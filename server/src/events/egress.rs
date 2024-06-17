use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::{
    types::{UserDB, UserID},
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

#[async_trait]
impl UserEvent for RoomEgress {
    type State = (SqliteDB, UserDB);
    async fn handle(self, user_id: &UserID, (db, user_db): &Self::State) {
        let Self {
            room_id,
            user_id,
            action,
        } = self;

        match action {
            RoomEvent::Leave => {
                if let Err(e) = db
                    .run(move |d| {
                        d.execute(
                            "delete from chatroom_users where chatroom_id = ? and user_id = ?",
                            params![room_id, user_id],
                        )
                    })
                    .await
                {
                    log::error!("Failed to remove user from room: {}", e);
                };
            }
            RoomEvent::Join => {
                if let Err(e) = db
                    .run(move |d| {
                        d.execute(
                            "insert into chatroom_users (chatroom_id, user_id) values (?, ?)",
                            params![room_id, user_id],
                        )
                    })
                    .await
                {
                    log::error!("Failed to add user to room: {}", e);
                };
            }
        };
    }
}
