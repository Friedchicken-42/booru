use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Tag {
    #[serde(skip_serializing)]
    pub id: Option<String>,
    pub name: String,
    pub category: String,
    pub description: String,
    pub count: u32,
    #[serde(skip_serializing, skip_deserializing)]
    pub user: String,
}

impl Tag {
    pub fn new(name: String, category: String, description: String) -> Self {
        Self {
            id: None,
            name,
            category,
            description,
            count: 0,
            user: String::new(),
        }
    }
}

impl ToString for Tag {
    fn to_string(&self) -> String {
        match &self.id {
            Some(x) => x.clone(),
            None => unreachable!(),
        }
    }
}
