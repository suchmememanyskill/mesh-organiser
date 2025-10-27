use serde::{Serialize, Serializer, ser::SerializeStruct};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("Failed to open or read file")]
    FileSystemFault(#[from] std::io::Error),
    #[error("Failed to read or write zip file")]
    ZipError(#[from] zip::result::ZipError),
    #[error("Internal error")]
    InternalError(String),
    #[error("Failed to launch thumbnail generator")]
    SidecarError(#[from] tauri_plugin_shell::Error),
    #[error("Failed to download file")]
    DownloadError(#[from] reqwest::Error),
    #[error("Failed to process JSON")]
    JsonError(#[from] serde_json::Error),
    #[error("Framework error")]
    FrameworkError(#[from] tauri::Error),
}

impl Serialize for ApplicationError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ApplicationError", 3)?;
        match self {
            ApplicationError::FileSystemFault(inner) => {
                state.serialize_field("error_type", "FileSystemFault")?;
                state.serialize_field("error_message", &self.to_string())?;
                state.serialize_field("error_inner_message", &inner.to_string())?;
            }
            ApplicationError::ZipError(inner) => {
                state.serialize_field("error_type", "ZipError")?;
                state.serialize_field("error_message", &self.to_string())?;
                state.serialize_field("error_inner_message", &inner.to_string())?;
            }
            ApplicationError::InternalError(s) => {
                state.serialize_field("error_type", "InternalError")?;
                state.serialize_field("error_message", &self.to_string())?;
                state.serialize_field("error_inner_message", s)?;
            }
            ApplicationError::SidecarError(inner) => {
                state.serialize_field("error_type", "SidecarError")?;
                state.serialize_field("error_message", &self.to_string())?;
                state.serialize_field("error_inner_message", &inner.to_string())?;
            }
            ApplicationError::DownloadError(inner) => {
                state.serialize_field("error_type", "DownloadError")?;
                state.serialize_field("error_message", &self.to_string())?;
                state.serialize_field("error_inner_message", &inner.to_string())?;
            }
            ApplicationError::JsonError(inner) => {
                state.serialize_field("error_type", "JsonError")?;
                state.serialize_field("error_message", &self.to_string())?;
                state.serialize_field("error_inner_message", &inner.to_string())?;
            }
            ApplicationError::FrameworkError(inner) => {
                state.serialize_field("error_type", "FrameworkError")?;
                state.serialize_field("error_message", &self.to_string())?;
                state.serialize_field("error_inner_message", &inner.to_string())?;
            }
        }
        state.end()
    }
}
