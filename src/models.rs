use rocket::serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;
use memra::*;

#[model]
#[derive(Debug, Clone, Deserialize, Serialize)]
// #[derive(Read)]
#[serde(crate = "rocket::serde")]
pub struct User {
    pub username: String,
    pub email: String,
    pub real_name: Option<String>,
    pub verified: Option<bool>,
}

#[model(table = "credentials")]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[derive(Related)]
#[serde(crate = "rocket::serde")]
pub struct Credentials {
    #[foreign(type = "User")]
    pub user_id: i32,
    pub password: String,
}

#[model]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[derive(Related)]
#[serde(crate = "rocket::serde")]
pub struct Course {
    #[foreign(type = "User")]
    pub user_id: i32,
    pub visibility: Option<bool>,
    pub name: String,
    pub image: Vec<u8>,
}

#[model]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[derive(Related)]
// #[derive(Related, CreateWithUser, ReadIfVisible, UpdateIfOwner, DeleteIfOwner)]
#[serde(crate = "rocket::serde")]
pub struct Deck {
    #[foreign(type = "User")]
    pub user_id: i32,
    pub visibility: Option<bool>,
    pub name: String,
    pub image: Vec<u8>,
}

#[model]
#[derive(Related, Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Card {
    #[foreign(type = "User")]
    pub user_id: i32,
    #[foreign(type = "Deck")]
    pub deck_id: i32,
    pub front: Vec<u8>,
    pub back: Vec<u8>,
}

#[model]
#[derive(Related, Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct History {
    #[foreign(type = "User")]
    pub user_id: i32,
    #[foreign(type = "Card")]
    pub card_id: i32,
    pub ts: NaiveDateTime,
    pub num_confident: i32,
    pub num_correct: i32,
    pub num_wrong: i32,
}

#[model]
#[derive(Related, Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Settings {
    #[foreign(type = "User")]
    pub user_id: i32,
    pub public_profile: Option<bool>,
    pub avatar: Vec<u8>,
}

#[model]
#[derive(Related, Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Notification {
    #[foreign(type = "User")]
    pub user_id: i32,
    pub ts: NaiveDateTime,
    pub message: String,
    pub icon: Vec<u8>,
}

#[model]
#[derive(Related, Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Addon {
    #[foreign(type = "User")]
    pub user_id: i32,
    pub visibility: Option<bool>,
    pub name: String,
    pub description: String,
    pub data: Vec<u8>,
}

#[model]
#[derive(Related, Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct CourseDeck {
    #[foreign(type = "Course")]
    pub course_id: i32,
    #[foreign(type = "Deck")]
    pub deck_id: i32,
}

#[model]
#[derive(Related, Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Followers {
    #[foreign(type = "User", collect = "followers")]
    pub follower_id: i32,
    #[foreign(type = "User", collect = "following")]
    pub following_id: i32,
}

#[model]
#[derive(Related, Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct CourseSubscription {
    #[foreign(type = "User")]
    pub user_id: i32,
    #[foreign(type = "Course")]
    pub course_id: i32,
}

#[model]
#[derive(Related, Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct DeckSubscription {
    #[foreign(type = "User")]
    pub user_id: i32,
    #[foreign(type = "Course")]
    pub deck_id: i32,
}
