use crate::{
    types::{ChatMessage, UserDB, UserID},
    ws_handler::UserEvent,
    SqliteDB,
};
use rusqlite::params;

async fn send_msg(msg: ChatMessage, db: &SqliteDB, user_db: &UserDB) {
    let chatroom_id = msg.room.clone();
    let users: Vec<_> = db
        .run(move |d| {
            d.prepare("select user_id from chatroom_users where chatroom_id = ?")?
                .query_map(params![chatroom_id.0], |r| {
                    r.get(0).map(crate::types::UserID)
                })?
                .collect::<Result<Vec<_>, _>>()
        })
        .await
        .unwrap_or_default();
    user_db.write_to(msg.clone(), &users).await;
    if let Err(e) = db
        .run(move |d| {
            d.execute(
                "INSERT INTO messages (user_id, chatroom_id, message, created_at) VALUES (?, ?, ?, ?)",
                params![msg.sender.0, msg.room.0, msg.content, msg.timestamp],
            )
        })
        .await
    {
        log::error!("Failed to insert message into database: {}", e);
    };
}

// Implement the `UserEvent` trait for each event type
#[async_trait]
impl UserEvent for ChatMessage {
    type State = (SqliteDB, UserDB);
    async fn handle(self, user_id: &UserID, (db, user_db): &Self::State) {
        send_msg(self, db, user_db).await;
    }
}
