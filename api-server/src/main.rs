mod schema;
mod models;

mod storage;
mod aigc;

mod routes;

use axum::{
    routing::get,
    Router,
};
use dotenvy::dotenv;
use diesel::prelude::*;
use std::env;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};


#[tokio::main]
async fn main() {
    dotenv().ok();
    init_tracing();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    MysqlConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, KFC!" }))
        .merge(crate::routes::text::get_routes())
        .merge(crate::routes::image::get_routes());

    let port = env::var("PORT").unwrap_or_else(|_| String::from("3000"));
    let host = env::var("HOST").unwrap_or_else(|_| String::from("0.0.0.0"));
    let socket_addr: std::net::SocketAddr = format!("{}:{}", host, port).parse().unwrap();
    tracing::info!("listening on http://{}", socket_addr);
    axum::Server::bind(&socket_addr).serve(app.into_make_service()).await.unwrap();
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            // load filters from the `RUST_LOG` environment variable.
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "api_server=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_ansi(false))
        .init();
}
