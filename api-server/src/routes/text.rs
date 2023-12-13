use serde_json::json;
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
    let json_body: serde_json::Value = serde_json::from_str(&body).unwrap();
    let params = &json_body["params"];
    let prompt = json_body["params"]["prompt"].as_str().unwrap();
    let message = match crate::aigc::openai::request(
        &SYSTEM_PROMPT, prompt, 1.0, false
    ).await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Error: {:?}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, "Server Error").into_response());
        }
    };
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
