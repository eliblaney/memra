#[macro_use]
extern crate rocket;
extern crate rocket_cors;

mod auth;

use rocket::fs::{FileServer, NamedFile};
use rocket::response::status::Custom;
use rocket::serde::json::Json;
use rocket::http::Method;
use rocket_cors::{AllowedHeaders, AllowedOrigins, Cors, CorsOptions};
use serde::{Deserialize, Serialize};

use auth::{User, LoginRequest};

fn make_cors() -> Cors {
    let allowed_origins = AllowedOrigins::some_exact(&[
        "http://localhost:3000",
        "http://127.0.0.1:3000",
        "http://localhost:8000",
        "http://0.0.0.0:8000",
        "https://memra.app",
        "http://memra.app",
        "https://memra.herokuapp.com",
        "http://memra.herokuapp.com",
    ]);

    CorsOptions {
        allowed_origins,
        allowed_methods: vec![Method::Get].into_iter().map(From::from).collect(), 
        allowed_headers: AllowedHeaders::some(&[
            "Authorization", "Accept", "Access-Control-Allow-Origin",
        ]),
        allow_credentials: true,
        ..Default::default()
    }.to_cors().expect("Error while building CORS")
}

#[derive(Serialize, Deserialize, Debug)]
struct Item {
    id: usize,
    name: String
}

#[get("/")]
async fn index() -> Option<NamedFile> {
    NamedFile::open("app/build/index.html").await.ok()
}

#[get("/item/<id>")]
fn item(id: usize) -> Json<Item> {
    Json(Item { id, name: "Example item".into() })
}

#[derive(Serialize)]
struct PrivateResponse {
    message: String,
    user: String,
}

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

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/public", FileServer::from("app/build"))
        .mount("/api", routes![item, private, login])
        .mount("/", routes![index]).attach(make_cors())
}
