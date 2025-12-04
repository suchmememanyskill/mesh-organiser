use axum::{Router, extract::{Path, State}, response::{Html, Response}, routing::get};
use db::{share_db, user_db};
use tokio::fs;
use tower_http::services::ServeFile;

use crate::{error::ApplicationError, web_app_state::WebAppState};

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
        .route_service("/group/", sub_index.clone())
        .route_service("/label/", sub_index.clone())
        .route_service("/share/", sub_index.clone())
        .route_service("/group/{group_id}", sub_index.clone())
        .route_service("/label/{label_id}", sub_index.clone())
        .route("/share/{share_id}", get(serve_share_page))
}

async fn serve_share_page(
    Path(share_id): Path<String>,
    State(app_state): State<WebAppState>,
) -> Result<Html<String>, ApplicationError> {
    let mut html = fs::read_to_string("www/group/1.html").await?;

    let share = match share_db::get_share_via_id(&app_state.app_state.db, &share_id).await {
        Ok(s) => s,
        Err(_) => {
            return Ok(Html(html));
        }
    };

    let user = match user_db::get_user_by_id(&app_state.app_state.db, share.user_id).await {
        Ok(Some(u)) => u,
        _ => {
            return Ok(Html(html));
        }
    };

    html = html
        .replace("content=\"Mesh Organiser\"", &format!("content=\"Share: {}\"", htmlescape::encode_attribute(&share.share_name)))
        .replace("content=\"A personal 3d printing model library.\"", &format!("content=\"By {}. Contains {} model{}.\"", htmlescape::encode_attribute(&user.username), share.model_ids.len(), if share.model_ids.len() >= 2 { "s" } else { "" }));

    Ok(Html(html))
}