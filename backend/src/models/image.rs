use axum::body::Bytes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use super::{tag::Tag, user::User};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Image {
    #[serde(skip_serializing)]
    pub id: Option<String>,
    pub hash: String,
    pub created_at: DateTime<Utc>,
    pub content_type: String,
    #[serde(skip_serializing, skip_deserializing)]
    pub tags: Vec<Tag>,
    #[serde(skip_serializing, skip_deserializing)]
    pub user: String,
}

impl Image {
    pub fn new(data: &Bytes, content_type: String) -> Self {
        let hash = format!("{:x}", md5::compute(data));

        Self {
            id: None,
            hash,
            created_at: Utc::now(),
            content_type,
            tags: vec![],
            user: String::new(),
        }
    }

    pub fn tagged(image: Image, tags: Vec<Tag>, user: User) -> Self {
        Self {
            tags,
            user: user.name,
            ..image
        }
    }
}
