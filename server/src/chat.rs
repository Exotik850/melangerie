use crate::logger::Log;
use crate::types::{message_from_row, ChatRoomID, ServerAction, UserAction, UserDB};
use crate::ws_handler::WebSocketHandler;
use crate::SqliteDB;
use crate::{
    auth::{decode_jwt, Jwt},
    types::{ChatMessage, UserStatus},
    UserID,
};
use rocket::futures::{SinkExt, StreamExt};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::tokio::select;
use rocket::{Shutdown, State};
use rusqlite::params;
use std::path::PathBuf;
use ws::stream::DuplexStream;
use ws::Message;

#[get("/connect")]
pub async fn connect<'r>(
    ws: ws::WebSocket,
    db: SqliteDB,
    user_db: &'r State<UserDB>,
    log: &'r State<Log>,
    shutdown: Shutdown,
) -> ws::Channel<'r> {
    ws.channel(move |stream| {
        Box::pin(async move { handle_connection(stream, shutdown, db, user_db, log).await })
    })
}

#[get("/list")]
pub async fn list_rooms(db: SqliteDB, user: Jwt) -> Json<Vec<String>> {
    let rooms: Vec<_> = db
        .run(move |d| {
            let mut stmt = d.prepare("select chatroom_id from chatrooms where user_id = ?")?;
            let stmt = stmt.query_map(params![user.name.0], |r| r.get(0))?;
            stmt.collect::<Result<Vec<_>, _>>()
        })
        .await
        .unwrap_or_default();
    Json(rooms)
}

#[post("/adduser/<room>/<user_id>")]
pub async fn add_user_to_room(
    room: ChatRoomID,
    user_id: UserID,
    db: SqliteDB,
    user_db: &State<UserDB>,
    auth_user: Jwt,
) -> Status {
    // if !add_user(user_id.clone(), room.clone(), &db).await {
    //     return Status::NotFound;
    // };
    // send_msg(
    //     ChatMessage {
    //         // id: MessageID(0),
    //         sender: "Server".into(),
    //         room,
    //         content: format!("{} added {} to the room", auth_user.name.0, user_id.0),
    //         timestamp: jsonwebtoken::get_current_timestamp(),
    //     },
    //     &db,
    //     user_db,
    // )
    // .await;

    Status::Ok
}

#[post("/chatroom", data = "<msg>")]
pub async fn send_message(msg: Json<ChatMessage>, user_db: &State<UserDB>, db: SqliteDB) {
    let chatroom_id = msg.0.room.clone();
    let users = db
        .run(move |d| {
            let mut stmt = d.prepare("select user_id from chatroom_users where chatroom_id = ?")?;
            let stmt = stmt.query_map(params![chatroom_id.0], |r| r.get(0).map(UserID))?;
            stmt.collect::<Result<Vec<_>, _>>()
        })
        .await
        .unwrap_or_default();
    let user_db = user_db.read().await;
    let action = ServerAction::Message(msg.0);
    for user in &users {
        let Some(user) = user_db.get(user) else {
            log::error!("User not found: {:?}", user);
            continue;
        };

        if let UserStatus::Active(sender) = &user.status {
            let Ok(_) = sender.send(action.clone()) else {
                log::error!("Failed to send message to user: {:?}", user.name);
                continue;
            };
        }
    }
}

#[post("/create/<name>/<users..>")]
pub async fn create_room(
    name: ChatRoomID,
    db: SqliteDB,
    user_db: &State<UserDB>,
    users: PathBuf,
    user: Jwt,
) -> Status {
    let room = name.0.replace('\'', "\'");
    let r2 = room.clone();
    if let Err(e) = db
        .run(move |d| {
            d.execute(
                "INSERT INTO chatrooms (chatroom_id) values (?)",
                params![room],
            )
        })
        .await
    {
        println!("Error creating room: {:?}", e);
        return Status::InternalServerError;
    };

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

    let uclone = users.clone();

    if let Err(e) = db
        .run(move |d| {
            let mut batch = String::from("BEGIN TRANSACTION;\n");
            for user in users {
                batch += &format!(
                    "INSERT INTO chatroom_users (chatroom_id, user_id) VALUES ('{}', '{user}');\n",
                    &r2
                );
            }
            batch += "COMMIT;";
            d.execute_batch(&batch)
        })
        .await
    {
        println!("Error adding users to room: {:?}", e);
        return Status::InternalServerError;
    };

    user_db
        .write_to(
            ChatMessage {
                // id: MessageID(0),
                sender: "admin".into(),
                room: name,
                content: format!("{} created the room", user.name.0),
                timestamp: jsonwebtoken::get_current_timestamp() as f64,
            },
            &uclone,
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

async fn send_action(action: ServerAction, user_db: &UserDB, id: &UserID) {
    let user_db = user_db.write().await;
    let Some(user) = user_db.get(id) else {
        log::error!("User not found: {:?}", id);
        return;
    };

    if let UserStatus::Active(sender) = &user.status {
        let Ok(_) = sender.send(action.clone()) else {
            log::error!("Failed to send message to user: {:?}", id);
            return;
        };
    }

    // user.messages.push(action);
}

async fn handle_connection(
    mut stream: DuplexStream,
    mut shutdown: Shutdown,
    db: SqliteDB,
    user_db: &UserDB,
    log: &Log,
) -> ws::result::Result<()> {
    let Some(id) = get_auth(&mut stream).await else {
        let _ = stream.send(Message::Close(None)).await;
        return Ok(());
    };
    let uida = id.clone();
    let uidb = id.clone();

    let (mut rx, chat) = {
        let mut user_db = user_db.write().await;
        let Some(user) = user_db.get_mut(&id) else {
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

        let messages = db.run(move |d| {
          d.prepare("SELECT m.* FROM messages m INNER JOIN chatroom_users cu ON m.chatroom_id = cu.chatroom_id WHERE cu.user_id = ?")?
            .query_map(params![uida.0], message_from_row)?
            .collect::<Result<Vec<_>, _>>()
        }).await.map_err(|e| {
          println!("Error gathering messages: {:?}", e);
          ws::result::Error::Utf8
        })?;

        if messages.is_empty() {
            log::error!("No messages found for user: {:?}", id);
        } else {
            log::info!("Found {} messages for {:?}", messages.len(), id);
        }

        user.status = UserStatus::Active(tx);
        (rx, messages)
    };

    for room in db
        .run(move |d| {
            d.prepare("select chatroom_id from chatroom_users where user_id = ?")?
                .query_map(params![uidb.0], |r| r.get(0).map(ChatRoomID))?
                .collect::<Result<Vec<_>, _>>()
        })
        .await
        .map_err(|e| {
            println!("Error getting chatroom ids: {:?}", e);
            ws::result::Error::Utf8
        })?
    // TODO : Handle error correctly
    {
        let msg = ServerAction::Add {
            room: room.clone(),
            added: id.clone(),
            adder: None,
        };
        let _ = stream
            .send(Message::binary(serde_json::to_vec(&msg).unwrap()))
            .await;
    }

    for msg in chat {
        let action = ServerAction::Message(msg);
        let _ = stream
            .send(Message::binary(serde_json::to_vec(&action).unwrap()))
            .await;
    }

    let state = (db, user_db.clone());
    loop {
        select! {
            // Shutdown the connection if the server is shutting down
            _ = &mut shutdown => {
                let _ = stream.send(Message::Close(None)).await;
                log::info!("Shutting down connection: {:?}", id);
                user_db.close_user(&id).await;
                break;
            },
            // A message has been sent to this user
            recv_msg = rx.recv() => if let Ok(msg) = recv_msg {
                let _ = stream.send(Message::binary(serde_json::to_vec(&msg).unwrap())).await;
            },
            // A message has been received from the user
            sent_msg = stream.next() => if let Some(Ok(msg)) = sent_msg {
                // if handle_user_message(msg, &id, &db, user_db, log).await {
                //     let _ = stream.send(Message::Close(None)).await;
                //     user_db.close_user(&id).await;
                //     break;
                // }
                state.handle_message(&id, msg).await;
            }
        }
        stream.flush().await?;
    }

    Ok(())
}

async fn handle_user_message(
    msg: ws::Message,
    id: &UserID,
    db: &SqliteDB,
    user_db: &UserDB,
    log: &Log,
) -> bool {
    if let Message::Close(_) = msg {
        return true;
    }
    log::info!("Received message: {:?}", msg);
    let msg: UserAction = serde_json::from_slice(&msg.into_data()).unwrap();
    match msg {
        // UserAction::Message(msg) => send_msg(msg, db, user_db).await,
        UserAction::Report(msg) => {
            let _ = log.write(format!("Report: {}", msg)).await;
        }
        // UserAction::Leave(room) => {
        //     {
        //         // let mut cdb = chatrooms.write().await;
        //         // let Some(users) = cdb.get_mut(&room) else {
        //         //     log::error!("Room not found: {:?}", room);
        //         //     return false;
        //         // };
        //         // users.retain(|user| user != id);
        //         let (room, id) = (room.clone(), id.clone());

        //     }
        //     send_msg(
        //         ChatMessage {
        //             // id: MessageID(0),
        //             sender: "admin".into(),
        //             room,
        //             content: format!("{} left the room", id.0),
        //             timestamp: jsonwebtoken::get_current_timestamp(),
        //         },
        //         db,
        //         user_db,
        //     )
        //     .await;
        // }
        // UserAction::Add((room, user)) => {
        //     if !add_user(user.clone(), room.clone(), db).await {
        //         return false;
        //     }
        //     send_msg(
        //         ChatMessage {
        //             // id: MessageID(0),
        //             sender: "admin".into(),
        //             room,
        //             content: format!("{} added {} to the room", id.0, user.0),
        //             timestamp: jsonwebtoken::get_current_timestamp(),
        //         },
        //         db,
        //         user_db,
        //     )
        //     .await;
        // }
        UserAction::ListUsers => {
            let user: Vec<_> = user_db.read().await.keys().cloned().collect();
            send_action(ServerAction::List(user), user_db, id).await;
        }
        UserAction::TimeIn(note) => {}
        UserAction::TimeOut(note) => {}
        UserAction::CheckTime => {
            // let timed_in = server_state.time.is_active(id).await.unwrap_or(false);
            let aid = id.clone();
        }
        // UserAction::AllowTime()
        _ => {
            log::error!("Invalid action: {:?}", msg);
        }
    }
    false
}
