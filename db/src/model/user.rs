use password_auth::generate_hash;
use serde::{Deserialize, Serialize};
use bitflags::bitflags;
use sha2::{Digest, Sha256};

bitflags! {
    #[derive(Debug)]
    pub struct UserPermissions: u32 {
        const Admin = 0b00000001;
        const Sync  = 0b00000010;
        const OnlineAccount = 0b00000100;
    }
}

impl Serialize for UserPermissions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut flags = Vec::new();
        if self.contains(UserPermissions::Admin) {
            flags.push("Admin");
        }
        if self.contains(UserPermissions::Sync) {
            flags.push("Sync");
        }
        if self.contains(UserPermissions::OnlineAccount) {
            flags.push("OnlineAccount");
        }
        flags.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for UserPermissions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let flags: Vec<String> = Vec::deserialize(deserializer)?;
        let mut result = UserPermissions::empty();
        for flag in flags {
            match flag.as_str() {
                "Admin" => result.insert(UserPermissions::Admin),
                "Sync" => result.insert(UserPermissions::Sync),
                "OnlineAccount" => result.insert(UserPermissions::OnlineAccount),
                _ => {}
            }
        }
        Ok(result)
    }
}

#[derive(Serialize, Debug)]
pub struct User {
    pub id : i64,
    pub username : String,
    pub email : String,
    pub created_at : String,
    pub sync_url : Option<String>,
    pub sync_token : Option<String>,
    pub last_sync : Option<String>,
    pub permissions : UserPermissions,
    #[serde(skip_serializing)]
    pub password_hash : String,
}

impl Clone for User {
    fn clone(&self) -> Self {
        User {
            id: self.id,
            username: self.username.clone(),
            email: self.email.clone(),
            created_at: self.created_at.clone(),
            sync_url: self.sync_url.clone(),
            sync_token: self.sync_token.clone(),
            last_sync: self.last_sync.clone(),
            password_hash: self.password_hash.clone(),
            permissions: unsafe { UserPermissions::from_bits(self.permissions.bits()).unwrap_unchecked() },
        }
    }
}

impl Default for User {
    fn default() -> Self {
        User {
            id: 1,
            username: "".into(),
            email: "".into(),
            created_at: "".into(),
            sync_url: None,
            sync_token: None,
            last_sync: None,
            password_hash: "".into(),
            permissions: UserPermissions::empty(),
        }
    }
}

pub fn hash_password(password: &str) -> String {
    generate_hash(password)
}