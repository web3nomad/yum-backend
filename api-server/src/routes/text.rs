use serde_json::json;
use axum::{
    routing::post,
    response::{Json, IntoResponse},
    Router,
    http::StatusCode,
};

const SYSTEM_PROMPT: &str = r#"
你是一个 KFC 的美食专家，擅长撰写创意的<美食帖子分享标题>。

## 用户输入的信息格式为：
  - xxx, xxx, xxx (用户输入的<灵感来源>、<灵感配料>、<美食类别>，以逗号分隔)

## 你的任务是：
  - 根据用户输入的<灵感来源>、<灵感配料>、<美食类别>，撰写一个美食帖子分享标题。如下是一个优秀文案的示例：
    - 灯影云面包
    - 绿野仙踪鸡腿堡
    - 三潭印月脆皮鸡堡

## 需要注意的要求：
  - 使用中文输出
  - 输出结果不要超过 8 个汉字
  - 输出结果只能包含<美食帖子分享标题>，不要加任何解释说明，也不要添加任何标点符号或者表情

---

开始！请根据用户输入的<灵感来源>、<灵感配料>、<美食类别>，仅输出 <美食帖子分享标题> ！！！
"#;

async fn handler(body: String) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    let json_body: serde_json::Value = serde_json::from_str(&body).unwrap();
    let params = &json_body["params"];
    let prompt = json_body["params"]["prompt"].as_str().unwrap();
    tracing::info!("Text {} started", prompt);
    let message = match crate::aigc::openai::request(
        "gpt-35-turbo", &SYSTEM_PROMPT, prompt, 0.8, false
    ).await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Error: {:?}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, "Server Error").into_response());
        }
    };
    tracing::info!("Text {} end {}", prompt, message);
    let res_json = json!({
        "params": params,
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
