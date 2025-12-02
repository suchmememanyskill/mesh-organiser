use axum::Router;
use tower_http::services::ServeFile;

use crate::web_app_state::WebAppState;

pub fn router() -> Router<WebAppState> {
    let index = ServeFile::new("www/index.html");
    let sub_index = ServeFile::new("www/group/1.html");

    Router::new()
        .route_service("/about", index.clone())
        .route_service("/settings", index.clone())
        .route_service("/favorite", index.clone())
        .route_service("/group", index.clone())
        .route_service("/import", index.clone())
        .route_service("/login", index.clone())
        .route_service("/model", index.clone())
        .route_service("/printed", index.clone())
        .route_service("/resource", index.clone())
        .route_service("/group/{group_id}", sub_index.clone())
        .route_service("/label/{label_id}", sub_index.clone())
}