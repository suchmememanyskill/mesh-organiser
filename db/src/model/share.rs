use serde::Serialize;

#[derive(Serialize)]
pub struct Share {
    pub id: String,
    pub created_at: String,
    pub share_name: String,
    pub user_id: i64,
    pub model_ids: Vec<i64>,
}

#[derive(Serialize)]
pub struct ShareDto {
    pub id: String,
    pub created_at: String,
    pub share_name: String,
    pub user_name: String,
    pub model_ids: Vec<i64>,
}

impl Share {
    pub fn to_dto(self, user_name: String) -> ShareDto {
        ShareDto {
            id: self.id,
            created_at: self.created_at,
            share_name: self.share_name,
            user_name,
            model_ids: self.model_ids,
        }
    }
}