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


pub fn router() -> Router<WebAppState> {
    Router::new()
        .nest(
            "/api/v1",
            Router::new()
                .route("/models/{model_id}/bytes", get(get::get_model_bytes))
                .route("/blobs/{sha256}/bytes", get(get::get_blob_bytes))
                .route_layer(login_required!(Backend))
        )
}
mod get {
    use async_zip::tokio::read::seek::ZipFileReader;
    use axum::{body::Body, extract::{Path, State}};
    use db::model::User;
    use service::is_zipped_file_extension;
    use tokio::{fs::File, io::BufReader};
    use tokio_util::{compat::FuturesAsyncReadCompatExt, io::ReaderStream};

    use crate::{error::ApplicationError, web_app_state::WebAppState};

    use super::*;

    pub async fn get_model_bytes(
        auth_session: AuthSession,
        Path(model_id): Path<i64>,
        State(app_state) : State<WebAppState>,
    ) -> impl IntoResponse {
        let user = auth_session.user.unwrap().to_user();

        let model = match db::model_db::get_models_via_ids(&app_state.app_state.db, &user, vec![model_id]).await {
            Ok(m) => m,
            Err(_) => return StatusCode::NOT_FOUND.into_response(),
        };

        if model.len() <= 0 {
            return StatusCode::NOT_FOUND.into_response();
        }

        let model = &model[0];

        get_blob_bytes_inner(&model.blob.sha256, &model.blob.filetype, &app_state).await.into_response()
    }

    pub async fn get_blob_bytes(
        auth_session: AuthSession,
        Path(sha256): Path<String>,
        State(app_state) : State<WebAppState>,
    ) -> impl IntoResponse {
        let _user = auth_session.user.unwrap().to_user();

        let blob = match db::blob_db::get_blob_via_sha256(&app_state.app_state.db, &sha256).await {
            Ok(Some(b)) => b,
            _ => return StatusCode::NOT_FOUND.into_response(),
        };

        get_blob_bytes_inner(&blob.sha256, &blob.filetype, &app_state).await.into_response()
    }

    pub async fn get_blob_thumb(
        auth_session: AuthSession,
        Path(sha256): Path<String>,
        State(app_state) : State<WebAppState>,
    ) -> impl IntoResponse {
        let base_dir = app_state.get_image_dir();
        let src_file_path = base_dir.join(format!("{}.png", sha256));

        let file = match File::open(src_file_path).await {
            Ok(f) => f,
            Err(_) => return StatusCode::NOT_FOUND.into_response(),
        };

        let buffered_reader = BufReader::new(file);
        let stream = ReaderStream::new(buffered_reader);

        return Body::from_stream(stream)
            .into_response();
    }

    async fn get_blob_bytes_inner(
        sha256: &String,
        filetype: &String,
        app_state : &WebAppState,
    ) -> impl IntoResponse {
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

            return Body::from_stream(stream)
                .into_response();
        } 
        else {
            let stream = ReaderStream::new(buffered_reader);

            return Body::from_stream(stream)
                .into_response();
        }
    }
}