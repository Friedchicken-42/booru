use axum::body::Bytes;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Image {
    #[serde(rename = "_id")]
    pub id: String,
    pub content_type: String,
}

impl Image {
    pub fn new(data: &Bytes, content_type: String) -> Image {
        let id = format!("{:x}", md5::compute(data));

        Image { id, content_type }
    }
}
