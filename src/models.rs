use rocket::serde::{Deserialize, Serialize};
use memra::model;

#[model]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct User {
    pub username: String,
    pub real_name: Option<String>,
    pub verified: Option<bool>,
}

#[model]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Credentials {
    pub user_id: i32,
    pub email: String,
    pub password: String,
}
