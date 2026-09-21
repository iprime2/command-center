use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ProjectRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn create_project(
    pool: &PgPool,
    user_id: Uuid,
    name: &str,
    description: Option<&str>,
) -> Result<ProjectRecord, AppError> {
    let id = Uuid::new_v4();
    let now = Utc::now();

    let project = sqlx::query_as::<_, ProjectRecord>(
        r#"
        INSERT INTO projects (id, user_id, name, description, status, created_at, updated_at)
        VALUES ($1, $2, $3, $4, 'active', $5, $5)
        RETURNING id, user_id, name, description, status, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(user_id)
    .bind(name)
    .bind(description)
    .bind(now)
    .fetch_one(pool)
    .await?;

    Ok(project)
}

pub async fn list_projects_for_user(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<ProjectRecord>, AppError> {
    let projects = sqlx::query_as::<_, ProjectRecord>(
        r#"
        SELECT id, user_id, name, description, status, created_at, updated_at
        FROM projects
        WHERE user_id = $1 AND deleted_at IS NULL
        ORDER BY created_at DESC
        LIMIT 500
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(projects)
}

pub async fn find_project_for_user(
    pool: &PgPool,
    user_id: Uuid,
    project_id: Uuid,
) -> Result<Option<ProjectRecord>, AppError> {
    let project = sqlx::query_as::<_, ProjectRecord>(
        r#"
        SELECT id, user_id, name, description, status, created_at, updated_at
        FROM projects
        WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL
        "#,
    )
    .bind(project_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(project)
}

pub const INBOX_PROJECT_NAME: &str = "Inbox";

async fn find_inbox_for_user(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<ProjectRecord>, AppError> {
    let project = sqlx::query_as::<_, ProjectRecord>(
        r#"
        SELECT id, user_id, name, description, status, created_at, updated_at
        FROM projects
        WHERE user_id = $1 AND name = $2 AND deleted_at IS NULL
        "#,
    )
    .bind(user_id)
    .bind(INBOX_PROJECT_NAME)
    .fetch_optional(pool)
    .await?;

    Ok(project)
}

fn is_unique_violation(err: &sqlx::Error) -> bool {
    match err {
        sqlx::Error::Database(db) => db.code().as_deref() == Some("23505"),
        _ => false,
    }
}

/// Return the user's active Inbox project, creating it if missing.
/// Soft-deleted Inboxes are ignored; a new Inbox is created instead.
pub async fn find_or_create_inbox(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<ProjectRecord, AppError> {
    if let Some(existing) = find_inbox_for_user(pool, user_id).await? {
        return Ok(existing);
    }

    match create_project(pool, user_id, INBOX_PROJECT_NAME, None).await {
        Ok(created) => Ok(created),
        Err(AppError::Db(ref e)) if is_unique_violation(e) => find_inbox_for_user(pool, user_id)
            .await?
            .ok_or_else(|| {
                AppError::Internal("inbox unique conflict but row missing".to_string())
            }),
        Err(e) => Err(e),
    }
}
