use std::sync::Arc;

use axum_login::{AuthUser as AxumAuthUser, AuthnBackend, UserId};
use db::{db_context::DbContext, model::User, user_db};
use password_auth::verify_password;
use serde::{Deserialize, Serialize};
use tokio::task;

use crate::error::ApplicationError;

#[derive(Clone, Serialize, Deserialize)]
pub struct AuthUser {
    id: i64,
    pub username: String,
    pub email: String,
    pub permissions: usize,
    pub validity_token: Vec<u8>,
}

// Here we've implemented `Debug` manually to avoid accidentally logging the
// password hash.
impl std::fmt::Debug for AuthUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("User")
            .field("id", &self.id)
            .field("username", &self.username)
            .field("permissions", &self.permissions)
            .finish()
    }
}

impl AuthUser {
    pub fn to_user(&self) -> User {
        User {
            id: self.id,
            username: self.username.clone(),
            email: self.email.clone(),
            permissions: db::model::UserPermissions::from_bits_truncate(self.permissions as u32),
            password_hash: String::new(),
            last_sync: None,
            sync_token: None,
            sync_url: None,
            created_at: String::new(),
        }
    }
}

impl AxumAuthUser for AuthUser {
    type Id = i64;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        &self.validity_token
    }
}

#[derive(Debug, Clone, Deserialize)]
pub enum Credentials {
    Password(PasswordCredentials),
    Token(TokenCredentials),
}

#[derive(Debug, Clone, Deserialize)]
pub struct PasswordCredentials {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TokenCredentials {
    pub token: String,
}

#[derive(Debug, Clone)]
pub struct Backend {
    db: Arc<DbContext>,
}

impl Backend {
    pub fn new(db: Arc<DbContext>) -> Self {
        Self { db }
    }

    fn convert_user(user: User) -> AuthUser {
        AuthUser {
            id: user.id,
            username: user.username,
            email: user.email,
            permissions: user.permissions.bits() as usize,
            validity_token: match user.sync_url {
                Some(token) => token.into_bytes(),
                None => vec![],
            },
        }
    }
}

impl AuthnBackend for Backend {
    type User = AuthUser;
    type Credentials = Credentials;
    type Error = ApplicationError;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        match creds {
            Credentials::Password(password_credentials) => {
                let user =
                    user_db::get_user_by_email(&self.db, &password_credentials.email).await?;

                let user = task::spawn_blocking(|| {
                    user.filter(|user| {
                        verify_password(password_credentials.password, &user.password_hash).is_ok()
                    })
                })
                .await?;

                match user {
                    Some(user) => Ok(Some(Self::convert_user(user))),
                    None => Ok(None),
                }
            }
            Credentials::Token(token_credentials) => {
                let user =
                    user_db::get_user_by_sync_token(&self.db, &token_credentials.token).await?;

                match user {
                    Some(user) => Ok(Some(Self::convert_user(user))),
                    None => Ok(None),
                }
            }
        }
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        let user = user_db::get_user_by_id(&self.db, *user_id).await?;

        Ok(user.map(Self::convert_user))
    }
}

// We use a type alias for convenience.
//
// Note that we've supplied our concrete backend here.
pub type AuthSession = axum_login::AuthSession<Backend>;
