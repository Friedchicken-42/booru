use bson::{doc, oid::ObjectId};
use futures::StreamExt;
use uuid::Uuid;

use crate::{
    errors::Error,
    models::{image::Image, tag::Tag},
};

use super::tag::Tags;

#[derive(Clone)]
pub struct Images {
    collection: mongodb::Collection<Image>,
    client: mongodb::Client,
}

impl Images {
    pub fn new(db: &mongodb::Database, client: mongodb::Client) -> Images {
        Images {
            collection: db.collection::<Image>("images"),
            client,
        }
    }

    pub async fn insert(&self, image: &Image) -> Result<(), Error> {
        self.collection
            .insert_one(image, None)
            .await
            .map_err(|_| Error::DatabaseError)?;

        Ok(())
    }

    pub async fn delete(&self, id: &Uuid) -> Result<(), Error> {
        self.collection
            .delete_one(doc! {"_id": id}, None)
            .await
            .map_err(|_| Error::DatabaseError)?;

        Ok(())
    }

    pub async fn get(&self, id: &Uuid) -> Result<Option<Image>, Error> {
        self.collection
            .find_one(doc! {"_id": id}, None)
            .await
            .map_err(|_| Error::DatabaseError)
    }

    pub async fn set(
        &self,
        id: &Uuid,
        tag_list: Vec<Tag>,
        tag_coll: &Tags,
    ) -> Result<Image, Error> {

        #[cfg(session)]
        let mut session = self
            .client
            .start_session(None)
            .await
            .map_err(|_| Error::SessionCreate)?;

        #[cfg(session)]
        session
            .start_transaction(None)
            .await
            .map_err(|_| Error::SessionCreate)?;

        let image = self.get(id).await?.ok_or(Error::ImageNotFound)?;

        let ids: Vec<ObjectId> = tag_list.iter().map(|t| t.id).collect();

        for id in &ids {
            if !image.tags.contains(id) {
                tag_coll.increment(id, #[cfg(session)]&mut session).await?;
            }
        }

        for tag in image.tags {
            if !ids.contains(&tag) {
                tag_coll.decrement(&tag, #[cfg(session)]&mut session).await?;
            }
        }

        #[cfg(not(session))]
        self.collection
            .update_one(doc! {"_id": id}, doc! {"$set": {"tags": ids}}, None)
            .await
            .map_err(|_| Error::DatabaseError)?;

        #[cfg(session)]
        self.collection
            .update_one_with_session(doc! {"_id": id}, doc! {"$set": {"tags": ids}}, None, &mut session)
            .await
            .map_err(|_| Error::DatabaseError)?;

        #[cfg(session)]
        session
            .commit_transaction()
            .await
            .map_err(|_| Error::SessionCommit)?;

        self.get(id).await?.ok_or(Error::DatabaseError)
    }

    pub async fn search(
        &self,
        include: Vec<Tag>,
        exclude: Vec<Tag>,
        previous: Option<Image>,
    ) -> Result<Vec<Image>, Error> {
        println!("{include:?} {exclude:?} {previous:?}");
        let limit: u32 = 5;

        let mut pipeline = vec![];

        let include: Vec<ObjectId> = include.iter().map(|t| t.id).collect();
        let exclude: Vec<ObjectId> = exclude.iter().map(|t| t.id).collect();

        if !include.is_empty() {
            pipeline.append(&mut vec![doc! {
                "$match": {
                    "tags": {
                        "$in": include
                    }
                }
            }]);
        }

        if !exclude.is_empty() {
            pipeline.append(&mut vec![doc! {
                "$match": {
                    "tags": {
                        "$not": {
                            "$in": exclude
                        }
                    }
                }
            }]);
        }

        pipeline.append(&mut vec![
            doc! { "$sort": { "created_at": -1 } },
            doc! { "$limit": limit },
        ]);

        if let Some(prev) = previous {
            pipeline.push(doc! {
                "$match": {
                    "created_at": { "$lt": prev.created_at }
                }
            });
        }

        let mut cursor = self
            .collection
            .aggregate(pipeline, None)
            .await
            .map_err(|_| Error::DatabaseError)?;

        let mut images = Vec::with_capacity(limit as usize);
        while let Some(document) = cursor.next().await {
            let document = document.map_err(|_| Error::DatabaseError)?;
            let image = bson::from_document(document).map_err(|_| Error::DatabaseError)?;
            images.push(image);
        }

        Ok(images)
    }
}
