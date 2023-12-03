use chrono;
use axum::{
    routing::post,
    response::Json,
    Router,
};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/callback", post(|body: String| async move {
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
            println!("{} - Received: {}", timestamp, body);
            Json(serde_json::json!({
                "success": true,
            }))
        }));

    axum::Server::bind(&"0.0.0.0:3001".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
