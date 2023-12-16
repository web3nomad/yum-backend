use serde_json::json;
use std::sync::Arc;
use tokio::sync::broadcast::Sender;
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
use super::task_pool::TaskPayload;
use super::database::establish_connection;

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

async fn handle_generate_image_request(
    body: String,
    tx: Arc<Sender<TaskPayload>>,
    comfy_count: usize,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    let timestamp = chrono::Utc::now().timestamp_millis();
    let random = rand::random::<u32>() % 1000000;
    let task_id = format!("{}-{:0>6}", timestamp, random);

    let body_json = serde_json::from_str::<serde_json::Value>(&body).unwrap();
    let params = body_json["params"].to_owned();
    let callback_url = body_json["resultCallbackUrl"].as_str().unwrap().to_string();

    let task_payload = TaskPayload {
        channel: (timestamp % (comfy_count as i64)) as usize,
        task_id: task_id.clone(),
        params,
        callback_url,
    };

    let conn = &mut establish_connection();
    let new_task = NewTask {
        task_id: &task_id,
        params: &serde_json::to_string(&task_payload.params).unwrap(),
        generation_params: "",
        result: "",
        callback_url: &task_payload.callback_url,
    };
    diesel::insert_into(tasks::table)
        .values(&new_task)
        .execute(conn)
        .expect("Error saving new task");

    match tx.send(task_payload) {
        Ok(rem) => {
            tracing::info!("Task {} queued, remaining receivers {}", &task_id, rem);
            let response = json!({ "taskId": &task_id });
            return Ok(Json(response));
        },
        Err(e) => {
            tracing::error!("Failed to queue task {}: {}", &task_id, e);
            Err(BadRequest { message: "Queue is full".to_string() })
        }
    }
}

pub fn get_routes() -> Router {
    let (tx, comfy_count) = super::task_pool::init_task_pool();

    let router: Router = Router::new()
        .route("/api/yum/generate/image", post({
            let tx = Arc::clone(&tx);
            move |body: String| handle_generate_image_request(body, tx, comfy_count)
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
