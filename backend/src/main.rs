mod database;
mod errors;
mod jwt;
mod models;
mod routes;

use axum::{
    routing::{post, put},
    Router, Server,
};
use database::Database;
use dotenv::dotenv;
use errors::Error;
use std::{env, net::SocketAddr};

use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();
    let url = env::var("DATABASE_URL").map_err(|_| Error::DatabaseConnection)?;

    let db = Database::new(url)
        .await?
        .signin("root", "root")
        .await?
        .connect("booru", "booru")
        .await?;

    let app = Router::new()
        .nest(
            "/api/v1",
            Router::new()
                .route("/login", post(routes::user::login))
                .route(
                    "/image",
                    put(routes::image::create)
                        .post(routes::image::post)
                        .delete(routes::image::delete)
                        .patch(routes::image::update)
                )
                .route(
                    "/tag",
                    put(routes::tag::create)
                        .post(routes::tag::post)
                        .delete(routes::tag::delete)
                )
                .route("/search/image", post(routes::search::image))
                .route("/search/tag", post(routes::search::tag)),
        )
        .layer(CorsLayer::very_permissive())
        .with_state(db);

    let addr = SocketAddr::from(([127, 0, 0, 1], 5000));

    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .map_err(|_| Error::ServerCreate)?;

    Ok(())
}

/* Routes

   [ ] post signup
   [-] post login
   [x] post/delete/get/patch image
   [x] post/delete/get tag
   [ ] post search tag            (autocomplete)
   [x] post search image          (search with tags)
     - post: search {include: [{name: a, category: b}], exclude: []}

    get image by id
     - get : /image/:id
     - get : /image?id

*/
