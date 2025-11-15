use serde::{Serialize, Serializer, ser::SerializeStruct};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Failed to open or read file")]
    FileSystemFault(#[from] std::io::Error),
    #[error("Failed to read or write zip file")]
    ZipError(#[from] async_zip::error::ZipError),
    #[error("Internal error")]
    InternalError(String),
    #[error("Failed to download file")]
    DownloadError(#[from] reqwest::Error),
    #[error("Failed to process JSON")]
    JsonError(#[from] serde_json::Error),
    #[error("Database error")]
    DatabaseError(#[from] db::DbError),
    #[error("TaskExecutionFailedError")]
    TaskExecutionFailedError(#[from] tokio::task::JoinError),
}

impl Serialize for ServiceError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ServiceError", 3)?;
        match self {
            ServiceError::FileSystemFault(inner) => {
                state.serialize_field("error_type", "FileSystemFault")?;
                state.serialize_field("error_message", &self.to_string())?;
                state.serialize_field("error_inner_message", &inner.to_string())?;
            }
            ServiceError::ZipError(inner) => {
                state.serialize_field("error_type", "ZipError")?;
                state.serialize_field("error_message", &self.to_string())?;
                state.serialize_field("error_inner_message", &inner.to_string())?;
            }
            ServiceError::InternalError(s) => {
                state.serialize_field("error_type", "InternalError")?;
                state.serialize_field("error_message", &self.to_string())?;
                state.serialize_field("error_inner_message", s)?;
            }
            ServiceError::DownloadError(inner) => {
                state.serialize_field("error_type", "DownloadError")?;
                state.serialize_field("error_message", &self.to_string())?;
                state.serialize_field("error_inner_message", &inner.to_string())?;
            }
            ServiceError::JsonError(inner) => {
                state.serialize_field("error_type", "JsonError")?;
                state.serialize_field("error_message", &self.to_string())?;
                state.serialize_field("error_inner_message", &inner.to_string())?;
            }
            ServiceError::DatabaseError(inner) => {
                state.serialize_field("error_type", "DatabaseError")?;
                state.serialize_field("error_message", &self.to_string())?;
                state.serialize_field("error_inner_message", &inner.to_string())?;
            }
            ServiceError::TaskExecutionFailedError(inner) => {
                state.serialize_field("error_type", "TaskExecutionFailedError")?;
                state.serialize_field("error_message", &self.to_string())?;
                state.serialize_field("error_inner_message", &inner.to_string())?;
            }
        }
        state.end()
    }
}
