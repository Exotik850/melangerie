use std::{collections::HashMap, ops::Deref, sync::Arc};

use rocket::{
    request::{FromParam, FromRequest, Outcome},
    tokio::sync::{broadcast::Sender, RwLock},
    Request,
};
use rusqlite::types::FromSql;
use serde::{Deserialize, Serialize};

use crate::SqliteDB;

#[derive(Default, Clone)]
pub struct UserDB(Arc<RwLock<HashMap<UserID, User>>>);
impl Deref for UserDB {
    type Target = Arc<RwLock<HashMap<UserID, User>>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl UserDB {
    pub async fn get_user(&self, id: &UserID) -> Option<User> {
        self.read().await.get(id).cloned()
    }
    pub async fn add_user(&self, user: User) {
        self.write().await.insert(user.name.clone(), user);
    }
    pub async fn write_to(&self, message: ChatMessage, users: &[UserID]) {
        let udb = self.read().await;
        for user in users {
            if let Some(UserStatus::Active(sender)) = udb.get(user).map(|u| &u.status) {
                if let Err(e) = sender.send(ServerAction::Message(message.clone())) {
                    log::error!("Failed to send message to user: {e:?}");
                };
            } else {
                log::warn!("User not found or inactive: {:?}", user);
            }
        }
    }
    pub async fn close_user(&self, id: &UserID) {
        if let Some(user) = self.write().await.get_mut(id) {
            user.status = UserStatus::Inactive;
            log::info!("User disconnected: {:?}", id);
        }
    }
    pub async fn all_users(&self) -> Vec<UserID> {
        self.read().await.keys().cloned().collect()
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct User {
    pub name: UserID,
    pub status: UserStatus,
    // Hashed password
    pub password: String,
}

fn user_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<User> {
    Ok(User {
        name: row.get(0)?,
        status: UserStatus::Inactive,
        password: row.get(1)?,
    })
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum UserStatus {
    #[serde(skip)]
    Active(Sender<ServerAction>),
    Inactive,
}

impl PartialEq for UserStatus {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (UserStatus::Active(a), UserStatus::Active(b)) => a.same_channel(b),
            (UserStatus::Inactive, UserStatus::Inactive) => true,
            _ => false,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Eq, Hash, Clone, Ord, PartialOrd)]
pub struct UserID(pub(crate) String);
impl FromSql for UserID {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        value.as_str().map(|s| UserID(s.to_string()))
    }
}

impl<T: Into<String>> From<T> for UserID {
    fn from(val: T) -> Self {
        UserID(val.into())
    }
}

impl std::fmt::Display for UserID {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl<'r> FromParam<'r> for UserID {
    type Error = &'static str;
    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        Ok(UserID(param.to_string()))
    }
}

#[async_trait]
impl<'r> FromRequest<'r> for UserID {
    type Error = &'static str;
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let cookie_jar = req.cookies();
        let user_id = cookie_jar.get_private("user_id");
        let id = match user_id {
            Some(id) => UserID(id.value().to_string()),
            None => {
                return Outcome::Error((
                    rocket::http::Status::Unauthorized,
                    "No user_id cookie found",
                ))
            }
        };
        let user_db: &SqliteDB = req.rocket().state().unwrap();
        let query_id = id.clone();
        let Ok(_): Result<UserID, _> = user_db
            .0
            .run(|conn| {
                conn.query_row("select id from Users where id=?", [query_id.0], |row| {
                    row.get(0)
                })
            })
            .await
        else {
            return Outcome::Error((rocket::http::Status::Unauthorized, "Invalid user_id cookie"));
        };
        Outcome::Success(id)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Eq, Hash, Clone, Copy)]
pub struct MessageID(pub(crate) i64);
impl FromSql for MessageID {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        value.as_i64().map(MessageID)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Eq, Hash, Clone)]
pub struct ChatRoomID(pub(crate) String);
impl FromSql for ChatRoomID {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        value.as_str().map(|i| ChatRoomID(i.into()))
    }
}
impl std::fmt::Display for ChatRoomID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl<'r> FromParam<'r> for ChatRoomID {
    type Error = &'static str;
    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        Ok(ChatRoomID(param.to_string()))
    }
}
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct ChatMessage {
    // pub id: MessageID,
    pub sender: UserID,
    pub room: ChatRoomID,
    pub content: String,
    pub timestamp: f64,
}

pub fn message_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ChatMessage> {
    Ok(ChatMessage {
        // id: row.get(0)?,
        sender: row.get(0)?,
        room: row.get(1)?,
        content: row.get(2)?,
        timestamp: row.get(3)?,
    })
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(tag = "action", content = "data")]
pub enum UserAction {
    Message(ChatMessage),
    Report(String),
    Leave(ChatRoomID),
    Add((ChatRoomID, UserID)),
    ListUsers,
    TimeIn(Option<String>),
    TimeOut(Option<String>),
    CheckTime,
    AllowTime(UserID),
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(tag = "action", content = "data")]
pub enum ServerAction {
    Message(ChatMessage),
    Add {
        room: ChatRoomID,
        adder: Option<UserID>,
        added: UserID,
    },
    List(Vec<UserID>),
    TimedIn(bool),
    Leave((ChatRoomID, UserID)),
    Error(String),
}

impl ServerAction {
    pub fn room(&self) -> Option<&ChatRoomID> {
        match self {
            ServerAction::Message(msg) => Some(&msg.room),
            ServerAction::Add { room, .. } => Some(room),
            ServerAction::Leave((room, _)) => Some(room),
            _ => None,
        }
    }
}

impl From<ChatMessage> for ServerAction {
    fn from(msg: ChatMessage) -> Self {
        ServerAction::Message(msg)
    }
}

// impl From<ChatMessage> for Vec<u8> {
//     fn from(val: ChatMessage) -> Self {
//         serde_json::to_vec(&val).unwrap()
//     }
// }

impl std::fmt::Display for ChatMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.content)
    }
}
