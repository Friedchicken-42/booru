use axum::{extract::State, Json};
use serde::Deserialize;

use crate::{
    database::Database,
    errors::Error,
    jwt::Claims,
    models::{tag::Tag, tagresponse::TagResponse},
};

#[derive(Deserialize)]
pub struct Create {
    name: String,
    category: String,
    description: String,
}

pub async fn create(
    claims: Claims,
    State(db): State<Database>,
    Json(query): Json<Create>,
) -> Result<TagResponse, Error> {
    let tag = Tag::new(query.name, query.category, query.description);

    if db.tag().get(&tag.name, &tag.category).await?.is_some() {
        return Err(Error::TagExists);
    }

    let name = claims.sub;

    let user = db.user().get(&name).await?.ok_or(Error::UserNotFound)?;

    let tag = db.tag().create(&tag).await?;

    match db.tag().user_set(&tag, &user).await {
        Ok(t) => Ok(TagResponse::new(t)),
        Err(_) => {
            db.tag().delete(tag).await?;
            return Err(Error::InvalidId);
        }
    }
}

#[derive(Deserialize)]
pub struct Delete {
    name: String,
    category: String,
}

pub async fn delete(
    _: Claims,
    State(db): State<Database>,
    Json(query): Json<Delete>,
) -> Result<(), Error> {
    let tag = db
        .tag()
        .get(&query.name, &query.category)
        .await?
        .ok_or(Error::TagNotFound)?;

    db.tag().delete(tag).await?;

    Ok(())
}

#[derive(Deserialize)]
pub struct Post {
    name: String,
    category: String,
}

pub async fn post(
    _: Claims,
    State(db): State<Database>,
    Json(query): Json<Post>,
) -> Result<TagResponse, Error> {
    let tag = db
        .tag()
        .get(&query.name, &query.category)
        .await?
        .ok_or(Error::TagNotFound)?;

    Ok(TagResponse::with_description(tag))
}
