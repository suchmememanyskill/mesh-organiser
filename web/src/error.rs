use axum::{Json, extract::multipart::MultipartError, response::IntoResponse};
use serde::{Serialize, Serializer, ser::SerializeStruct};
use thiserror::Error;
use tokio::task;

#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("Failed to open or read file")]
    FileSystemFault(#[from] std::io::Error),
    #[error("Internal error")]
    InternalError(String),
    #[error("Failed to process JSON")]
    JsonError(#[from] serde_json::Error),
    #[error("Database error")]
    DatabaseError(#[from] db::DbError),
    #[error("Service error")]
    ServiceError(#[from] service::ServiceError),
    #[error(transparent)]
    TaskJoinError(#[from] task::JoinError),
    #[error("Upload error")]
    MultipartError(#[from] MultipartError),
}

impl Serialize for ApplicationError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let ApplicationError::ServiceError(inner) = self {
            return inner.serialize(serializer);
        }

        let mut state = serializer.serialize_struct("ApplicationError", 3)?;
        match self {
            ApplicationError::FileSystemFault(inner) => {
                state.serialize_field("error_type", "FileSystemFault")?;
                state.serialize_field("error_message", &self.to_string())?;
                state.serialize_field("error_inner_message", &inner.to_string())?;
            }
            ApplicationError::InternalError(s) => {
                state.serialize_field("error_type", "InternalError")?;
                state.serialize_field("error_message", &self.to_string())?;
                state.serialize_field("error_inner_message", s)?;
            }
            ApplicationError::JsonError(inner) => {
                state.serialize_field("error_type", "JsonError")?;
                state.serialize_field("error_message", &self.to_string())?;
                state.serialize_field("error_inner_message", &inner.to_string())?;
            }
            ApplicationError::DatabaseError(inner) => {
                state.serialize_field("error_type", "DatabaseError")?;
                state.serialize_field("error_message", &self.to_string())?;
                state.serialize_field("error_inner_message", &inner.to_string())?;
            }
            ApplicationError::TaskJoinError(inner) => {
                state.serialize_field("error_type", "TaskJoinError")?;
                state.serialize_field("error_message", &self.to_string())?;
                state.serialize_field("error_inner_message", &inner.to_string())?;
            }
            ApplicationError::MultipartError(inner) => {
                state.serialize_field("error_type", "MultipartError")?;
                state.serialize_field("error_message", &self.to_string())?;
                state.serialize_field("error_inner_message", &inner.to_string())?;
            }
            _ => {}
        }
        state.end()
    }
}

impl IntoResponse for ApplicationError {
    fn into_response(self) -> axum::response::Response {
        let json = serde_json::to_string(&self).unwrap_or("Failed to serialize error".to_string());
        println!("[Error] {}", json);
        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, Json(self)).into_response()
    }
}
