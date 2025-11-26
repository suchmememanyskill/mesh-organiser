use crate::{user::AuthSession, web_app_state::WebAppState};
use axum::{
    extract::Query,
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
    Form, Router,
};
use crate::user::{Credentials, PasswordCredentials, TokenCredentials};

pub fn router() -> Router<WebAppState> {
    Router::new()
        .nest(
            "/api/v1",
            Router::new()
                .route("/login/password", post(post::password))
                .route("/login/token", post(post::token))
                .route("/logout", post(post::logout))
        )
}

mod post {
    use axum::Json;

    use super::*;

    pub async fn password(
        mut auth_session: AuthSession,
        Json(creds): Json<PasswordCredentials>,
    ) -> impl IntoResponse {
        let user = match auth_session
            .authenticate(Credentials::Password(creds))
            .await {
                Ok(Some(user)) => user,
                Ok(None) => {
                    return (StatusCode::UNAUTHORIZED, "Invalid username or password").into_response();
                },
                Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            };

        if auth_session.login(&user).await.is_err() {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }

        StatusCode::OK.into_response()
    }

    pub async fn token(
        mut auth_session: AuthSession,
        Json(creds): Json<TokenCredentials>,
    ) -> impl IntoResponse {
        let user = match auth_session
            .authenticate(Credentials::Token(creds))
            .await {
                Ok(Some(user)) => user,
                Ok(None) => {
                    return (StatusCode::UNAUTHORIZED, "Invalid token").into_response();
                },
                Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            };

        if auth_session.login(&user).await.is_err() {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }

        StatusCode::OK.into_response()
    }

    pub async fn logout(
        mut auth_session: AuthSession,
    ) -> impl IntoResponse {
        if auth_session.logout().await.is_err() {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }

        StatusCode::OK.into_response()
    }
}