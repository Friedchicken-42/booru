use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Tag {
    #[serde(skip_serializing)]
    pub id: Option<String>,
    pub name: String,
    pub category: String,
    pub description: String,
    pub count: u32,
    pub user: Vec<String>,
}

impl Tag {
    pub fn new(name: String, category: String, description: String) -> Self {
        Self {
            id: None,
            name,
            category,
            description,
            count: 0,
            user: vec![],
        }
    }
}
