use rocket::futures::TryFutureExt;
use rocket_db_pools::Connection;
use rocket_db_pools::sqlx::{self, Row};
use rocket::response::status::Created;
use rocket::serde::{Deserialize, Serialize, json::Json};
use super::{Db, Result};
use super::auth::AuthenticatedUser;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct User {
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    pub username: String,
    pub real_name: Option<String>,
    pub verified: Option<bool>,
}

impl User {
    fn new(r: sqlx::postgres::PgRow) -> Self {
        Self {
            id: Some(r.get("id")),
            username: r.get("username"),
            real_name: r.get("real_name"),
            verified: r.get("verified"),
        }
    }
}

#[get("/<id>")]
pub async fn read(mut db: Connection<Db>, id: i32) -> Option<Json<User>> {
    sqlx::query("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_one(&mut *db)
        .map_ok(|r| Json(User::new(r)))
        .await.ok()
}

#[post("/", data = "<user>")]
pub async fn create(mut db: Connection<Db>, user: Json<User>) -> Result<Created<Json<User>>> {
    sqlx::query("INSERT INTO users (username, real_name, verified) VALUES ($1, $2, $3)")
        .bind(&user.username).bind(&user.real_name).bind(false)
        .execute(&mut *db)
        .await?;

    Ok(Created::new("/user").body(user))
}

#[delete("/")]
async fn delete(mut db: Connection<Db>, user: AuthenticatedUser) -> Result<Option<()>> {
    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user.data.id)
        .execute(&mut *db)
        .await?;

    Ok((result.rows_affected() == 1).then(|| ()))
}
