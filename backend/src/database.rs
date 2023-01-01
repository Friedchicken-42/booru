use mongodb::{
    bson::{doc, oid::ObjectId, DateTime},
    options::ClientOptions,
    Client, Collection,
};

use crate::{
    errors::Error,
    models::{image::Image, user::User},
};

#[derive(Clone)]
pub struct Database {
    client: Client,
    pub db: mongodb::Database,
}

impl Database {
    pub async fn connect(url: String) -> Result<Self, Error> {
        let mut options = ClientOptions::parse(url)
            .await
            .map_err(|_| Error::DatabaseConnection)?;

        options.app_name = Some("Booru".to_string());

        let client = Client::with_options(options).map_err(|_| Error::DatabaseConnection)?;

        let db = client.database("booru");

        Ok(Database { client, db })
    }

    pub async fn ping(self) -> Result<Self, Error> {
        self.client
            .database("admin")
            .run_command(doc! {"ping": 1}, None)
            .await
            .map_err(|_| Error::DatabaseConnection)?;

        Ok(self)
    }

    fn collection<T>(&self, name: &str) -> Collection<T> {
        self.db.collection(name)
    }

    pub async fn authenticate(&self, name: String, password: String) -> Result<User, Error> {
        // temp
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

    pub async fn image_exists(&self, id: String) -> Result<bool, Error> {
        let collection = self.collection::<Image>("images");

        let filter = doc! {"_id": id};
        let found = collection
            .find_one(filter, None)
            .await
            .map_err(|_| Error::DatabaseError)?;

        Ok(found.is_some())
    }

    pub async fn image_insert(&self, image: &Image) -> Result<(), Error> {
        let collection = self.collection::<Image>("images");

        collection
            .insert_one(image, None)
            .await
            .map_err(|_| Error::DatabaseError)?;

        Ok(())
    }

    pub async fn image_delete(&self, id: String) -> Result<(), Error> {
        let collection = self.collection::<Image>("images");

        collection
            .delete_one(doc! {"_id": id}, None)
            .await
            .map_err(|_| Error::DatabaseError)?;

        Ok(())
    }
}
