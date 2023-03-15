use surrealdb::{engine::remote::ws::Client, Surreal};

use crate::errors::Error;
use crate::models::user::User;

#[derive(Clone)]
pub struct UserDB(pub Surreal<Client>);

impl UserDB {
    pub async fn authenticate(&self, user: User) -> Result<User, Error> {
        // hmmmm
        let query = format!(
            "select * from user where name = '{}' and password = '{}'",
            user.name, user.password
        );

        let mut res = self.0.query(query).await?;

        let user: Option<User> = res.take(0)?;

        user.ok_or(Error::UserNotFound)
    }
}
