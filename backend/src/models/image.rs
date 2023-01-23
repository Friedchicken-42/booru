use axum::{
    body::Bytes,
    http::StatusCode,
    response::{IntoResponse, Response}, async_trait,
};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{database::Database, errors::Error, models::tag::TagResponse};

use super::tag::Convert;

#[derive(Debug, Serialize, Deserialize)]
pub struct Image {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub hash: String,
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
        let hash = format!("{:x}", md5::compute(data));

        Image {
            id: ObjectId::new(),
            hash,
            content_type,
            tags: vec![],
        }
    }
}

#[async_trait]
impl Convert<ImageResponse> for Image {
    async fn convert(self, db: &Database) -> Result<ImageResponse, Error> {
        let url = format!("http://localhost:4000/{}", self.id);

        let mut tags = Vec::with_capacity(self.tags.len());
        for id in self.tags {
            let tag = db.tag.get(&id).await?;
            let tag = tag.convert(db).await?;
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
