use axum::{extract::State, Json};
use axum_macros::debug_handler;
use futures::future::try_join_all;
use serde::Deserialize;

use crate::{
    database::Database,
    errors::Error,
    jwt::Claims,
    models::{
        image::ImageResponse,
        tag::{Convert, TagResponse},
    },
};

#[derive(Debug, Deserialize)]
pub struct SearchImage {
    #[serde(default)]
    include: Vec<TagResponse>,
    #[serde(default)]
    exclude: Vec<TagResponse>,
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

    let include = try_join_all(query.include.into_iter().map(|t| t.convert(&db))).await?;
    let exclude = try_join_all(query.exclude.into_iter().map(|t| t.convert(&db))).await?;

    let previous = match query.previous {
        Some(hash) => {
            let image = db.image.get(&hash).await?.ok_or(Error::ImageNotFound)?;
            Some(image.id)
        }
        None => None,
    };

    println!("{include:?} {exclude:?}");

    let images = db.image.search(include, exclude, previous).await?;
    let images = try_join_all(images.into_iter().map(|i| i.convert(&db))).await?;

    Ok(Json(images))
}
