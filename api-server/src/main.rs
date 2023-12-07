mod schema;
mod models;

mod storage;
mod stablediffusion;

mod routes;

// use crate::routes::test_routes::get_test_routes;
use axum::{
    routing::get,
    Router,
};
use dotenvy::dotenv;
use diesel::prelude::*;
use std::env;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    MysqlConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, KFC!" }))
        .merge(crate::routes::test_routes::get_test_routes())
        .merge(crate::routes::text::get_routes())
        .merge(crate::routes::image::get_routes());

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
