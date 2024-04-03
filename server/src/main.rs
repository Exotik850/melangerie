use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use rocket::futures::{pin_mut, Stream, StreamExt, TryStreamExt};
use rocket::http::{Cookie, CookieJar};
use rocket::response::stream::{Event, EventStream};
use rocket::tokio::select;
use rocket::tokio::sync::broadcast::Sender;
use rocket::Shutdown;
use rocket::{
    fs::NamedFile, http::Status, tokio::sync::broadcast::channel, tokio::sync::RwLock, Request,
    State,
};
use types::{ChatMessage, UserID};
use ws::{Message, WebSocket};
mod types;

#[macro_use]
extern crate rocket;

const FILE_PATH: &str = "./public";

type LockedMap<K, V> = RwLock<HashMap<K, V>>;
type LockedSet<T> = RwLock<HashSet<T>>;

type ChatroomsDB = LockedMap<String, (Sender<ChatMessage>, Vec<ChatMessage>)>;
type UserDB = LockedSet<UserID>;
// type UserDB = LockedMap<UserID, Vec<ChatMessage>>;

#[get("/connect/<room>")]
async fn connect<'r>(
    ws: WebSocket,
    room: String,
    db: &State<ChatroomsDB>,
    mut shutdown: Shutdown,
    user: UserID,
) -> Option<ws::Channel<'static>> {
    use rocket::futures::{SinkExt, StreamExt};
    let (tx, mut rx, chat) = {
        let db = db.read().await;
        let (tx, chat) = db.get(&room)?;
        (tx.clone(), tx.subscribe(), chat.clone())
    };

    let stream = ws.channel(move |mut stream| {
        Box::pin(async move {
            for msg in chat {
                let _ = stream.send(Message::binary(serde_json::to_vec(&msg).unwrap()));
            }

            loop {
                select! {
                    _ = &mut shutdown => {
                        let _ = stream.send(Message::Close(None)).await;
                        break;
                    },
                    recv_msg = rx.recv() => if let Ok(msg) = recv_msg {
                        let _ = stream.send(Message::binary(serde_json::to_vec(&msg).unwrap())).await;
                    },
                    sent_msg = stream.next() => if let Some(Ok(msg)) = sent_msg {
                        if let Message::Close(_) = msg {
                            break;
                        }

                        log::info!("Received message: {:?}", msg);
                        if !msg.is_binary() && !msg.is_text() {
                            continue;
                        }
                        let mut msg: ChatMessage = serde_json::from_slice(&msg.into_data()).unwrap();
                        // Make sure the sender is the user who sent the message
                        msg.sender = user.clone();
                        let _ = tx.send(msg);
                    }
                }
            }

            Ok(())
        })
    });

    Some(stream)
}

use rocket::serde::json::Json;

#[post("/chatroom", data = "<msg>")]
async fn send_message(msg: Json<ChatMessage>, db: &State<ChatroomsDB>) -> Status {
    let mut db = db.write().await;
    if let Some((tx, chat)) = db.get_mut(&msg.0.room.0) {
        if let Ok(_) = tx.send(msg.0.clone()) {
            chat.push(msg.0);
            Status::Ok
        } else {
            Status::InternalServerError
        }
    } else {
        Status::NotFound
    }
}

#[post("/create/<name>")]
async fn create_room(name: String, db: &State<ChatroomsDB>) -> Status {
    let mut db = db.write().await;
    let (tx, _rx) = channel(16);
    if db.contains_key(&name) {
        return Status::Conflict;
    }
    db.insert(name, (tx, vec![]));
    Status::Ok
}

#[get("/<file..>")]
async fn file_server(file: PathBuf) -> std::io::Result<NamedFile> {
    let path = if file.to_str().map_or(false, str::is_empty) {
        PathBuf::from("index.html")
    } else {
        file
    };
    NamedFile::open(PathBuf::from(FILE_PATH).join(path)).await
}

#[get("/checkuser/<name>")]
async fn check_user(name: String, db: &State<UserDB>) -> &'static str {
    if db.read().await.contains(&UserID(name)) {
        "found"
    } else {
        "not found"
    }
}

#[post("/createuser/<name>")]
async fn create_user(name: String, db: &State<UserDB>, cookies: &CookieJar<'_>) -> Status {
    let mut db = db.write().await;
    let id = UserID(name.clone());
    if db.contains(&id) {
        return Status::Conflict;
    }
    db.insert(id);
    cookies.add_private(Cookie::new("user_id", name));
    Status::Ok
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(UserDB::default())
        .manage(ChatroomsDB::default())
        .mount(
            "/",
            routes![
                file_server,
                create_room,
                connect,
                send_message,
                check_user,
                create_user
            ],
        )
    // .register("/", catchers![echo_catcher])
}
