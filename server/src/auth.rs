use jsonwebtoken::Algorithm;
use rocket::{form::Form, http::Status, request::{FromRequest, Outcome}, Request};
use crate::{types::{InactiveUser, User, UserStatus}, UserDB, UserID};
use rocket::State;

const HASH_COST: u32 = 12;

#[derive(FromForm)]
pub struct Credentials<'a> {
    name: &'a str,
    password: &'a str,
}


#[derive(serde::Deserialize, serde::Serialize)]
pub struct JWT {
    pub name: UserID,
    pub exp: u64,
}

#[async_trait]
impl<'r> FromRequest<'r> for JWT {
    type Error = &'static str;
    async fn from_request(r: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let Some(token) = r.headers().get_one("authorization") else {
            return Outcome::Error((Status::Unauthorized, "No Authorization Header"));
        };
        let Ok(secret) = std::env::var("JWT_SECRET") else {
            return Outcome::Error((Status::InternalServerError, "JWT_SECRET not set"));
        };
        let Some(token) = decode_jwt(token, secret) else {
            return Outcome::Error((Status::Unauthorized, "Invalid Token"));
        };
        if token.exp < jsonwebtoken::get_current_timestamp() {
            return Outcome::Error((Status::Unauthorized, "Token Expired"));
        }
        Outcome::Success(token)
    }
}

#[post("/login", data = "<login>")]
pub async fn login_user(login: Form<Credentials<'_>>, db: &State<UserDB>) -> Option<String> {
    let Credentials { name, password } = login.into_inner();
    let Ok(secret) = std::env::var("JWT_SECRET") else {
        log::error!("JWT_SECRET not set");
        return None;
    };    
    let id = UserID(name.into());
    let db = db.read().await;
    let user = db.get(&id)?;
    bcrypt::verify(password, &user.password).unwrap_or(false).then(|| encode_jwt(name, secret))
}    

#[get("/checkuser/<name>")]
pub async fn check_user(name: String, db: &State<UserDB>) -> &'static str {
    if db.read().await.contains_key(&UserID(name)) {
        "found"
    } else {  
        "not found"
    }    
}    

#[post("/createuser", data = "<form>")]
pub async fn create_user(form: Form<Credentials<'_>>, db: &State<UserDB>) -> (Status, Option<String>) {
    let Credentials { name, password } = form.into_inner();
    let mut db = db.write().await;
    let id = UserID(name.into());
    if db.contains_key(&id) {
        return (Status::Conflict, None);
    }    
    let Ok(hashed) = bcrypt::hash(password, HASH_COST) else {
        return (Status::InternalServerError, None);
    };    
    let Ok(secret) = std::env::var("JWT_SECRET") else {
        return (Status::InternalServerError, None);
    };    
    // Insert the user into the database
    db.insert(
        id.clone(),
        User {
            name: id,
            password: hashed,
            status: UserStatus::Inactive(InactiveUser {
                messages: Vec::new(),
            }),    
        },    
    );    
    (Status::Ok, Some(encode_jwt(name, secret)))
}    
pub fn encode_jwt<T: AsRef<[u8]>>(name: &str, secret: T) -> String {
    let exp = jsonwebtoken::get_current_timestamp() + 3600;
    let header = jsonwebtoken::Header::new(Algorithm::HS512);
    jsonwebtoken::encode(
        &header,
        &JWT {
            name: name.to_string().into(),
            exp,
        },
        &jsonwebtoken::EncodingKey::from_secret(secret.as_ref()),
    )
    .expect("Shouldnt fail?")
}

pub fn decode_jwt<T: AsRef<[u8]>>(token: &str, secret: T) -> Option<JWT> {
    let validation = jsonwebtoken::Validation::new(Algorithm::HS512);
    let token = jsonwebtoken::decode::<JWT>(
        token.trim(),
        &jsonwebtoken::DecodingKey::from_secret(secret.as_ref()),
        &validation,
    )
    .map_err(|e| dbg!(e))
    .ok()?;
    Some(token.claims)
}
