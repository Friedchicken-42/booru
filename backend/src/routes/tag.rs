use axum::{extract::State, Json};
use serde::Deserialize;

use crate::{database::Database, errors::Error, jwt::Claims, models::tag::Tag};

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
) -> Result<String, Error> {
    let tag = Tag::new(query.name, query.category, query.description);

    db.tag.insert(&tag).await?;

    Ok(format!("{}:{}", tag.category, tag.name))
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
) -> Result<String, Error> {
    let option = db.tag.get(&query.category, &query.name).await?;

    let Some(tag) = option else {
        return Err(Error::TagNotFound);
    };

    db.tag.delete(&query.category, &query.name).await?;

    Ok(format!("{}:{}", tag.category, tag.name))
}
