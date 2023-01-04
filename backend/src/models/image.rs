use axum::{
    body::Bytes,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::errors::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct Image {
    #[serde(rename = "_id")]
    pub id: String,
    pub content_type: String,
    pub tags: Vec<ObjectId>,
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

impl IntoResponse for Image {
    fn into_response(self) -> Response {
        match serde_json::to_string(&self) {
            Ok(data) => (StatusCode::OK, data).into_response(),
            Err(_) => Error::Serialize.into_response(),
        }
    }
}
