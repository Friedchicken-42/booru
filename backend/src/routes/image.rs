use axum::{
    body::Bytes,
    extract::{multipart::Field, Multipart, State, Query},
};
use reqwest::{Client, multipart::{Form, Part}};
use serde::Deserialize;

use crate::{database::Database, errors::Error, jwt::Claims, models::image::Image};

async fn upload(image: &Image, data: Bytes) -> Result<(), Error> {
    let part = Part::bytes(data.to_vec())
        .file_name(image.id.clone());

    let multipart = Form::new()
        .part("file", part);

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
        let Some((name, _, content_type, data)) = parse_field(field).await else {
            continue;
        };

        if name != "image" {
            continue;
        }

        let image = Image::new(&data, content_type);
        let exists = db
            .image_exists(image.id.clone())
            .await?;

        if exists {
            return Err(Error::ImageExists);
        }

        upload(&image, data).await?;

        db.image_insert(&image).await?;

        return Ok(image.id);
    }

    Err(Error::MissingField)
}

#[derive(Deserialize)]
pub struct QueryDelete {
    id: String,
}

pub async fn delete(
    _: Claims,
    State(db): State<Database>,
    Query(query): Query<QueryDelete>,
) -> Result<String, Error> {

    let id = query.id;

    db
        .image_delete(id)
        .await?;

    Ok(id)
}
