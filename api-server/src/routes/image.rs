use serde_json::json;
use std::env;
use std::sync::Arc;
use tokio::sync::mpsc;
use axum::{
    routing::post,
    response::{Json, IntoResponse, Response},
    Router,
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

async fn handler_generate_image(
    body: String,
    tx: Arc<mpsc::Sender<String>>
) -> Result<Json<serde_json::Value>, BadRequest> {
    if let Ok(()) = tx.try_send(body) {
        let res_json = json!({
            "taskId": "fed8d585-4ca7-4cda-a41e-16bb9a7c93c3"
        });
        return Ok(Json(res_json));
    } else {
        return Err(BadRequest { message: "Queue is full".to_string() });
    }
}

async fn request_webui(params: &serde_json::Value) -> Vec<String> {
    let prompt = params["prompt"].as_str().unwrap();
    let webui_origin = env::var("SD_WEBUI_TEST_ORIGIN").unwrap();
    let api_auth = Some(sdwebuiapi::OpenApiV1Auth {
        username: env::var("SD_WEBUI_TEST_AUTH_USERNAME").unwrap(),
        password: env::var("SD_WEBUI_TEST_AUTH_PASSWORD").unwrap(),
    });
    let mut payload = sdwebuiapi::TextToImagePayload {
        prompt: prompt.to_string(),
        batch_size: 4,
        ..Default::default()
    };
    payload.set_base_model("DreamShaper_6_BakedVae.safetensors [b76cc78ad9]");
    let client = sdwebuiapi::Client::new(&webui_origin, api_auth);
    let response = client.txt2img(payload).await;
    let images = response.images.iter().map(|raw_b64_str| {
        raw_b64_str.to_string()
    }).collect::<Vec<String>>();
    return images;
}

async fn handler_generate_image_callback(
    task_id: &str,
    params: &serde_json::Value,
    images: &Vec<String>,
    callback_url: &str,
) {
    let mut images_payload: Vec<String> = vec![];
    for (i, raw_b64_str) in images.iter().enumerate() {
        // let raw_b64_str = &response.images[i];
        // TODO: remove this debug code in production
        let output_image = data_encoding::BASE64.decode(raw_b64_str.as_bytes()).unwrap();
        std::fs::create_dir_all("test-client/output").unwrap();
        let filename = format!("test-client/output/{}-{}.png", task_id, i);
        std::fs::write(&filename, output_image).unwrap();
        images_payload.push(filename);
    }
    let payload = json!({
        "taskId": task_id,
        "params": params,
        "result": {
            "images": images_payload
        }
    });
    let client = reqwest::Client::new();
    client.post(callback_url).json(&payload).send().await.unwrap();
}

pub fn get_routes() -> Router {
    let (tx, mut rx) = mpsc::channel::<String>(2);

    let tx = Arc::new(tx);

    tokio::spawn(async move {
        while let Some(body) = rx.recv().await {
            let body_json = serde_json::from_str::<serde_json::Value>(&body).unwrap();
            let callback_url = body_json["resultCallbackUrl"].as_str().unwrap();
            let params = body_json["params"].clone();
            let task_id = body_json["timestamp"].as_str().unwrap();
            println!("Start! {}", task_id);
            let images = request_webui(&params).await;
            handler_generate_image_callback(
                task_id, &params, &images, callback_url
            ).await;
            println!("End! {}", task_id);
        }
    });

    let router: Router = Router::new()
        .route("/api/yum/generate/image", post({
            let tx = Arc::clone(&tx);
            |body: String| handler_generate_image(body, tx)
        }));
    return router;
}
