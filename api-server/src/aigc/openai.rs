use serde_json::json;
use std::env;
use reqwest;

#[derive(Debug)]
pub enum OpenAIError {
    ReqwestError(reqwest::Error),
    Error(String),
}

pub async fn request(
    system_prompt: &str,
    prompt: &str,
    temprature: f32,
) -> Result<String, OpenAIError> {
    let openai_token = env::var("OPENAI_TOKEN").unwrap();
    let model_name = "gpt-4";
    let version = "2023-12-01-preview";
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
        "max_tokens": 800,
        "temperature": temprature,
        "frequency_penalty": 0,
        "presence_penalty": 0,
        "top_p": 0.95,
        "stop": null
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
                return Err(OpenAIError::ReqwestError(e));
            }
        };

    let result_str = res.text().await.unwrap();
    let json_data: serde_json::Value = serde_json::from_str(&result_str).unwrap();

    match json_data["choices"][0]["message"]["content"].as_str() {
        Some(v) => Ok(v.to_string()),
        None => {
            return Err(OpenAIError::Error(result_str));
        }
    }
}
