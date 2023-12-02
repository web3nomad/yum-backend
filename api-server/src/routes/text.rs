use serde_json::json;
use std::env;
use reqwest;
use axum::{
    routing::post,
    Router,
};

async fn handler(body: String) -> String {
    // println!("Send: {}", body);
    // format!("Hello World {}\n", &body)
    let openai_token = env::var("OPENAI_TOKEN").unwrap();
    let model_name = "gpt-4";
    let version = "2023-07-01-preview";
    let endpoint = "museai1";

    let json_body: serde_json::Value = serde_json::from_str(&body).unwrap();
    let prompt = json_body["params"]["prompt"].as_str().unwrap();
    let messages = json!([{
        "role": "system",
        "content": "你是一个 KFC 的美食专家，你的任务是根据需求，生成一个创意的 KFC 商品的名字，不超过 10 个中文汉字。直接输出名字，不要包含任何标点符号。"
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
    let res = reqwest::Client::new()
        .post(url)
        .header("Content-Type", "application/json")
        .header("api-key", openai_token)
        .json(&payload)
        .send()
        .await
        .unwrap();
    // println!("status = {}", res.status());
    let result_str = res.text().await.unwrap();
    println!("res = {:?}", result_str);
    let json_data: serde_json::Value = serde_json::from_str(&result_str).unwrap();
    // println!("json_data = {:?}", json_data["choices"][0]["message"]["content"]);
    let message = json_data["choices"][0]["message"]["content"].as_str().unwrap().to_string();
    message
}

pub fn get_routes() -> Router {
    let router: Router = Router::new().route("/api/yum/generate/text", post({
        |body: String| handler(body)
    }));
    return router;
}
