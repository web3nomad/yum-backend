use serde_json::json;
use std::env;
use std::sync::Arc;
use tokio::sync::broadcast;
use diesel::prelude::*;
use crate::schema::tasks;
use crate::aigc::text2prompt::{self, GenerationParams};
use crate::aigc::comfy;
use crate::storage::azure;
use super::database::establish_connection;

#[derive(Clone)]
pub struct TaskPayload {
    pub channel: usize,
    pub task_id: String,
    pub params: serde_json::Value,
    pub callback_url: String,
}

async fn callback_generate_image(
    task_payload: &TaskPayload,
    generation_params: &GenerationParams,
    result: &serde_json::Value
) {
    let callback_res = reqwest::Client::new()
        .post(&task_payload.callback_url)
        .json(&json!({
            "taskId": task_payload.task_id,
            "params": task_payload.params,
            "result": result,
            "generationParams": generation_params,
        }))
        .send().await;
    match callback_res {
        Ok(r) => {
            let res_code = r.status();
            let res_text = r.text().await.unwrap_or("".to_string());
            tracing::info!(task_id=task_payload.task_id, "Task callback success {} {}", res_code, res_text);
        },
        Err(e) => {
            tracing::error!(task_id=task_payload.task_id, "Task callback failed {}", e);
        }
    }
}

async fn process_task(comfy_origins: &Vec<String>, task_payload: &TaskPayload) {
    let conn: &mut MysqlConnection = &mut establish_connection();
    let task_id = &task_payload.task_id;

    async fn on_task_start(conn: &mut MysqlConnection, task_id: &str) {
        tracing::info!(task_id, "Task started");
        diesel::update(tasks::table)
            .filter(tasks::task_id.eq(task_id))
            .set(tasks::starts_at.eq(chrono::Utc::now().naive_utc()))
            .execute(conn)
            .unwrap_or_else(|e| {
                tracing::error!(task_id, "Error updating task starts_at {:?}", e);
                0
            });
    }

    async fn on_task_end(
        conn: &mut MysqlConnection,
        task_id: &str,
        task_payload: &TaskPayload,
        result: &serde_json::Value,
        generation_params: &GenerationParams,
    ) {
        diesel::update(tasks::table)
            .filter(tasks::task_id.eq(task_id))
            .set((
                tasks::generation_params.eq(serde_json::to_string(&generation_params).unwrap_or("".to_string())),
                tasks::result.eq(serde_json::to_string(&result).unwrap_or("".to_string())),
                tasks::ends_at.eq(chrono::Utc::now().naive_utc())
            ))
            .execute(conn)
            .unwrap_or_else(|e| {
                tracing::error!(task_id, "Error updating task ends_at {:?}", e);
                0
            });

        callback_generate_image(&task_payload, &generation_params, &result).await;

        tracing::info!(task_id, "Task end");
    }

    on_task_start(conn, task_id).await;

    let (
        generation_params, theme
    ) = match text2prompt::request(&task_payload.params).await {
        Ok(v) => {
            tracing::info!(task_id, "text2prompt success");
            v
        },
        Err(e) => {
            tracing::error!(task_id, "text2prompt failed {:?}", e);
            let generation_params = GenerationParams {
                positive: String::from(""),
                negative: String::from(""),
            };
            let result = json!({
                "theme": "",
                "images": json!([
                    { "src": "", "filtered": true },
                    { "src": "", "filtered": true },
                    { "src": "", "filtered": true },
                    { "src": "", "filtered": true },
                ])
            });
            on_task_end(conn, task_id, &task_payload, &result, &generation_params).await;
            return;
        }
    };

    if let Ok(base64_images) = comfy::request(comfy_origins, &generation_params).await {
        tracing::info!(task_id, "Task comfy success {:?}", comfy_origins);

        let task_id: &str = &task_payload.task_id;
        let format = "jpeg";
        let images = base64_images
            .iter()
            .enumerate()
            .map(|(i, base64_image)| {
                let filename = format!("{}-{}.{}", task_id, i, format);
                ( filename, base64_image )
            }).collect::<Vec<_>>();
        let image_urls = azure::upload_images(&images, format).await;

        let result = json!({
            "theme": theme,
            "images": image_urls.iter().map(|image_url| {
                json!({
                    "src": image_url,
                    "filtered": if image_url == "" { true } else { false },
                })
            }).collect::<Vec<_>>()
        });

        on_task_end(conn, task_id, &task_payload, &result, &generation_params).await;
    } else {
        tracing::info!(task_id, "Task comfy failed {:?}", comfy_origins);
    }
}

pub fn init_task_pool() -> (Arc<broadcast::Sender<TaskPayload>>, usize) {
    let comfy_origins = env::var("COMFYUI_ORIGINS").unwrap()
        .split(",").map(|s| s.to_string()).collect::<Vec<_>>();

    let (tx, _rx) = broadcast::channel::<TaskPayload>(500);
    let tx = Arc::new(tx);

    let comfy_count = comfy_origins.len();
    let (workers_count, comfy_origins_group): (usize, Vec<Vec<String>>) = if comfy_count == 4 {
        (1, vec![comfy_origins])
    } else {
        let comfy_origins = comfy_origins
            .iter()
            .map(|comfy_origin| vec![comfy_origin.clone()])
            .collect::<Vec<_>>();
        (comfy_count, comfy_origins)
    };
    comfy_origins_group.iter().enumerate().for_each(|(index, comfy_origins)| {
        let comfy_origins = comfy_origins.clone();
        let mut rx = tx.subscribe();
        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(task_payload) => {
                        if task_payload.channel != index {
                            continue;
                        }
                        tracing::info!(task_id=task_payload.task_id, "Task received by {:?}", comfy_origins);
                        process_task(&comfy_origins, &task_payload).await;
                    },
                    Err(e) => {
                        tracing::error!("Task receive error {:?}", e);
                    }
                }
            }
        });
    });

    return (tx, workers_count);
}
