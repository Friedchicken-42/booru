mod database;
mod errors;
mod jwt;
mod models;
mod routes;

use axum::{
    routing::post,
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

    let db = Database::connect(url).await?.ping().await?;

    let app = Router::new()
        .nest(
            "/api/v1",
            Router::new()
                .route("/login", post(routes::user::login))
                .route(
                    "/image",
                    post(routes::image::create)
                        .delete(routes::image::delete)
                        .get(routes::image::get)
                        .patch(routes::image::update),
                )
                .route(
                    "/tag",
                    post(routes::tag::create)
                        .delete(routes::tag::delete)
                        .get(routes::tag::get),
                )
                .route("/search/image", post(routes::search::image))
                .route("/search/tag", post(routes::search::tag))
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

    TODO: Should fix get requests w/ json 

*/
