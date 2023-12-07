use serde_json::json;
use std::env;
use reqwest;
use axum::{
    routing::post,
    response::{Json, IntoResponse},
    Router,
    http::StatusCode,
};

const SYSTEM_PROMPT: &str = r#"
你是一个 KFC 的美食专家，擅长撰写创意的 KFC 食物的名字。
我将提供一些灵感来源，口味，和食物的类型，你的任务是：
  - 生成一个创意的 KFC 食物的名字
  - 直接输出名字，不要包含任何标点符号或者表情
  - 长度在 4 到 8 个汉字之间
如下是一个优秀文案的示例：
  - 灯影云面包
  - 绿野仙踪鸡腿堡
  - 三潭印月脆皮鸡堡
"#;

async fn handler(body: String) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    let openai_token = env::var("OPENAI_TOKEN").unwrap();
    let model_name = "gpt-4";
    let version = "2023-07-01-preview";
    let endpoint = "museai1";

    let json_body: serde_json::Value = serde_json::from_str(&body).unwrap();
    let prompt = json_body["params"]["prompt"].as_str().unwrap();
    let messages = json!([{
        "role": "system",
        "content": SYSTEM_PROMPT
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
                return Err((StatusCode::INTERNAL_SERVER_ERROR, "Server Error").into_response());
            }
        };
    // println!("status = {}", res.status());
    let result_str = res.text().await.unwrap();
    // println!("res = {:?}", result_str);
    let json_data: serde_json::Value = serde_json::from_str(&result_str).unwrap();
    // println!("json_data = {:?}", json_data["choices"][0]["message"]["content"]);
    let message = json_data["choices"][0]["message"]["content"].as_str().unwrap().to_string();

    // merge json_body to {"message": "Hello World"}
    let res_json = json!({
        "params": json_body["params"],
        "result": {
            "text": message
        }
    });
    return Ok(Json(res_json));
}

pub fn get_routes() -> Router {
    let router: Router = Router::new().route("/api/yum/generate/text", post({
        |body: String| handler(body)
    }));
    return router;
}
