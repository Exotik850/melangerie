mod auth;
mod chat;
mod cors;
mod events;
mod logger;
#[cfg(test)]
mod test;
mod timing;
mod types;
mod ws_handler;
#[macro_use]
extern crate rocket;
const FILE_PATH: &str = "./public";
type LockedSet<T> = RwLock<HashSet<T>>;
use std::collections::HashSet;

use logger::Log;
// Map Users to their sender which is sending to their active websocket connection
// and a Vec of messages that have been sent to them while they were offline
use rocket::fairing::AdHoc;
use rocket::tokio;
use rocket::{
    fs::NamedFile,
    http::Status,
    serde::json::Json,
    tokio::{io::AsyncWriteExt, sync::RwLock},
    State,
};
use rocket_sync_db_pools::{database, rusqlite::Connection as SqliteConnection};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use types::{ChatMessage, ChatRoomID, User, UserDB, UserID, UserStatus};

#[database("sqlite_db")]
pub struct SqliteDB(SqliteConnection);

impl SqliteDB {
    async fn send_msg(&self, msg: ChatMessage, user_db: &UserDB) {
        let chatroom_id = msg.room.clone();
        let users: Vec<_> = match self
            .run(move |d| {
                d.prepare("select user_id from chatroom_users where chatroom_id = ?")?
                    .query_map(params![chatroom_id.0], |r| {
                        r.get(0).map(crate::types::UserID)
                    })?
                    .collect::<Result<Vec<_>, _>>()
            })
            .await
        {
            Ok(users) => users,
            Err(e) => {
                log::error!("Failed to get users from chatroom: {}", e);
                return;
            }
        };
        user_db.write_to(msg.clone(), &users).await;
        if let Err(e) = self.run(move |d| {
          d.execute(
              "INSERT INTO messages (user_id, chatroom_id, message, created_at) VALUES (?, ?, ?, ?)",
              params![msg.sender.0, msg.room.0, msg.content, msg.timestamp],
          )
      })
      .await
  {
      log::error!("Failed to insert message into database: {}", e);
  };
    }
}

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
    let udb = UserDB::default();
    rocket::build()
        // .manage(server_state)
        .manage(log)
        .manage(udb.clone())
        .attach(cors::Cors)
        .attach(SqliteDB::fairing())
        .attach(AdHoc::on_liftoff("Load DB", move |rocket| {
            Box::pin(async move {
                let Some(db) = SqliteDB::get_one(rocket).await else {
                    panic!("Failed to get database");
                };
                db.run(move |d| d.execute_batch(include_str!("../migrations/up.sql")))
                    .await
                    .unwrap();
                let users: Vec<(UserID, User)> = db
                    .run(move |d| {
                        d.prepare("SELECT user_id, password FROM users")
                            .unwrap()
                            .query_map([], |row| {
                                let id: UserID = row.get::<_, String>(0).unwrap().into();
                                let pass: String = row.get(1).unwrap();
                                Ok((
                                    id.clone(),
                                    User {
                                        name: id,
                                        password: pass,
                                        status: UserStatus::Inactive,
                                    },
                                ))
                            })
                            .unwrap()
                            .map(Result::unwrap)
                            .collect::<Vec<_>>()
                    })
                    .await;
                let mut udb = udb.write().await;
                udb.extend(users.into_iter());
            })
        }))
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
