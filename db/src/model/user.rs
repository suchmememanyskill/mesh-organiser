use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct User {
    pub id : i64,
    pub username : String,
    pub email : String,
    pub password_hash : String,
    pub created_at : String,
    pub sync_url : Option<String>,
    pub sync_token : Option<String>,
    pub last_sync : Option<String>,
}

impl Default for User {
    fn default() -> Self {
        User {
            id: 1,
            username: "".into(),
            email: "".into(),
            password_hash: "".into(),
            created_at: "".into(),
            sync_url: None,
            sync_token: None,
            last_sync: None,
        }
    }
}