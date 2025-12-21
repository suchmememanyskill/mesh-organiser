use crate::{
    user::{AuthSession, Backend},
    web_app_state::WebAppState,
};
use axum::extract::Path;
use axum::extract::{Multipart, State};
use axum::{Json, response::Response};
use axum::{
    Router,
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
};
use axum_login::login_required;
use db::model::ModelFlags;
use db::model_db;
use serde::Deserialize;
use service::{cleanse_evil_from_name, import_service, import_state::ImportState};
use std::path::PathBuf;
use std::str::FromStr;
use time::OffsetDateTime;
use tokio::fs;

use crate::error::ApplicationError;
use db::blob_db;
use db::model_db::{ModelFilterOptions, ModelOrderBy};
use serde::Serialize;
use service::export_service;
use service::threemf_service;

pub fn router() -> Router<WebAppState> {
    Router::new().nest(
        "/api/v1",
        Router::new()
            .route(
                "/models/{model_id}/3mf_metadata",
                get(get::get_threemf_metadata),
            )
            .route(
                "/models/{model_id}/3mf_extract",
                post(post::extract_threemf_models),
            )
            .route_layer(login_required!(Backend)),
    )
}

mod get {
    use super::*;

    pub async fn get_threemf_metadata(
        auth_session: AuthSession,
        Path(model_id): Path<i64>,
        State(app_state): State<WebAppState>,
    ) -> Result<Response, ApplicationError> {
        let user = auth_session.user.unwrap().to_user();

        let model = model_db::get_models_via_ids(&app_state.app_state.db, &user, vec![model_id]).await?;

        if model.is_empty() {
            return Ok((StatusCode::NOT_FOUND, "Model not found").into_response());
        }

        let threemf_metadata =
            threemf_service::extract_metadata(&model[0], &app_state.app_state).await?;

        Ok(Json(threemf_metadata).into_response())
    }
}

mod post {
    use db::{model::{Blob, ModelGroupMeta}, random_hex_32, time_now};
    use service::thumbnail_service;

    use crate::web_import_state::WebImportStateEmitter;

    use super::*;

    pub async fn extract_threemf_models(
        auth_session: AuthSession,
        Path(model_id): Path<i64>,
        State(app_state): State<WebAppState>,
    ) -> Result<Response, ApplicationError> {
        let user = auth_session.user.unwrap().to_user();

        let model = model_db::get_models_via_ids(&app_state.app_state.db, &user, vec![model_id]).await?;

        if model.is_empty() {
            return Ok((StatusCode::NOT_FOUND, "Model not found").into_response());
        }

        let mut import_state =
            threemf_service::extract_models(&model[0], &user, &app_state.app_state).await?;

        import_state.set_emitter(Box::new(WebImportStateEmitter {}));

        let model_ids: Vec<i64> = import_state
            .imported_models
            .iter()
            .flat_map(|f| f.model_ids.clone())
            .collect();

        let models = model_db::get_models_via_ids(&app_state.app_state.db, &user, model_ids).await?;
        let blobs: Vec<&Blob> = models.iter().map(|m| &m.blob).collect();

        thumbnail_service::generate_thumbnails(&blobs, &app_state.app_state, false, &mut import_state).await?;

        Ok(Json(ModelGroupMeta {
            id: import_state.imported_models[0].group_id.unwrap(),
            name: import_state.imported_models[0].group_name.clone().unwrap(),
            created: time_now(),
            last_modified: time_now(),
            resource_id: None,
            unique_global_id: random_hex_32(),
        }).into_response())
    }
}