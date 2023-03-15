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
use surrealdb::sql::statements::{BeginStatement, CommitStatement};

use crate::{
    database::Database,
    errors::Error,
    jwt::Claims,
    models::{image::Image, tag::Tag, tagresponse::TagResponse, imageresponse::ImageResponse},
};

async fn upload(image: &Image, data: Bytes) -> Result<(), Error> {
    let hash = image.hash.clone();
    let part = Part::bytes(data.to_vec()).file_name(hash);

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
    // fix large image
    while let Ok(Some(field)) = multipart.next_field().await {
        let Some((name, _, content_type, data)) = parse_field(field).await else {
            continue;
        };

        if name != "image" {
            continue;
        }

        let image = Image::new(&data, content_type);

        if db.image.get(&image.hash).await?.is_some() {
            return Err(Error::ImageExists);
        }

        // upload(&image, data).await?;

        db.image.create(&image).await?;

        return Ok(image.hash);
    }

    Err(Error::MissingField)
}

#[derive(Deserialize)]
pub struct Delete {
    hash: String,
}

pub async fn delete(
    _: Claims,
    State(db): State<Database>,
    Json(query): Json<Delete>,
) -> Result<String, Error> {
    let hash = query.hash;

    let image = db.image.get(&hash).await?.ok_or(Error::ImageNotFound)?;

    db.image.delete(image).await?;

    Ok(hash)
}

#[derive(Deserialize)]
pub struct Post {
    hash: String,
}

#[debug_handler]
pub async fn post(
    _: Claims,
    State(db): State<Database>,
    Json(query): Json<Post>,
) -> Result<ImageResponse, Error> {
    let image = db
        .image
        .get(&query.hash)
        .await?
        .ok_or(Error::ImageNotFound)?;

    println!("{:?}", image);

    let image = db.image.tagged(image).await?;

    Ok(ImageResponse::new(image))
}

#[derive(Deserialize)]
pub struct Update {
    hash: String,
    #[serde(default)]
    tags: Vec<TagResponse>,
}

pub async fn update(
    _: Claims,
    State(db): State<Database>,
    Json(query): Json<Update>,
) -> Result<ImageResponse, Error> {
    let Update { hash, tags } = query;

    let image = db.image.get(&hash).await?.ok_or(Error::ImageNotFound)?;
    println!("image: {:?}", image);

    let old_tags = db.tag.from_image(&image).await?;
    println!("old_tags: {:?}", old_tags);

    let tags = try_join_all(tags.iter().map(|t| db.tag.get(&t.name, &t.category))).await?;
    println!("tags: {:?}", tags);

    let new_tags = tags
        .into_iter()
        .collect::<Option<Vec<Tag>>>()
        .ok_or(Error::DatabaseError)?;

    println!("new_tags: {:?}", new_tags);

    let mut session = db.client.query(BeginStatement);

    for old in &old_tags {
        if !new_tags.contains(old) {
            session = db.image.unrelate(&image, old, session)?;
            session = db.tag.update(old, -1, session)?;
        }
    }
    
    for new in &new_tags {
        if !old_tags.contains(new) {
            session = db.image.relate(&image, new, session)?;
            session = db.tag.update(new, 1, session)?;
        }
    }

    let response = session.query(CommitStatement).await?;
    response.check()?; 

    let image = db.image.tagged(image).await?;

    Ok(ImageResponse::new(image))
}
