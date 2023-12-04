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

async fn request_webui(params: &serde_json::Value) -> Vec<String> {
    let prompt = params["prompt"].as_str().unwrap();
    let webui_origin = env::var("SD_WEBUI_TEST_ORIGIN").unwrap();
    let api_auth = Some(sdwebuiapi::OpenApiV1Auth {
        username: env::var("SD_WEBUI_TEST_AUTH_USERNAME").unwrap(),
        password: env::var("SD_WEBUI_TEST_AUTH_PASSWORD").unwrap(),
    });
    let mut payload = sdwebuiapi::TextToImagePayload {
        prompt: prompt.to_string(),
        batch_size: 4,
        ..Default::default()
    };
    payload.set_base_model("DreamShaper_6_BakedVae.safetensors [b76cc78ad9]");
    let client = sdwebuiapi::Client::new(&webui_origin, api_auth);
    let response = client.txt2img(payload).await;
    let images = response.images.iter().map(|raw_b64_str| {
        raw_b64_str.to_string()
    }).collect::<Vec<String>>();
    return images;
}

async fn callback_generate_image(
    task_id: &str,
    params: &serde_json::Value,
    images: &Vec<String>,
    callback_url: &str,
) {
    let mut builder = opendal::services::Azblob::default();
    let azblob_endpoint = env::var("AZBLOB_ENDPOINT").unwrap();
    let azblob_key = env::var("AZBLOB_KEY").unwrap();
    let azblob_container = env::var("AZBLOB_CONTAINER").unwrap();
    let azblob_account = env::var("AZBLOB_ACCOUNT").unwrap();
    builder.root("/");
    builder.container(&azblob_container);
    builder.endpoint(&azblob_endpoint);
    builder.account_name(&azblob_account);
    builder.account_key(&azblob_key);
    let op = opendal::Operator::new(builder).unwrap().finish();

    let mut images_payload: Vec<serde_json::Value> = vec![];
    for (i, raw_b64_str) in images.iter().enumerate() {
        // let raw_b64_str = &response.images[i];
        // TODO: remove this debug code in production
        let output_image = data_encoding::BASE64.decode(raw_b64_str.as_bytes()).unwrap();
        std::fs::create_dir_all("test-client/output").unwrap();
        let filename = format!("{}-{}.png", task_id, i);
        std::fs::write(&format!("test-client/output/{}", &filename), &output_image).unwrap();
        op.write(&filename, output_image).await.unwrap();
        let image_url = format!("{}{}/{}", &azblob_endpoint, &azblob_container, &filename);
        images_payload.push(json!({
            "src": &image_url
        }));
    }
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
            let images = request_webui(&params).await;
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
