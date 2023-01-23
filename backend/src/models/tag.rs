use axum::{
    async_trait,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{database::Database, errors::Error};

#[derive(Debug, Serialize, Deserialize)]
pub struct Tag {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub name: String,
    pub category: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
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
    pub fn clean(self) -> Self {
        Self {
            name: self.name,
            category: self.category,
            description: None,
        }
    }
}

#[async_trait]
pub trait Convert<T> {
    async fn convert(self, db: &Database) -> Result<T, Error>;
}

#[async_trait]
impl Convert<TagResponse> for Tag {
    async fn convert(self, _: &Database) -> Result<TagResponse, Error> {
        Ok(TagResponse {
            name: self.name,
            category: self.category,
            description: Some(self.description),
        })
    }
}

#[async_trait]
impl Convert<Tag> for TagResponse {
    async fn convert(self, db: &Database) -> Result<Tag, Error> {
        db.tag
            .find(&self.category, &self.name)
            .await?
            .ok_or(Error::TagNotFound)
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
