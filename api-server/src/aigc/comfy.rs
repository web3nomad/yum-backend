use serde_json::json;
use futures::future::join_all;
use std::env;

pub enum ComfyError {
    ReqwestError(reqwest::Error),
    SerdeJsonError(serde_json::Error),
}

async fn fetch_images(images_urls: Vec<String>) -> Vec<String> {
    async fn fetch(image_url: &str) -> String {
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

#[allow(dead_code)]
pub async fn request(prompt: &str) -> Result<Vec<String>, ComfyError> {
    let comfy_origin = env::var("COMFYUI_TEST_ORIGIN").unwrap();
    // let mut params: serde_json::Value = serde_json::from_str(COMFY_API_TPL_SDXL_TURBO).unwrap();
    // params["6"]["inputs"]["text"] = json!(prompt);
    let mut params: serde_json::Value = serde_json::from_str(COMFY_API_TPL_SDXL).unwrap();
    params["22"]["inputs"]["positive"] = json!(prompt);
    let payload = json!({
        "prompt": params,
    });
    let client = reqwest::Client::new();
    let url = format!("{}/prompt", comfy_origin);
    let res = match client.post(url).json(&payload).send().await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Error: {:?}", e);
            return Err(ComfyError::ReqwestError(e));
        }
    };
    let res_body_text = res.text().await.unwrap();
    // println!("body: {:?}", &res_body_text);
    let res_body_json: serde_json::Value = match serde_json::from_str(&res_body_text) {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Error: {:?}", e);
            return Err(ComfyError::SerdeJsonError(e));
        }
    };
    let prompt_id = res_body_json["prompt_id"].as_str().unwrap();
    let base64_images: Vec<String>;
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        let url = format!("{}/history/{}", comfy_origin, prompt_id);
        let res = match client.get(&url).send().await {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("Error: {:?}", e);
                return Err(ComfyError::ReqwestError(e));
            }
        };
        let res_body_text = res.text().await.unwrap();
        let res_body_json: serde_json::Value = serde_json::from_str(&res_body_text).unwrap();
        if let Some(result) = res_body_json.get(prompt_id) {
            let images_urls = result["outputs"]["final"]["images"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| {
                    let filename = v["filename"].as_str().unwrap();
                    format!("{}/view?filename={}&subfolder=&type=output", comfy_origin, filename)
                })
                .collect();
            base64_images = fetch_images(images_urls).await;
            break;
        }
    }
    return Ok(base64_images);
}

#[allow(dead_code)]
const COMFY_API_TPL_SDXL_TURBO: &'static str = include_str!("./workflows/sdxl_turbo.json");

#[allow(dead_code)]
const COMFY_API_TPL_SDXL: &'static str = include_str!("./workflows/sdxl_base.json");
