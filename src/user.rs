use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};

use rocket::http::Status;
use rocket::futures::TryFutureExt;
use rocket_db_pools::Connection;
use rocket_db_pools::sqlx::{self, Row};
use rocket::response::status::Created;
use rocket::serde::{Deserialize, Serialize, json::Json};
use super::{Db, Result};
use super::auth::AuthenticatedUser;
use rocket::response::status::Custom;
use super::auth;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct User {
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    pub username: String,
    pub real_name: Option<String>,
    pub verified: Option<bool>,
}

impl From<sqlx::postgres::PgRow> for User {
    fn from(r: sqlx::postgres::PgRow) -> Self {
        Self {
            id: Some(r.get("id")),
            username: r.get("username"),
            real_name: r.get("real_name"),
            verified: r.get("verified"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Registration {
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    pub real_name: Option<String>,
    pub username: String,
    pub email: String,
    pub password: String,
}

impl From<sqlx::postgres::PgRow> for Registration {
    fn from(r: sqlx::postgres::PgRow) -> Self {
        Self {
            id: Some(r.get("id")),
            real_name: r.get("real_name"),
            username: r.get("username"),
            email: r.get("email"),
            password: r.get("password"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Credentials {
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    pub user_id: i32,
    pub email: String,
    pub password: String,
}

impl From<sqlx::postgres::PgRow> for Credentials {
    fn from(r: sqlx::postgres::PgRow) -> Self {
        Self {
            id: Some(r.get("id")),
            user_id: r.get("user_id"),
            email: r.get("email"),
            password: r.get("password"),
        }
    }
}

#[get("/<id>")]
pub async fn read(mut db: Connection<Db>, id: i32) -> Option<Json<User>> {
    sqlx::query("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_one(&mut *db)
        .map_ok(|r| Json(User::from(r)))
        .await.ok()
}

#[delete("/")]
pub async fn delete(mut db: Connection<Db>, user: AuthenticatedUser) -> Result<Option<()>> {
    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user.data.id)
        .execute(&mut *db)
        .await?;

    Ok((result.rows_affected() == 1).then(|| ()))
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
pub async fn login(mut db: Connection<Db>, credentials: Json<LoginRequest>) -> Result<Json<JwtToken>, Custom<String>> {
    let user: Option<User> = sqlx::query("SELECT id FROM users WHERE username = $1")
        .bind(&credentials.username)
        .fetch_one(&mut *db)
        .map_ok(|r| User::from(r))
        .await.ok();

    if user.is_none() {
        return Err(Custom(
            Status::Unauthorized,
            "Invalid login credentials.".to_string(),
        ));
    }

    let user = user.unwrap();

    let creds: Option<Credentials> = sqlx::query("SELECT * FROM credentials WHERE user_id = $1")
        .bind(&user.id)
        .fetch_one(&mut *db)
        .map_ok(|r| Credentials::from(r))
        .await.ok();

    if creds.is_none() {
        return Err(Custom(
            Status::InternalServerError,
            "Could not verify login credentials.".to_string(),
        ));
    }

    let creds = creds.unwrap();
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

#[post("/register", data = "<registration>")]
pub async fn register(mut db: Connection<Db>, registration: Json<Registration>) -> Result<Created<Json<JwtToken>>, Custom<String>> {
    let user: Option<User> = sqlx::query("INSERT INTO users (username, real_name, verified) VALUES ($1, $2, $3) RETURNING *")
        .bind(&registration.username).bind(&registration.real_name).bind(false)
        .fetch_one(&mut *db)
        .map_ok(|r| User::from(r))
        .await.ok();

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

    let response = sqlx::query("INSERT INTO credentials (user_id, email, password) VALUES ($1, $2)")
        .bind(&user.id).bind(&registration.email).bind(&password_hash.unwrap().to_string())
        .execute(&mut *db)
        .await;

    if response.is_err() {
        return Err(Custom(
            Status::InternalServerError,
            "Could not register user credentials. Please try again.".to_string(),
        ));
    }

    let claim = auth::AuthenticatedUser::from_user(user);

    Ok(Created::new("/user").body(Json(JwtToken { token: claim.to_token()? })))
}
