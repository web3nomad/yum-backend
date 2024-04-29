pub mod database;
pub mod task_pool;
pub mod text;
pub mod image;
pub mod admin;

use axum::Router;

pub fn get_routes() -> Router {
    Router::new()
        .merge(text::get_routes())
        .merge(image::get_routes())
        // .merge(admin::get_routes())
}
