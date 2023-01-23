use std::env;

use axum::{
    async_trait,
    extract::FromRequestParts,
    headers::{authorization::Bearer, Authorization},
    http::request::Parts,
    RequestPartsExt, TypedHeader,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::errors::Error;

struct Keys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

static KEYS: Lazy<Keys> = Lazy::new(|| {
    let secret = env::var("JWT_SECRET").expect("JsonWebToken Secret not found");

    Keys {
        encoding: EncodingKey::from_secret(secret.as_bytes()),
        decoding: DecodingKey::from_secret(secret.as_bytes()),
    }
});

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Token {
    pub access_token: String,
    pub token_type: String,
    pub expire: i64,
}

#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| Error::InvalidToken)?;

        Claims::decode(bearer.token())
    }
}

impl Claims {
    pub fn new(sub: String) -> Claims {
        let exp = Utc::now() + Duration::days(1);
        let exp = exp.timestamp();

        Claims { sub, exp }
    }

    pub fn encode(&self) -> Result<Token, Error> {
        let access_token =
            encode(&Header::default(), self, &KEYS.encoding).map_err(|_| Error::InvalidToken)?;

        Ok(Token {
            access_token,
            token_type: "Bearer".to_string(),
            expire: self.exp,
        })
    }

    pub fn decode(token: &str) -> Result<Claims, Error> {
        let claims = decode::<Claims>(token, &KEYS.decoding, &Validation::default())
            .map_err(|_| Error::InvalidToken)?
            .claims;

        Ok(claims)
    }
}
