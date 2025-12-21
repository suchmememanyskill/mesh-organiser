use crate::{
    user::{AuthSession, Backend},
    web_app_state::WebAppState,
};
use axum::{Router, http::StatusCode, response::IntoResponse, routing::{get, post}};
use axum_login::login_required;
use async_zip::tokio::read::seek::ZipFileReader;
use axum::{
    body::Body,
    extract::{Path, State}, response::Response,
};

use axum_extra::extract::Query;
use db::{model::{Blob, User}, model_db, user_db};
use serde::Deserialize;
use service::{cleanse_evil_from_name, convert_zip_to_extension, export_service::get_model_path_for_blob, is_zipped_file_extension};
use tokio::{fs::File, io::BufReader};
use tokio_util::{compat::FuturesAsyncReadCompatExt, io::ReaderStream};
use axum::Json;
use service::export_service;
use crate::error::ApplicationError;

pub fn router() -> Router<WebAppState> {
    Router::new().nest(
        "/api/v1",
        Router::new()
            .route("/models/{model_id}/bytes", get(get::get_model_bytes))
            .route("/blobs/{sha256}/bytes", get(get::get_blob_bytes))
            .route("/blobs/download", post(post::create_blobs_zip_download))
            .route_layer(login_required!(Backend))
            .route("/blobs/{sha256}/thumb", get(get::get_blob_thumb))
            .route("/blobs/{sha256}/download", get(get::download_model))
            .route("/blobs/download/{zip_dir}", get(get::get_blobs_zip_download))
    )
}

mod get {
    use super::*;

    #[derive(Deserialize)]
    pub struct DownloadModelParams {
        pub user_id: Option<i64>,
        pub user_hash: Option<String>,
        pub share_id: Option<String>,
    }

    async fn extract_user_via_id_and_hash(
        app_state: &WebAppState,
        user_id: i64,
        user_hash: &String,
    ) -> Option<User> {
        let user = match user_db::get_user_by_id(&app_state.app_state.db, user_id).await {
            Ok(Some(u)) => u,
            _ => return None,
        };

        if user.sync_url.is_none() || user.sync_url.clone().unwrap() != *user_hash {
            return None;
        }

        Some(user)
    }

    async fn extract_user_via_share_id(
        app_state: &WebAppState,
        share_id: &String,
    ) -> Option<User> {
        let share = match db::share_db::get_share_via_id(&app_state.app_state.db, share_id).await {
            Ok(s) => s,
            _ => return None,
        };

        let user = match user_db::get_user_by_id(&app_state.app_state.db, share.user_id).await {
            Ok(Some(u)) => u,
            _ => return None,
        };

        Some(user)
    }

    pub async fn download_model(
        Path(blob_sha256): Path<String>,
        State(app_state): State<WebAppState>,
        Query(params): Query<DownloadModelParams>,
    ) -> Response {

        let user = match params {
            DownloadModelParams {
                user_id: Some(user_id),
                user_hash: Some(user_hash),
                share_id: None,
            } => {
                match extract_user_via_id_and_hash(&app_state, user_id, &user_hash).await {
                    Some(u) => u,
                    None => return StatusCode::NOT_FOUND.into_response(),
                }
            }
            DownloadModelParams {
                user_id: None,
                user_hash: None,
                share_id: Some(share_id),
            } => {
                match extract_user_via_share_id(&app_state, &share_id).await {
                    Some(u) => u,
                    None => return StatusCode::NOT_FOUND.into_response(),
                }
            }
            _ => return StatusCode::NOT_FOUND.into_response(),
        };

        let model_id = match model_db::get_model_id_via_sha256(&app_state.app_state.db, &user, &blob_sha256).await {
            Ok(Some(m)) => m,
            _ => return StatusCode::NOT_FOUND.into_response(),
        };

        let model = match model_db::get_models_via_ids(&app_state.app_state.db, &user, vec![model_id]).await {
            Ok(m) => m,
            Err(_) => return StatusCode::NOT_FOUND.into_response(),
        };

        if model.len() <= 0 {
            return StatusCode::NOT_FOUND.into_response();
        }

        let model = &model[0];

        if model.blob.sha256 != blob_sha256 {
            return StatusCode::NOT_FOUND.into_response();
        }

        let filename = format!("{}.{}", cleanse_evil_from_name(&model.name).trim(), convert_zip_to_extension(&model.blob.filetype)).to_ascii_lowercase();
        let mut response = get_blob_bytes_inner(&model.blob, &app_state)
            .await;

        response.headers_mut().insert(
            axum::http::header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", filename)
                .parse()
                .unwrap(),
        );
        
        response
    }

    pub async fn get_model_bytes(
        auth_session: AuthSession,
        Path(model_id): Path<i64>,
        State(app_state): State<WebAppState>,
    ) -> Response {
        let user = auth_session.user.unwrap().to_user();

        let model =
            match db::model_db::get_models_via_ids(&app_state.app_state.db, &user, vec![model_id])
                .await
            {
                Ok(m) => m,
                Err(_) => return StatusCode::NOT_FOUND.into_response(),
            };

        if model.len() <= 0 {
            return StatusCode::NOT_FOUND.into_response();
        }

        let model = &model[0];

        get_blob_bytes_inner(&model.blob, &app_state)
            .await
    }

    pub async fn get_blob_bytes(
        auth_session: AuthSession,
        Path(sha256): Path<String>,
        State(app_state): State<WebAppState>,
    ) -> Response {
        let user = auth_session.user.unwrap().to_user();

        // Verify that the user has access to a model with this blob
        match model_db::get_model_id_via_sha256(&app_state.app_state.db, &user, &sha256).await {
            Ok(Some(m)) => m,
            _ => return StatusCode::NOT_FOUND.into_response(),
        };

        let blob = match db::blob_db::get_blob_via_sha256(&app_state.app_state.db, &sha256).await {
            Ok(Some(b)) => b,
            _ => return StatusCode::NOT_FOUND.into_response(),
        };

        get_blob_bytes_inner(&blob, &app_state)
            .await
            .into_response()
    }

    pub async fn get_blob_thumb(
        Path(sha256): Path<String>,
        State(app_state): State<WebAppState>,
    ) -> Response {
        let base_dir = app_state.get_image_dir();
        let src_file_path = base_dir.join(format!("{}.png", sha256));

        let file = match File::open(src_file_path).await {
            Ok(f) => f,
            Err(_) => return StatusCode::NOT_FOUND.into_response(),
        };

        let buffered_reader = BufReader::new(file);
        let stream = ReaderStream::new(buffered_reader);

        return Body::from_stream(stream).into_response();
    }

    async fn get_blob_bytes_inner(
        blob: &Blob,
        app_state: &WebAppState,
    ) -> Response {
        let src_file_path = get_model_path_for_blob(&blob, &app_state.app_state);

        let file = match File::open(src_file_path).await {
            Ok(f) => f,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };

        let buffered_reader = BufReader::new(file);

        if is_zipped_file_extension(&blob.filetype) {
            let archive = match ZipFileReader::with_tokio(buffered_reader).await {
                Ok(a) => a,
                Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            };
            let file = match archive.into_entry(0).await {
                Ok(f) => f,
                Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            };

            let stream = ReaderStream::new(file.compat());

            return Body::from_stream(stream).into_response();
        } else {
            let stream = ReaderStream::new(buffered_reader);

            return Body::from_stream(stream).into_response();
        }
    }

    pub async fn get_blobs_zip_download(
        Path(zip_dir): Path<String>,
    ) -> Result<Response, ApplicationError> {
        if !zip_dir.starts_with("meshorganiser_") {
            return Ok(StatusCode::BAD_REQUEST.into_response());
        }

        let path = std::env::temp_dir().join(zip_dir);

        if !path.exists() {
            return Ok(StatusCode::NOT_FOUND.into_response());
        }

        let mut list_dir = tokio::fs::read_dir(&path).await?;
        let next = list_dir.next_entry().await?;

        let zip_file = match next {
            Some(f) => f,
            None => return Ok(StatusCode::NOT_FOUND.into_response()),
        };

        let t = zip_file.file_name();
        let filename = t.to_string_lossy();
        if !filename.ends_with(".zip") {
            return Ok(StatusCode::NOT_FOUND.into_response());
        }

        let path = zip_file.path();

        if !path.exists() {
            return Ok(StatusCode::NOT_FOUND.into_response());
        }

        let file = File::open(path).await?;
        let buffered_reader = BufReader::new(file);
        let stream = ReaderStream::new(buffered_reader);

        let mut response = Body::from_stream(stream).into_response();

        response.headers_mut().insert(
            axum::http::header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", filename)
                .parse()
                .unwrap(),
        );

        return Ok(response);
    }
}


mod post {
    use super::*;

    pub async fn create_blobs_zip_download(
        auth_session: AuthSession,
        State(app_state): State<WebAppState>,
        Json(blob_sha256s): Json<Vec<String>>,
    ) -> Result<Response, ApplicationError> {
        let mut model_ids = Vec::with_capacity(blob_sha256s.len());
        let user = auth_session.user.unwrap().to_user();

        // TODO: This is slow, optimise later
        for sha256 in blob_sha256s {
            let id = match model_db::get_model_id_via_sha256(&app_state.app_state.db, &user, &sha256).await {
                Ok(Some(m)) => m,
                _ => return Ok(StatusCode::NOT_FOUND.into_response()),
            };

            model_ids.push(id);
        }
        let model_ids_len = model_ids.len();
        let models = match model_db::get_models_via_ids(&app_state.app_state.db, &user, model_ids).await {
            Ok(m) => m,
            Err(_) => return Ok(StatusCode::NOT_FOUND.into_response()),
        };
        
        if models.len() != model_ids_len {
            return Ok(StatusCode::NOT_FOUND.into_response());
        }
        
        let path = export_service::export_zip_to_temp_folder(models, &app_state.app_state).await?;

        Ok(Json(path.temp_dir.file_name().unwrap().to_string_lossy().to_string()).into_response())
    }
}