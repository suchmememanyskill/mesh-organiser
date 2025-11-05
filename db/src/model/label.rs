use indexmap::IndexMap;
use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub struct LabelMeta {
    pub id: i64,
    pub name: String,
    pub color: i64,
    pub unique_global_id: String,
}

pub fn convert_label_meta_list_to_map(labels: Vec<LabelMeta>) -> IndexMap<i64, LabelMeta> {
    let mut label_map: IndexMap<i64, LabelMeta> = IndexMap::with_capacity(labels.len());

    for label in labels {
        label_map.insert(
            label.id,
            LabelMeta {
                id: label.id,
                name: label.name,
                color: label.color,
                unique_global_id: label.unique_global_id,
            },
        );
    }

    label_map
}

#[derive(Serialize, Debug)]
pub struct Label {
    pub meta : LabelMeta,
    pub children: Vec<LabelMeta>,
    pub effective_labels: Vec<LabelMeta>,
    pub has_parent: bool,
    pub model_count: i64,
    pub group_count: i64,
    pub self_model_count: i64,
    pub self_group_count: i64,
}