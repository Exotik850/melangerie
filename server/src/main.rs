mod auth;
mod chat;
mod cors;
mod log;
mod timing;
use timing::TimeState;
#[cfg(test)]
mod test;
mod types;
#[macro_use]
extern crate rocket;
const FILE_PATH: &str = "./public";
type LockedMap<K, V> = RwLock<HashMap<K, V>>;
type LockedSet<T> = RwLock<HashSet<T>>;
use std::collections::{HashMap, HashSet};
type ChatroomsDB = LockedMap<ChatRoomID, Vec<UserID>>;
// Map Users to their sender which is sending to their active websocket connection
// and a Vec of messages that have been sent to them while they were offline
type UserDB = LockedMap<UserID, User>;
use log::Log;
use rocket::{fs::NamedFile, http::Status, serde::json::Json, tokio::sync::RwLock, State};
use serde::Deserialize;
use std::path::PathBuf;
use types::{ChatRoomID, User, UserID};

use rocket::tokio::runtime::{Handle, Runtime};
pub fn get_runtime_handle() -> (Handle, Option<Runtime>) {
    match Handle::try_current() {
        Ok(h) => (h, None),
        Err(_) => {
            let rt = Runtime::new().expect("Failed to create runtime");
            (rt.handle().clone(), Some(rt))
        }
    }
}


use std::future::Future;
pub fn run_or_block<F, T>(f: F) -> T
where
    F: Future<Output = T>,
{
    if let Ok(handle) = Handle::try_current() {
        handle.block_on(f)
    } else {
        futures::executor::block_on(f)
    }
}

#[get("/<file..>")]
async fn file_server(file: PathBuf) -> std::io::Result<NamedFile> {
    let path = if file.to_str().map_or(false, str::is_empty) {
        PathBuf::from("index.html")
    } else {
        file
    };
    NamedFile::open(PathBuf::from(FILE_PATH).join(path)).await
}

#[derive(Deserialize)]
struct ReportInfo {
    name: String,
    issue: String,
}

#[post("/report", data = "<info>")]
async fn report(info: Json<ReportInfo>, log: &State<Log>) -> Status {
    match log
        .write(format!("Report: {} - {}", info.name, info.issue))
        .await
    {
        Ok(_) => Status::Ok,
        Err(_) => Status::InternalServerError,
    }
}

async fn periodic_flush(log: Log) {
    loop {
        rocket::tokio::time::sleep(rocket::tokio::time::Duration::from_secs(5)).await;
        log.flush().await.unwrap();
    }
}

#[launch]
async fn rocket() -> _ {
    dotenvy::dotenv().ok();
    let log = Log::new().unwrap();
    rocket::tokio::spawn(periodic_flush(log.clone()));
    rocket::build()
        .manage(UserDB::default())
        .manage(ChatroomsDB::default())
        .manage(log)
        .manage(TimeState::new())
        .attach(cors::Cors)
        .mount(
            "/chat",
            routes![
                chat::connect,
                chat::create_room,
                chat::add_user_to_room,
                chat::send_message,
                chat::list_rooms
            ],
        )
        .mount(
            "/auth",
            routes![auth::create_user, auth::login_user, auth::check_user],
        )
        .mount("/", routes![file_server, report, timing::get_time])

    // .register("/", catchers![echo_catcher])
}
