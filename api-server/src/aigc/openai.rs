use serde_json::json;
use std::{env, fmt::Display};
use reqwest;

#[derive(Debug)]
pub enum OpenAIError {
    ReqwestError(reqwest::Error),
    Filtered(String),
    Error(String),
}

impl Display for OpenAIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpenAIError::ReqwestError(e) => write!(f, "OpenAIError::ReqwestError: {:?}", e),
            OpenAIError::Filtered(e) => write!(f, "OpenAIError::Filtered: {:?}", e),
            OpenAIError::Error(e) => write!(f, "OpenAIError::Error: {:?}", e),
        }
    }
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
    let json_object = env::var("OPENAI_JSON_OBJECT").unwrap();
    let deployment_name = {
        let deployments = env::var("OPENAI_DEPLOYMENTS").unwrap()
            .split(",").map(|s| s.to_string()).collect::<Vec<_>>();
        match model_name {
            "gpt-3.5" => deployments[0].to_string(),
            "gpt-4" => deployments[1].to_string(),
            _ => model_name.to_string(),
        }
    };

    let response_format = if json && json_object == "true" {
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
        endpoint, deployment_name, version);
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
    let res = reqwest::Client::new()
        .post(url)
        .header("Content-Type", "application/json")
        .header("api-key", openai_token)
        .json(&payload)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Failed request OpenAI: {:?}", e);
            OpenAIError::ReqwestError(e)
        })?;
    let t = (chrono::Utc::now().naive_utc() - t).num_seconds();

    let result_str = res.text().await.map_err(|e| {
        tracing::error!("Failed reading OpenAI response: {:?}", e);
        OpenAIError::Error(format!("Failed reading OpenAI response: {:?}", e))
    })?;

    tracing::debug!("OpenAI Response ({}s): {}", t, &result_str);

    let json_data: serde_json::Value = serde_json::from_str(&result_str).map_err(|e| {
        tracing::error!("Failed parsing OpenAI response: {:?}", e);
        OpenAIError::Error(format!("Failed parsing OpenAI response: {:?}", e))
    })?;

    if let Some(reason) = json_data["choices"][0]["finish_reason"].as_str() {
        if reason == "content_filter" {
            let msg = format!("OpenAI content filter triggered: {}", prompt);
            tracing::warn!("{}", msg);
            return Err(OpenAIError::Filtered(msg));
        }
    };

    if let Some(code) = json_data["error"]["code"].as_str() {
        if code == "content_filter" {
            let msg = format!("OpenAI content filter triggered: {}", prompt);
            tracing::warn!("{}", msg);
            return Err(OpenAIError::Filtered(msg));
        }
    };

    match json_data["choices"][0]["message"]["content"].as_str() {
        Some(v) => Ok(v.to_string()),
        None => {
            let msg = format!(
                r#"Failed reading json_data["choices"][0]["message"]["content"] in OpenAI response: {:?}"#,
                json_data
            );
            tracing::error!("{}", msg);
            return Err(OpenAIError::Error(msg));
        }
    }
}
