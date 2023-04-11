use surrealdb::{engine::remote::ws::Client, Surreal};

use crate::errors::Error;
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
