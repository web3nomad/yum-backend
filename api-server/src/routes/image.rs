use serde_json::json;
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

struct TaskPayload {
    task_id: String,
    params: serde_json::Value,
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

    let task_payload = TaskPayload {
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

    if let Ok(()) = tx.try_send(task_payload) {
        let res_json = json!({
            "taskId": &task_id
        });
        return Ok(Json(res_json));
    } else {
        return Err(BadRequest { message: "Queue is full".to_string() });
    }
}

async fn callback_generate_image(
    task_payload: &TaskPayload,
    generation_params: &crate::aigc::text2prompt::GenerationParams,
    result: &serde_json::Value
) {
    let callback_res = reqwest::Client::new()
        .post(&task_payload.callback_url)
        .json(&json!({
            "taskId": task_payload.task_id,
            "params": task_payload.params,
            "result": result,
            "generation_params": generation_params,
        }))
        .send().await;
    match callback_res {
        Ok(_) => tracing::info!("Task {} callback success", task_payload.task_id),
        Err(e) => tracing::error!("Task {} callback failed: {}", task_payload.task_id, e)
    }
}

async fn process_task(task_payload: &TaskPayload) {
    let conn = &mut establish_connection();
    let task_id = &task_payload.task_id;

    tracing::info!("Task {} started", task_id);
    diesel::update(tasks::table)
        .filter(tasks::task_id.eq(task_id))
        .set(tasks::starts_at.eq(chrono::Utc::now().naive_utc()))
        .execute(conn).unwrap();

    let generation_params = match crate::aigc::text2prompt::request(&task_payload.params).await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Task {} Error: {:?}", task_id, e);
            return;
        }
    };

    tracing::info!("Task {} text2prompt success", task_id);

    let prompt = &generation_params.prompt;

    if let Ok(base64_images) = crate::aigc::comfy::request(prompt).await {
        tracing::info!("Task {} comfy success", task_id);

        let task_id: &str = &task_payload.task_id;
        let format = "jpeg";
        let images = base64_images
            .iter()
            .enumerate()
            .map(|(i, base64_image)| {
                let filename = format!("{}-{}.{}", task_id, i, format);
                ( filename, base64_image )
            }).collect::<Vec<_>>();
        let image_urls = crate::storage::azure::upload_images(&images, format).await;

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
            .set((
                tasks::generation_params.eq(serde_json::to_string(&generation_params).unwrap()),
                tasks::result.eq(serde_json::to_string(&result).unwrap()),
                tasks::ends_at.eq(chrono::Utc::now().naive_utc())
            ))
            .execute(conn)
            .expect("Error updating task");

        callback_generate_image(&task_payload, &generation_params, &result).await;

        tracing::info!("Task {} end", task_id);
    } else {
        tracing::info!("Task {} comfy failed", task_id);
    }
}

pub fn get_routes() -> Router {
    let (
        tx,
        mut rx
    ) = mpsc::channel::<TaskPayload>(10);

    let tx = Arc::new(tx);

    tokio::spawn(async move {
        while let Some(task_payload) = rx.recv().await {
            process_task(&task_payload).await;
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
