use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::errors::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct Tag {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub name: String,
    pub category: String,
    pub description: String,
}

#[derive(Serialize)]
pub struct TagResponse {
    pub name: String,
    pub category: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Tag {
    pub fn new(name: String, category: String, description: String) -> Tag {
        Tag {
            id: ObjectId::new(),
            name,
            category,
            description,
        }
    }
}

impl TagResponse {
    pub fn from(tag: Tag) -> Self {
        Self {
            name: tag.name,
            category: tag.category,
            description: Some(tag.description),
        }
    }

    pub fn clean(mut self) -> Self {
        self.description = None;
        self
    }
}

impl IntoResponse for TagResponse {
    fn into_response(self) -> Response {
        match serde_json::to_string(&self) {
            Ok(data) => (StatusCode::OK, data).into_response(),
            Err(_) => Error::Serialize.into_response(),
        }
    }
}
