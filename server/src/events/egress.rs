use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::{
    types::{ChatMessage, ChatRoomID, UserDB, UserID},
    ws_handler::UserEvent,
    SqliteDB,
};

#[derive(Serialize, Deserialize)]
pub struct RoomEgress {
    room_id: String,
    user_id: String,
    action: RoomEvent,
}

#[derive(Serialize, Deserialize)]
pub enum RoomEvent {
    Leave,
    Join,
}

impl std::fmt::Display for RoomEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RoomEvent::Leave => write!(f, "left"),
            RoomEvent::Join => write!(f, "joined"),
        }
    }
}

#[async_trait]
impl UserEvent for RoomEgress {
    type State = (SqliteDB, UserDB);
    async fn handle(self, user_id: &UserID, (db, user_db): &Self::State) {
        let Self {
            room_id,
            user_id,
            action,
        } = self;
        log::info!("User {} {} room {}", user_id, action, room_id);
        match action {
            RoomEvent::Leave => {
                // if let Err(e) = db
                //     .run(move |d| {
                //         d.execute(
                //             "delete from chatroom_users where chatroom_id = ? and user_id = ?",
                //             params![room_id, user_id],
                //         )
                //     })
                //     .await
                // {
                //     log::error!("Failed to remove user from room: {}", e);
                // };
                if !remove_user(
                    UserID(user_id.clone()),
                    ChatRoomID(room_id.clone()),
                    db,
                    user_db,
                )
                .await
                {
                    log::error!("Failed to remove {user_id} from {room_id}");
                }
            }
            RoomEvent::Join => {
                if !add_user(
                    UserID(user_id.clone()),
                    ChatRoomID(room_id.clone()),
                    db,
                    user_db,
                )
                .await
                {
                    log::error!("Failed to add {user_id} to {room_id}");
                };
            }
        };
    }
}

async fn add_user(user_id: UserID, room: ChatRoomID, db: &SqliteDB, user_db: &UserDB) -> bool {
    // Add the user to the room

    let content = format!("User {} joined room {}", user_id, room);
    let room_id = room.clone();
    db
.run(move |d| {
          let Ok(_): Result<String, _> = d.query_row(
              "select chatroom_id from chatrooms c inner join chatroom_users cu on c.chatroom_id = cu.chatroom_id where cu.user_id != ? and c.chatroom_id = ?",
              params![room.0],
              |r| r.get(0),
          ) else {
            return false;
          };
          if d.execute("insert into chatroom_users (chatroom_id, user_id) values (?, ?)", params![room.0, user_id.0]).is_err() {
              return false;
          };
          true
      })
      .await;

    db.send_msg(
        ChatMessage {
            sender: "admin".into(),
            content,
            room: room_id,
            timestamp: jsonwebtoken::get_current_timestamp() as f64,
        },
        user_db,
    )
    .await;
    true
}

async fn remove_user(user_id: UserID, room: ChatRoomID, db: &SqliteDB, user_db: &UserDB) -> bool {
    // Remove the user from the room
    let content = format!("User {} left room {}", user_id, room);
    let room_id = room.clone();
    let removed = db
        .run(move |db| {
            if db
                .execute(
                    "delete from chatroom_users where chatroom_id = ? and user_id = ?",
                    params![room.0, user_id.0],
                )
                .is_err()
            {
                return false;
            };
            true
        })
        .await;
    if removed {
        db.send_msg(
            ChatMessage {
                sender: "admin".into(),
                content,
                room: room_id,
                timestamp: jsonwebtoken::get_current_timestamp() as f64,
            },
            user_db,
        )
        .await;
    }
    removed
}
