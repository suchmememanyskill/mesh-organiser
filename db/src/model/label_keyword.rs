use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct LabelKeyword {
    pub id: i64,
    pub name: String,
}