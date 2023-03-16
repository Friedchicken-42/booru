use surrealdb::{engine::remote::ws::Client, Surreal};

use crate::errors::Error;
use crate::models::user::User;

#[derive(Clone)]
pub struct UserDB(pub Surreal<Client>);

impl UserDB {
    pub async fn contains(&self, name: &String) -> Result<bool, Error> {
        // could user count or something else
        let query = format!("select * from user where name = '{}'", name);

        let mut res = self.0.query(query).await?;
        let user: Option<User> = res.take(0)?;

        Ok(user.is_some())
    }

    pub async fn create(&self, name: String, password: String) -> Result<User, Error> {
        if self.contains(&name).await? {
            return Err(Error::UserExists);
        }

        let user = User::hash(name, password)?;
        let user: User = self.0.create("user").content(user).await?;

        Ok(user)
    }

    pub async fn authenticate(&self, name: String, password: String) -> Result<User, Error> {
        if !self.contains(&name).await? {
            return Err(Error::UserNotFound);
        }

        let query = format!("select * from user where name = '{}'", name);

        let mut res = self.0.query(query).await?;
        let user: Option<User> = res.take(0)?;
        let user = user.ok_or(Error::UserNotFound)?;
        user.verify(password)?;

        Ok(user)
    }
}
