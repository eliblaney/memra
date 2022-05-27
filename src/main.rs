#[macro_use]
extern crate rocket;

mod auth;

// use std::path::{Path, PathBuf};
use rocket::fs::{FileServer, NamedFile};
use rocket::response::status::Custom;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
// use shuttle_service::ShuttleRocket;
// use shuttle_service::{ShuttleRocket, SecretStore, error::CustomError};
// use sqlx::{Executor, FromRow, PgPool};
//
// use rocket_include_static_resources::{EtagIfNoneMatch, StaticContextManager, StaticResponse};
// use rocket::State;

use auth::{User, LoginRequest};

#[derive(Serialize, Deserialize, Debug)]
struct Item {
    id: usize,
    name: String
}

/*
#[get("/")]
async fn index() -> Option<NamedFile> {
    let page_directory_path = format!("{}/static", env!("CARGO_MANIFEST_DIR"));
    NamedFile::open(Path::new(&page_directory_path).join("index.html")).await.ok()
}

#[get("/")]
fn index(sr: &State<StaticContextManager>, etag: EtagIfNoneMatch) -> StaticResponse {
    sr.build(&etag, "html")
}
*/

#[get("/")]
async fn index() -> Option<NamedFile> {
    NamedFile::open("app/build/index.html").await.ok()
}

/*
#[get("/public/<file..>")]
async fn public(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("public").join(file)).await.ok()
}
*/

#[get("/item/<id>")]
fn item(id: usize) -> Json<Item> {
    Json(Item { id, name: "Example item".into() })
}

#[derive(Serialize)]
struct PrivateResponse {
    message: String,
    user: String,
}

// More details on Rocket request guards can be found here
// https://rocket.rs/v0.5-rc/guide/requests/#request-guards
#[get("/private")]
fn private(user: User) -> Json<PrivateResponse> {
    match user {
        User::Authenticated(u) => Json(PrivateResponse {
            message: "Authenticated User.".to_string(),
                user: u.name,
        }),
        User::Guest => 
            Json(PrivateResponse {
                message: "Unauthenticated.".to_string(),
                user: "None".to_string()
            })
    }
}

#[derive(Serialize)]
pub(crate) struct LoginResponse {
    token: String,
}

/// Tries to authenticate a user. Successful authentications get a JWT
#[post("/login", data = "<credentials>")]
fn login(credentials: Json<LoginRequest>) -> Result<Json<LoginResponse>, Custom<String>> {
    let response = auth::login(credentials)?;
    Ok(Json(LoginResponse { token: response }))
}

// struct State {
    // pool: PgPool
// }

#[launch]
fn rocket() -> _ {
    // pool.execute(include_str!("../schema.sql")).await
        // .map_err(CustomError::new)?;

    // let state = State { pool };

    rocket::build()
        .mount("/public", FileServer::from("app/build"))
        .mount("/api", routes![item, private, login])
        .mount("/", routes![index])
        // .manage(state)
}
