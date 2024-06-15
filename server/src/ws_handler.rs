use std::{
    ops::Deref,
    sync::{Arc, RwLock},
};

use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::{types::UserDB, SqliteDB};

impl WebSocketHandler<Event> for (SqliteDB, UserDB) {}

#[async_trait]
trait WebSocketHandler<T: UserEvent<State = Self> + for<'a> Deserialize<'a> + Send> {
    // Method to process incoming messages and dispatch to event handlers
    async fn handle_message(&self, msg: ws::Message) {
        if msg.is_text() || msg.is_binary() {
            if let Ok(event) = serde_json::from_slice::<T>(msg.into_data().as_slice()) {
                event.handle(self).await;
            }
        }
    }
}

// Trait defining the interface for user events
#[async_trait]
trait UserEvent {
    type State;
    async fn handle(self, state: &Self::State);
}

// Macro to simplify event handler registration
macro_rules! impl_user_event {
  ($($event_name:ident:$event:ty),+; $state:ty) => {
      #[derive(Serialize, Deserialize)]
      #[serde(tag = "action", content = "data")]
      enum Event {
          $($event_name($event)),+
      }
      #[async_trait]
      impl UserEvent for Event {
          type State = $state;
          async fn handle(self, state: &Self::State) {
              match self {
                  $(Event::$event_name(event) => event.handle(state).await),+
              }
          }
      }
  }
}

// Example event types
#[derive(Serialize, Deserialize)]
struct MessageEvent {
    room_id: String,
    content: String,
}

#[derive(Serialize, Deserialize)]
struct LeaveEvent {
    room_id: String,
}

#[derive(Serialize, Deserialize)]
struct JoinEvent {
    room_id: String,
    user_id: String,
}

// Implement the `UserEvent` trait for each event type
#[async_trait]
impl UserEvent for MessageEvent {
    type State = (SqliteDB, UserDB);
    async fn handle(self, (db, user_db): &Self::State) {
        let s2 = (self.room_id.clone(), self.content.clone());
        let added = db
            .run(move |d| {
                d.execute(
                    "INSERT INTO messages (room_id, content) VALUES (?, ?)",
                    params![self.room_id, self.content],
                )
            })
            .await
            .is_ok();
        if added {
            let users: Vec<String> = db
                .run(move |d| {
                        d.prepare("select user_id from chatrooms_users where chatroom_id=?")?
                            .query_map(params![s2.0, s2.1], |r| r.get(0))?
                            .collect::<Result<Vec<_>, _>>()
                })
                .await
                .unwrap();
        }
    }
}

#[async_trait]
impl UserEvent for LeaveEvent {
    type State = (SqliteDB, UserDB);
    async fn handle(self, state: &Self::State) {
        // Handle the leave event
        // Example: Remove the user from the room
        // ...
    }
}

#[async_trait]
impl UserEvent for JoinEvent {
    type State = (SqliteDB, UserDB);
    async fn handle(self, state: &Self::State) {
        // Handle the join event
        // Example: Add the user to the room
        // ...
    }
}

// Register the event handlers using the macro
impl_user_event!(Message:MessageEvent, Leave:LeaveEvent, Join:JoinEvent; (SqliteDB, UserDB));
