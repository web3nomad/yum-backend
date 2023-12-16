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
        Ok(_) => tracing::info!("Task {} callback success", task_payload.task_id),
        Err(e) => tracing::error!("Task {} callback failed: {}", task_payload.task_id, e)
    }
}

async fn process_task(comfy_origin: &str, task_payload: &TaskPayload) {
    let conn = &mut establish_connection();
    let task_id = &task_payload.task_id;

    tracing::info!("Task {} started", task_id);
    diesel::update(tasks::table)
        .filter(tasks::task_id.eq(task_id))
        .set(tasks::starts_at.eq(chrono::Utc::now().naive_utc()))
        .execute(conn).unwrap();

    let (generation_params, theme) =
        match text2prompt::request(&task_payload.params).await
    {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Task {} Error: {:?}", task_id, e);
            return;
        }
    };

    tracing::info!("Task {} text2prompt success", task_id);

    if let Ok(base64_images) = comfy::request(comfy_origin, &generation_params).await {
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
        let image_urls = azure::upload_images(&images, format).await;

        let result = json!({
            "theme": theme,
            "images": image_urls.iter().map(|image_url| {
                json!({
                    "src": image_url,
                    "is_filtered": if image_url == "" { true } else { false },
                })
            }).collect::<Vec<_>>()
        });

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

pub fn init_task_pool() -> (Arc<broadcast::Sender<TaskPayload>>, usize) {
    let comfy_origins = env::var("COMFYUI_ORIGINS").unwrap()
        .split(",").map(|s| s.to_string()).collect::<Vec<_>>();

    let (tx, _rx) = broadcast::channel::<TaskPayload>(500);
    let tx = Arc::new(tx);

    let comfy_count = comfy_origins.len();
    comfy_origins.iter().enumerate().for_each(|(index, comfy_origin)| {
        let comfy_origin = comfy_origin.clone();
        let mut rx = tx.subscribe();
        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(task_payload) => {
                        if task_payload.channel != index {
                            continue;
                        }
                        tracing::info!("Task {} received by {}", task_payload.task_id, comfy_origin);
                        process_task(&comfy_origin, &task_payload).await;
                    },
                    Err(e) => {
                        tracing::error!("No Task Error: {} {:?}", comfy_origin, e);
                    }
                }
            }
        });
    });

    return (tx, comfy_count);
}
