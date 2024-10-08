use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::{
    types::{ServerAction, UserDB, UserID, UserStatus},
    ws_handler::UserEvent,
    SqliteDB,
};

#[derive(Serialize, Deserialize)]
pub enum ClockType {
    TimeIn,
    TimeOut,
}

#[derive(Serialize, Deserialize)]
pub struct TimingAction {
    action: ClockType,
    note: Option<String>,
}

pub struct CheckTime;

#[async_trait]
impl UserEvent for CheckTime {
    type State = (SqliteDB, UserDB);
    async fn handle(self, user_id: &UserID, state: &Self::State) {
        let (db, users) = state;
        let id = user_id.clone();
        let timed_in = db
            .run(move |d| {
                d.query_row(
                    "select clocked_in from timesheets where user_id = ?",
                    params![id.0],
                    |r| r.get(0),
                )
                .inspect_err(|e| log::error!("Failed to check time: {}", e))
                .unwrap_or(false)
            })
            .await;
        let Some(user) = users.get_user(user_id).await else {
            log::error!("User not found: {user_id}");
            return;
        };
        let UserStatus::Active(tx) = user.status else {
            log::error!("User not active: {user_id}");
            return;
        };
        if let Err(e) = tx.send(ServerAction::TimedIn(timed_in)) {
            log::error!("Failed to send time status: {e:?}");
        };
    }
}

#[async_trait]
impl UserEvent for TimingAction {
    type State = (SqliteDB, UserDB);
    async fn handle(self, user_id: &UserID, state: &Self::State) {
        let (db, users) = state;
        let id = user_id.clone();
        match self.action {
            ClockType::TimeIn => {
                log::info!("{} Timed in: {:?}", id, self.note);
                if let Err(e) = db
                    .run(move |d| {
                        let statement = format!(
                            "BEGIN TRANSACTION; \
                INSERT INTO time_entries (timesheet_id, start_time, start_note) \
                SELECT timesheet_id, CURRENT_TIMESTAMP, '{}' \
                FROM timesheets WHERE user_id = '{id}' AND clocked_in = 0;
                UPDATE timesheets SET clocked_in = 1 WHERE user_id = '{id}' AND clocked_in = 0;
                COMMIT;",
                            self.note.as_deref().unwrap_or("NULL")
                        );
                        d.execute_batch(&statement)
                    })
                    .await
                {
                    log::error!("Failed to insert time entry: {}", e);
                };
            }
            ClockType::TimeOut => {
                log::info!("{} Timed out: {:?}", id, self.note);
                if let Err(e) = db
                    .run(move |d| {
                        let statement = format!(
                            "BEGIN TRANSACTION; \
                UPDATE time_entries \
                SET end_time = CURRENT_TIMESTAMP, end_note = '{}' \
                WHERE time_entry_id = (
                  SELECT current_id
                  FROM timesheets
                  WHERE user_id = '{id}' AND clocked_in = 1
                );
                UPDATE timesheets
                SET clocked_in = 0, current_id = NULL
                WHERE user_id = '{id}' AND clocked_in = 1;
                COMMIT;",
                            self.note.as_deref().unwrap_or("NULL")
                        );
                        d.execute_batch(&statement)
                    })
                    .await
                {
                    log::error!("Failed to insert time entry: {}", e);
                };
            }
        };
    }
}
