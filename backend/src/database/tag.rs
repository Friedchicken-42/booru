use bson::{doc, oid::ObjectId};
use futures::{StreamExt, TryStreamExt};
use mongodb::{ClientSession, options::FindOptions};

use crate::{errors::Error, models::tag::Tag};

#[derive(Clone)]
pub struct Tags {
    collection: mongodb::Collection<Tag>,
    client: mongodb::Client,
}

impl Tags {
    pub fn new(db: &mongodb::Database, client: mongodb::Client) -> Tags {
        Tags {
            collection: db.collection::<Tag>("tags"),
            client,
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

    pub async fn search(&self, category: &String, name: &String) -> Result<Vec<Tag>, Error> {
        let mut filter = doc! {};

        if !category.is_empty() {
            filter.insert("category", doc! { "$regex": category });
        }

        if !name.is_empty() {
            filter.insert("name", doc! { "$regex": name });
        }

        let options = FindOptions::builder().limit(10).build();

        let x = self.collection
            .find(filter, options)
            .await
            .map_err(|_| Error::DatabaseError)?;

        x.try_collect().await.map_err(|_| Error::DatabaseError)
    }

    #[cfg(not(session))]
    pub async fn increment(&self, id: &ObjectId) -> Result<(), Error> {
        self.collection
            .update_one(doc! {"_id": id }, doc! {"$inc": { "count": 1 } }, None)
            .await
            .map_err(|_| Error::DatabaseError)?;

        Ok(())
    }

    #[cfg(session)]
    pub async fn increment(&self, id: &ObjectId, session: &mut ClientSession) -> Result<(), Error> {
        self.collection
            .update_one_with_session(
                doc! {"_id": id },
                doc! {"$inc": { "count": 1 } },
                None,
                session,
            )
            .await
            .map_err(|_| Error::DatabaseError)?;

        Ok(())
    }

    #[cfg(not(session))]
    pub async fn decrement(&self, id: &ObjectId) -> Result<(), Error> {
        self.collection
            .update_one(doc! {"_id": id }, doc! {"$inc": { "count": -1 } }, None)
            .await
            .map_err(|_| Error::DatabaseError)?;

        Ok(())
    }

    #[cfg(session)]
    pub async fn decrement(&self, id: &ObjectId, session: &mut ClientSession) -> Result<(), Error> {
        self.collection
            .update_one_with_session(
                doc! {"_id": id },
                doc! {"$inc": { "count": -1 } },
                None,
                session,
            )
            .await
            .map_err(|_| Error::DatabaseError)?;

        Ok(())
    }
}
