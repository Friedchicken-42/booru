use bson::{doc, oid::ObjectId};

use crate::{models::tag::Tag, errors::Error};

#[derive(Clone)]
pub struct Tags {
    collection: mongodb::Collection<Tag>,
}

impl Tags {
    pub fn new(db: &mongodb::Database) -> Tags {
        Tags {
            collection: db.collection::<Tag>("tags"),
        }
    }

    pub async fn insert(&self, tag: &Tag) -> Result<(), Error> {
        self.collection
            .insert_one(tag, None)
            .await
            .map_err(|_| Error::DatabaseError)?;

        Ok(())
    }

    pub async fn delete(&self, category: &String, name: &String) -> Result<(), Error> {
        self.collection
            .delete_one(doc! {"category": category, "name": name}, None)
            .await
            .map_err(|_| Error::DatabaseError)?;

        Ok(())
    }

    pub async fn get(&self, id: &ObjectId) -> Result<Tag, Error> {
        self.collection
            .find_one(doc! {"_id": id}, None)
            .await
            .map_err(|_| Error::DatabaseError)?
            .ok_or(Error::TagNotFound)
    }

    pub async fn find(&self, category: &String, name: &String) -> Result<Option<Tag>, Error> {
        self.collection
            .find_one(doc! {"category": category, "name": name}, None)
            .await
            .map_err(|_| Error::DatabaseError)
    }
}
