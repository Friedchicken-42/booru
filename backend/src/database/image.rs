use futures::future::try_join_all;
use surrealdb::{engine::remote::ws::Client, Surreal};

use crate::{
    errors::Error,
    models::{image::Image, tag::Tag, taggedimage::TaggedImage, user::User},
};

use super::Session;

#[derive(Clone)]
pub struct ImageDB(pub Surreal<Client>);

impl ImageDB {
    pub async fn create(&self, image: &Image) -> Result<Image, Error> {
        let image: Image = self
            .0
            .create(("image", image.hash.clone()))
            .content(image)
            .await?;

        Ok(image)
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

    pub async fn search(
        &self,
        include: Vec<Tag>,
        exclude: Vec<Tag>,
        previous: Option<Image>,
    ) -> Result<Vec<TaggedImage>, Error> {
        let include = include
            .into_iter()
            .map(|t| t.id)
            .collect::<Option<Vec<String>>>()
            .ok_or(Error::TagNotFound)?;

        let query = format!(
            "select * from image where {:?} allinside ->tagged->tag.*.id",
            include
        );
        let mut res = self.0.query(query).await?;
        let images: Vec<Image> = res.take(0)?;

        try_join_all(images.into_iter().map(|image| self.tagged(image))).await
    }

    pub async fn user(&self, image: &Image, user: &User) -> Result<(), Error> {
        let image_id = image.id.clone().ok_or(Error::InvalidId)?;
        let user_id = user.id.clone().ok_or(Error::InvalidId)?;

        let query = format!("relate {}->upload->{};", user_id, image_id);
        self.0.query(query).await?;

        Ok(())
    }

    pub async fn tagged(&self, image: Image) -> Result<TaggedImage, Error> {
        let id = image.id.ok_or(Error::ImageNotFound)?;

        let mut res = self
            .0
            .query("select *, ->tagged->tag.* as tags, <-upload<-user.name as user from $image")
            .bind(("image", id))
            .await?;

        let image: Option<TaggedImage> = res.take(0)?;

        image.ok_or(Error::InvalidId)
    }

    pub fn tag<'a>(
        &self,
        image: &Image,
        tag: &Tag,
        session: Session<'a>,
    ) -> Result<Session<'a>, Error> {
        let image_id = image.id.clone().ok_or(Error::InvalidId)?;
        let tag_id = tag.id.clone().ok_or(Error::InvalidId)?;

        let query = format!("relate {}->tagged->{};", image_id, tag_id);
        let s = session.query(query);

        Ok(s)
    }

    pub fn untag<'a>(
        &self,
        image: &Image,
        tag: &Tag,
        session: Session<'a>,
    ) -> Result<Session<'a>, Error> {
        let image_id = image.id.clone().ok_or(Error::InvalidId)?;
        let tag_id = tag.id.clone().ok_or(Error::InvalidId)?;

        let query = format!(
            "delete tagged where in = {} and out = {};",
            image_id, tag_id
        );
        let s = session.query(query);

        Ok(s)
    }
}
