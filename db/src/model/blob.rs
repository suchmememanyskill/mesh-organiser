use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct Blob {
    pub id: i64,
    pub sha256: String,
    pub filetype: String,
    pub size: i64,
    pub added: String,
    pub disk_path: Option<String>
}