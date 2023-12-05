use std::env;

#[allow(dead_code)]
pub async fn request(prompt: &str) -> Vec<String> {
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
