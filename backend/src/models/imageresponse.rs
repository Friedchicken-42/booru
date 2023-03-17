use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::errors::Error;

use super::{taggedimage::TaggedImage, tagresponse::TagResponse};

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageResponse {
    pub hash: String,
    pub url: String,
    pub created_at: DateTime<Utc>,
    pub tags: Vec<TagResponse>,
    pub user: String,
}

impl ImageResponse {
    pub fn new(image: TaggedImage) -> Self {
        let url = format!("http://localhost:4000/{}", image.hash);
        let tags = image.tags.into_iter().map(TagResponse::new).collect();
        let user = image.user[0].clone();

        Self {
            hash: image.hash,
            url,
            created_at: image.created_at,
            tags,
            user,
        }
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
