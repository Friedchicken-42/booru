use axum::{extract::State, Json};
use serde::Deserialize;

use crate::{
    database::Database,
    errors::Error,
    jwt::Claims,
    models::tag::{Tag, TagResponse},
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

    db.tag.insert(&tag).await?;

    Ok(TagResponse::from(tag).clean())
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
) -> Result<TagResponse, Error> {
    let option = db.tag.search(&query.category, &query.name).await?;

    let Some(tag) = option else {
        return Err(Error::TagNotFound);
    };

    db.tag.delete(&tag.category, &tag.name).await?;

    Ok(TagResponse::from(tag).clean())
}

#[derive(Deserialize)]
pub struct Get {
    name: String,
    category: String,
}

pub async fn get(
    _: Claims,
    State(db): State<Database>,
    Json(query): Json<Get>,
) -> Result<TagResponse, Error> {
    let tag = db
        .tag
        .search(&query.category, &query.name)
        .await?
        .ok_or(Error::TagNotFound)?;

    Ok(TagResponse::from(tag))
}
