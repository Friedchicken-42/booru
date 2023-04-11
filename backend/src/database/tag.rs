use surrealdb::{engine::remote::ws::Client, Surreal};

use crate::{
    errors::Error,
    models::{tag::Tag, user::User},
};

use super::{Database, Session};

pub struct TagDB<'a> {
    pub client: &'a Surreal<Client>,
    pub db: &'a Database,
}

impl<'a> TagDB<'a> {
    pub async fn create(&self, tag: &Tag) -> Result<Tag, Error> {
        let tag: Tag = self.client.create("tag").content(tag).await?;
        Ok(tag)
    }

    pub async fn get(&self, name: &String, category: &String) -> Result<Option<Tag>, Error> {
        let mut res = self
            .client
            .query("select *, <-upload<-user.name as user from tag where name = $name and category = $category")
            .bind(("name", name))
            .bind(("category", category))
            .await?;

        Ok(res.take(0)?)
    }

    pub async fn search(&self, category: &String, name: &String) -> Result<Vec<Tag>, Error> {
        let query = format!(
            "select * from tag where category = /^{}/ and name = /^{}/;",
            category, name
        );

        let mut res = self.client.query(query).await?;

        Ok(res.take(0)?)
    }

    pub async fn user(&self, tag: &Tag, user: &User) -> Result<Tag, Error> {
        let tag_id = tag.id.clone().ok_or(Error::InvalidId)?;
        let user_id = user.id.clone().ok_or(Error::InvalidId)?;

        let query = format!("relate {}->upload->{};", user_id, tag_id);
        self.client.query(query).await?;

        self.get(&tag.name, &tag.category)
            .await?
            .ok_or(Error::TagNotFound)
    }

    pub async fn delete(&self, tag: Tag) -> Result<(), Error> {
        let tag = self
            .get(&tag.name, &tag.category)
            .await?
            .ok_or(Error::TagNotFound)?;

        let id = tag.id.ok_or(Error::TagNotFound)?;
        let (_, id) = id.split_at(4);

        self.client.delete(("tag", id)).await?;

        Ok(())
    }

    pub fn update<'b>(
        &self,
        tag: &Tag,
        offset: i32,
        session: Session<'b>,
    ) -> Result<Session<'b>, Error> {
        let id = tag.id.clone().ok_or(Error::InvalidId)?;

        let query = format!("update {} set count += {};", id, offset);
        let s = session.query(query);

        Ok(s)
    }
}
