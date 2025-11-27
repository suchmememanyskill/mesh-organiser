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

use crate::web_thumbnail_service;

pub fn router() -> Router<WebAppState> {
    Router::new().nest(
        "/api/v1",
        Router::new()
            .route("/models", post(post::add_model))
            .route("/models", get(get::get_models))
            .route("/models/count", get(get::get_model_count))
            .route("/models/disk_usage", get(get::get_model_disk_space_usage))
            .route("/models/{model_id}", put(put::edit_model))
            .route("/models/{model_id}", delete(delete::delete_model))
            .route_layer(login_required!(Backend)),
    )
}

mod get {
    use axum_extra::extract::Query;

    use super::*;

    #[derive(Deserialize)]
    pub struct GetModelParams {
        #[serde(default)]
        pub model_ids: Vec<i64>,
        #[serde(default)]
        pub group_ids: Vec<i64>,
        #[serde(default)]
        pub label_ids: Vec<i64>,
        pub order_by: Option<String>,
        pub text_search: Option<String>,
        #[serde(default)]
        pub model_flags: ModelFlags,
        pub page: u32,
        pub page_size: u32,
    }

    pub async fn get_models(
        auth_session: AuthSession,
        State(app_state): State<WebAppState>,
        Query(params): Query<GetModelParams>,
    ) -> Result<Response, ApplicationError> {
        let user = auth_session.user.unwrap().to_user();
        let flags = params.model_flags;
        println!("Flags: {:?}", flags);
        let models = model_db::get_models(
            &app_state.app_state.db,
            &user,
            ModelFilterOptions {
                model_ids: if params.model_ids.is_empty() { None } else { Some(params.model_ids) },
                group_ids: if params.group_ids.is_empty() { None } else { Some(params.group_ids) },
                label_ids: if params.label_ids.is_empty() { None } else { Some(params.label_ids) },
                order_by: params
                    .order_by
                    .map(|s| ModelOrderBy::from_str(&s).unwrap_or(ModelOrderBy::AddedDesc)),
                model_flags: if flags.is_empty() { None } else { Some(flags) },
                text_search: params.text_search,
                page: params.page,
                page_size: params.page_size,
            },
        )
        .await?;

        Ok(Json(models.items).into_response())
    }

    #[derive(Deserialize)]
    pub struct GetModelCountParams {
        #[serde(default)]
        pub model_flags: ModelFlags,
    }

    #[derive(Serialize)]
    pub struct GetModelCountResponse {
        pub count: usize,
    }

    pub async fn get_model_count(
        auth_session: AuthSession,
        State(app_state): State<WebAppState>,
        Query(params): Query<GetModelCountParams>,
    ) -> Result<Response, ApplicationError> {
        let user = auth_session.user.unwrap().to_user();
        let count = model_db::get_model_count(&app_state.app_state.db, &user, if params.model_flags.is_empty() { None } else { Some(params.model_flags)}).await?;

        Ok(Json(GetModelCountResponse { count }).into_response())
    }

    #[derive(Serialize)]
    pub struct GetModelDiskSpaceUsageResponse {
        pub size_compressed: u64,
        pub size_uncompressed: u64,
    }

    pub async fn get_model_disk_space_usage(
        auth_session: AuthSession,
        State(app_state): State<WebAppState>,
    ) -> Result<Response, ApplicationError> {
        let user = auth_session.user.unwrap().to_user();
        let data = model_db::get_size_of_models(&app_state.app_state.db, &user).await?;
        let local = export_service::get_size_of_blobs(&data.blob_sha256, &app_state.app_state)?;

        Ok(Json(GetModelDiskSpaceUsageResponse {
            size_uncompressed: data.total_size as u64,
            size_compressed: local,
        })
        .into_response())
    }
}

mod put {
    use super::*;

    #[derive(Deserialize)]
    pub struct PutModelParams {
        pub model_name: String,
        pub model_url: Option<String>,
        pub model_description: Option<String>,
        pub model_flags: Option<ModelFlags>,
    }

    pub async fn edit_model(
        auth_session: AuthSession,
        Path(model_id): Path<i64>,
        State(app_state): State<WebAppState>,
        Json(params): Json<PutModelParams>,
    ) -> Result<Response, ApplicationError> {
        let user = auth_session.user.unwrap().to_user();
        model_db::edit_model(
            &app_state.app_state.db,
            &user,
            model_id,
            &params.model_name,
            params.model_url.as_deref(),
            params.model_description.as_deref(),
            params.model_flags.unwrap_or(ModelFlags::empty()),
            None,
        )
        .await?;

        Ok(StatusCode::NO_CONTENT.into_response())
    }
}

mod delete {
    use super::*;

    pub async fn delete_model(
        auth_session: AuthSession,
        Path(model_id): Path<i64>,
        State(app_state): State<WebAppState>,
    ) -> Result<Response, ApplicationError> {
        let user = auth_session.user.unwrap().to_user();
        let model =
            model_db::get_models_via_ids(&app_state.app_state.db, &user, vec![model_id]).await?;

        if model.len() != 1 {
            return Err(ApplicationError::InternalError(String::from(
                "Failed to find model to delete",
            )));
        }

        let model = &model[0];

        model_db::delete_model(&app_state.app_state.db, &user, model_id).await?;

        if blob_db::get_blob_model_usage_count(&app_state.app_state.db, model.blob.id).await? <= 0 {
            let model_path = PathBuf::from(app_state.get_model_dir())
                .join(format!("{}.{}", model.blob.sha256, model.blob.filetype));
            let image_path =
                PathBuf::from(app_state.get_image_dir()).join(format!("{}.png", model.blob.sha256));

            if model_path.exists() {
                std::fs::remove_file(model_path)?;
            }

            if image_path.exists() {
                std::fs::remove_file(image_path)?;
            }

            blob_db::delete_blob(&app_state.app_state.db, model.blob.id).await?;
        }

        Ok(StatusCode::NO_CONTENT.into_response())
    }
}

mod post {
    use super::*;

    pub async fn add_model(
        auth_session: AuthSession,
        State(app_state): State<WebAppState>,
        mut multipart: Multipart,
    ) -> Result<Response, ApplicationError> {
        let user = auth_session.user.unwrap().to_user();
        let mut paths = vec![];
        let config = app_state.get_configuration();

        let temp_dir = std::env::temp_dir().join(format!(
            "meshorganiser_import_action_{}",
            OffsetDateTime::now_utc().unix_timestamp()
        ));

        std::fs::create_dir(&temp_dir)?;

        while let Some(field) = multipart.next_field().await.unwrap_or(None) {
            let file_name = match field.file_name() {
                Some(name) => name.to_string(),
                None => continue,
            };

            let data = match field.bytes().await {
                Ok(d) => d,
                Err(_) => continue,
            };

            let file_path = temp_dir.join(cleanse_evil_from_name(&file_name));

            if !import_service::is_supported_extension(&file_path, &config) {
                continue;
            }

            match fs::write(&file_path, &data).await {
                Ok(_) => paths.push(file_path),
                Err(_) => continue,
            };
        }

        if paths.is_empty() {
            return Ok((StatusCode::BAD_REQUEST, "No files uploaded").into_response());
        }

        let mut model_ids: Vec<i64> = vec![];

        for path in paths {
            let mut import_state = ImportState::new(None, false, true, user.clone());
            import_state = import_service::import_path(
                &path.to_string_lossy().to_string(),
                &app_state.app_state,
                import_state,
            )
            .await?;

            model_ids.extend(&import_state.imported_models[0].model_ids);
        }

        let models =
            model_db::get_models_via_ids(&app_state.app_state.db, &user, model_ids.clone()).await?;

        web_thumbnail_service::generate_thumbnails(&models, &app_state, false).await?;

        Ok(Json(model_ids).into_response())
    }
}
