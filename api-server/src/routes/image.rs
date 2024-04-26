use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::broadcast::Sender;
use axum::{
    routing::{post, get},
    response::{Json, IntoResponse, Response},
    extract::Path,
    Router,
    http::StatusCode,
};
use diesel::prelude::*;
use crate::schema::tasks;
use crate::models::NewTask;
use crate::models::Task;
use super::database::establish_connection;
use super::task_pool::TaskPayload;

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

#[derive(Deserialize)]
struct RequestPayload {
    pub params: serde_json::Value,
    #[serde(rename = "resultCallbackUrl")]
    pub result_callback_url: String,
}

async fn handle_generate_image_request(
    body: String,
    tx: Arc<Sender<TaskPayload>>,
    workers_count: usize,
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    let timestamp = chrono::Utc::now().timestamp_millis();
    let rand_suffix = rand::random::<usize>() % 1000000;
    let task_id = format!("{}-{:0>6}", timestamp, rand_suffix);

    let req_payload = serde_json::from_str::<RequestPayload>(&body).map_err(|_| BadRequest {
        message: "Invalid request body".to_string()
    })?;
    let params = req_payload.params;
    let callback_url = req_payload.result_callback_url;

    let conn = &mut establish_connection();
    let new_task = NewTask {
        task_id: &task_id,
        params: &serde_json::to_string(&params).unwrap_or(format!("error: {}", &body)),
        generation_params: "",
        result: "",
        callback_url: &callback_url,
    };
    diesel::insert_into(tasks::table)
        .values(&new_task)
        .execute(conn)
        .expect("Error saving new task");

    let task_payload = TaskPayload {
        channel: rand_suffix % workers_count,
        task_id: task_id.clone(),
        params,
        callback_url,
    };

    match tx.send(task_payload) {
        Ok(rem) => {
            tracing::info!(task_id, "Task queued, remaining receivers {}", rem);
            let response = json!({ "taskId": &task_id });
            Ok(Json(response))
        },
        Err(e) => {
            tracing::error!(task_id, "Failed to queue task {}", e);
            // 这里只是接口返回 Queue is full, 但实际可能是 panic 了以后 channel closed, 具体要看日志
            Err(BadRequest { message: "Queue is full".to_string() })
        }
    }
}

async fn fetch_task_result(task_id: String) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    let conn = &mut establish_connection();
    let queryset =
        tasks::table.filter(tasks::task_id.eq(task_id)).first::<Task>(conn);
    let task = match queryset {
        Ok(t) => t,
        Err(_) => {
            let response = (StatusCode::NOT_FOUND, "Task not found").into_response();
            return Err(response);
        }
    };
    let json_or_null = |s: &str|
        serde_json::from_str::<serde_json::Value>(s)
        .unwrap_or_else(|_| serde_json::Value::Null);
    let task_json = json!({
        "taskId": &task.task_id,
        "params": json_or_null(&task.params),
        "result": json_or_null(&task.result),
        "generationParams": json_or_null(&task.generation_params),
    });
    Ok(Json(task_json))
}

async fn retry_task(
    task_id: String,
    tx: Arc<Sender<TaskPayload>>,
    workers_count: usize
) -> Result<Json<serde_json::Value>, impl IntoResponse> {
    let conn = &mut establish_connection();
    let queryset = tasks::table.filter(tasks::task_id.eq(&task_id)).first::<Task>(conn);
    let task = match queryset {
        Ok(t) => t,
        Err(_) => {
            let response = (StatusCode::NOT_FOUND, "Task not found").into_response();
            return Err(response);
        }
    };
    if task.ends_at != None {
        let response = (StatusCode::BAD_REQUEST, "Task already finished").into_response();
        return Err(response);
    };
    let params = match serde_json::from_str::<serde_json::Value>(&task.params) {
        Ok(p) => p,
        Err(_) => {
            let response = (StatusCode::INTERNAL_SERVER_ERROR, "Failed to parse task params").into_response();
            return Err(response);
        }
    };
    let task_payload = TaskPayload {
        channel: rand::random::<usize>() % workers_count,
        task_id: task.task_id,
        params: params,
        callback_url: task.callback_url,
    };
    match tx.send(task_payload) {
        Ok(rem) => {
            tracing::info!(task_id, "Task queued, remaining receivers {}", rem);
            let response = json!({ "taskId": &task_id });
            Ok(Json(response))
        },
        Err(e) => {
            tracing::error!(task_id, "Failed to queue task {}", e);
            let response = (StatusCode::BAD_REQUEST, "Failed to queue task").into_response();
            Err(response)
        }
    }
}

async fn fetch_queue_info() -> Json<serde_json::Value> {
    let conn = &mut establish_connection();
    let pending_tasks = tasks::table
        .filter(tasks::starts_at.is_null())
        .count().get_result::<i64>(conn).unwrap_or(0);
    let executing_tasks = tasks::table
        .filter(tasks::starts_at.is_not_null())
        .filter(tasks::ends_at.is_null())
        .count().get_result::<i64>(conn).unwrap_or(0);
    let queue_info = json!({
        "pendingTasks": pending_tasks,
        "executingTasks": executing_tasks,
    });
    Json(queue_info)
}

pub fn get_routes() -> Router {
    let (tx, workers_count) = super::task_pool::init_task_pool();

    let router: Router = Router::new()
        .route("/api/yum/generate/image", post({
            let tx = Arc::clone(&tx);
            move |body: String| handle_generate_image_request(body, tx, workers_count)
        }))
        .route("/api/yum/generate/result/:id", get({
            |Path(task_id): Path<String>| fetch_task_result(task_id)
        }))
        .route("/api/yum/generate/result/:id/retry", post({
            move |Path(task_id): Path<String>| retry_task(task_id, tx, workers_count)
        }))
        .route("/api/yum/generate/queueInfo", get(fetch_queue_info));

    return router;
}
