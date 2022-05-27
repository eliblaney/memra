use chrono::{Duration, Utc};
use jsonwebtoken::{
    decode, encode, errors::ErrorKind, DecodingKey, EncodingKey, Header, Validation,
};
use lazy_static::lazy_static;
use rocket::{
    http::Status,
    request::{FromRequest, Outcome},
    response::status::Custom,
    serde::json::Json
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
pub(crate) enum AuthenticationError {
    Missing,
    Decoding(String),
    Expired,
}

// Basic claim object. Only the `exp` claim (field) is required. Consult the `jsonwebtoken` documentation for other claims that can be validated.
// The `name` is a custom claim for this API
#[derive(Serialize, Deserialize)]
pub(crate) struct AuthenticatedUser {
    pub(crate) name: String,
    exp: usize,
}

impl AuthenticatedUser {
    pub(crate) fn from_name(name: &str) -> Self {
        Self {
            name: name.to_string(),
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

pub(crate) enum User {
    Authenticated(AuthenticatedUser),
    Guest
}

/*
// Optional implementation for helper methods
impl User {
    pub fn is_authenticated(&self) -> bool {
        match self {
            User::Authenticated(_) => true,
            User::Guest => false
        }
    }
}
*/

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

#[derive(Deserialize)]
pub(crate) struct LoginRequest {
    username: String,
    password: String,
}

pub(crate) fn login(credentials: Json<LoginRequest>) -> Result<String, Custom<String>> {
    // This should be real user validation code, but is left simple for this example
    if credentials.username != "username" || credentials.password != "password" {
        return Err(Custom(
            Status::Unauthorized,
            "account was not found".to_string(),
        ));
    }

    let claim = AuthenticatedUser::from_name(&credentials.username);

    claim.to_token()
}
