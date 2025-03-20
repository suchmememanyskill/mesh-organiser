use serde::{Serialize, Serializer, ser::SerializeStruct};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("Failed to open or read file")]
    FileSystemFault(#[from] std::io::Error),
    #[error("Failed to read or write zip file")]
    ZipError(#[from] zip::result::ZipError),
    #[error("Internal error")]
    InternalError,
    #[error("Failed to launch thumbnail generator")]
    SidecarError(#[from] tauri_plugin_shell::Error),
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
            },
            ApplicationError::ZipError(inner) => {
                state.serialize_field("error_type", "ZipError")?;
                state.serialize_field("error_message", &self.to_string())?;
                state.serialize_field("error_inner_message", &inner.to_string())?;
            },
            ApplicationError::InternalError => {
                state.serialize_field("error_type", "InternalError")?;
                state.serialize_field("error_message", &self.to_string())?;
                state.serialize_field("error_inner_message", "")?;
            },
            ApplicationError::SidecarError(inner) => {
                state.serialize_field("error_type", "SidecarError")?;
                state.serialize_field("error_message", &self.to_string())?;
                state.serialize_field("error_inner_message", &inner.to_string())?;
            },
        }
        state.end()
    }
}