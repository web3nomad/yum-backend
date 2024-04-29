use serde::Deserialize;
use axum::{
    extract::Query,
    routing::get,
    response::Html,
    Router,
};
use diesel::prelude::*;
use crate::schema::tasks;
use crate::models::Task;
use super::database::establish_connection;

#[derive(Deserialize)]
struct Pagination {
    page_size: Option<usize>,
    page: Option<usize>,
    task_id: Option<String>,
}

fn task_to_html(task: &Task) -> String {
    if task.result == "" {
        return String::from("");
    }
    let result = serde_json::from_str::<serde_json::Value>(&task.result).unwrap();
    let params = serde_json::from_str::<serde_json::Value>(&task.params).unwrap();
    let user_input = params["prompt"].as_str().unwrap();
    let generation_params = serde_json::from_str::<serde_json::Value>(&task.generation_params).unwrap();
    let positive = generation_params["positive"].as_str().unwrap();
    let negative = generation_params["negative"].as_str().unwrap();
    let images_html: String = result["images"]
        .as_array().unwrap()
        .iter().map(|image| {
            let src = image["src"].as_str().unwrap();
            format!(r#"<img src="{}" style="margin-right:2px;" />"#, src)
        })
        .collect::<Vec<String>>()
        .join("\n");
    let theme = result["theme"].as_str().unwrap();
    let duration = if task.ends_at.is_some() && task.starts_at.is_some() {
        task.ends_at.unwrap().and_utc() - task.starts_at.unwrap().and_utc()
    } else {
        chrono::Duration::zero()
    };
    let images_line_html = format!(r#"
        <div style="margin-bottom:30px;">
            <div style="font-weight:bold;margin-bottom:10px;">{} | {} | {} | {} | ({}s)</div>
            <div style="margin-bottom:10px;font-size:14px;"><strong>Positive</strong>: {}</div>
            <div style="margin-bottom:10px;font-size:14px;"><strong>Negative</strong>: {}</div>
            <div style="margin-top:20px;display:flex;height:300px;">{}</div>
        </div>"#,
        &task.task_id,
        user_input,
        theme,
        &task.starts_at.unwrap_or_default().and_utc(),
        duration.num_seconds(),
        positive,
        negative,
        images_html
    );
    return images_line_html;
}

async fn handler(pagination: Query<Pagination>) -> Html<String> {
    let conn = &mut establish_connection();
    let page_size: usize = if let Some(page_size) = pagination.page_size {
        page_size
    } else {
        10
    };
    let offset: usize = if let Some(page) = pagination.page {
        (page - 1) * page_size
    } else {
        0
    };
    // println!("offset: {}", offset);
    let tasks = if let Some(task_id) = &pagination.task_id {
        tasks::table
            .filter(tasks::ends_at.is_not_null())
            .filter(tasks::task_id.eq(task_id))
            .order(tasks::updated_at.desc())
            .limit(page_size.try_into().unwrap())
            .offset(offset.try_into().unwrap())
            .load::<Task>(conn)
            .unwrap()
    } else {
        tasks::table
            .filter(tasks::ends_at.is_not_null())
            .order(tasks::updated_at.desc())
            .limit(page_size.try_into().unwrap())
            .offset(offset.try_into().unwrap())
            .load::<Task>(conn)
            .unwrap()
    };
    let html = tasks
        .iter()
        .map(|task| task_to_html(task))
        .collect::<Vec<String>>()
        .join("\n");
    let html = format!(r#"
        <html>
            <head>
                <title>Yum Admin</title>
                <style>body {{ font-family: sans-serif; }}</style>
            </head>
            <body>{}</body>
        </html>"#,
        html
    );
    return Html(html);
}

pub fn get_routes() -> Router {
    let router: Router = Router::new().route("/api/yum/admin/tasks", get(handler));
    return router;
}
