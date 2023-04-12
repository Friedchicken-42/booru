use serde::Deserialize;
use surrealdb::{engine::remote::ws::Client, Surreal};

use crate::errors::Error;
use crate::models::image::Image;
use crate::models::tag::Tag;
use crate::models::user::User;

use super::Database;

pub struct UserDB<'a> {
    pub client: &'a Surreal<Client>,
    pub db: &'a Database,
}

impl<'a> UserDB<'a> {
    pub async fn get(&self, name: &String) -> Result<Option<User>, Error> {
        let query = format!("select * from user where name = '{}'", name);

        let mut res = self.client.query(query).await?;
        let user: Option<User> = res.take(0)?;

        Ok(user)
    }

    pub async fn from_image(&self, image: &Image) -> Result<User, Error> {
        let id = image.id.clone().ok_or(Error::InvalidId)?;

        let mut res = self
            .client
            .query("select <-upload<-user.* as users from $image")
            .bind(("image", id))
            .await?;

        #[derive(Deserialize)]
        struct Container {
            users: Vec<User>,
        }

        let cont: Option<Container> = res.take(0)?;
        let cont = cont.ok_or(Error::UserNotFound)?;
        if cont.users.len() != 1 {
            return Err(Error::DatabaseError);
        }

        Ok(cont.users[0].clone())
    }

    pub async fn from_tag(&self, tag: &Tag) -> Result<User, Error> {
        let id = tag.id.clone().ok_or(Error::InvalidId)?;

        let mut res = self
            .client
            .query("select <-upload<-user.* as users from $tag")
            .bind(("tag", id))
            .await?;

        #[derive(Deserialize)]
        struct Container {
            users: Vec<User>,
        }

        let cont: Option<Container> = res.take(0)?;
        let cont = cont.ok_or(Error::UserNotFound)?;
        if cont.users.len() != 1 {
            return Err(Error::DatabaseError);
        }

        Ok(cont.users[0].clone())
    }

    pub async fn create(&self, name: String, password: String) -> Result<User, Error> {
        if self.get(&name).await?.is_some() {
            return Err(Error::UserExists);
        }

        let user = User::hash(name, password)?;
        let user: User = self.client.create("user").content(user).await?;

        Ok(user)
    }

    pub async fn authenticate(&self, name: String, password: String) -> Result<User, Error> {
        if self.get(&name).await?.is_none() {
            return Err(Error::UserNotFound);
        }

        let query = format!("select * from user where name = '{}'", name);

        let mut res = self.client.query(query).await?;
        let user: Option<User> = res.take(0)?;
        let user = user.ok_or(Error::UserNotFound)?;
        user.verify(password)?;

        Ok(user)
    }
}
