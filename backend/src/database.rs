use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    Surreal, method::Query,
};

use crate::errors::Error;

// use self::{image::Images, tag::Tags, user::Users};
use self::{image::ImageDB, tag::TagDB};

pub mod image;
pub mod tag;
// pub mod user;

pub type Session<'a> = Query<'a, Client>;

#[derive(Clone)]
pub struct Database {
    pub client: Surreal<Client>,
    // pub user: Users,
    pub image: ImageDB,
    pub tag: TagDB,
}

impl Database {
    pub async fn new(url: String) -> Result<Self, Error> {
        let client = Surreal::new::<Ws>(url)
            .await
            .map_err(|_| Error::DatabaseConnection)?;

        Ok(Self {
            client: client.clone(),
            // user: Users(db.clone()),
            image: ImageDB(client.clone()),
            tag: TagDB(client.clone()),
        })
    }

    pub async fn signin(self, username: &str, password: &str) -> Result<Self, Error> {
        self.client
            .signin(Root { username, password })
            .await
            .map_err(|_| Error::DatabaseConnection)?;

        Ok(self)
    }

    pub async fn connect(self, namespace: &str, database: &str) -> Result<Self, Error> {
        self.client
            .use_ns(namespace)
            .use_db(database)
            .await
            .map_err(|_| Error::DatabaseConnection)?;

        Ok(self)
    }
}
