use crate::error::ApplicationError;
use crate::user::Backend;
use crate::{user::AuthSession, web_app_state::WebAppState};
use axum::extract::Path;
use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
};
use axum_login::login_required;
use db::model::{ResourceFlags, ResourceMeta};
use db::{random_hex_32, resource_db, time_now};
use serde::Deserialize;
use service::resource_service;

pub fn router() -> Router<WebAppState> {
    Router::new().nest(
        "/api/v1",
        Router::new()
            .route("/resources", get(get::get_resources))
            .route("/resources", post(post::add_resource))
            .route("/resources/{resource_id}", put(put::edit_resource))
            .route("/resources/{resource_id}", delete(delete::delete_resource))
            .route(
                "/resources/{resource_id}/groups",
                get(get::get_groups_for_resource),
            )
            .route(
                "/groups/{group_id}/resource",
                put(put::set_resource_on_group),
            )
            .route_layer(login_required!(Backend)),
    )
}

mod get {
    use super::*;

    pub async fn get_resources(
        auth_session: AuthSession,
        State(app_state): State<WebAppState>,
    ) -> Result<Response, ApplicationError> {
        let user = auth_session.user.unwrap().to_user();
        let resources = resource_db::get_resources(&app_state.app_state.db, &user).await?;

        Ok(Json(resources).into_response())
    }

    pub async fn get_groups_for_resource(
        auth_session: AuthSession,
        Path(resource_id): Path<i64>,
        State(app_state): State<WebAppState>,
    ) -> Result<Response, ApplicationError> {
        let user = auth_session.user.unwrap().to_user();
        let groups =
            resource_db::get_groups_for_resource(&app_state.app_state.db, &user, resource_id)
                .await?;

        Ok(Json(groups).into_response())
    }
}

mod post {
    use super::*;

    #[derive(Deserialize)]
    pub struct PostResourceParams {
        pub resource_name: String,
    }

    pub async fn add_resource(
        auth_session: AuthSession,
        State(app_state): State<WebAppState>,
        Json(params): Json<PostResourceParams>,
    ) -> Result<Response, ApplicationError> {
        let user = auth_session.user.unwrap().to_user();
        let id =
            resource_db::add_resource(&app_state.app_state.db, &user, &params.resource_name, None)
                .await?;

        let resource_meta = ResourceMeta {
            id,
            name: params.resource_name,
            flags: ResourceFlags::empty(),
            created: time_now(),
            unique_global_id: random_hex_32(),
            last_modified: time_now(),
        };

        Ok(Json(resource_meta).into_response())
    }
}

mod put {
    use crate::controller::EditGlobalId;

    use super::*;

    #[derive(Deserialize)]
    pub struct PutResourceParams {
        pub resource_name: String,
        pub resource_flags: ResourceFlags,
        pub resource_timestamp: Option<String>,
        pub resource_global_id: Option<String>,
    }

    pub async fn edit_resource(
        auth_session: AuthSession,
        Path(resource_id): Path<i64>,
        State(app_state): State<WebAppState>,
        Json(params): Json<PutResourceParams>,
    ) -> Result<Response, ApplicationError> {
        let user = auth_session.user.unwrap().to_user();
        resource_db::edit_resource(
            &app_state.app_state.db,
            &user,
            resource_id,
            &params.resource_name,
            params.resource_flags,
            params.resource_timestamp.as_deref(),
        )
        .await?;

        if let Some(new_global_id) = params.resource_global_id {
            resource_db::edit_resource_global_id(
                &app_state.app_state.db,
                &user,
                resource_id,
                &new_global_id,
            )
            .await?;
        }

        Ok(StatusCode::NO_CONTENT.into_response())
    }
    
    #[derive(Deserialize)]
    pub struct SetResourceOnGroupParams {
        pub resource_id: Option<i64>,
    }

    pub async fn set_resource_on_group(
        auth_session: AuthSession,
        Path(group_id): Path<i64>,
        State(app_state): State<WebAppState>,
        Json(params): Json<SetResourceOnGroupParams>,
    ) -> Result<Response, ApplicationError> {
        let user = auth_session.user.unwrap().to_user();
        resource_db::set_resource_on_group(
            &app_state.app_state.db,
            &user,
            params.resource_id,
            group_id,
            None,
        )
        .await?;

        Ok(StatusCode::NO_CONTENT.into_response())
    }
}

mod delete {
    use super::*;

    pub async fn delete_resource(
        auth_session: AuthSession,
        Path(resource_id): Path<i64>,
        State(app_state): State<WebAppState>,
    ) -> Result<Response, ApplicationError> {
        let user = auth_session.user.unwrap().to_user();
        let resource =
            resource_db::get_resource_meta_by_id(&app_state.app_state.db, &user, resource_id)
                .await?;

        if resource.is_none() {
            return Err(ApplicationError::InternalError(String::from(
                "Resource not found",
            )));
        }

        let resource = resource.unwrap();

        resource_service::delete_resource_folder(&resource, &user, &app_state.app_state).await?;
        resource_db::delete_resource(&app_state.app_state.db, &user, resource.id).await?;

        Ok(StatusCode::NO_CONTENT.into_response())
    }
}
