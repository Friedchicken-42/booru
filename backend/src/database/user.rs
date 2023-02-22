use bson::{oid::ObjectId, DateTime};

use crate::{models::user::User, errors::Error};

#[derive(Clone)]
pub struct Users {
    collection: mongodb::Collection<User>,
    client: mongodb::Client,
}

impl Users {
    pub fn new(db: &mongodb::Database, client: mongodb::Client) -> Users {
        Users {
            collection: db.collection::<User>("users"),
            client,
        }
    }

    pub async fn authenticate(&self, name: String, password: String) -> Result<User, Error> {
        if name != "a" || password != "b" {
            return Err(Error::WrongCredential);
        }

        Ok(User {
            id: ObjectId::new(),
            name,
            password,
            created_at: DateTime::now(),
        })
    }
}
