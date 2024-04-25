use crate::types::{ChatMessage, ChatRoomID, ServerAction, UserAction, UserID};

#[test]
fn what_is_the_json() {
    let action = UserAction::Message(ChatMessage {
        sender: UserID("What".into()),
        room: ChatRoomID("Is".into()),
        content: "The JSON".into(),
        timestamp: 0,
    });
    let json = serde_json::to_string(&action).unwrap();
    println!("{}", json);

    // let action = ServerAction::Join((ChatRoomID("The".into()), UserID("JSON".into())));
    // let json = serde_json::to_string(&action).unwrap();
    // println!("{}", json);
}
