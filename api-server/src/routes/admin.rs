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
    page: Option<usize>,
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
            format!(r#"<img src="{}" style="margin:2px;" />"#, src)
        })
        .collect::<Vec<String>>()
        .join("\n");
    let theme = result["theme"].as_str().unwrap();
    let images_line_html = format!(r#"
        <div>
            <div style="font-weight:bold;">{} | {} | {}</div>
            <div>Positive: {}</div>
            <div>Negative: {}</div>
            <div style="display:flex;height:300px;">{}</div>
        </div>"#,
        &task.task_id,
        user_input,
        theme,
        positive,
        negative,
        images_html
    );
    return images_line_html;
}

async fn handler(pagination: Query<Pagination>) -> Html<String> {
    let conn = &mut establish_connection();
    let page_size: usize = 10;
    let offset: usize = if let Some(page) = pagination.page {
        (page - 1) * page_size
    } else {
        0
    };
    // println!("offset: {}", offset);
    let tasks = tasks::table
        .order(tasks::id.desc())
        .limit(page_size.try_into().unwrap())
        .offset(offset.try_into().unwrap())
        .load::<Task>(conn)
        .unwrap();
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
