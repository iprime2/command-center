use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TaskRecord {
    pub id: Uuid,
    pub project_id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub due_on: Option<NaiveDate>,
    pub position: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub fn validate_status(status: &str) -> Result<(), AppError> {
    match status {
        "backlog" | "doing" | "done" => Ok(()),
        _ => Err(AppError::Validation(
            "status must be one of: backlog, doing, done".to_string(),
        )),
    }
}

pub async fn create_task(
    pool: &PgPool,
    user_id: Uuid,
    project_id: Uuid,
    title: &str,
    description: Option<&str>,
    status: &str,
    due_on: Option<NaiveDate>,
) -> Result<TaskRecord, AppError> {
    validate_status(status)?;
    let id = Uuid::new_v4();
    let now = Utc::now();

    let task = sqlx::query_as::<_, TaskRecord>(
        r#"
        INSERT INTO tasks (
            id, project_id, user_id, title, description, status, due_on, position,
            created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, 0, $8, $8)
        RETURNING id, project_id, user_id, title, description, status, due_on, position,
                  created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(project_id)
    .bind(user_id)
    .bind(title)
    .bind(description)
    .bind(status)
    .bind(due_on)
    .bind(now)
    .fetch_one(pool)
    .await?;

    Ok(task)
}

pub async fn list_tasks_for_project(
    pool: &PgPool,
    user_id: Uuid,
    project_id: Uuid,
) -> Result<Vec<TaskRecord>, AppError> {
    let tasks = sqlx::query_as::<_, TaskRecord>(
        r#"
        SELECT id, project_id, user_id, title, description, status, due_on, position,
               created_at, updated_at
        FROM tasks
        WHERE project_id = $1 AND user_id = $2 AND deleted_at IS NULL
        ORDER BY position ASC, created_at ASC
        LIMIT 1000
        "#,
    )
    .bind(project_id)
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(tasks)
}

pub async fn update_task_for_user(
    pool: &PgPool,
    user_id: Uuid,
    task_id: Uuid,
    title: Option<&str>,
    description: Option<Option<&str>>,
    status: Option<&str>,
    due_on: Option<Option<NaiveDate>>,
) -> Result<Option<TaskRecord>, AppError> {
    if let Some(s) = status {
        validate_status(s)?;
    }

    let existing = sqlx::query_as::<_, TaskRecord>(
        r#"
        SELECT id, project_id, user_id, title, description, status, due_on, position,
               created_at, updated_at
        FROM tasks
        WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL
        "#,
    )
    .bind(task_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    let Some(existing) = existing else {
        return Ok(None);
    };

    let new_title = title.unwrap_or(existing.title.as_str());
    let new_description = match description {
        Some(d) => d.map(|s| s.to_string()),
        None => existing.description.clone(),
    };
    let new_status = status.unwrap_or(existing.status.as_str());
    let new_due_on = match due_on {
        Some(d) => d,
        None => existing.due_on,
    };
    let now = Utc::now();

    let task = sqlx::query_as::<_, TaskRecord>(
        r#"
        UPDATE tasks
        SET title = $1,
            description = $2,
            status = $3,
            due_on = $4,
            updated_at = $5
        WHERE id = $6 AND user_id = $7 AND deleted_at IS NULL
        RETURNING id, project_id, user_id, title, description, status, due_on, position,
                  created_at, updated_at
        "#,
    )
    .bind(new_title)
    .bind(new_description)
    .bind(new_status)
    .bind(new_due_on)
    .bind(now)
    .bind(task_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(task)
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TodayTaskRecord {
    pub id: Uuid,
    pub project_id: Uuid,
    pub title: String,
    pub status: String,
    pub due_on: Option<NaiveDate>,
    pub project_name: String,
}

/// Non-deleted tasks that are in progress or due on/before `today` (UTC calendar date).
pub async fn list_today_tasks(
    pool: &PgPool,
    user_id: Uuid,
    today: NaiveDate,
) -> Result<Vec<TodayTaskRecord>, AppError> {
    let tasks = sqlx::query_as::<_, TodayTaskRecord>(
        r#"
        SELECT t.id,
               t.project_id,
               t.title,
               t.status,
               t.due_on,
               p.name AS project_name
        FROM tasks t
        INNER JOIN projects p
            ON p.id = t.project_id
           AND p.user_id = t.user_id
           AND p.deleted_at IS NULL
        WHERE t.user_id = $1
          AND t.deleted_at IS NULL
          AND (
                t.status = 'doing'
                OR (t.due_on IS NOT NULL AND t.due_on <= $2)
              )
        ORDER BY t.due_on ASC NULLS LAST, t.updated_at DESC
        LIMIT 1000
        "#,
    )
    .bind(user_id)
    .bind(today)
    .fetch_all(pool)
    .await?;

    Ok(tasks)
}
