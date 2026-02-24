// src/models/user.rs
use axum_login::{AuthUser, AuthnBackend};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::fmt::Debug;

#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub github_id: Option<i64>,
    pub avatar_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl AuthUser for User {
    type Id = Uuid;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        b""
    }
}

impl AuthnBackend for User {
    type User = Self;
    type Credentials = ();
    type Error = sqlx::Error;

    async fn get_user(&self, _user_id: &Self::Id) -> Result<Option<Self>, Self::Error> {
        Ok(None)
    }

    async fn authenticate(&self, _credentials: Self::Credentials) -> Result<Option<Self::User>, Self::Error> {
        Ok(None)
    }
}
