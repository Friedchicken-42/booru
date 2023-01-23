use axum::{
    body::Bytes,
    extract::{multipart::Field, Multipart, State},
    Json,
};
use axum_macros::debug_handler;
use futures::future::try_join_all;
use reqwest::{
    multipart::{Form, Part},
    Client,
};
use serde::Deserialize;

use crate::{
    database::Database,
    errors::Error,
    jwt::Claims,
    models::{
        image::{Image, ImageResponse},
        tag::{Convert, TagResponse},
    },
};

async fn upload(image: &Image, data: Bytes) -> Result<(), Error> {
    let part = Part::bytes(data.to_vec()).file_name(image.hash.clone());

    let multipart = Form::new().part("file", part);

    Client::new()
        .post("http://localhost:4000")
        .multipart(multipart)
        .send()
        .await
        .map_err(|_| Error::WrongCredential)?
        .error_for_status()
        .map_err(|_| Error::Upload)?;

    Ok(())
}

async fn parse_field(field: Field<'_>) -> Option<(String, String, String, Bytes)> {
    let name = match field.name() {
        Some(n) => n.to_string(),
        None => return None,
    };

    let filename = match field.file_name() {
        Some(n) => n.to_string(),
        None => return None,
    };

    let content_type = match field.content_type() {
        Some(n) => n.to_string(),
        None => return None,
    };

    let data = match field.bytes().await {
        Ok(n) => n,
        Err(_) => return None,
    };

    Some((name, filename, content_type, data))
}

pub async fn create(
    _: Claims,
    State(db): State<Database>,
    mut multipart: Multipart,
) -> Result<String, Error> {
    while let Ok(Some(field)) = multipart.next_field().await {
        println!("{field:?}");
        let Some((name, _, content_type, data)) = parse_field(field).await else {
            continue;
        };
        println!("{name:?}");

        if name != "image" {
            continue;
        }

        let image = Image::new(&data, content_type);
        let option = db.image.get(&image.hash).await?;

        if option.is_some() {
            return Err(Error::ImageExists);
        }

        upload(&image, data).await?;

        db.image.insert(&image).await?;

        return Ok(image.hash);
    }

    Err(Error::MissingField)
}

#[derive(Deserialize)]
pub struct Delete {
    id: String,
}

pub async fn delete(
    _: Claims,
    State(db): State<Database>,
    Json(query): Json<Delete>,
) -> Result<String, Error> {
    let hash = query.id;

    let option = db.image.get(&hash).await?;

    if option.is_none() {
        return Err(Error::ImageNotFound);
    }

    db.image.delete(&hash).await?;

    Ok(hash)
}

#[derive(Deserialize)]
pub struct Get {
    id: String,
}

#[debug_handler]
pub async fn get(
    _: Claims,
    State(db): State<Database>,
    Json(query): Json<Get>,
) -> Result<ImageResponse, Error> {
    let hash = query.id;

    let image = db.image.get(&hash).await?.ok_or(Error::ImageNotFound)?;

    image.convert(&db).await
}

#[derive(Deserialize)]
pub struct Update {
    id: String,
    tags: Vec<TagResponse>,
}

pub async fn update(
    _: Claims,
    State(db): State<Database>,
    Json(query): Json<Update>,
) -> Result<ImageResponse, Error> {
    let id = query.id;

    let tags = try_join_all(query.tags.into_iter().map(|t| t.convert(&db))).await?;

    let image = db.image.set(&id, tags).await?;
    image.convert(&db).await
}
