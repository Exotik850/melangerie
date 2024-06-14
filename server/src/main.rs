mod auth;
mod chat;
mod cors;
mod log;
#[cfg(test)]
mod test;
mod timing;
mod types;
#[macro_use]
extern crate rocket;
const FILE_PATH: &str = "./public";
type LockedSet<T> = RwLock<HashSet<T>>;
use std::collections::HashSet;

// Map Users to their sender which is sending to their active websocket connection
// and a Vec of messages that have been sent to them while they were offline
use log::Log;
use rocket::tokio;
use rocket::{
    fs::NamedFile,
    http::Status,
    serde::json::Json,
    tokio::{io::AsyncWriteExt, sync::RwLock},
    State,
};
use rocket_sync_db_pools::{database, rusqlite::Connection as SqliteConnection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use types::{UserDB, UserID};

#[database("sqlite_db")]
struct SqliteDB(SqliteConnection);

use tokio::runtime::{Handle, Runtime};
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
    let file_str = file.to_str();
    dbg!(&file_str);
    let path = if file_str.map_or(false, str::is_empty) {
        PathBuf::from("index.html")
    } else if file.components().count() == 1 {
        PathBuf::from(format!("{}.html", file_str.unwrap()))
    } else {
        file
    };
    NamedFile::open(PathBuf::from(FILE_PATH).join(path)).await
}

#[derive(Deserialize, Serialize)]
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
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        log.flush().await.unwrap();
    }
}

#[launch]
async fn rocket() -> _ {
    dotenvy::dotenv().ok();
    let log = Log::new().unwrap();

    tokio::spawn(periodic_flush(log.clone()));
    // tokio::spawn(timing_flush(time_db.clone()));
    rocket::build()
        // .manage(server_state)
        .manage(log)
        .manage(UserDB::default())
        .attach(cors::Cors)
        .attach(SqliteDB::fairing())
        // .attach(AdHoc::on_shutdown("Save Dbs", |rocket| {
        //     Box::pin(async {
        //         let server_state: &ServerState = rocket.state().unwrap();
        //         server_state.to_file(".server").await.unwrap();
        //     })
        // }))
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
