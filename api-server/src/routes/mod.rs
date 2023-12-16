pub mod text;
pub mod image;

use axum::Router;

pub fn get_routes() -> Router {
    Router::new()
        .merge(text::get_routes())
        .merge(image::get_routes())
}
