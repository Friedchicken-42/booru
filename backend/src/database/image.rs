use futures::future::try_join_all;
use surrealdb::{engine::remote::ws::Client, Surreal};

use crate::{
    errors::Error,
    models::{image::Image, tag::Tag, user::User},
    pattern::Pattern,
};

use super::{Database, Session};

pub struct ImageDB<'a> {
    pub client: &'a Surreal<Client>,
    pub db: &'a Database,
}

impl<'a> ImageDB<'a> {
    pub async fn create(&self, image: &Image) -> Result<Image, Error> {
        let image: Image = self
            .client
            .create(("image", image.hash.clone()))
            .content(image)
            .await?;

        Ok(image)
    }

    pub async fn get(&self, hash: &String) -> Result<Option<Image>, Error> {
        Ok(self.client.select(("image", hash.to_owned())).await?)
    }

    pub async fn delete(&self, image: Image) -> Result<(), Error> {
        let id = image.id.ok_or(Error::ImageNotFound)?;
        let (_, id) = id.split_at(6);
        self.client.delete(("image", id)).await?;

        Ok(())
    }

    pub async fn search(
        &self,
        pattern: Option<Pattern<Tag>>,
        previous: Option<Image>,
    ) -> Result<Vec<Image>, Error> {
        // TODO: add limit
        let mut query =
            String::from("select * from (select *, ->tagged->tag.*.id as tag from image)");

        if let Some(p) = pattern {
            query = format!("{} where {}", query, p.serialize("tag"));
        }

        let mut res = self.client.query(query).await?;
        let images: Vec<Image> = res.take(0)?;

        try_join_all(images.into_iter().map(|image| self.tagged(image))).await
    }

    pub async fn tagged(&self, image: Image) -> Result<Image, Error> {
        let tags = self.db.tag().from_image(&image).await?;
        let user = self.db.user().from_image(&image).await?;

        Ok(Image::tagged(image, tags, user))
    }

    pub async fn user(&self, image: &Image, user: &User) -> Result<(), Error> {
        let image_id = image.id.clone().ok_or(Error::InvalidId)?;
        let user_id = user.id.clone().ok_or(Error::InvalidId)?;

        let query = format!("relate {}->upload->{};", user_id, image_id);
        self.client.query(query).await?;

        Ok(())
    }

    pub fn tag<'b>(
        &self,
        image: &Image,
        tag: &Tag,
        session: Session<'b>,
    ) -> Result<Session<'b>, Error> {
        let image_id = image.id.clone().ok_or(Error::InvalidId)?;
        let tag_id = tag.id.clone().ok_or(Error::InvalidId)?;

        let query = format!("relate {}->tagged->{};", image_id, tag_id);
        let s = session.query(query);

        Ok(s)
    }

    pub fn untag<'b>(
        &self,
        image: &Image,
        tag: &Tag,
        session: Session<'b>,
    ) -> Result<Session<'b>, Error> {
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
