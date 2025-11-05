use serde::Serialize;

use crate::model::{LabelMeta, Model};

#[derive(Serialize, Clone)]
pub struct ModelGroupMeta {
    pub id: i64,
    pub name: String,
    pub created: String,
    pub resource_id: Option<i64>,
    pub unique_global_id: String,
}

#[derive(Serialize)]
pub struct ModelGroup {
    pub meta: ModelGroupMeta,
    pub models: Vec<Model>,
    pub labels: Vec<LabelMeta>,
}

impl ModelGroup
{
    pub fn from_meta(meta : ModelGroupMeta) -> ModelGroup
    {
        ModelGroup { meta, models: Vec::new(), labels: Vec::new() }
    }
}

// TODO: Add impl for ModelGroup to fetch effective labels, etc.