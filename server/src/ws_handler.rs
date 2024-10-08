
use serde::{Deserialize, Serialize};

use crate::{
    types::{ChatMessage, UserDB, UserID},
    SqliteDB,
};

impl WebSocketHandler<Event> for (SqliteDB, UserDB) {}

#[async_trait]
pub trait WebSocketHandler<T: UserEvent<State = Self> + for<'a> Deserialize<'a> + Send> {
    // Method to process incoming messages and dispatch to event handlers
    async fn handle_message(&self, user: &UserID, msg: ws::Message) {
        if msg.is_text() || msg.is_binary() {
            let bytes = msg.into_data();
            if let Ok(event) = serde_json::from_slice::<T>(&bytes) {
                event.handle(user, self).await;
            } else {
                log::error!(
                    "Failed to parse message: {:?}",
                    String::from_utf8_lossy(&bytes)
                );
            }
        }
    }
}

// Trait defining the interface for user events
#[async_trait]
pub trait UserEvent {
    type State;
    async fn handle(self, user: &UserID, state: &Self::State);
}

// Macro to simplify event handler registration
macro_rules! impl_user_event {
  ($($event_name:ident:$event:ty),+; $($unit_event_name:ident:$unit_event:expr),+; $state:ty) => {
      #[derive(Serialize, Deserialize)]
      #[serde(tag = "action", content = "data")]
      pub enum Event {
          $($event_name($event),)+
          $($unit_event_name),+
      }
      #[async_trait]
      impl UserEvent for Event {
          type State = $state;
          async fn handle(self, user_id: &UserID, state: &Self::State) {
              match self {
                  $(Event::$event_name(event) => event.handle(user_id, state).await,)+
                  $(Event::$unit_event_name => $unit_event.handle(user_id, state).await,)+
              }
          }
      }
  }
}
use crate::events::*;
// Register the event handlers using the macro
impl_user_event!(Message:ChatMessage, Egress:RoomEgress, TimingAction:TimingAction; CheckTime:CheckTime; (SqliteDB, UserDB));
