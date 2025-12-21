use crate::user::{Credentials, PasswordCredentials, TokenCredentials};
use crate::{user::AuthSession, web_app_state::WebAppState};
use axum::{Router, http::StatusCode, response::IntoResponse, routing::{post, get}};
use axum::Json;

pub fn router() -> Router<WebAppState> {
    Router::new().nest(
        "/api/v1",
        Router::new()
            .route("/login/password", post(post::password))
            .route("/login/token", post(post::token))
            .route("/users/me", get(get::me))
            .route("/logout", post(post::logout)),
    )
}

mod get {
    use axum::{extract::State, response::Response};
    use db::user_db;

    use crate::error::ApplicationError;

    use super::*;

    pub async fn me(
        auth_session: AuthSession,
        State(app_state): State<WebAppState>,
    ) -> Result<Response, ApplicationError> {
        let user = match auth_session.user {
            Some(u) => u.to_user(),
            None => return Ok(StatusCode::UNAUTHORIZED.into_response()),
        };

        let user = user_db::get_user_by_id(&app_state.app_state.db, user.id).await?;

        let user = match user {
            Some(u) => u,
            None => return Ok(StatusCode::UNAUTHORIZED.into_response()),
        };

        Ok(Json(user).into_response())
    }
}

mod post {
    use super::*;

    pub async fn password(
        mut auth_session: AuthSession,
        Json(creds): Json<PasswordCredentials>,
    ) -> impl IntoResponse {
        let user = match auth_session
            .authenticate(Credentials::Password(creds))
            .await
        {
            Ok(Some(user)) => user,
            Ok(None) => {
                return (StatusCode::UNAUTHORIZED, "Invalid username or password").into_response();
            }
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };

        if auth_session.login(&user).await.is_err() {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }

        StatusCode::NO_CONTENT.into_response()
    }

    pub async fn token(
        mut auth_session: AuthSession,
        Json(creds): Json<TokenCredentials>,
    ) -> impl IntoResponse {
        let user = match auth_session.authenticate(Credentials::Token(creds)).await {
            Ok(Some(user)) => user,
            Ok(None) => {
                return (StatusCode::UNAUTHORIZED, "Invalid token").into_response();
            }
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };

        if auth_session.login(&user).await.is_err() {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }

        StatusCode::NO_CONTENT.into_response()
    }

    pub async fn logout(mut auth_session: AuthSession) -> impl IntoResponse {
        if auth_session.logout().await.is_err() {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }

        StatusCode::NO_CONTENT.into_response()
    }
}
