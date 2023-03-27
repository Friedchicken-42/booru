use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{tag::Tag, image::Image};

#[derive(Debug, Serialize, Deserialize)]
pub struct TaggedImage {
    pub hash: String,
    pub created_at: DateTime<Utc>,
    pub tags: Vec<Tag>,
    pub user: String,
}

impl TaggedImage {
    pub fn new(image: Image, tags: Vec<Tag>, user: String) -> Self {
        Self {
            hash: image.hash,
            created_at: image.created_at,
            tags,
            user,
        }
    }
}
