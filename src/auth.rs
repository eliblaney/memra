use super::models;
use chrono::{Duration, Utc};
use jsonwebtoken::{
    decode, encode, errors::ErrorKind, DecodingKey, EncodingKey, Header, Validation,
};
use lazy_static::lazy_static;
use rocket::{
    http::Status,
    request::{FromRequest, Outcome},
    response::status::Custom
};
use serde::{Deserialize, Serialize};

const BEARER: &str = "Bearer ";
const AUTHORIZATION: &str = "Authorization";

/// Key used for symmetric token encoding
const SECRET: &str = "secret";

lazy_static! {
    /// Time before token expires (aka exp claim)
    static ref TOKEN_EXPIRATION: Duration = Duration::days(1);
}

// Used when decoding a token to `AuthenticatedUser`
#[derive(Debug)]
pub enum AuthenticationError {
    Missing,
    Decoding(String),
    Expired,
}

// Only the `exp` claim (field) is required. Consult the `jsonwebtoken` documentation for other claims that can be validated.
#[derive(Serialize, Deserialize)]
pub struct AuthenticatedUser {
    pub data: models::User,
    exp: usize,
}

impl Into<models::User> for AuthenticatedUser {
    fn into(self) -> models::User {
        self.data
    }
}

impl AuthenticatedUser {
    pub fn id(&self) -> i32 {
        self.data.id.unwrap()
    }

    pub(crate) fn from_user(user: models::User) -> Self {
        Self {
            data: user,
            exp: 0,
        }
    }

    /// Create a `AuthenticatedUser` from a 'Bearer <token>' value
    fn from_authorization(value: &str) -> Result<Self, AuthenticationError> {
        let token = value.strip_prefix(BEARER);

        if token.is_none() {
            return Err(AuthenticationError::Missing);
        }

        // Safe to unwrap as we just confirmed it is not none
        let token = token.unwrap();

        // Use `jsonwebtoken` to get the claims from a JWT
        // Consult the `jsonwebtoken` documentation for using other algorithms and validations (the default validation just checks the expiration claim)
        let token = decode::<AuthenticatedUser>(
            token,
            &DecodingKey::from_secret(SECRET.as_ref()),
            &Validation::default(),
        )
        .map_err(|e| match e.kind() {
            ErrorKind::ExpiredSignature => AuthenticationError::Expired,
            _ => AuthenticationError::Decoding(e.to_string()),
        })?;

        Ok(token.claims)
    }

    /// Converts this claims into a token string
    pub(crate) fn to_token(mut self) -> Result<String, Custom<String>> {
        let expiration = Utc::now()
            .checked_add_signed(*TOKEN_EXPIRATION)
            .expect("failed to create an expiration time")
            .timestamp();

        self.exp = expiration as usize;

        // Construct and return JWT using `jsonwebtoken`
        // Consult the `jsonwebtoken` documentation for using other algorithms and asymmetric keys
        let token = encode(
            &Header::default(),
            &self,
            &EncodingKey::from_secret(SECRET.as_ref()),
        )
        .map_err(|e| Custom(Status::BadRequest, e.to_string()))?;

        Ok(token)
    }
}

pub enum User {
    Authenticated(AuthenticatedUser),
    Guest
}

impl User {
    pub fn id(&self) -> Option<i32> {
        match self {
            User::Authenticated(u) => Some(u.data.id.unwrap()),
            Guest => None
        }
    }
}

// Rocket specific request guard implementations
#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedUser {
    type Error = AuthenticationError;

    async fn from_request(request: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        match request.headers().get_one(AUTHORIZATION) {
            None => Outcome::Failure((Status::Forbidden, AuthenticationError::Missing)),
            Some(value) => match AuthenticatedUser::from_authorization(value) {
                Err(e) => Outcome::Failure((Status::Forbidden, e)),
                Ok(claims) => Outcome::Success(claims),
            },
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = AuthenticationError;

    async fn from_request(request: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        match request.headers().get_one(AUTHORIZATION) {
            None => Outcome::Success(User::Guest),
            Some(value) => match AuthenticatedUser::from_authorization(value) {
                Err(e) => Outcome::Failure((Status::Forbidden, e)),
                Ok(claims) => Outcome::Success(User::Authenticated(claims)),
            },
        }
    }
}

