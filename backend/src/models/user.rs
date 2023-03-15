use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct User {
    pub name: String,
    pub password: String,
}

impl User {
    pub fn new(name: String, password: String) -> Self {
        // TODO: hash password here
        Self { name, password }
    }
}
