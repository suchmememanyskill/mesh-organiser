use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApplicationError 
{
    #[error("Failed to open or read file")]
    FileSystemFault(#[from] std::io::Error),
    #[error("Failed to close worker thread")]
    ThreadJoinError(#[from] tokio::task::JoinError),
}