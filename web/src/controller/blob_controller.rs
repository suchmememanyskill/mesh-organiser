use crate::{
    user::{AuthSession, Backend},
    web_app_state::WebAppState,
};
use axum::{Router, http::StatusCode, response::IntoResponse, routing::get};
use axum_login::login_required;

pub fn router() -> Router<WebAppState> {
    Router::new().nest(
        "/api/v1",
        Router::new()
            .route("/models/{model_id}/bytes", get(get::get_model_bytes))
            .route("/blobs/{sha256}/bytes", get(get::get_blob_bytes))
            .route_layer(login_required!(Backend))
            .route("/blobs/{sha256}/thumb", get(get::get_blob_thumb))
            .route("/blobs/{sha256}/download", get(get::download_model)),
    )
}

mod get {
    use async_zip::tokio::read::seek::ZipFileReader;
    use axum::{
        body::Body,
        extract::{Path, State}, response::Response,
    };

    use axum_extra::extract::Query;
    use db::{model_db, user_db};
    use serde::Deserialize;
    use service::{cleanse_evil_from_name, convert_zip_to_extension, is_zipped_file_extension};
    use tokio::{fs::File, io::BufReader};
    use tokio_util::{compat::FuturesAsyncReadCompatExt, io::ReaderStream};

    use crate::web_app_state::WebAppState;

    use super::*;

    #[derive(Deserialize)]
    pub struct DownloadModelParams {
        pub user_id: i64,
        pub user_hash: String,
    }

    pub async fn download_model(
        Path(blob_sha256): Path<String>,
        State(app_state): State<WebAppState>,
        Query(params): Query<DownloadModelParams>,
    ) -> Response {
        let user = match user_db::get_user_by_id(&app_state.app_state.db, params.user_id).await {
            Ok(Some(u)) => u,
            _ => return StatusCode::NOT_FOUND.into_response(),
        };

        if user.sync_url.is_none() || user.sync_url.clone().unwrap() != params.user_hash {
            return StatusCode::NOT_FOUND.into_response();
        }

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
        let mut response = get_blob_bytes_inner(&model.blob.sha256, &model.blob.filetype, &app_state)
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

        get_blob_bytes_inner(&model.blob.sha256, &model.blob.filetype, &app_state)
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

        get_blob_bytes_inner(&blob.sha256, &blob.filetype, &app_state)
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
        sha256: &String,
        filetype: &String,
        app_state: &WebAppState,
    ) -> Response {
        let base_dir = app_state.get_model_dir();
        let src_file_path = base_dir.join(format!("{}.{}", sha256, filetype));

        let file = match File::open(src_file_path).await {
            Ok(f) => f,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };

        let buffered_reader = BufReader::new(file);

        if is_zipped_file_extension(&filetype) {
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
}
