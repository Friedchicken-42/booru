use axum::{extract::State, Json};
use serde::Deserialize;

use crate::{
    database::Database,
    errors::Error,
    jwt::Claims,
    models::{tag::Tag, tagresponse::TagResponse},
    // models::tag::{Tag, TagResponse, Convert},
};

#[derive(Deserialize)]
pub struct Create {
    name: String,
    category: String,
    description: String,
}

pub async fn create(
    _: Claims,
    State(db): State<Database>,
    Json(query): Json<Create>,
) -> Result<TagResponse, Error> {
    let tag = Tag::new(query.name, query.category, query.description);

    if db.tag.get(&tag.name, &tag.category).await?.is_some() {
        return Err(Error::TagExists);
    }

    db.tag.create(&tag).await?;

    Ok(TagResponse::new(tag))
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
    let tag = db.tag
        .get(&query.name, &query.category)
        .await?
        .ok_or(Error::TagNotFound)?;

    db.tag.delete(tag).await?;

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
    let tag = db.tag
        .get(&query.name, &query.category)
        .await?
        .ok_or(Error::TagNotFound)?;

    Ok(TagResponse::with_description(tag))
}
