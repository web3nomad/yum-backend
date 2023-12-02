mod routes;
use crate::routes::test_routes::get_test_routes;

use axum::Router;

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new()
        .merge(get_test_routes());

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
