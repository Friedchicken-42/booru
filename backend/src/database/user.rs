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
    pub async fn authenticate(&self, user: User) -> Result<User, Error> {
        if self.contains(&user.name).await? {
            return Err(Error::UserNotFound);
        }

        // hmmmm
        let query = format!(
            "select * from user where name = '{}' and password = '{}'",
            user.name, user.password
        );

        let mut res = self.0.query(query).await?;

        let user: Option<User> = res.take(0)?;

        user.ok_or(Error::WrongCredential)
    }
}
