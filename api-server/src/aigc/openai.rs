use serde_json::json;
use std::env;
use reqwest;

pub async fn request(system_prompt: &str, prompt: &str) -> Result<String, reqwest::Error> {
    let openai_token = env::var("OPENAI_TOKEN").unwrap();
    let model_name = "gpt-4";
    let version = "2023-07-01-preview";
    let endpoint = "museai1";

    let messages = json!([{
        "role": "system",
        "content": system_prompt
    }, {
        "role": "user",
        "content": prompt
    }]);

    let url = format!(
        "https://{}.openai.azure.com/openai/deployments/{}/chat/completions?api-version={}",
        endpoint, model_name, version);
    let payload = json!({
        "messages": messages,
        "temperature": 1,
    });
    let res = match reqwest::Client::new()
        .post(url)
        .header("Content-Type", "application/json")
        .header("api-key", openai_token)
        .json(&payload)
        .send()
        .await {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("Error: {:?}", e);
                return Err(e);
            }
        };

    let result_str = res.text().await.unwrap();
    tracing::info!("openai res: {:?}", result_str);
    let json_data: serde_json::Value = serde_json::from_str(&result_str).unwrap();
    let message = json_data["choices"][0]["message"]["content"].as_str().unwrap().to_string();
    return Ok(message);
}
