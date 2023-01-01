mod database;
mod errors;
mod jwt;
mod models;
mod routes;

use axum::{routing::post, Router, Server};
use database::Database;
use dotenv::dotenv;
use errors::Error;
use std::{env, net::SocketAddr};

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();

    let url = env::var("DATABASE_URL").map_err(|_| Error::DatabaseConnection)?;

    let db = Database::connect(url).await?.ping().await?;

    let app = Router::new()
        .nest(
            "/api/v1",
            Router::new()
                .route("/login", post(routes::user::login))
                .route(
                    "/image",
                    post(routes::image::create).delete(routes::image::delete),
                ),
        )
        .with_state(db);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .map_err(|_| Error::ServerCreate)?;

    Ok(())
}

/* Routes

    post signup
    post login
    post/delete/get image
    post/delete/get/put tag
    get search tag            (autocomplete)
    get search image          (search with tags)
     - get : search {include: [{name: a, category: b}], exclude: []}

    get image by id
     - get : /image/:id
     - get : /image?id

*/
