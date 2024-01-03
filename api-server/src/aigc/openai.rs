use serde_json::json;
use std::env;
use reqwest;

#[derive(Debug)]
pub enum OpenAIError {
    ReqwestError(reqwest::Error),
    Error(String),
}

pub async fn request(
    model_name: &str,
    system_prompt: &str,
    prompt: &str,
    temprature: f32,
    json: bool,
) -> Result<String, OpenAIError> {
    let openai_token = env::var("OPENAI_TOKEN").unwrap();
    let version = env::var("OPENAI_API_VERSION").unwrap();
    let endpoint = env::var("OPENAI_ENDPOINT").unwrap();

    let response_format = if json {
        "json_object"
    } else {
        "text"
    };

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
        "response_format": {
            "type": response_format
        },
        "max_tokens": 800,
        "temperature": temprature,
        "frequency_penalty": 0,
        "presence_penalty": 0,
        "top_p": 0.95,
        "stop": null
    });

    let t = chrono::Utc::now().naive_utc();
    let res = match reqwest::Client::new()
        .post(url)
        .header("Content-Type", "application/json")
        .header("api-key", openai_token)
        .json(&payload)
        .send()
        .await {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("OpenAI Request Error: {:?}", e);
                return Err(OpenAIError::ReqwestError(e));
            }
        };
    let t = (chrono::Utc::now().naive_utc() - t).num_seconds();

    let result_str = res.text().await.unwrap();
    tracing::debug!("OpenAI Response ({}s): {:?}", t, &result_str);
    let json_data: serde_json::Value = serde_json::from_str(&result_str).unwrap();

    match json_data["choices"][0]["message"]["content"].as_str() {
        Some(v) => Ok(v.to_string()),
        None => {
            return Err(OpenAIError::Error(result_str));
        }
    }
}
