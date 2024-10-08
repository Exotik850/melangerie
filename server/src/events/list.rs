use crate::{
    types::{ServerAction, UserDB, UserID, UserStatus},
    ws_handler::UserEvent,
    SqliteDB,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ListUsers;

#[async_trait]
impl UserEvent for ListUsers {
    type State = (SqliteDB, UserDB);

    async fn handle(self, user_id: &UserID, state: &Self::State) {
        let (sqldb, udb) = state;
        let Some(user) = udb.get_user(user_id).await else {
            println!("User not found: {user_id}");
            return;
        };
        let UserStatus::Active(tx) = user.status else {
            println!("User not active: {user_id}");
            return;
        };
        let res = sqldb
            .run(move |db| {
                db.prepare("SELECT user_id FROM users")?
                    .query_map([], |row| row.get(0))?
                    .collect::<Result<Vec<UserID>, _>>()
            })
            .await;
        let Ok(users) = res else {
            println!("Failed to list users: {}", res.unwrap_err());
            return;
        };

        if let Err(e) = tx.send(ServerAction::List(users)) {
            log::error!("Failed to send list of users: {e:?}");
        };
    }
}
