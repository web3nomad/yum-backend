use serde_json::json;
use serde::Serialize;
use std::env;
use std::sync::Arc;
use tokio::sync::mpsc;
use axum::{
    routing::{post, get},
    response::{Json, IntoResponse, Response},
    Router,
    http::StatusCode,
};
use diesel::prelude::*;
use crate::schema::tasks;
use crate::models::NewTask;
// use crate::models::Task;

fn establish_connection() -> MysqlConnection {
    let database_url = env::var("DATABASE_URL").unwrap();
    MysqlConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

#[derive(Serialize)]
struct GenerationParams {
    prompt: String,
    negative_prompt: String,
}

struct TaskPayload {
    task_id: String,
    params: serde_json::Value,
    generation_params: GenerationParams,
    callback_url: String,
}

struct BadRequest {
    message: String,
}

impl IntoResponse for BadRequest {
    fn into_response(self) -> Response {
        // let body = format!("Bad request: {}", self.message);
        let body = Json(json!({
            "error": self.message
        }));
        let status = StatusCode::BAD_REQUEST;
        (status, body).into_response()
    }
}

const SYSTEM_PROMPT: &str = r#"
你是一个 KFC 的美食专家，擅长编写 Stable Diffusion 的 prompt 来生成 KFC 的食物图片。
我将提供一些灵感来源，口味，和食物的类型，你的任务是：
  - 生成一段可以生成创意 KFC 食物的 Stable Diffusion prompt
  - 直接输出 prompt
如下是一个优秀文案的示例：
Food photography style.
Chicken popcorn coated with a black Oreo-style crumb mixture.
Appetizing, professional, culinary, high-resolution, commercial, highly detailed
"#;

async fn get_generation_params(params: &serde_json::Value) -> GenerationParams {
    let prompt = params["prompt"].as_str().unwrap();
    let message = crate::aigc::openai::request(&SYSTEM_PROMPT, prompt).await.unwrap();
    GenerationParams {
        prompt: message,
        negative_prompt: String::from(""),
    }
}

async fn handle_generate_image_request(
    body: String,
    tx: Arc<mpsc::Sender<TaskPayload>>
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    let timestamp = chrono::Utc::now().timestamp_millis();
    let random = rand::random::<u32>() % 1000000;
    let task_id = format!("{}-{:0>6}", timestamp, random);

    let body_json = serde_json::from_str::<serde_json::Value>(&body).unwrap();
    let params = body_json["params"].to_owned();
    let callback_url = body_json["resultCallbackUrl"].as_str().unwrap().to_string();
    let generation_params = get_generation_params(&params).await;

    let task_payload = TaskPayload {
        task_id: task_id.clone(),
        params,
        generation_params,
        callback_url,
    };

    let conn = &mut establish_connection();
    let new_task = NewTask {
        task_id: &task_id,
        params: &serde_json::to_string(&task_payload.params).unwrap(),
        generation_params: &serde_json::to_string(&task_payload.generation_params).unwrap(),
        result: "",
        callback_url: &task_payload.callback_url,
    };
    diesel::insert_into(tasks::table)
        .values(&new_task)
        .execute(conn)
        .expect("Error saving new task");

    if let Ok(()) = tx.try_send(task_payload) {
        let res_json = json!({"taskId": &task_id});
        return Ok(Json(res_json));
    } else {
        return Err(BadRequest { message: "Queue is full".to_string() });
    }
}

async fn callback_generate_image(task_payload: &TaskPayload, base64_images: &Vec<String>) {
    let task_id: &str = &task_payload.task_id;
    let images = base64_images
        .iter()
        .enumerate()
        .map(|(i, base64_image)| {
            let filename = format!("{}-{}.png", task_id, i);
            ( filename, base64_image )
        }).collect::<Vec<_>>();
    let image_urls = crate::storage::azure::upload_images(&images).await;

    let result = json!({
        "images": image_urls.iter().map(|image_url| {
            json!({
                "src": image_url
            })
        }).collect::<Vec<_>>()
    });

    let conn = &mut establish_connection();
    diesel::update(tasks::table)
        .filter(tasks::task_id.eq(task_id))
        .set(tasks::result.eq(serde_json::to_string(&result).unwrap()))
        .execute(conn)
        .expect("Error updating task");

    let callback_res = reqwest::Client::new()
        .post(&task_payload.callback_url)
        .json(&json!({
            "taskId": task_id,
            "params": task_payload.params,
            "result": result,
        }))
        .send().await;
    match callback_res {
        Ok(_) => tracing::info!("Task {} callback success", task_id),
        Err(e) => tracing::info!("Task {} callback failed: {}", task_id, e)
    }
}


pub fn get_routes() -> Router {
    let (tx, mut rx) = mpsc::channel::<TaskPayload>(2);

    let tx = Arc::new(tx);

    tokio::spawn(async move {
        while let Some(task_payload) = rx.recv().await {
            let conn = &mut establish_connection();
            let task_id = &task_payload.task_id;
            tracing::info!("Task {} started", task_id);
            diesel::update(tasks::table)
                .filter(tasks::task_id.eq(task_id))
                .set(tasks::starts_at.eq(chrono::Utc::now().naive_utc()))
                .execute(conn).unwrap();
            let prompt = &task_payload.generation_params.prompt;
            if let Ok(base64_images) = crate::aigc::comfy::request(prompt).await {
                tracing::info!("Task {} comfy success", task_id);
                callback_generate_image(&task_payload, &base64_images).await;
                diesel::update(tasks::table)
                    .filter(tasks::task_id.eq(task_id))
                    .set(tasks::ends_at.eq(chrono::Utc::now().naive_utc()))
                    .execute(conn).unwrap();
                tracing::info!("Task {} end", task_id);
            } else {
                tracing::info!("Task {} comfy failed", task_id);
            }
        }
    });

    let router: Router = Router::new()
        .route("/api/yum/generate/image", post({
            let tx = Arc::clone(&tx);
            |body: String| handle_generate_image_request(body, tx)
        }))
        .route("/api/yum/generate/queueInfo", get(|| async {
            let queue_info = json!({
                "pendingTasks": 5,
                "executingTasks": 1,
            });
            Json(queue_info)
        }));
    return router;
}
