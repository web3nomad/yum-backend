use chrono;
use rand;
use axum::{
    // routing::get,
    routing::post,
    Router,
};

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new().route("/", post(handler));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler(body: String) -> &'static str {
    tokio::spawn(async move {
        let n = rand::random::<u8>() % 5 + 1;
        tokio::time::sleep(tokio::time::Duration::from_secs(n.into())).await;
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        println!("{} Hello, World Done! {}", timestamp, body);
    });
    return "Hello, World!"
}
