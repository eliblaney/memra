#[macro_use]
extern crate rocket;
extern crate rocket_cors;

mod models;
mod auth;
mod user;

use rocket::fs::{FileServer, NamedFile};
use rocket::http::Method;
use rocket_cors::{AllowedHeaders, AllowedOrigins, Cors, CorsOptions};
use rocket_db_pools::{sqlx, Database};
use memra::router;

type Result<T, E = rocket::response::Debug<sqlx::Error>> = std::result::Result<T, E>;

#[derive(Database)]
#[database("main")]
pub struct Db(sqlx::PgPool);

#[router(models, Course, Deck, Card, History, Settings, Notification, Addon)]
pub struct MemraRouter;

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

#[get("/")]
async fn index() -> Option<NamedFile> {
    NamedFile::open("app/build/index.html").await.ok()
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(Db::init())
        .attach(make_cors())
        .attach(MemraRouter)
        .mount("/public", FileServer::from("app/build"))
        .mount("/api/users", routes![user::read_user, user::delete_user, user::login, user::register, user::change_password])
        .mount("/", routes![index])
}
