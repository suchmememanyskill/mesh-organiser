use crate::{user::{AuthSession, Backend}, web_app_state::WebAppState};
use axum::{
    extract::Query,
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
    Form, Router,
};
use axum_login::login_required;
use crate::user::{Credentials, PasswordCredentials, TokenCredentials};
use axum::extract::{Multipart, State};
use db::model_db;
use service::{cleanse_evil_from_name, import_service, import_state::ImportState};
use time::OffsetDateTime;
use tokio::fs;

use crate::web_thumbnail_service;

pub fn router() -> Router<WebAppState> {
    Router::new()
        .nest(
            "/api/v1",
            Router::new()
                .route("/models", post(post::add_model))
                .route_layer(login_required!(Backend))
        )
}

mod post {
    use super::*;

    pub async fn add_model(
        auth_session: AuthSession,
        State(app_state) : State<WebAppState>,
        mut multipart: Multipart
    ) -> impl IntoResponse {
        let user = auth_session.user.unwrap().to_user();
        let mut paths = vec![];
        
        let temp_dir = std::env::temp_dir().join(format!(
            "meshorganiser_import_action_{}",
            OffsetDateTime::now_utc().unix_timestamp()
        ));

        match std::fs::create_dir(&temp_dir) {
            Ok(_) => {},
            Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create temporary directory").into_response(),
        };

        while let Some(field) = multipart.next_field().await.unwrap_or(None) {
            let file_name = match field.file_name() {
                Some(name) => name.to_string(),
                None => continue
            };

            let data = match field.bytes().await {
                Ok(d) => d,
                Err(_) => continue,
            };

            let file_path = temp_dir.join(cleanse_evil_from_name(&file_name));
            match fs::write(&file_path, &data).await {
                Ok(_) => paths.push(file_path),
                Err(_) => continue,
            };
        }

        if paths.is_empty() {
            return (StatusCode::BAD_REQUEST, "No files uploaded").into_response();
        }

        let mut model_ids: Vec<i64> = vec![];
        
        for path in paths {
            let mut import_state = ImportState::new(None, false, true, user.clone());
            import_state = match import_service::import_path(&path.to_string_lossy().to_string(), &app_state.app_state, import_state).await {
                Ok(state) => state,
                Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            };

            model_ids.extend(&import_state.imported_models[0].model_ids);
        }

        let models = match model_db::get_models_via_ids(&app_state.app_state.db, &user, model_ids).await {
            Ok(m) => m,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };

        match web_thumbnail_service::generate_thumbnails(&models, &app_state, false).await {
            Ok(_) => StatusCode::OK.into_response(),
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}