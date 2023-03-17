use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::tag::Tag;

#[derive(Debug, Serialize, Deserialize)]
pub struct TaggedImage {
    pub hash: String,
    pub created_at: DateTime<Utc>,
    pub tags: Vec<Tag>,
    pub user: Vec<String>,
}
