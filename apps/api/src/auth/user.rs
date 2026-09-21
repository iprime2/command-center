use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, sqlx::FromRow, ToSchema)]
pub struct UserRecord {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    #[schema(value_type = String)]
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PublicUser {
    pub id: Uuid,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<UserRecord> for PublicUser {
    fn from(u: UserRecord) -> Self {
        Self {
            id: u.id,
            email: u.email,
            created_at: u.created_at,
            updated_at: u.updated_at,
        }
    }
}

pub async fn create_user(
    pool: &PgPool,
    email: &str,
    password_hash: &str,
) -> Result<UserRecord, AppError> {
    let id = Uuid::new_v4();
    let now = Utc::now();

    let user = sqlx::query_as::<_, UserRecord>(
        r#"
        INSERT INTO users (id, email, password_hash, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $4)
        RETURNING id, email, password_hash, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(email)
    .bind(password_hash)
    .bind(now)
    .fetch_one(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(db) if db.constraint().is_some() => {
            AppError::Conflict("email already registered".to_string())
        }
        other => AppError::Db(other),
    })?;

    Ok(user)
}

pub async fn find_active_user_by_email(
    pool: &PgPool,
    email: &str,
) -> Result<Option<UserRecord>, AppError> {
    let user = sqlx::query_as::<_, UserRecord>(
        r#"
        SELECT id, email, password_hash, created_at, updated_at
        FROM users
        WHERE email = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;
    Ok(user)
}
