use axum::{http::StatusCode, response::{IntoResponse, Response}};
use serde::{Deserialize, Serialize};

use crate::errors::Error;

use super::tag::Tag;

#[derive(Debug, Serialize, Deserialize)]
pub struct TagResponse {
    pub name: String,
    pub category: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub count: u32,
}

impl TagResponse {
    pub fn new(tag: Tag) -> Self {
        Self {
            name: tag.name,
            category: tag.category,
            description: None,
            count: tag.count,
        }
    }

    pub fn with_description(tag: Tag) -> Self {
        let description = Some(tag.description.clone());
        Self {
            description,
            ..Self::new(tag)
        }
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
