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
use std::str::FromStr;
use time::OffsetDateTime;
use tokio::fs;

use crate::error::ApplicationError;
use db::blob_db;
use db::model_db::{ModelFilterOptions, ModelOrderBy};
use serde::Serialize;
use service::export_service;

pub fn router() -> Router<WebAppState> {
    Router::new().nest(
        "/api/v1",
        Router::new()
            .route("/models", post(post::add_model))
            .route("/models", get(get::get_models))
            .route("/models", delete(delete::delete_models))
            .route("/models/count", get(get::get_model_count))
            .route("/models/disk_usage", get(get::get_model_disk_space_usage))
            .route("/models/{model_id}", put(put::edit_model))
            .route("/models/{model_id}", delete(delete::delete_model))
            .route_layer(login_required!(Backend))
            .route("/shares/{share_id}/models", get(get::get_share_models)),
    )
}

mod get {
    use axum_extra::extract::Query;
    use db::{model::{FileType, User}, share_db};

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
        #[serde(default)]
        pub file_types: Vec<FileType>,
    }

    async fn get_models_inner(
        app_state: &WebAppState,
        user: &User,
        params: GetModelParams,
    ) -> Result<Response, ApplicationError> {
        let flags = params.model_flags;

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
                file_types: if params.file_types.is_empty() { None } else { Some(params.file_types) },
            },
        )
        .await?;

        Ok(Json(models.items).into_response())
    }

    pub async fn get_models(
        auth_session: AuthSession,
        State(app_state): State<WebAppState>,
        Query(params): Query<GetModelParams>,
    ) -> Result<Response, ApplicationError> {
        let user = auth_session.user.unwrap().to_user();

        get_models_inner(&app_state, &user, params).await
    }

    pub async fn get_share_models(
        Path(share_id): Path<String>,
        State(app_state): State<WebAppState>,
        Query(mut params): Query<GetModelParams>,
    ) -> Result<Response, ApplicationError> {
        let share = share_db::get_share_via_id(&app_state.app_state.db, &share_id).await?;

        params.model_ids = match params.model_ids.is_empty() {
            true => vec![],
            false => share.model_ids.into_iter().filter(|x| params.model_ids.contains(x)).collect(),
        };

        params.group_ids = vec![];
        params.label_ids = vec![];
        
        get_models_inner(&app_state, &User { 
            id: share.user_id,
            ..Default::default()
        }, params).await
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
        pub model_timestamp: Option<String>,
        pub model_global_id: Option<String>,
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
            params.model_timestamp.as_deref(),
        )
        .await?;

        if let Some(new_global_id) = params.model_global_id {
            model_db::edit_model_global_id(
                &app_state.app_state.db,
                &user,
                model_id,
                &new_global_id,
            )
            .await?;
        }

        Ok(StatusCode::NO_CONTENT.into_response())
    }
}

mod delete {
    use db::model::User;
    use service::export_service::delete_dead_blobs;

    use super::*;

    pub async fn delete_model(
        auth_session: AuthSession,
        Path(model_id): Path<i64>,
        State(app_state): State<WebAppState>,
    ) -> Result<Response, ApplicationError> {
        let user = auth_session.user.unwrap().to_user();
        
        delete_model_inner(&app_state, &user, vec![model_id]).await?;
        delete_dead_blobs(&app_state.app_state).await?;

        Ok(StatusCode::NO_CONTENT.into_response())
    }

    #[derive(Deserialize)]
    pub struct DeleteModelsParams {
        pub model_ids: Vec<i64>,
    }

    pub async fn delete_models(
        auth_session: AuthSession,
        State(app_state): State<WebAppState>,
        Json(params): Json<DeleteModelsParams>,
    ) -> Result<Response, ApplicationError> {
        let user = auth_session.user.unwrap().to_user();
        
        delete_model_inner(&app_state, &user, params.model_ids).await?;
        delete_dead_blobs(&app_state.app_state).await?;

        Ok(StatusCode::NO_CONTENT.into_response())
    }

    async fn delete_model_inner(
        app_state: &WebAppState,
        user: &User,
        model_ids: Vec<i64>,
    ) -> Result<(), ApplicationError> {
        let ids_len = model_ids.len();
        let models =
            model_db::get_models_via_ids(&app_state.app_state.db, &user, model_ids).await?;

        if models.len() != ids_len {
            return Err(ApplicationError::InternalError(String::from(
                "Failed to find model to delete",
            )));
        }

        let ids = models.iter().map(|m| m.id).collect::<Vec<i64>>();

        model_db::delete_models(&app_state.app_state.db, user, &ids).await?;

        Ok(())
    }
}

mod post {
    use db::{model::Blob, random_hex_32};
    use service::thumbnail_service;
    use tokio::io::AsyncWriteExt;

    use crate::web_import_state::WebImportStateEmitter;

    use super::*;

    pub async fn add_model(
        auth_session: AuthSession,
        State(app_state): State<WebAppState>,
        mut multipart: Multipart,
    ) -> Result<Response, ApplicationError> {
        let user = auth_session.user.unwrap().to_user();
        let mut paths = vec![];

        let temp_dir = std::env::temp_dir().join(format!(
            "meshorganiser_import_action_{}_{}",
            OffsetDateTime::now_utc().unix_timestamp(),
            random_hex_32()
        ));

        std::fs::create_dir(&temp_dir)?;

        let mut link = None;

        while let Some(mut field) = multipart.next_field().await? {
            if let Some("source_url") = field.name() {
                link = Some(field.text().await?);
                continue;
            };

            let file_name = match field.file_name() {
                Some(name) => name.to_string(),
                None => continue,
            };

            let file_path = temp_dir.join(cleanse_evil_from_name(&file_name));

            if !(import_service::is_supported_extension(&file_path) 
                || file_path.extension().unwrap().to_str().unwrap().to_lowercase() == "zip") {
                continue;
            }

            let mut file = fs::File::create(&file_path).await?;

            while let Some(chunk) = field.chunk().await? {
                #[cfg(debug_assertions)]
                println!("Writing chunk of size {} for file {}", chunk.len(), file_name);
                file.write(&chunk).await?;
            }

            file.flush().await?;

            paths.push(file_path);
        }

        drop(multipart);

        if paths.is_empty() {
            return Ok((StatusCode::BAD_REQUEST, "No files uploaded").into_response());
        }

        let mut model_ids: Vec<i64> = vec![];

        let mut import_state = ImportState::new_with_emitter(None, false, true, false, user.clone(), Box::new(WebImportStateEmitter {}));

        for path in paths {
            println!("Importing file: {}", path.to_string_lossy());
            import_state = ImportState::new_with_emitter(link.clone(), false, true, false, user.clone(), Box::new(WebImportStateEmitter {}));
            import_state = import_service::import_path(
                &path.to_string_lossy(),
                &app_state.app_state,
                import_state,
            )
            .await?;

            model_ids.extend(&import_state.imported_models[0].model_ids);
        }

        let models = model_db::get_models_via_ids(&app_state.app_state.db, &user, model_ids.clone()).await?;
        let blobs: Vec<&Blob> = models.iter().map(|m| &m.blob).collect();

        thumbnail_service::generate_thumbnails(&blobs, &app_state.app_state, false, &mut import_state).await?;

        Ok(Json(model_ids).into_response())
    }
}
