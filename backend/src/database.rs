use mongodb::{
    bson::{doc, oid::ObjectId, DateTime},
    options::ClientOptions,
    Client,
};

use crate::{
    errors::Error,
    models::{image::Image, user::User},
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
pub struct Database {
    client: Client,
    pub user: Users,
    pub image: Images,
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

    pub async fn exists(&self, id: &String) -> Result<bool, Error> {
        let filter = doc! {"_id": id};
        let found = self
            .collection
            .find_one(filter, None)
            .await
            .map_err(|_| Error::DatabaseError)?;

        Ok(found.is_some())
    }

    pub async fn insert(&self, image: &Image) -> Result<(), Error> {
        self.collection
            .insert_one(image, None)
            .await
            .map_err(|_| Error::DatabaseError)?;

        Ok(())
    }

    pub async fn delete(&self, id: &String) -> Result<(), Error> {
        self.collection
            .delete_one(doc! {"_id": id}, None)
            .await
            .map_err(|_| Error::DatabaseError)?;

        Ok(())
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

        Ok(Database {
            client,
            user,
            image,
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
