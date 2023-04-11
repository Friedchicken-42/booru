use axum::{extract::State, Json};
use serde::Deserialize;

use crate::{
    database::Database,
    errors::Error,
    jwt::{Claims, Token},
};

#[derive(Debug, Deserialize)]
pub struct Login {
    pub name: String,
    pub password: String,
}

pub async fn login(
    State(db): State<Database>,
    Json(user): Json<Login>,
) -> Result<Json<Token>, Error> {
    let Login { name, password } = user;

    if name.is_empty() || password.is_empty() {
        return Err(Error::MissingCredential);
    }

    let user = db.user().authenticate(name, password).await?;

    let claims = Claims::new(user.name);
    let token = claims.encode()?;

    Ok(Json(token))
}

#[derive(Debug, Deserialize)]
pub struct Signup {
    pub name: String,
    pub password: String,
}

pub async fn signup(
    State(db): State<Database>,
    Json(user): Json<Signup>,
) -> Result<Json<Token>, Error> {
    let Signup { name, password } = user;

    if name.is_empty() || password.is_empty() {
        return Err(Error::MissingCredential);
    }

    let user = db.user().create(name, password).await?;

    let claims = Claims::new(user.name);
    let token = claims.encode()?;

    Ok(Json(token))
}
