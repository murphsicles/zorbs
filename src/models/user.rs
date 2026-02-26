// src/models/user.rs
use axum_login::{AuthUser, AuthnBackend};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;
use time::OffsetDateTime;

#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub provider: String,
    pub provider_id: String,
    pub avatar_url: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
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

#[derive(Clone)]
pub struct UserBackend {
    pub pool: PgPool,
}

impl UserBackend {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl AuthnBackend for UserBackend {
    type User = User;
    type Credentials = ();
    type Error = sqlx::Error;

    async fn get_user(&self, user_id: &Uuid) -> Result<Option<User>, Self::Error> {
        sqlx::query_as("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await
    }

    async fn authenticate(&self, _credentials: Self::Credentials) -> Result<Option<User>, Self::Error> {
        Ok(None)
    }
}
