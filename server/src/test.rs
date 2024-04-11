use crate::types::{Action, ChatMessage, ChatRoomID, UserID};

#[test]
fn what_is_the_json() {
    let action = Action::Message(ChatMessage {
        sender: UserID("What".into()),
        room: ChatRoomID("Is".into()),
        content: "The JSON".into(),
        timestamp: 0,
    });
    let json = serde_json::to_string(&action).unwrap();
    println!("{}", json);

    let action = Action::Add((ChatRoomID("The".into()), UserID("JSON".into())));
    let json = serde_json::to_string(&action).unwrap();
    println!("{}", json);
}
