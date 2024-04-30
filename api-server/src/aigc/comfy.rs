use serde_json::json;
use futures::future::join_all;
use crate::aigc::text2prompt::GenerationParams;

pub enum ComfyError {
    ReqwestError(reqwest::Error),
    SerdeJsonError(serde_json::Error),
    Error(String),
}

#[allow(dead_code)]
async fn request_one_comfy(
    comfy_origin: &str,
    generation_params: &GenerationParams,
    batch_size: usize
) -> Result<Vec<String>, ComfyError> {

    let comfy_request_error = |e: reqwest::Error| {
        tracing::error!("Failed request comfy: {} {:?}", comfy_origin, e);
        ComfyError::ReqwestError(reqwest::Error::from(e))
    };
    let comfy_parse_error = |e: serde_json::Error| {
        tracing::error!("Failed parsing comfy response: {} {:?}", comfy_origin, e);
        ComfyError::SerdeJsonError(e)
    };

    let params = get_sdxl_base_params(generation_params, batch_size);
    let payload = json!({
        "prompt": params,
    });
    // println!("payload: {:?}", &payload);
    let client = reqwest::Client::new();
    let url = format!("{}/prompt", comfy_origin);
    let res = client.post(url).json(&payload).send().await.map_err(comfy_request_error)?;
    let res_body_text = res.text().await.map_err(comfy_request_error)?;
    // println!("body: {:?}", &res_body_text);
    let res_body_json: serde_json::Value = serde_json::from_str(&res_body_text).map_err(comfy_parse_error)?;
    let prompt_id = res_body_json["prompt_id"].as_str().ok_or_else(|| {
        tracing::error!("Failed parsing comfy response prompt_id: {} {:?}", comfy_origin, res_body_json);
        ComfyError::Error("Failed parsing comfy response prompt_id".to_owned())
    })?;
    let base64_images: Vec<String>;
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        let url = format!("{}/history/{}", comfy_origin, prompt_id);
        let res = client.get(&url).send().await.map_err(comfy_request_error)?;
        let res_body_text = res.text().await.map_err(comfy_request_error)?;
        let res_body_json: serde_json::Value = serde_json::from_str(&res_body_text).map_err(comfy_parse_error)?;
        if let Some(result) = res_body_json.get(prompt_id) {
            let images = result["outputs"]["final"]["images"].as_array().ok_or_else(|| {
                tracing::error!("Failed parsing comfy result images: {} {:?}", comfy_origin, serde_json::to_string(&result));
                ComfyError::Error("Failed parsing comfy result images".to_owned())
            })?;
            base64_images = images
                .iter()
                .map(|base64_str| {
                    base64_str.as_str().unwrap_or("").to_owned()
                })
                .collect();
            break;
        }
    }
    return Ok(base64_images);
}

pub async fn request(
    comfy_origins: &Vec<String>,
    generation_params: &GenerationParams
) -> Result<Vec<String>, ComfyError> {
    if comfy_origins.len() == 4 {
        let futures = comfy_origins.iter().map(|comfy_origin| {
            request_one_comfy(comfy_origin, generation_params, 1)
        }).collect::<Vec<_>>();
        let results = join_all(futures).await;
        let mut base64_images: Vec<String> = vec![];
        for result in results {
            match result {
                Ok(mut v) => base64_images.append(&mut v),
                Err(e) => return Err(e),
            }
        }
        Ok(base64_images)
    } else if comfy_origins.len() == 2 {
        let futures = comfy_origins.iter().map(|comfy_origin| {
            request_one_comfy(comfy_origin, generation_params, 2)
        }).collect::<Vec<_>>();
        let results = join_all(futures).await;
        let mut base64_images: Vec<String> = vec![];
        for result in results {
            match result {
                Ok(mut v) => base64_images.append(&mut v),
                Err(e) => return Err(e),
            }
        }
        Ok(base64_images)
    } else if comfy_origins.len() == 1 {
        let comfy_origin = comfy_origins[0].as_str();
        let batch_size = 4;
        let base64_images = request_one_comfy(comfy_origin, generation_params, batch_size).await?;
        Ok(base64_images)
    } else {
        panic!("comfy_origins.len() must be 1 or 4");
    }
}

async fn _fetch_images(images_urls: Vec<String>) -> Vec<String> {
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

const COMFY_API_TPL_SDXL_TURBO: &'static str = include_str!("./workflows/sdxl_turbo.json");
#[allow(dead_code)]
fn get_sdxl_turbo_params(generation_params: &GenerationParams, batch_size: usize) -> serde_json::Value {
    let mut params: serde_json::Value = serde_json::from_str(COMFY_API_TPL_SDXL_TURBO).unwrap();
    params["5"]["inputs"]["batch_size"] = serde_json::Value::from(batch_size);
    params["6"]["inputs"]["text"] = json!(generation_params.positive);
    params["7"]["inputs"]["text"] = json!(generation_params.negative);
    params["13"]["inputs"]["noise_seed"] = serde_json::Value::from(rand::random::<u32>());
    return params;
}

const COMFY_API_TPL_SDXL_LCM: &'static str = include_str!("./workflows/sdxl_lcm_lora.json");
#[allow(dead_code)]
fn get_sdxl_lcm_params(generation_params: &GenerationParams, batch_size: usize) -> serde_json::Value {
    let mut params: serde_json::Value = serde_json::from_str(COMFY_API_TPL_SDXL_LCM).unwrap();
    params["22"]["inputs"]["batch_size"] = serde_json::Value::from(batch_size);
    params["22"]["inputs"]["positive"] = json!(generation_params.positive);
    params["22"]["inputs"]["negative"] = json!(generation_params.negative);
    params["31"]["inputs"]["noise_seed"] = serde_json::Value::from(rand::random::<u32>());
    params["33"]["inputs"]["noise_seed"] = serde_json::Value::from(rand::random::<u32>());
    return params;
}

const COMFY_API_TPL_SDXL_BASE64: &'static str = include_str!("./workflows/sdxl_base_base64.json");
#[allow(dead_code)]
fn get_sdxl_base_params(generation_params: &GenerationParams, batch_size: usize) -> serde_json::Value {
    let mut params: serde_json::Value = serde_json::from_str(COMFY_API_TPL_SDXL_BASE64).unwrap();
    params["22"]["inputs"]["batch_size"] = serde_json::Value::from(batch_size);
    params["22"]["inputs"]["positive"] = json!(generation_params.positive);
    params["22"]["inputs"]["negative"] = json!(generation_params.negative);
    params["39"]["inputs"]["noise_seed"] = serde_json::Value::from(rand::random::<u32>());
    return params;
}
