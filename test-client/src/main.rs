use chrono;
use futures::future::join_all;
use axum::{
    routing::post,
    response::Json,
    Router,
};

async fn fetch_images(images_urls: Vec<&str>) -> Vec<String> {
    async fn fetch(image_url: &str) -> String {
        if image_url == "" {
            return String::from("");
        }
        // fetch image file from image_url and convert to base64
        let response = reqwest::get(image_url).await.unwrap();
        let image_bytes = response.bytes().await.unwrap();
        // println!("image_url: {:?}", image_url);
        return data_encoding::BASE64.encode(&image_bytes);
    }
    let base64_images = images_urls
        .iter()
        .map(|url| fetch(url))
        .collect::<Vec<_>>();
    return join_all(base64_images).await;
}

async fn handle_callback(body: String) {
    let body_json = serde_json::from_str::<serde_json::Value>(&body).unwrap();
    let user_prompt: &str = body_json["params"]["prompt"].as_str().unwrap();
    let generation_prompt = body_json["generationParams"]["prompt"].as_str().unwrap();
    let generation_negative_prompt = body_json["generationParams"]["negative_prompt"].as_str().unwrap();
    if generation_prompt == "" {
        return;
    }
    let images_urls: Vec<&str> = body_json["result"]["images"]
        .as_array().unwrap()
        .iter().map(|image| image["src"].as_str().unwrap())
        .collect::<Vec<&str>>();
    let theme = body_json["result"]["theme"].as_str().unwrap();
    let base64_images = fetch_images(images_urls).await;
    base64_images.iter().enumerate().for_each(|(index, base64_image)| {
        let image_bytes = data_encoding::BASE64.decode(base64_image.as_bytes()).unwrap();
        let image_path = format!("test-client/output/{} {} {}.jpeg", user_prompt, theme, index);
        std::fs::write(image_path, &image_bytes).unwrap();
    });
    let text_content = format!(
        "Prompt:\n{}\n\nNegativePrompt:\n{}\n", generation_prompt, generation_negative_prompt);
    let image_path = format!("test-client/output/{} {}.txt", user_prompt, theme);
    std::fs::write(image_path, text_content).unwrap();
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/callback", post(|body: String| async move {
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
            println!("{} - Received: {}", timestamp, body);
            handle_callback(body).await;
            Json(serde_json::json!({
                "success": true,
            }))
        }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
