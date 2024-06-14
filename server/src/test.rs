use std::time::Duration;

use crate::{run_or_block, timing::TimeState, types::UserID};

#[test]
fn what_is_the_json() {
    let ts = TimeState::new();
    let uids = vec![
        UserID("Jimby".into()),
        UserID("Johny".into()),
        UserID("Hundo".into()),
    ];
    for uid in uids.clone() {
        run_or_block(ts.start(uid, None));
    }
    std::thread::sleep(Duration::from_secs(1));
    for uid in uids.iter().take(2) {
        run_or_block(ts.stop(uid, None));
    }
    // let json = serde_json::to_string_pretty(&ts).unwrap();
    // println!("{}", json);
}
