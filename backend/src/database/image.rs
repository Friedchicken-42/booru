use surrealdb::{engine::remote::ws::Client, Surreal};

use crate::{errors::Error, models::{image::Image, tag::Tag, taggedimage::TaggedImage}};

use super::Session;

#[derive(Clone)]
pub struct ImageDB(pub Surreal<Client>);

impl ImageDB {
    pub async fn create(&self, image: &Image) -> Result<(), Error> {
        let _: Image = self.0
            .create(("image", image.hash.clone()))
            .content(image)
            .await?;

        Ok(())
    }

    pub async fn get(&self, hash: &String) -> Result<Option<Image>, Error> {
        Ok(self.0.select(("image", hash.to_owned())).await?)
    }

    pub async fn delete(&self, image: Image) -> Result<(), Error> {
        let id = image.id.ok_or(Error::ImageNotFound)?;
        let (_, id) = id.split_at(6);
        self.0.delete(("image", id)).await?;

        Ok(())
    }

    pub async fn tagged(&self, image: Image) -> Result<TaggedImage, Error> {
        let id = image.id.ok_or(Error::ImageNotFound)?;

        let mut res = self.0.query("select *, ->tagged->tag.* as tags from $image")
            .bind(("image", id))
            .await?;

        let image: Option<TaggedImage> = res.take(0)?;

        image.ok_or(Error::InvalidId)
    }

    pub fn relate<'a>(&self, image: &Image, tag: &Tag, session: Session<'a>) -> Result<Session<'a>, Error> {
        let image_id = image.id.clone().ok_or(Error::InvalidId)?;
        let tag_id = tag.id.clone().ok_or(Error::InvalidId)?;

        let query = format!("relate {}->tagged->{};", image_id, tag_id);
        let s = session.query(query);

        Ok(s)
    }
    
    pub fn unrelate<'a>(&self, image: &Image, tag: &Tag, session: Session<'a>) -> Result<Session<'a>, Error> {
        let image_id = image.id.clone().ok_or(Error::InvalidId)?;
        let tag_id = tag.id.clone().ok_or(Error::InvalidId)?;

        let query = format!("delete tagged where in = {} and out = {};", image_id, tag_id);
        let s = session.query(query);

        Ok(s)
    }
}
