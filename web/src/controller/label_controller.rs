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
use db::{label_db, label_keyword_db};
use db::model::{Label, LabelKeyword, LabelMeta};
use db::random_hex_32;
use serde::{Deserialize, Serialize};
use crate::error::ApplicationError;

pub fn router() -> Router<WebAppState> {
    Router::new()
        .nest(
            "/api/v1",
            Router::new()
                .route("/labels", get(get::get_labels))
                .route("/labels", post(post::add_label))
                .route("/labels/{label_id}", put(put::edit_label))
                .route("/labels/{label_id}", delete(delete::delete_label))
                .route("/labels/{label_id}/models", post(post::set_label_on_models))
                .route("/labels/{label_id}/models", delete(delete::remove_label_from_models))
                .route("/labels/{label_id}/childs", post(post::add_childs_to_label))
                .route("/labels/{label_id}/childs", put(put::set_childs_on_label))
                .route("/labels/{label_id}/childs", delete(delete::remove_childs_from_label))
                .route("/labels/{label_id}/keywords", get(get::get_keywords_for_label))
                .route("/labels/{label_id}/keywords", put(put::set_keywords_on_label))
                .route("/models/{model_id}/labels", put(put::set_labels_on_model))
                .route_layer(login_required!(Backend))
        )
}

mod get {
    use super::*;

    #[derive(Deserialize)]
    pub struct GetLabelsParams {
        pub include_ungrouped_models: Option<bool>,
    }

    pub async fn get_labels(
        auth_session: AuthSession,
        State(app_state): State<WebAppState>,
        Json(params): Json<GetLabelsParams>,
    ) -> Result<Response, ApplicationError> {
        let user = auth_session.user.unwrap().to_user();
        let labels = label_db::get_labels(
            &app_state.app_state.db,
            &user,
            params.include_ungrouped_models.unwrap_or(false),
        ).await?;

        Ok(Json(labels).into_response())
    }

    pub async fn get_keywords_for_label(
        auth_session: AuthSession,
        Path(label_id): Path<i64>,
        State(app_state): State<WebAppState>,
    ) -> Result<Response, ApplicationError> {
        let user = auth_session.user.unwrap().to_user();
        let keywords = label_keyword_db::get_keywords_for_label(
            &app_state.app_state.db,
            &user,
            label_id,
        ).await
        .map_err(|e| ApplicationError::InternalError(e.to_string()))?;

        Ok(Json(keywords).into_response())
    }
}

mod post {
    use super::*;

    #[derive(Deserialize)]
    pub struct PostLabelParams {
        pub label_name: String,
        pub label_color: i64,
    }

    pub async fn add_label(
        auth_session: AuthSession,
        State(app_state): State<WebAppState>,
        Json(params): Json<PostLabelParams>,
    ) -> Result<Response, ApplicationError> {
        let user = auth_session.user.unwrap().to_user();
        let id = label_db::add_label(
            &app_state.app_state.db,
            &user,
            &params.label_name,
            params.label_color,
            None,
        ).await?;

        let label_meta = LabelMeta {
            id,
            name: params.label_name,
            color: params.label_color,
            unique_global_id: random_hex_32(),
        };

        Ok(Json(label_meta).into_response())
    }

    #[derive(Deserialize)]
    pub struct SetLabelOnModelsParams {
        pub model_ids: Vec<i64>,
    }

    pub async fn set_label_on_models(
        auth_session: AuthSession,
        Path(label_id): Path<i64>,
        State(app_state): State<WebAppState>,
        Json(params): Json<SetLabelOnModelsParams>,
    ) -> Result<Response, ApplicationError> {
        let user = auth_session.user.unwrap().to_user();
        label_db::remove_labels_from_models(
            &app_state.app_state.db,
            &user,
            &[label_id],
            &params.model_ids,
            None,
        ).await?;
        label_db::add_labels_on_models(
            &app_state.app_state.db,
            &user,
            &[label_id],
            &params.model_ids,
            None,
        ).await?;

        Ok(StatusCode::OK.into_response())
    }

    #[derive(Deserialize)]
    pub struct AddChildsToLabelParams {
        pub child_label_ids: Vec<i64>,
    }

    pub async fn add_childs_to_label(
        auth_session: AuthSession,
        Path(parent_label_id): Path<i64>,
        State(app_state): State<WebAppState>,
        Json(params): Json<AddChildsToLabelParams>,
    ) -> Result<Response, ApplicationError> {
        let user = auth_session.user.unwrap().to_user();
        label_db::add_childs_to_label(
            &app_state.app_state.db,
            &user,
            parent_label_id,
            params.child_label_ids,
            None,
        ).await
        .map_err(|e| ApplicationError::InternalError(e.to_string()))?;

        Ok(StatusCode::OK.into_response())
    }
}

mod put {
    use super::*;

    #[derive(Deserialize)]
    pub struct PutLabelParams {
        pub label_name: String,
        pub label_color: i64,
    }

    pub async fn edit_label(
        auth_session: AuthSession,
        Path(label_id): Path<i64>,
        State(app_state): State<WebAppState>,
        Json(params): Json<PutLabelParams>,
    ) -> Result<Response, ApplicationError> {
        let user = auth_session.user.unwrap().to_user();
        label_db::edit_label(
            &app_state.app_state.db,
            &user,
            label_id,
            &params.label_name,
            params.label_color,
            None,
        ).await?;

        Ok(StatusCode::OK.into_response())
    }

    #[derive(Deserialize)]
    pub struct SetLabelsOnModelParams {
        pub label_ids: Vec<i64>,
    }

    pub async fn set_labels_on_model(
        auth_session: AuthSession,
        Path(model_id): Path<i64>,
        State(app_state): State<WebAppState>,
        Json(params): Json<SetLabelsOnModelParams>,
    ) -> Result<Response, ApplicationError> {
        let user = auth_session.user.unwrap().to_user();
        label_db::remove_all_labels_from_models(
            &app_state.app_state.db,
            &user,
            &[model_id],
            None,
        ).await?;
        label_db::add_labels_on_models(
            &app_state.app_state.db,
            &user,
            &params.label_ids,
            &[model_id],
            None,
        ).await?;

        Ok(StatusCode::OK.into_response())
    }

    #[derive(Deserialize)]
    pub struct SetChildsOnLabelParams {
        pub child_label_ids: Vec<i64>,
    }

    pub async fn set_childs_on_label(
        auth_session: AuthSession,
        Path(parent_label_id): Path<i64>,
        State(app_state): State<WebAppState>,
        Json(params): Json<SetChildsOnLabelParams>,
    ) -> Result<Response, ApplicationError> {
        let user = auth_session.user.unwrap().to_user();
        label_db::remove_all_childs_from_label(
            &app_state.app_state.db,
            &user,
            parent_label_id,
            None,
        ).await
        .map_err(|e| ApplicationError::InternalError(e.to_string()))?;

        if !params.child_label_ids.is_empty() {
            label_db::add_childs_to_label(
                &app_state.app_state.db,
                &user,
                parent_label_id,
                params.child_label_ids,
                None,
            ).await
            .map_err(|e| ApplicationError::InternalError(e.to_string()))?;
        }

        Ok(StatusCode::OK.into_response())
    }

    #[derive(Deserialize)]
    pub struct SetKeywordsOnLabelParams {
        pub keywords: Vec<String>,
    }

    pub async fn set_keywords_on_label(
        auth_session: AuthSession,
        Path(label_id): Path<i64>,
        State(app_state): State<WebAppState>,
        Json(params): Json<SetKeywordsOnLabelParams>,
    ) -> Result<Response, ApplicationError> {
        let user = auth_session.user.unwrap().to_user();
        label_keyword_db::set_keywords_for_label(
            &app_state.app_state.db,
            &user,
            label_id,
            params.keywords,
            None,
        ).await
        .map_err(|e| ApplicationError::InternalError(e.to_string()))?;

        Ok(StatusCode::OK.into_response())
    }
}

mod delete {
    use super::*;

    pub async fn delete_label(
        auth_session: AuthSession,
        Path(label_id): Path<i64>,
        State(app_state): State<WebAppState>,
    ) -> Result<Response, ApplicationError> {
        let user = auth_session.user.unwrap().to_user();
        label_db::delete_label(&app_state.app_state.db, &user, label_id).await?;

        Ok(StatusCode::OK.into_response())
    }

    #[derive(Deserialize)]
    pub struct RemoveLabelFromModelsParams {
        pub model_ids: Vec<i64>,
    }

    pub async fn remove_label_from_models(
        auth_session: AuthSession,
        Path(label_id): Path<i64>,
        State(app_state): State<WebAppState>,
        Json(params): Json<RemoveLabelFromModelsParams>,
    ) -> Result<Response, ApplicationError> {
        let user = auth_session.user.unwrap().to_user();
        label_db::remove_labels_from_models(
            &app_state.app_state.db,
            &user,
            &[label_id],
            &params.model_ids,
            None,
        ).await?;

        Ok(StatusCode::OK.into_response())
    }

    #[derive(Deserialize)]
    pub struct RemoveChildsFromLabelParams {
        pub child_label_ids: Vec<i64>,
    }

    pub async fn remove_childs_from_label(
        auth_session: AuthSession,
        Path(parent_label_id): Path<i64>,
        State(app_state): State<WebAppState>,
        Json(params): Json<RemoveChildsFromLabelParams>,
    ) -> Result<Response, ApplicationError> {
        let user = auth_session.user.unwrap().to_user();
        label_db::remove_childs_from_label(
            &app_state.app_state.db,
            &user,
            parent_label_id,
            params.child_label_ids,
            None,
        ).await
        .map_err(|e| ApplicationError::InternalError(e.to_string()))?;

        Ok(StatusCode::OK.into_response())
    }
}
