use axum::{extract::State, Json};
use serde::Deserialize;

use crate::{
    database::Database,
    errors::Error,
    jwt::{Claims, Token},
};

#[derive(Debug, Deserialize)]
pub struct ApiUser {
    pub name: String,
    pub password: String,
}

pub async fn login(
    State(db): State<Database>,
    Json(user): Json<ApiUser>,
) -> Result<Json<Token>, Error> {
    let ApiUser { name, password } = user;

    if name.is_empty() || password.is_empty() {
        return Err(Error::MissingCredential);
    }

    // let user = db.user.authenticate(name.clone(), password).await?;

    // let claims = Claims::new(user.id.to_string());
    let claims = Claims::new(name);
    let token = claims.encode()?;

    Ok(Json(token))
}
