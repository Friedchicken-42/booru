use surrealdb::{
    engine::remote::ws::{Client, Ws},
    method::Query,
    opt::auth::Root,
    Surreal,
};

use crate::errors::Error;

use self::{image::ImageDB, tag::TagDB, user::UserDB};

pub mod image;
pub mod tag;
pub mod user;

pub type Session<'a> = Query<'a, Client>;

#[derive(Clone)]
pub struct Database {
    pub client: Surreal<Client>,
}

impl Database {
    pub async fn new(url: String) -> Result<Self, Error> {
        let client = Surreal::new::<Ws>(url)
            .await
            .map_err(|_| Error::DatabaseConnection)?;

        Ok(Self { client })
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

    pub fn image(&self) -> ImageDB {
        ImageDB {
            client: &self.client,
            db: &self,
        }
    }

    pub fn tag(&self) -> TagDB {
        TagDB {
            client: &self.client,
            db: &self,
        }
    }

    pub fn user(&self) -> UserDB {
        UserDB {
            client: &self.client,
            db: &self,
        }
    }
}
