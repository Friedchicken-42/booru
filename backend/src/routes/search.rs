use std::{rc::Rc, sync::Arc};

use axum::{extract::State, Json};
use axum_macros::debug_handler;
use serde::Deserialize;

use crate::{
    database::Database,
    errors::Error,
    jwt::Claims,
    models::{imageresponse::ImageResponse, tag::Tag, tagresponse::TagResponse},
    pattern::Pattern,
};

#[derive(Debug, Deserialize)]
pub struct PatternTag {
    pub name: String,
    pub category: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchImage {
    #[serde(default)]
    pattern: Option<Pattern<PatternTag>>,
    #[serde(default)]
    previous: Option<String>,
}

#[debug_handler]
pub async fn image(
    _: Claims,
    State(db): State<Database>,
    Json(query): Json<SearchImage>,
) -> Result<Json<Vec<ImageResponse>>, Error> {
    let dbarc = Arc::new(&db);

    let mut pattern = None;
    if let Some(p) = query.pattern {
        let closure = move |tag: PatternTag| {
            let inside = Arc::clone(&dbarc);
            async move {
                inside
                    .tag
                    .get(&tag.name, &tag.category)
                    .await
                    .map_err(|_| ())?
                    .ok_or(())
            }
        };
        let closure = Arc::new(closure);
        let res = p.convert(closure).await.map_err(|_| Error::TagNotFound)?;
        pattern = Some(res);
    }

    let previous = match query.previous {
        Some(hash) => Some(db.image.get(&hash).await?.ok_or(Error::ImageNotFound)?),
        None => None,
    };

    let images = db.image.search(pattern, previous).await?;
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
