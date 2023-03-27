use axum::body::Bytes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Image {
    #[serde(skip_serializing)]
    pub id: Option<String>,
    pub hash: String,
    pub created_at: DateTime<Utc>,
    pub content_type: String,
}

impl Image {
    pub fn new(data: &Bytes, content_type: String) -> Self {
        let hash = format!("{:x}", md5::compute(data));

        Self {
            id: None,
            hash,
            created_at: Utc::now(),
            content_type,
        }
    }
}

impl Clone for Image {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            hash: self.hash.clone(),
            created_at: self.created_at.clone(),
            content_type: self.content_type.clone(),
        }
    }
}
