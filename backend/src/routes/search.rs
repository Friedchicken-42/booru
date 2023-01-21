use axum::{extract::State, Json};
use serde::Deserialize;

use crate::{models::{image::ImageResponse, tag::TagResponse}, errors::Error, jwt::Claims, database::Database};

#[derive(Debug, Deserialize)]
pub struct SearchImage {
    #[serde(default)]
    include: Vec<TagResponse>,
    #[serde(default)]
    exclude: Vec<TagResponse>,
    #[serde(default)]
    offset: usize,
}

pub async fn image(
    _: Claims,
    State(db): State<Database>,
    Json(query): Json<SearchImage>
) -> Result<ImageResponse, Error> {
    println!("{:?} {:?} {}", query.include, query.exclude, query.offset);
    Err(Error::ImageNotFound)
}
