use mongodb::{bson::doc, options::ClientOptions, Client};

use crate::errors::Error;

use self::{image::Images, tag::Tags, user::Users};

pub mod image;
pub mod tag;
pub mod user;

#[derive(Clone)]
pub struct Database {
    client: Client,
    pub user: Users,
    pub image: Images,
    pub tag: Tags,
}

impl Database {
    pub async fn connect(url: String) -> Result<Self, Error> {
        let mut options = ClientOptions::parse(url)
            .await
            .map_err(|_| Error::DatabaseConnection)?;

        options.app_name = Some("Booru".to_string());

        let client = Client::with_options(options).map_err(|_| Error::DatabaseConnection)?;

        let db = client.database("booru");
        let user = Users::new(&db, client.clone());
        let image = Images::new(&db, client.clone());
        let tag = Tags::new(&db, client.clone());

        Ok(Database {
            client,
            user,
            image,
            tag,
        })
    }

    pub async fn ping(self) -> Result<Self, Error> {
        self.client
            .database("admin")
            .run_command(doc! {"ping": 1}, None)
            .await
            .map_err(|_| Error::DatabaseConnection)?;

        Ok(self)
    }
}
