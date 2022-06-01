use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};

use rocket::http::Status;
use rocket_db_pools::Connection;
use rocket::serde::{Deserialize, Serialize, json::Json};
use super::{Db, Result};
use super::auth::AuthenticatedUser;
use rocket::response::status::{Created, Custom};
use super::auth;
use super::models::*;

#[get("/<id>")]
pub async fn read_user(db: rocket_db_pools::Connection<crate::Db>, user: crate::auth::AuthenticatedUser, id: i32) -> Option<rocket::serde::json::Json<User>> {
    let (m, _db) = <User>::read(id, db).await;
    if m.is_none() {
        return None;
    }
    let m = m.unwrap();
    if m.id.is_none() || (m.visibility.is_some() && m.visibility.unwrap() && m.id.unwrap() != user.id()) {
        return None;
    }
    Some(m.json())
}

#[delete("/")]
pub async fn delete_user(db: Connection<Db>, user: AuthenticatedUser) -> Result<Option<()>> {
    let (rows_affected, _db) = User::delete(user.id(), db).await;
    Ok((rows_affected? == 1).then(|| ()))
}

#[derive(Serialize)]
pub struct JwtToken {
    token: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

#[post("/login", data = "<credentials>")]
pub async fn login(db: Connection<Db>, credentials: Json<LoginRequest>) -> Result<Json<JwtToken>, Custom<String>> {
    let (user, db) = User::find_where("username", &credentials.username, db).await;

    if user.is_none() {
        return Err(Custom(
            Status::Unauthorized,
            "Invalid login credentials.".to_string(),
        ));
    }

    let user = user.unwrap();

    let (creds, _db) = user.find_credentials(db).await;

    if creds.len() == 0 {
        return Err(Custom(
            Status::InternalServerError,
            "Could not verify login credentials.".to_string(),
        ));
    }

    let creds = &creds[0];
    let password_hash = PasswordHash::new(creds.password.as_str());

    if password_hash.is_err() {
        return Err(Custom(
            Status::InternalServerError,
            "Could not verify login credentials.".to_string(),
        ));
    }

    let password_hash = password_hash.ok();

    if password_hash.is_none() {
        return Err(Custom(
            Status::InternalServerError,
            "Could not verify login credentials.".to_string(),
        ));
    }
    
    if Argon2::default().verify_password(credentials.password.as_bytes(), &password_hash.unwrap()).is_err() {
        return Err(Custom(
            Status::Unauthorized,
            "Invalid login credentials.".to_string(),
        ));
    }

    let claim = auth::AuthenticatedUser::from_user(user);

    Ok(Json(JwtToken { token: claim.to_token()? }))
}

#[derive(Deserialize)]
pub struct Registration {
    pub real_name: Option<String>,
    pub username: String,
    pub email: String,
    pub password: String,
}

#[post("/register", data = "<registration>")]
pub async fn register(db: Connection<Db>, registration: Json<Registration>) -> Result<Created<Json<JwtToken>>, Custom<String>> {
    let (user, db) = User::new(
        registration.username.to_string(),
        registration.email.to_string(),
        match &registration.real_name {
            None => None,
            Some(real_name) => Some(real_name.to_string())
        },
        Some(true),
        Some(false),
        chrono::Utc::now(),
        chrono::Utc::now(),
    ).save(db).await;

    if user.is_none() {
        return Err(Custom(
            Status::InternalServerError,
            "Could not register user. Please try again.".to_string(),
        ));
    }

    let user = user.unwrap();
    let salt = SaltString::generate(&mut OsRng);
    // Argon2 with default params (Argon2id v19)
    let argon2 = Argon2::default();
    // Hash password to PHC string ($argon2id$v=19$...)
    let password_hash = argon2.hash_password(&registration.password.as_bytes(), &salt);

    if password_hash.is_err() {
        return Err(Custom(
            Status::InternalServerError,
            "Could not create user credentials. Please try again.".to_string(),
        ));
    }

    let password_hash = password_hash.ok();

    if password_hash.is_none() {
        return Err(Custom(
            Status::InternalServerError,
            "Could not verify user credentials. Please try again.".to_string(),
        ));
    }

    let (creds, _db) = Credentials::new_from(&user, password_hash.unwrap().to_string())
        .unwrap().save(db).await;

    if creds.is_none() {
        return Err(Custom(
            Status::InternalServerError,
            "Could not register user credentials. Please try again.".to_string(),
        ));
    }

    let claim = auth::AuthenticatedUser::from_user(user);

    Ok(Created::new("/user").body(Json(JwtToken { token: claim.to_token()? })))
}
