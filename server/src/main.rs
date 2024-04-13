mod cors;
mod chat;
mod auth;
mod types;
#[cfg(test)]
mod test;
#[macro_use]
extern crate rocket;
const FILE_PATH: &str = "./public";
type LockedMap<K, V> = RwLock<HashMap<K, V>>;
type LockedSet<T> = RwLock<HashSet<T>>;
use std::collections::{HashMap, HashSet};
type ChatroomsDB = LockedMap<String, Vec<UserID>>;
// Map Users to their sender which is sending to their active websocket connection
// and a Vec of messages that have been sent to them while they were offline
type UserDB = LockedMap<UserID, User>;
use rocket::{fs::NamedFile, tokio::sync::RwLock};
use std::path::PathBuf;
use types::{User, UserID};

#[get("/<file..>")]
async fn file_server(file: PathBuf) -> std::io::Result<NamedFile> {
    let path = if file.to_str().map_or(false, str::is_empty) {
        PathBuf::from("index.html")
    } else {
        file
    };
    NamedFile::open(PathBuf::from(FILE_PATH).join(path)).await
}




#[launch]
fn rocket() -> _ {
    dotenvy::dotenv().ok();
    rocket::build()
        .manage(UserDB::default())
        .manage(ChatroomsDB::default())
        .attach(cors::CORS)
        .mount(
            "/",
            routes![
                file_server,
                chat::create_room,
                chat::connect,
                chat::add_user_to_room,
                chat::send_message,
                auth::check_user,
                auth::create_user,
                auth::login_user,
            ],
        )
        
    // .register("/", catchers![echo_catcher])
}
