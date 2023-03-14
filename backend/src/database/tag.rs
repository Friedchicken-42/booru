use futures::future::try_join_all;
use serde::Deserialize;
use surrealdb::{engine::remote::ws::Client, Surreal};

use crate::{
    errors::Error,
    models::{image::Image, tag::Tag, tagresponse::TagResponse},
};

use super::Session;

#[derive(Clone)]
pub struct TagDB(pub Surreal<Client>);

impl TagDB {
    pub async fn create(self, tag: &Tag) -> Result<(), Error> {
        let _: Tag = self.0.create("tag").content(tag).await?;
        Ok(())
    }

    pub async fn get(&self, name: &String, category: &String) -> Result<Option<Tag>, Error> {
        let mut res = self
            .0
            .query("select * from tag where name = $name and category = $category")
            .bind(("name", name))
            .bind(("category", category))
            .await?;

        Ok(res.take(0)?)
    }
    
    pub async fn search(&self, category: &String, name: &String) -> Result<Vec<Tag>, Error> {
        let query = format!("select * from tag where category = /^{}/ and name = /^{}/;", category, name);

        let mut res = self
            .0
            .query(query)
            .await?;

        Ok(res.take(0)?)        
    }

    pub async fn convert(&self, tags: Vec<TagResponse>) -> Result<Vec<Tag>, Error> {
        let tags = try_join_all(tags.iter().map(|t| self.get(&t.name, &t.category))).await?;
        let tags = tags.into_iter().collect::<Option<Vec<Tag>>>();

        tags.ok_or(Error::TagNotFound)
    }

    pub async fn from_image(&self, image: &Image) -> Result<Vec<Tag>, Error> {
        let id = image.id.clone().ok_or(Error::ImageNotFound)?;

        let mut res = self
            .0
            .query("select ->tagged->tag.* as tagged from $image")
            .bind(("image", id))
            .await?;

        #[derive(Deserialize)]
        struct Tags {
            tagged: Vec<Tag>,
        }

        let tags: Option<Tags> = res.take(0)?;

        match tags {
            Some(tags) => Ok(tags.tagged),
            None => Ok(vec![]),
        }
    }

    pub async fn delete(&self, tag: Tag) -> Result<(), Error> {
        let tag = self
            .get(&tag.name, &tag.category)
            .await?
            .ok_or(Error::TagNotFound)?;

        let id = tag.id.ok_or(Error::TagNotFound)?;
        let (_, id) = id.split_at(4);

        self.0.delete(("tag", id)).await?;

        Ok(())
    }

    pub fn update<'a>(
        &self,
        tag: &Tag,
        offset: i32,
        session: Session<'a>,
    ) -> Result<Session<'a>, Error> {
        let id = tag.id.clone().ok_or(Error::InvalidId)?;

        let query = format!("update {} set count += {};", id, offset);
        let s = session.query(query);

        Ok(s)
    }
}
