use serde_json::json;
use std::env;
use std::sync::Arc;
use tokio::sync::mpsc;
use diesel::prelude::*;
use axum::{
    routing::post,
    response::{Json, IntoResponse, Response},
    Router,
    http::StatusCode,
};

fn establish_connection() -> MysqlConnection {
    let database_url = env::var("DATABASE_URL").unwrap();
    MysqlConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

struct TaskPayload {
    task_id: String,
    body: String,
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
) -> Result<Json<serde_json::Value>, BadRequest> {
    let timestamp = chrono::Utc::now().timestamp_millis();
    let random = rand::random::<u32>() % 1000000;
    let task_id = &format!("{}-{:0>6}", timestamp, random);

    let body_json = serde_json::from_str::<serde_json::Value>(&body).unwrap();

    use crate::schema::tasks;
    use crate::models::NewTask;
    let conn = &mut establish_connection();
    let new_task = NewTask {
        task_id: task_id,
        params: &serde_json::to_string(&body_json["params"]).unwrap(),
        result: "",
        callback_url: body_json["resultCallbackUrl"].as_str().unwrap(),
    };
    diesel::insert_into(tasks::table)
        .values(&new_task)
        .execute(conn)
        .expect("Error saving new task");

    let task = TaskPayload {
        task_id: task_id.clone(),
        body,
    };

    if let Ok(()) = tx.try_send(task) {
        let res_json = json!({"taskId": &task_id});
        return Ok(Json(res_json));
    } else {
        return Err(BadRequest { message: "Queue is full".to_string() });
    }
}

async fn callback_generate_image(
    task_id: &str,
    params: &serde_json::Value,
    images: &Vec<String>,
    callback_url: &str,
) {
    let images = images.iter().enumerate().map(|(i, image)| {
        let filename = format!("{}-{}.png", task_id, i);
        let base64 = image;
        ( filename, base64 )
    }).collect::<Vec<_>>();
    let image_urls = crate::storage::azure::upload_images(&images).await;
    let images_payload = image_urls.iter().map(|image_url| {
        json!({
            "src": image_url
        })
    }).collect::<Vec<_>>();

    let result = json!({
        "images": images_payload
    });

    use crate::schema::tasks;
    let conn = &mut establish_connection();
    diesel::update(tasks::table)
        .filter(tasks::task_id.eq(&task_id))
        .set(tasks::result.eq(serde_json::to_string(&result).unwrap()))
        .execute(conn)
        .expect("Error updating task");

    let payload = json!({
        "taskId": task_id,
        "params": params,
        "result": result,
    });
    let client = reqwest::Client::new();
    client.post(callback_url).json(&payload).send().await.unwrap();
}

pub fn get_routes() -> Router {
    let (tx, mut rx) = mpsc::channel::<TaskPayload>(2);

    let tx = Arc::new(tx);

    tokio::spawn(async move {
        while let Some(task) = rx.recv().await {
            let body_json = serde_json::from_str::<serde_json::Value>(&task.body).unwrap();
            let callback_url = body_json["resultCallbackUrl"].as_str().unwrap();
            let params = body_json["params"].clone();
            let task_id = &task.task_id;
            println!("Start! {}", task_id);
            let prompt = params["prompt"].as_str().unwrap();
            let images = crate::stablediffusion::comfy::request(prompt).await;
            callback_generate_image(
                task_id, &params, &images, callback_url
            ).await;
            println!("End! {}", task_id);
        }
    });

    let router: Router = Router::new()
        .route("/api/yum/generate/image", post({
            let tx = Arc::clone(&tx);
            |body: String| handle_generate_image_request(body, tx)
        }));
    return router;
}
