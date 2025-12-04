use crate::{
    user::{AuthSession, Backend},
    web_app_state::WebAppState,
};
use axum::extract::Path;
use axum::extract::State;
use axum::{Json, response::Response};
use axum::{
    Router,
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
};
use axum_login::login_required;
use db::share_db;
use serde::Deserialize;

use crate::error::ApplicationError;
use serde::Serialize;

pub fn router() -> Router<WebAppState> {
    Router::new().nest(
        "/api/v1",
        Router::new()
            .route("/shares", get(get::get_shares))
            .route("/shares", post(post::create_share))
            .route("/shares/{share_id}", put(put::edit_share))
            .route("/shares/{share_id}", delete(delete::delete_share))
            .route("/shares/{share_id}/models", put(put::set_model_ids_on_share))
            .route_layer(login_required!(Backend))
            .route("/shares/{share_id}", get(get::get_share)),
    )
}

mod get {
    use db::{model::ShareDto, user_db};

    use super::*;

    pub async fn get_shares(
        auth_session: AuthSession,
        State(app_state): State<WebAppState>,
    ) -> Result<Response, ApplicationError> {
        let user = auth_session.user.unwrap().to_user();
        let shares = share_db::get_shares(&app_state.app_state.db, &user).await?;

        let shares : Vec<ShareDto> = shares.into_iter().map(|s| {
            s.to_dto(user.username.clone())
        }).collect();

        Ok(Json(shares).into_response())
    }

    pub async fn get_share(
        Path(share_id): Path<String>,
        State(app_state): State<WebAppState>,
    ) -> Result<Response, ApplicationError> {
        let share = share_db::get_share_via_id(&app_state.app_state.db, &share_id).await?;
        
        let user = match user_db::get_user_by_id(&app_state.app_state.db, share.user_id).await? {
            Some(u) => u,
            _ => return Err(ApplicationError::InternalError(
                "Share owner user not found.".into(),
            )),
        };

        let share = share.to_dto(user.username);

        Ok(Json(share).into_response())
    }
}

mod post {
    use db::{model::ShareDto, time_now};

    use super::*;

    #[derive(Deserialize)]
    pub struct CreateShareParams {
        pub share_name: String,
    }

    pub async fn create_share(
        auth_session: AuthSession,
        State(app_state): State<WebAppState>,
        Json(params): Json<CreateShareParams>,
    ) -> Result<Response, ApplicationError> {
        let user = auth_session.user.unwrap().to_user();
        let share_id = share_db::create_share(&app_state.app_state.db, &user, &params.share_name).await?;

        Ok(Json(ShareDto {
            id: share_id,
            share_name: params.share_name,
            user_name: user.username,
            model_ids: Vec::new(),
            created_at: time_now(),
        }).into_response())
    }
}

mod put {
    use super::*;

    #[derive(Deserialize)]
    pub struct EditShareParams {
        pub share_name: String,
    }

    pub async fn edit_share(
        auth_session: AuthSession,
        Path(share_id): Path<String>,
        State(app_state): State<WebAppState>,
        Json(params): Json<EditShareParams>,
    ) -> Result<Response, ApplicationError> {
        let user = auth_session.user.unwrap().to_user();
        share_db::rename_share(&app_state.app_state.db, &user, &share_id, &params.share_name).await?;

        Ok(StatusCode::NO_CONTENT.into_response())
    }

    #[derive(Deserialize)]
    pub struct SetModelIdsOnShareParams {
        pub model_ids: Vec<i64>,
    }

    pub async fn set_model_ids_on_share(
        auth_session: AuthSession,
        Path(share_id): Path<String>,
        State(app_state): State<WebAppState>,
        Json(params): Json<SetModelIdsOnShareParams>,
    ) -> Result<Response, ApplicationError> {
        let user = auth_session.user.unwrap().to_user();
        share_db::set_model_ids_on_share(&app_state.app_state.db, &user, &share_id, params.model_ids).await?;

        Ok(StatusCode::NO_CONTENT.into_response())
    }
}

mod delete {
    use super::*;

    pub async fn delete_share(
        auth_session: AuthSession,
        Path(share_id): Path<String>,
        State(app_state): State<WebAppState>,
    ) -> Result<Response, ApplicationError> {
        let user = auth_session.user.unwrap().to_user();
        share_db::delete_share(&app_state.app_state.db, &user, &share_id).await?;

        Ok(StatusCode::NO_CONTENT.into_response())
    }
}
