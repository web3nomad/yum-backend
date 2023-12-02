mod routes;
// use crate::routes::test_routes::get_test_routes;
use axum::Router;
use dotenv::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();
    // build our application with a single route
    let app = Router::new()
        .merge(crate::routes::test_routes::get_test_routes())
        .merge(crate::routes::text::get_routes());

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
