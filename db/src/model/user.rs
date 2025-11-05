use serde::Serialize;

#[derive(Serialize)]
pub struct User {
    pub id : i64,
    pub username : String,
    pub email : String,
    pub password_hash : String,
    pub user_created_at : String,
}

impl Default for User {
    fn default() -> Self {
        User {
            id: 1,
            username: "".into(),
            email: "".into(),
            password_hash: "".into(),
            user_created_at: "".into(),
        }
    }
}