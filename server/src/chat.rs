use crate::types::{ChatRoomID, ServerAction, UserAction};
use crate::{
    auth::{decode_jwt, JWT},
    types::{ChatMessage, UserStatus},
    ChatroomsDB, UserDB, UserID,
};
use rocket::futures::{SinkExt, StreamExt};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::tokio::select;
use rocket::{Shutdown, State};
use std::path::PathBuf;
use ws::stream::DuplexStream;
use ws::Message;

#[get("/connect")]
pub async fn connect<'r>(
    ws: ws::WebSocket,
    user_db: &'r State<UserDB>,
    chat_db: &'r State<ChatroomsDB>,
    shutdown: Shutdown,
) -> ws::Channel<'r> {
    ws.channel(move |stream| {
        Box::pin(async move { handle_connection(stream, shutdown, user_db, chat_db).await })
    })
}

#[get("/list")]
pub async fn list_rooms(chat_db: &State<ChatroomsDB>, user: JWT) -> Json<Vec<String>> {
    let db = chat_db.read().await;
    let rooms = db
        .iter()
        .filter_map(|(name, users)| users.contains(&user.name).then(|| name.0.clone()))
        .collect();
    Json(rooms)
}

#[post("/adduser/<room>/<user_id>")]
pub async fn add_user_to_room(
    room: ChatRoomID,
    user_id: UserID,
    user_db: &State<UserDB>,
    chat_db: &State<ChatroomsDB>,
    auth_user: JWT,
) -> Status {
    {
        let mut cdb = chat_db.write().await;

        // Check if the room exists
        let Some(users) = cdb.get_mut(&room) else {
            return Status::NotFound;
        };

        // Check if the user exists
        let Some(_) = user_db.read().await.get(&user_id) else {
            return Status::NotFound;
        };

        // Check if the user is already in the room
        if users.contains(&user_id) {
            return Status::Conflict;
        }
        // Add the user to the room
        users.push(user_id.clone());
    }

    send_msg(
        ChatMessage {
            sender: "Server".into(),
            room,
            content: format!("{} added {} to the room", auth_user.name.0, user_id.0),
            timestamp: jsonwebtoken::get_current_timestamp(),
        },
        chat_db,
        user_db,
    )
    .await;

    Status::Ok
}

#[post("/chatroom", data = "<msg>")]
pub async fn send_message(
    msg: Json<ChatMessage>,
    chat_db: &State<ChatroomsDB>,
    user_db: &State<UserDB>,
) -> Status {
    let db = chat_db.read().await;
    if let Some(users) = db.get(&msg.0.room) {
        let mut user_db = user_db.write().await;
        for user in users {
            let Some(user) = user_db.get_mut(user) else {
                log::error!("User not found: {:?}", user);
                continue;
            };

            if let UserStatus::Active(sender) = &user.status {
                let Ok(_) = sender.send(ServerAction::Message(msg.0.clone())) else {
                    log::error!("Failed to send message to user: {:?}", user.name);
                    continue;
                };
            }
            user.messages.push(msg.0.clone());
        }
        Status::Ok
    } else {
        Status::NotFound
    }
}

#[post("/create/<name>/<users..>")]
pub async fn create_room(
    name: ChatRoomID,
    chat_db: &State<ChatroomsDB>,
    user_db: &State<UserDB>,
    users: PathBuf,
    user: JWT,
) -> Status {
    {
        let mut cdb = chat_db.write().await;
        if cdb.contains_key(&name) {
            return Status::Conflict;
        }

        let udb = user_db.read().await;

        // Make sure all the users are valid
        let mut users: Vec<_> = users
            .iter()
            .map(|user| UserID(user.to_string_lossy().to_string()))
            .filter(|user| udb.contains_key(user))
            .collect();

        if !users.contains(&user.name) {
            users.push(user.name.clone());
        }

        if users.is_empty() {
            return Status::NotFound;
        }

        cdb.insert(name.clone(), users);
    }

    send_msg(
        ChatMessage {
            sender: "Server".into(),
            room: name,
            content: format!("{} created the room", user.name.0),
            timestamp: jsonwebtoken::get_current_timestamp(),
        },
        chat_db,
        user_db,
    )
    .await;

    Status::Ok
}

async fn get_auth(stream: &mut DuplexStream) -> Option<UserID> {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET not set");
    let Some(Ok(auth_token)) = stream.next().await else {
        log::error!("Received no auth token from client");
        return None;
    };
    let token_string = auth_token.into_text().ok()?;
    let Some(token) = decode_jwt(&token_string, &secret) else {
        log::error!("Failed to decode JWT token");
        return None;
    };
    Some(token.name)
}

async fn send_msg(msg: ChatMessage, chat_db: &State<ChatroomsDB>, user_db: &State<UserDB>) {
  let db = chat_db.read().await;
  let Some(room) = db.get(&msg.room) else {
    log::error!("Room not found: {:?}", msg.room);
    return;
  };
  let mut user_db = user_db.write().await;
  for user in room {
        let Some(user) = user_db.get_mut(user) else {
            log::error!("User not found: {:?}", user);
            continue;
        };

        if let UserStatus::Active(sender) = &user.status {
            let Ok(_) = sender.send(ServerAction::Message(msg.clone())) else {
                log::error!("Failed to send message to user: {:?}", user.name);
                continue;
            };
        }

        user.messages.push(msg.clone());
    }
}

async fn handle_user_close(id: UserID, user_db: &State<UserDB>) {
    if let Some(user) = user_db.write().await.get_mut(&id) {
        user.status = UserStatus::Inactive;
        log::info!("User disconnected: {:?}", id);
    }
}

async fn handle_connection(
    mut stream: DuplexStream,
    mut shutdown: Shutdown,
    user_db: &State<UserDB>,
    chat_db: &State<ChatroomsDB>,
) -> ws::result::Result<()> {
    let Some(id) = get_auth(&mut stream).await else {
        let _ = stream.send(Message::Close(None)).await;
        return Ok(());
    };

    let (mut rx, chat) = {
        let mut db = user_db.write().await;
        let Some(user) = db.get_mut(&id) else {
            log::error!("User not found: {:?}", id);
            let _ = stream.send(Message::Close(None)).await;
            return Ok(());
        };
        if let UserStatus::Active(_) = &user.status {
            let _ = stream.send(Message::Close(None)).await;
            log::error!("User already connected: {:?}", id);
            return Ok(());
        }
        let (tx, rx) = rocket::tokio::sync::broadcast::channel(16);
        user.status = UserStatus::Active(tx);
        (rx, user.messages.clone())
    };

    for room in chat_db.read().await.iter().filter_map(|f| f.1.contains(&id).then_some(f.0)) {
        let msg = ServerAction::Join((room.clone(), id.clone()));
        let _ = stream
            .send(Message::binary(serde_json::to_vec(&msg).unwrap()))
            .await;
    }

    for msg in chat {
        let msg = ServerAction::Message(msg);
        let _ = stream
            .send(Message::binary(serde_json::to_vec(&msg).unwrap()))
            .await;
    }

    loop {
        select! {
            // Shutdown the connection if the server is shutting down
            _ = &mut shutdown => {
                let _ = stream.send(Message::Close(None)).await;
                log::info!("Shutting down connection: {:?}", id);
                handle_user_close(id, user_db).await;
                break;
            },
            // A message has been sent to this user
            recv_msg = rx.recv() => if let Ok(msg) = recv_msg {
                log::info!("Sending message: {:?}", msg);
                let _ = stream.send(Message::binary(serde_json::to_vec(&msg).unwrap())).await;
            },
            // A message has been received from the user
            sent_msg = stream.next() => if let Some(Ok(msg)) = sent_msg {
                if handle_user_message(msg, chat_db, user_db).await {
                    let _ = stream.send(Message::Close(None)).await;
                    handle_user_close(id, user_db).await;
                    break;
                }
            }
        }
        stream.flush().await?;
    }

    Ok(())
}

async fn handle_user_message(
    msg: ws::Message,
    chat_db: &State<ChatroomsDB>,
    user_db: &State<UserDB>,
) -> bool {
    if let Message::Close(_) = msg {
        return true;
    }
    if !msg.is_binary() && !msg.is_text() {
        return false;
    }
    let data = msg.into_data();
    let Ok(msg) = serde_json::from_slice::<UserAction>(&data) else {
        log::error!(
            "Failed to parse message: {:?}",
            String::from_utf8_lossy(&data)
        );
        return false;
    };
    log::info!("Received message: {:?}", msg);
    match msg {
        UserAction::Message(msg) => send_msg(msg, chat_db, user_db).await,
        _ => {}
    }
    false
}
