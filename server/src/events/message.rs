use crate::{
    types::{ChatMessage, UserDB, UserID},
    ws_handler::UserEvent,
    SqliteDB,
};
use rusqlite::params;

// Implement the `UserEvent` trait for each event type
#[async_trait]
impl UserEvent for ChatMessage {
    type State = (SqliteDB, UserDB);
    async fn handle(self, user_id: &UserID, (db, user_db): &Self::State) {
        db.send_msg(self, user_db).await;
    }
}
