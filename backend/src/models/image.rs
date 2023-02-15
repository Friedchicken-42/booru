use axum::{
    async_trait,
    body::Bytes,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use bson::serde_helpers::uuid_1_as_binary;
use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{database::Database, errors::Error, models::tag::TagResponse};

use super::tag::Convert;

#[derive(Debug, Serialize, Deserialize)]
pub struct Image {
    #[serde(rename = "_id")]
    #[serde(with = "uuid_1_as_binary")]
    pub id: Uuid,
    pub content_type: String,
    pub tags: Vec<ObjectId>,
    pub created_at: DateTime,
}

#[derive(Serialize)]
pub struct ImageResponse {
    pub id: String,
    pub url: String,
    pub tags: Vec<TagResponse>,
    pub created_at: String,
}

impl Image {
    pub fn new(data: &Bytes, content_type: String) -> Image {
        let hash = Uuid::from_bytes(md5::compute(data).0);

        Image {
            id: hash,
            content_type,
            tags: vec![],
            created_at: DateTime::now(),
        }
    }
}

#[async_trait]
impl Convert<ImageResponse> for Image {
    async fn convert(self, db: &Database) -> Result<ImageResponse, Error> {
        let hash = self.id.simple().to_string();
        let url = format!("http://localhost:4000/{}", hash);

        let mut tags = Vec::with_capacity(self.tags.len());
        for id in self.tags {
            let tag = db.tag.get(&id).await?;
            let tag = tag.convert(db).await?;
            tags.push(tag);
        }

        let timestamp = self
            .created_at
            .try_to_rfc3339_string()
            .map_err(|_| Error::Serialize)?;

        Ok(ImageResponse {
            id: hash,
            url,
            tags,
            created_at: timestamp,
        })
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
