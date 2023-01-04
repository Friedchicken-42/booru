use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Tag {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub name: String,
    pub category: String,
    pub description: String,
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
