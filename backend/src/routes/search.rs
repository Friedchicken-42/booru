use axum::{extract::State, Json};
use axum_macros::debug_handler;
use serde::Deserialize;

use crate::{
    database::Database,
    errors::Error,
    jwt::Claims,
    models::{tagresponse::TagResponse, imageresponse::ImageResponse},
};

#[derive(Debug, Deserialize)]
pub struct SearchImage {
    #[serde(default)]
    include: Vec<TagResponse>,
    #[serde(default)]
    exclude: Vec<TagResponse>,
    #[serde(default)]
    previous: Option<String>,
}

#[debug_handler]
pub async fn image(
    _: Claims,
    State(db): State<Database>,
    Json(query): Json<SearchImage>,
) -> Result<Json<Vec<ImageResponse>>, Error> {
    println!(
        "{:?} {:?} {:?}",
        query.include, query.exclude, query.previous
    );

    let include = db.tag.convert(query.include).await?;
    let exclude = db.tag.convert(query.exclude).await?;
    let previous = match query.previous {
        Some(hash) => Some(db.image.get(&hash).await?.ok_or(Error::ImageNotFound)?),
        None => None,
    };

    let images = db.image.search(include, exclude, previous).await?;
    let images = images.into_iter().map(ImageResponse::new).collect(); 

    Ok(Json(images))
}

#[derive(Debug, Deserialize)]
pub struct SearchTag {
    #[serde(default)]
    name: String,
    #[serde(default)]
    category: String,
}

pub async fn tag(
    _: Claims,
    State(db): State<Database>,
    Json(query): Json<SearchTag>,
    ) -> Result<Json<Vec<TagResponse>>, Error> {
    let tags = db.tag.search(&query.category, &query.name).await?;
    let tags = tags.into_iter().map(TagResponse::new).collect();

    Ok(Json(tags))
}
