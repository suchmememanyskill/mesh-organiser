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
use db::group_db::{GroupFilterOptions, GroupOrderBy};
use db::model::ModelGroupMeta;
use db::{group_db, random_hex_32, time_now};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub fn router() -> Router<WebAppState> {
    Router::new().nest(
        "/api/v1",
        Router::new()
            .route("/groups", get(get::get_groups))
            .route("/groups/count", get(get::get_group_count))
            .route("/groups", post(post::add_group))
            .route(
                "/groups/detach_models",
                delete(delete::remove_models_from_group),
            )
            .route("/groups/{group_id}", put(put::edit_group))
            .route("/groups/{group_id}", delete(delete::delete_group))
            .route("/groups/{group_id}/models", post(post::add_models_to_group))
            .route_layer(login_required!(Backend))
            .route("/shares/{share_id}/groups", get(get::get_share_groups)),
    )
}

mod get {
    use axum_extra::extract::Query;
    use db::{share_db, user_db};

    use super::*;

    #[derive(Deserialize)]
    pub struct GetGroupParams {
        #[serde(default)]
        pub model_ids: Vec<i64>,
        pub model_ids_str: Option<String>,
        #[serde(default)]
        pub group_ids: Vec<i64>,
        #[serde(default)]
        pub label_ids: Vec<i64>,
        pub order_by: Option<String>,
        pub text_search: Option<String>,
        pub page: u32,
        pub page_size: u32,
        pub include_ungrouped_models: Option<bool>,
    }

    pub async fn get_groups(
        auth_session: AuthSession,
        State(app_state): State<WebAppState>,
        Query(params): Query<GetGroupParams>,
    ) -> Result<Response, ApplicationError> {
        let user = auth_session.user.unwrap().to_user();
        let model_ids_from_str = params.model_ids_str.map(|s| {
            s.split(',')
                .filter_map(|x| x.parse::<i64>().ok())
                .collect::<Vec<i64>>()
        });

        let groups = group_db::get_groups(
            &app_state.app_state.db,
            &user,
            GroupFilterOptions {
                model_ids: if params.model_ids.is_empty() { model_ids_from_str } else { Some(params.model_ids) },
                group_ids: if params.group_ids.is_empty() { None } else { Some(params.group_ids) },
                label_ids: if params.label_ids.is_empty() { None } else { Some(params.label_ids) },
                order_by: params
                    .order_by
                    .map(|s| GroupOrderBy::from_str(&s).unwrap_or(GroupOrderBy::NameAsc)),
                text_search: params.text_search,
                page: params.page,
                page_size: params.page_size,
                include_ungrouped_models: params.include_ungrouped_models.unwrap_or(false),
            },
        )
        .await?;

        Ok(Json(groups.items).into_response())
    }

    pub async fn get_share_groups(
        Path(share_id): Path<String>,
        State(app_state): State<WebAppState>,
        Query(params): Query<GetGroupParams>,
    ) -> Result<Response, ApplicationError> {
        let share = share_db::get_share_via_id(&app_state.app_state.db, &share_id).await?;
        let user = match user_db::get_user_by_id(&app_state.app_state.db, share.user_id).await? {
            Some(u) => u,
            _ => return Err(ApplicationError::InternalError(
                "Share owner user not found.".into(),
            )),
        };

        let groups = group_db::get_groups(
            &app_state.app_state.db, 
            &user, 
            GroupFilterOptions {
                model_ids: share.model_ids.into(),
                group_ids: if params.group_ids.is_empty() { None } else { Some(params.group_ids) },
                label_ids: None,
                order_by: params
                    .order_by
                    .map(|s| GroupOrderBy::from_str(&s).unwrap_or(GroupOrderBy::NameAsc)),
                text_search: params.text_search,
                page: params.page,
                page_size: params.page_size,
                include_ungrouped_models: params.include_ungrouped_models.unwrap_or(true),
            }
        ).await?;

        Ok(Json(groups.items).into_response())
    }

    #[derive(Deserialize)]
    pub struct GetGroupCountParams {
        pub include_ungrouped_models: Option<bool>,
    }

    #[derive(Serialize)]
    pub struct GetGroupCountResponse {
        pub count: usize,
    }

    pub async fn get_group_count(
        auth_session: AuthSession,
        State(app_state): State<WebAppState>,
        Query(params): Query<GetGroupCountParams>,
    ) -> Result<Response, ApplicationError> {
        let user = auth_session.user.unwrap().to_user();
        let count = group_db::get_group_count(
            &app_state.app_state.db,
            &user,
            params.include_ungrouped_models.unwrap_or(false),
        )
        .await?;

        Ok(Json(GetGroupCountResponse { count }).into_response())
    }
}

mod put {
    use super::*;

    #[derive(Deserialize)]
    pub struct PutGroupParams {
        pub group_name: String,
    }

    pub async fn edit_group(
        auth_session: AuthSession,
        Path(group_id): Path<i64>,
        State(app_state): State<WebAppState>,
        Json(params): Json<PutGroupParams>,
    ) -> Result<Response, ApplicationError> {
        let user = auth_session.user.unwrap().to_user();

        group_db::edit_group(
            &app_state.app_state.db,
            &user,
            group_id,
            &params.group_name,
            None,
        )
        .await?;

        Ok(StatusCode::NO_CONTENT.into_response())
    }
}

mod delete {
    use super::*;

    pub async fn delete_group(
        auth_session: AuthSession,
        Path(group_id): Path<i64>,
        State(app_state): State<WebAppState>,
    ) -> Result<Response, ApplicationError> {
        let user = auth_session.user.unwrap().to_user();
        group_db::delete_group(&app_state.app_state.db, &user, group_id).await?;

        Ok(StatusCode::NO_CONTENT.into_response())
    }

    #[derive(Deserialize)]
    pub struct RemoveModelsFromGroupParams {
        pub model_ids: Vec<i64>,
    }

    pub async fn remove_models_from_group(
        auth_session: AuthSession,
        State(app_state): State<WebAppState>,
        Json(params): Json<RemoveModelsFromGroupParams>,
    ) -> Result<Response, ApplicationError> {
        let user = auth_session.user.unwrap().to_user();
        group_db::set_group_id_on_models(
            &app_state.app_state.db,
            &user,
            None,
            params.model_ids,
            None,
        )
        .await?;

        Ok(StatusCode::NO_CONTENT.into_response())
    }
}

mod post {
    use super::*;

    #[derive(Deserialize)]
    pub struct PostGroupParams {
        pub group_name: String,
    }

    pub async fn add_group(
        auth_session: AuthSession,
        State(app_state): State<WebAppState>,
        Json(params): Json<PostGroupParams>,
    ) -> Result<Response, ApplicationError> {
        let user = auth_session.user.unwrap().to_user();
        let id =
            group_db::add_empty_group(&app_state.app_state.db, &user, &params.group_name, None)
                .await?;

        let group_meta = ModelGroupMeta {
            id,
            name: params.group_name,
            created: time_now(),
            unique_global_id: random_hex_32(),
            resource_id: None,
            last_modified: time_now(),
        };

        Ok(Json(group_meta).into_response())
    }

    #[derive(Deserialize)]
    pub struct AddModelsToGroupParams {
        pub model_ids: Vec<i64>,
    }

    pub async fn add_models_to_group(
        auth_session: AuthSession,
        Path(group_id): Path<i64>,
        State(app_state): State<WebAppState>,
        Json(params): Json<AddModelsToGroupParams>,
    ) -> Result<Response, ApplicationError> {
        let user = auth_session.user.unwrap().to_user();
        group_db::set_group_id_on_models(
            &app_state.app_state.db,
            &user,
            Some(group_id),
            params.model_ids,
            None,
        )
        .await?;

        Ok(StatusCode::NO_CONTENT.into_response())
    }
}
