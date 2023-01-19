use axum::{
    body::Bytes,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{database::Database, errors::Error, models::tag::TagResponse};

#[derive(Debug, Serialize, Deserialize)]
pub struct Image {
    #[serde(rename = "_id")]
    pub id: String,
    pub content_type: String,
    pub tags: Vec<ObjectId>,
}

#[derive(Serialize)]
pub struct ImageResponse {
    pub url: String,
    pub tags: Vec<TagResponse>,
}

impl Image {
    pub fn new(data: &Bytes, content_type: String) -> Image {
        let id = format!("{:x}", md5::compute(data));

        Image {
            id,
            content_type,
            tags: vec![],
        }
    }
}

impl ImageResponse {
    pub async fn from(image: Image, db: &Database) -> Result<Self, Error> {
        let url = format!("http://localhost:4000/{}", image.id);

        let mut tags = vec![];
        for id in image.tags {
            let tag = db
                .tag
                .get(&id)
                .await
                .map(TagResponse::from)?
                .clean();

            tags.push(tag);
        }

        Ok(ImageResponse { url, tags })
    }
}

impl IntoResponse for ImageResponse {
    fn into_response(self) -> Response {
        match serde_json::to_string(&self) {
            Ok(data) => (StatusCode::OK, data).into_response(),
            Err(_) => Error::Serialize.into_response(),
        }
    }
}
