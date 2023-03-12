use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use axum::body::Bytes;

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
