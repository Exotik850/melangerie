use rocket::{
    request::{FromParam, FromRequest, Outcome},
    tokio::sync::broadcast::Sender,
    Request,
};
use serde::{Deserialize, Serialize};

use crate::UserDB;

pub struct User {
    pub name: UserID,
    pub status: UserStatus,
    // Messages to this user, sent on reconnect
    pub messages: Vec<ServerAction>,
    // Hashed password
    pub password: String,
}

pub enum UserStatus {
    Active(Sender<ServerAction>),
    Inactive,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Eq, Hash, Clone)]
pub struct UserID(pub(crate) String);

impl<T: std::fmt::Display> From<T> for UserID {
    fn from(s: T) -> Self {
        UserID(s.to_string())
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
        let user_db: &UserDB = req.rocket().state().unwrap();
        if user_db.read().await.contains_key(&id) {
            Outcome::Success(id)
        } else {
            Outcome::Error((rocket::http::Status::Unauthorized, "Invalid user_id cookie"))
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Eq, Hash, Clone, Copy)]
pub struct MessageID(pub(crate) usize);

#[derive(Serialize, Deserialize, PartialEq, Debug, Eq, Hash, Clone)]
pub struct ChatRoomID(pub(crate) String);

impl<'r> FromParam<'r> for ChatRoomID {
    type Error = &'static str;
    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        Ok(ChatRoomID(param.to_string()))
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct ChatMessage {
    pub sender: UserID,
    pub room: ChatRoomID,
    pub content: String,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(tag = "action", content = "data")]
pub enum UserAction {
    Message(ChatMessage),
    Report(String),
    Leave(ChatRoomID),
    Add((ChatRoomID, UserID)),
    ListUsers,
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
    Leave((ChatRoomID, UserID)),
    Error(String),
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