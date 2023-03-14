use axum::{extract::State, Json};
use axum_macros::debug_handler;
use futures::future::try_join_all;
use serde::Deserialize;

use crate::{
    database::Database,
    errors::Error,
    jwt::Claims,
    models::{tag::Tag, tagresponse::TagResponse, imageresponse::ImageResponse},
    // models::{
    //     image::ImageResponse,
    //     tag::{Convert, TagResponse},
    // },
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
) -> Result<(), Error> {
    // let tags = db.tag.search(&query.category, &query.name).await?;
    // let tags = try_join_all(tags.into_iter().map(|tag| tag.convert(&db))).await?;

    // Ok(Json(tags))
    Err(Error::NotImplemented)
}
