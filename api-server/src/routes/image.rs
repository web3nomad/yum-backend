use serde_json::json;
use std::env;
// use reqwest;
use std::sync::Arc;
use tokio::sync::{
    // watch,
    mpsc,
};
use axum::{
    routing::post,
    response::{Json, IntoResponse, Response},
    Router,
    // body::Body,
    http::StatusCode,
};

struct BadRequest {
    message: String,
}

impl IntoResponse for BadRequest {
    fn into_response(self) -> Response {
        let body = format!("Bad request: {}", self.message);
        let status = StatusCode::BAD_REQUEST;
        (status, body).into_response()
    }
}

async fn handler(body: String, tx: Arc<mpsc::Sender<String>>) -> Result<Json<serde_json::Value>, BadRequest> {
    if let Ok(()) = tx.try_send(body) {
        let res_json = json!({
            "taskId": "fed8d585-4ca7-4cda-a41e-16bb9a7c93c3"
        });
        return Ok(Json(res_json));
    } else {
        // let res_json = json!({
        //     "success": false
        // });
        return Err(BadRequest { message: "Queue is full".to_string() });
    }
}

async fn request_webui(prompt: &str, timestamp: &str) {
    let webui_origin = env::var("SD_WEBUI_TEST_ORIGIN").unwrap();
    let api_auth = Some(sdwebuiapi::OpenApiV1Auth {
        username: env::var("SD_WEBUI_TEST_AUTH_USERNAME").unwrap(),
        password: env::var("SD_WEBUI_TEST_AUTH_PASSWORD").unwrap(),
    });
    let mut payload = sdwebuiapi::TextToImagePayload {
        prompt: prompt.to_string(),
        ..Default::default()
    };
    payload.set_base_model("DreamShaper_6_BakedVae.safetensors [b76cc78ad9]");
    let client = sdwebuiapi::Client::new(&webui_origin, api_auth);
    let response = client.txt2img(payload).await;
    let raw_b64_str = &response.images[0];
    // TODO: remove this debug code in production
    let output_image = data_encoding::BASE64.decode(raw_b64_str.as_bytes()).unwrap();
    std::fs::create_dir_all("test-client/output").unwrap();
    std::fs::write(format!("test-client/output/{}.png", timestamp), output_image).unwrap();
}

pub fn get_routes() -> Router {
    let (tx, mut rx) = mpsc::channel::<String>(2);

    let tx = Arc::new(tx);

    tokio::spawn(async move {
        while let Some(body) = rx.recv().await {
            // let n = rand::random::<u8>() % 5 + 1;
            // tokio::time::sleep(tokio::time::Duration::from_secs(n.into())).await;
            // let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
            // println!("{} Done! {}", timestamp, body);
            let body_json = serde_json::from_str::<serde_json::Value>(&body).unwrap();
            let prompt = body_json["params"]["prompt"].as_str().unwrap();
            let timestamp = body_json["timestamp"].as_str().unwrap();
            println!("Start! {}", timestamp);
            request_webui(prompt, timestamp).await;
            println!("End! {}", timestamp);
        }
    });

    let router: Router = Router::new()
        .route("/api/yum/generate/image", post({
            let tx = Arc::clone(&tx);
            |body: String| handler(body, tx)
        }));
    return router;
}
