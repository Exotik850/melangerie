use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use rocket::http::{Cookie, CookieJar};
use rocket::request::{FromRequest, Outcome};
use rocket::response::stream::{Event, EventStream};
use rocket::tokio::select;
use rocket::tokio::sync::broadcast::Sender;
use rocket::Shutdown;
use rocket::{
    fs::NamedFile, http::Status, tokio::sync::broadcast::channel, tokio::sync::RwLock, Request,
    State,
};
use types::{ChatMessage, InactiveUser, User, UserID, UserStatus};
use ws::{Message, WebSocket};
mod types;

#[macro_use]
extern crate rocket;

const FILE_PATH: &str = "./public";
const HASH_COST: u32 = 12;

type LockedMap<K, V> = RwLock<HashMap<K, V>>;
type LockedSet<T> = RwLock<HashSet<T>>;

type ChatroomsDB = LockedMap<String, Vec<UserID>>;

// Map Users to their sender which is sending to their active websocket connection
// and a Vec of messages that have been sent to them while they were offline
type UserDB = LockedMap<UserID, User>;

struct SecureUser(UserID);

#[async_trait]
impl<'r> FromRequest<'r> for SecureUser {
    type Error = &'static str;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let cookie_jar = req.cookies();
        let id = match cookie_jar.get_private("user_id") {
            Some(id) => UserID(id.value().to_string()),
            None => return Outcome::Error((Status::Unauthorized, "No user_id cookie found")),
        };
        let pwd = match cookie_jar.get_private("user_pwd") {
            Some(pwd) => pwd,
            None => return Outcome::Error((Status::Unauthorized, "No user_pwd cookie found")),
        };
        let user_db: &UserDB = req.rocket().state().expect("UserDB not found");
        let user_db = user_db.read().await;
        let password = match user_db.get(&id) {
            Some(user) => &user.password,
            None => return Outcome::Error((Status::Unauthorized, "Invalid user_id cookie")),
        };
        if bcrypt::verify(pwd.value(), password).unwrap_or(false) {
            Outcome::Success(SecureUser(id))
        } else {
            Outcome::Error((Status::Unauthorized, "Invalid user_pwd cookie"))
        }
    }
}

#[get("/connect")]
async fn connect<'r>(
    ws: WebSocket,
    user_db: &'r State<UserDB>,
    chat_db: &'r State<ChatroomsDB>,
    mut shutdown: Shutdown,
    user: SecureUser,
) -> Option<ws::Channel<'r>> {
    use rocket::futures::{SinkExt, StreamExt};
    let (mut rx, chat) = {
        let mut db = user_db.write().await;
        let user = db.get_mut(&user.0)?;
        let chat = match &mut user.status {
            UserStatus::Active(_) => {
                return None;
            }
            UserStatus::Inactive(chat) => &mut chat.messages,
        };
        let (tx, rx) = channel(16);
        let out: Vec<_> = chat.drain(..).collect();
        user.status = UserStatus::Active(ActiveUser { sender: tx });
        (rx, out)
    };

    let stream = ws.channel(move |mut stream| {
        Box::pin(async move {
            for msg in chat {
                let _ = stream.send(Message::binary(serde_json::to_vec(&msg).unwrap()));
            }

            loop {
                select! {
                    // Shutdown the connection if the server is shutting down
                    _ = &mut shutdown => {
                        let _ = stream.send(Message::Close(None)).await;
                        break;
                    },
                    // A message has been sent to this user
                    recv_msg = rx.recv() => if let Ok(msg) = recv_msg {
                        let _ = stream.send(Message::binary(serde_json::to_vec(&msg).unwrap())).await;
                    },
                    // A message has been received from the user
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
                        msg.sender = user.0.clone();

                        if let Some(users) = chat_db.read().await.get(&msg.room.0) {
                          let mut user_db = user_db.write().await;
                          for user in users {
                              let Some(user) = user_db.get_mut(user) else {
                                  log::error!("User not found: {:?}", user);
                                  continue;
                              };
                    
                              match &mut user.status {
                                  UserStatus::Active(ActiveUser { sender }) => {
                                      let Ok(_) = sender.send(msg.clone()) else {
                                          log::error!("Failed to send message to user: {:?}", user.name);
                                          continue;
                                      };
                                  }
                                  UserStatus::Inactive(InactiveUser { messages }) => {
                                      messages.push(msg.clone());
                                  }
                              }
                          }
                      }
                    }
                }
            }

            Ok(())
        })
    });

    Some(stream)
}

use rocket::serde::json::Json;

use crate::types::ActiveUser;

#[post("/chatroom", data = "<msg>")]
async fn send_message(
    msg: Json<ChatMessage>,
    chat_db: &State<ChatroomsDB>,
    user_db: &State<UserDB>,
) -> Status {
    let db = chat_db.read().await;
    if let Some(users) = db.get(&msg.0.room.0) {
        let mut user_db = user_db.write().await;
        for user in users {
            let Some(user) = user_db.get_mut(user) else {
                log::error!("User not found: {:?}", user);
                continue;
            };

            match &mut user.status {
                UserStatus::Active(ActiveUser { sender }) => {
                    let Ok(_) = sender.send(msg.0.clone()) else {
                        log::error!("Failed to send message to user: {:?}", user.name);
                        continue;
                    };
                }
                UserStatus::Inactive(InactiveUser { messages }) => {
                    messages.push(msg.0.clone());
                }
            }
        }
        Status::Ok
    } else {
        Status::NotFound
    }
}

#[post("/create/<name>/<users..>")]
async fn create_room(name: String, db: &State<ChatroomsDB>, users: PathBuf) -> Status {
    let mut db = db.write().await;
    if db.contains_key(&name) {
        return Status::Conflict;
    }

    // Make sure all the users are valid
    let users = users
        .iter()
        .map(|user| UserID(user.to_string_lossy().to_string()))
        .filter(|user| db.contains_key(&user.0))
        .collect();

    db.insert(name, users);
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

#[post("/adduser/<room>/<add_user>")]
async fn add_user_to_room(
    room: String,
    add_user: String,
    chat_db: &State<ChatroomsDB>,
    user_id: SecureUser,
) -> Status {
    let mut chat_db = chat_db.write().await;
    let add_user = UserID(add_user);

    // Check if the room exists
    let Some(users) = chat_db.get_mut(&room) else {
        return Status::NotFound;
    };
    // Check that the person adding the user is in the room
    if !users.contains(&user_id.0) {
        return Status::Unauthorized;
    }

    // Check if the user is already in the room
    if users.contains(&add_user) {
        return Status::Conflict;
    }
    // Add the user to the room
    users.push(add_user);
    Status::Ok
}

#[get("/checkuser/<name>")]
async fn check_user(name: String, db: &State<UserDB>) -> &'static str {
    if db.read().await.contains_key(&UserID(name)) {
        "found"
    } else {
        "not found"
    }
}

#[post("/createuser/<name>/<password>")]
async fn create_user(
    name: String,
    password: String,
    db: &State<UserDB>,
    cookies: &CookieJar<'_>,
) -> Status {
    let mut db = db.write().await;
    let id = UserID(name.clone());
    if db.contains_key(&id) {
        return Status::Conflict;
    }

    let Ok(hashed) = bcrypt::hash(&password, HASH_COST) else {
        return Status::InternalServerError;
    };

    // Insert the user into the database
    db.insert(
        id.clone(),
        User {
            name: id,
            password: hashed,
            status: UserStatus::Inactive(InactiveUser {
                messages: Vec::new(),
            }),
        },
    );
    // Set the user_id and user_pwd cookies
    cookies.add_private(Cookie::new("user_id", name));
    cookies.add_private(Cookie::new("user_pwd", password));
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
                create_user,
                add_user_to_room
            ],
        )
    // .register("/", catchers![echo_catcher])
}
