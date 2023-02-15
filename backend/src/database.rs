use futures::StreamExt;
use mongodb::{
    bson::{self, doc, oid::ObjectId, DateTime},
    options::ClientOptions,
    Client,
};
use uuid::Uuid;

use crate::{
    errors::Error,
    models::{image::Image, tag::Tag, user::User},
};

#[derive(Clone)]
pub struct Users {
    collection: mongodb::Collection<User>,
}

#[derive(Clone)]
pub struct Images {
    collection: mongodb::Collection<Image>,
}

#[derive(Clone)]
pub struct Tags {
    collection: mongodb::Collection<Tag>,
}

#[derive(Clone)]
pub struct Database {
    client: Client,
    pub user: Users,
    pub image: Images,
    pub tag: Tags,
}

impl Users {
    fn new(db: &mongodb::Database) -> Users {
        Users {
            collection: db.collection::<User>("users"),
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

impl Images {
    fn new(db: &mongodb::Database) -> Images {
        Images {
            collection: db.collection::<Image>("images"),
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

    pub async fn set(&self, id: &Uuid, tags: Vec<Tag>) -> Result<Image, Error> {
        let ids: Vec<ObjectId> = tags.iter().map(|t| t.id).collect();

        self.collection
            .update_one(doc! {"_id": id}, doc! {"$set": {"tags": ids}}, None)
            .await
            .map_err(|_| Error::DatabaseError)?;

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

        if include.len() > 0 {
            pipeline.append(&mut vec![doc! {
                "$match": {
                    "tags": {
                        "$in": include
                    }
                }
            }]);
        }

        if exclude.len() > 0 {
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

impl Tags {
    fn new(db: &mongodb::Database) -> Tags {
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

impl Database {
    pub async fn connect(url: String) -> Result<Self, Error> {
        let mut options = ClientOptions::parse(url)
            .await
            .map_err(|_| Error::DatabaseConnection)?;

        options.app_name = Some("Booru".to_string());

        let client = Client::with_options(options).map_err(|_| Error::DatabaseConnection)?;

        let db = client.database("booru");
        let user = Users::new(&db);
        let image = Images::new(&db);
        let tag = Tags::new(&db);

        Ok(Database {
            client,
            user,
            image,
            tag,
        })
    }

    pub async fn ping(self) -> Result<Self, Error> {
        self.client
            .database("admin")
            .run_command(doc! {"ping": 1}, None)
            .await
            .map_err(|_| Error::DatabaseConnection)?;

        Ok(self)
    }
}
