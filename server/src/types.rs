use rocket::{
    request::{FromRequest, Outcome},
    tokio::sync::broadcast::Sender,
    Request,
};
use serde::{Deserialize, Serialize};

use crate::UserDB;

pub struct User {
    pub name: UserID,
    pub status: UserStatus,
    // Hashed password
    pub password: String,
}

pub enum UserStatus {
    Active(ActiveUser),
    Inactive(InactiveUser),
}

pub struct ActiveUser {
    pub sender: Sender<ChatMessage>,
}
pub struct InactiveUser {
    pub messages: Vec<ChatMessage>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Eq, Hash, Clone)]
pub struct UserID(pub(crate) String);

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

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct ChatMessage {
    pub sender: UserID,
    pub room: ChatRoomID,
    pub content: String,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum Action {
  Message(ChatMessage),
  Join(ChatRoomID),
  Leave(ChatRoomID),
  Add(ChatRoomID, UserID),
  Remove(ChatRoomID, UserID),
  Create(ChatRoomID),
  Delete(ChatRoomID),
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

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ChatRoom {
    pub name: ChatRoomID,
    pub participants: Vec<UserID>,
    pub content: Vec<ChatMessage>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct UserData {
    pub name: UserID,
    pub rooms: Vec<ChatRoomID>,
    pub active: Option<ChatRoomID>,
}
