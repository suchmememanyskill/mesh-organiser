use crate::{user::AuthSession, web_app_state::WebAppState};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Json, Router,
};
use axum_login::login_required;
use crate::user::Backend;
use axum::extract::Path;
use db::model::{hash_password, User};
use db::user_db;
use service::export_service;
use serde::{Deserialize, Serialize};
use crate::error::ApplicationError;
use db::model::UserPermissions;

pub fn router() -> Router<WebAppState> {
    Router::new()
        .nest(
            "/api/v1",
            Router::new()
                .route("/users", get(get::get_users))
                .route("/users", post(post::add_user))
                .route("/users/{user_id}", put(put::edit_user))
                .route("/users/{user_id}", delete(delete::delete_user))
                .route("/users/{user_id}/password", put(put::edit_user_password))
                .route("/users/{user_id}/permissions", put(put::edit_user_permissions))
                .route_layer(login_required!(Backend))
        )
}

mod get {
    use super::*;

    pub async fn get_users(
        auth_session: AuthSession,
        State(app_state): State<WebAppState>,
    ) -> Result<Response, ApplicationError> {
        let user = auth_session.user.unwrap().to_user();

        if !user.permissions.contains(UserPermissions::Admin) {
            return Err(ApplicationError::InternalError(
                "Insufficient permissions to view users.".into(),
            ));
        }

        let users = user_db::get_users(&app_state.app_state.db).await?;

        Ok(Json(users).into_response())
    }
}

mod post {
    use super::*;

    #[derive(Deserialize)]
    pub struct PostUserParams {
        pub user_name: String,
        pub user_email: String,
        pub user_password: String,
    }

    #[derive(Serialize)]
    pub struct PostUserResponse {
        pub id: i64,
    }

    pub async fn add_user(
        auth_session: AuthSession,
        State(app_state): State<WebAppState>,
        Json(params): Json<PostUserParams>,
    ) -> Result<Response, ApplicationError> {
        let user = auth_session.user.unwrap().to_user();

        if !user.permissions.contains(UserPermissions::Admin) {
            return Err(ApplicationError::InternalError(
                "Insufficient permissions to add a new user.".into(),
            ));
        }

        let id = user_db::add_user(
            &app_state.app_state.db,
            &params.user_name,
            &params.user_email,
            &params.user_password,
        ).await?;

        user_db::scramble_validity_token(&app_state.app_state.db, id).await?;

        Ok(Json(PostUserResponse { id }).into_response())
    }
}

mod put {
    use super::*;

    #[derive(Deserialize)]
    pub struct PutUserParams {
        pub user_name: String,
        pub user_email: String,
    }

    pub async fn edit_user(
        auth_session: AuthSession,
        Path(user_id): Path<i64>,
        State(app_state): State<WebAppState>,
        Json(params): Json<PutUserParams>,
    ) -> Result<Response, ApplicationError> {
        let user = auth_session.user.unwrap().to_user();

        if !user.permissions.contains(UserPermissions::Admin) && user.id != user_id {
            return Err(ApplicationError::InternalError(
                "Insufficient permissions to change this user's password.".into(),
            ));
        }

        user_db::edit_user_min(
            &app_state.app_state.db,
            user_id,
            &params.user_name,
            &params.user_email,
        ).await?;

        Ok(StatusCode::OK.into_response())
    }

    #[derive(Deserialize)]
    pub struct PutUserPasswordParams {
        pub new_password: String,
    }

    pub async fn edit_user_password(
        auth_session: AuthSession,
        Path(user_id): Path<i64>,
        State(app_state): State<WebAppState>,
        Json(params): Json<PutUserPasswordParams>,
    ) -> Result<Response, ApplicationError> {
        let user = auth_session.user.unwrap().to_user();

        if !user.permissions.contains(UserPermissions::Admin) && user.id != user_id {
            return Err(ApplicationError::InternalError(
                "Insufficient permissions to change this user's password.".into(),
            ));
        }

        user_db::edit_user_password(
            &app_state.app_state.db,
            user_id,
            &hash_password(&params.new_password),
        ).await?;

        Ok(StatusCode::OK.into_response())
    }

    #[derive(Deserialize)]
    pub struct PutUserPermissionsParams {
        pub permissions: UserPermissions,
    }

    pub async fn edit_user_permissions(
        auth_session: AuthSession,
        Path(user_id): Path<i64>,
        State(app_state): State<WebAppState>,
        Json(params): Json<PutUserPermissionsParams>,
    ) -> Result<Response, ApplicationError> {
        let user = auth_session.user.unwrap().to_user();

        if !user.permissions.contains(UserPermissions::Admin) {
            return Err(ApplicationError::InternalError(
                "Insufficient permissions to change user permissions.".into(),
            ));
        }

        user_db::set_user_permissions(
            &app_state.app_state.db,
            user_id,
            params.permissions,
        ).await?;

        Ok(StatusCode::OK.into_response())
    }
}

mod delete {
    use super::*;

    pub async fn delete_user(
        auth_session: AuthSession,
        Path(user_id): Path<i64>,
        State(app_state): State<WebAppState>,
    ) -> Result<Response, ApplicationError> {
        let user = auth_session.user.unwrap().to_user();
        
        if !user.permissions.contains(UserPermissions::Admin) && user.id != user_id {
            return Err(ApplicationError::InternalError(
                "Insufficient permissions to delete this user.".into(),
            ));
        }
        
        user_db::delete_user(&app_state.app_state.db, user_id).await?;

        export_service::delete_dead_blobs(&app_state.app_state).await?;

        Ok(StatusCode::OK.into_response())
    }
}
