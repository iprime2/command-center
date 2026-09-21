use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::cookie::CookieJar;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::auth::require_user;
use crate::error::AppError;
use crate::projects::find_project_for_user;
use crate::state::AppState;
use crate::tasks::{create_task, list_tasks_for_project, update_task_for_user, TaskRecord};

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateTaskRequest {
    #[validate(length(min = 1, max = 500))]
    pub title: String,
    #[validate(length(max = 5000))]
    pub description: Option<String>,
    pub status: Option<String>,
    pub due_on: Option<NaiveDate>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct PatchTaskRequest {
    #[validate(length(min = 1, max = 500))]
    pub title: Option<String>,
    #[validate(length(max = 5000))]
    pub description: Option<String>,
    pub status: Option<String>,
    pub due_on: Option<NaiveDate>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TaskDto {
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

impl From<TaskRecord> for TaskDto {
    fn from(t: TaskRecord) -> Self {
        Self {
            id: t.id,
            project_id: t.project_id,
            user_id: t.user_id,
            title: t.title,
            description: t.description,
            status: t.status,
            due_on: t.due_on,
            position: t.position,
            created_at: t.created_at,
            updated_at: t.updated_at,
        }
    }
}

/// List non-deleted tasks for a project owned by the current user.
#[utoipa::path(
    get,
    path = "/projects/{id}/tasks",
    tag = "tasks",
    params(
        ("id" = Uuid, Path, description = "Project id")
    ),
    responses(
        (status = 200, description = "Project tasks", body = Vec<TaskDto>),
        (status = 401, description = "Not authenticated", body = crate::error::ErrorBody),
        (status = 404, description = "Project not found", body = crate::error::ErrorBody)
    ),
    security(("cc_session" = []))
)]
pub async fn list(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<TaskDto>>, AppError> {
    let user = require_user(&state.pool, &jar).await?;
    find_project_for_user(&state.pool, user.id, id)
        .await?
        .ok_or(AppError::NotFound)?;

    let tasks = list_tasks_for_project(&state.pool, user.id, id).await?;
    Ok(Json(tasks.into_iter().map(TaskDto::from).collect()))
}

/// Create a task on a project owned by the current user.
#[utoipa::path(
    post,
    path = "/projects/{id}/tasks",
    tag = "tasks",
    params(
        ("id" = Uuid, Path, description = "Project id")
    ),
    request_body = CreateTaskRequest,
    responses(
        (status = 201, description = "Task created", body = TaskDto),
        (status = 400, description = "Validation error", body = crate::error::ErrorBody),
        (status = 401, description = "Not authenticated", body = crate::error::ErrorBody),
        (status = 404, description = "Project not found", body = crate::error::ErrorBody)
    ),
    security(("cc_session" = []))
)]
pub async fn create(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(id): Path<Uuid>,
    Json(body): Json<CreateTaskRequest>,
) -> Result<(StatusCode, Json<TaskDto>), AppError> {
    let user = require_user(&state.pool, &jar).await?;
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    find_project_for_user(&state.pool, user.id, id)
        .await?
        .ok_or(AppError::NotFound)?;

    let title = body.title.trim();
    if title.is_empty() {
        return Err(AppError::Validation("title must not be empty".to_string()));
    }

    let description = body
        .description
        .as_ref()
        .map(|d| d.trim())
        .filter(|d| !d.is_empty());

    let status = body
        .status
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("backlog");

    let task = create_task(
        &state.pool,
        user.id,
        id,
        title,
        description,
        status,
        body.due_on,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(task.into())))
}

/// Update a task owned by the current user (404 if missing or not owned).
#[utoipa::path(
    patch,
    path = "/tasks/{id}",
    tag = "tasks",
    params(
        ("id" = Uuid, Path, description = "Task id")
    ),
    request_body = PatchTaskRequest,
    responses(
        (status = 200, description = "Task updated", body = TaskDto),
        (status = 400, description = "Validation error", body = crate::error::ErrorBody),
        (status = 401, description = "Not authenticated", body = crate::error::ErrorBody),
        (status = 404, description = "Not found", body = crate::error::ErrorBody)
    ),
    security(("cc_session" = []))
)]
pub async fn patch(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(id): Path<Uuid>,
    Json(body): Json<PatchTaskRequest>,
) -> Result<Json<TaskDto>, AppError> {
    let user = require_user(&state.pool, &jar).await?;
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    if body.title.is_none()
        && body.description.is_none()
        && body.status.is_none()
        && body.due_on.is_none()
    {
        return Err(AppError::Validation(
            "at least one field is required".to_string(),
        ));
    }

    let title = body
        .title
        .as_ref()
        .map(|t| t.trim())
        .filter(|t| !t.is_empty());
    if body.title.is_some() && title.is_none() {
        return Err(AppError::Validation("title must not be empty".to_string()));
    }

    let description = body.description.as_ref().map(|d| {
        let trimmed = d.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    });

    let status = body
        .status
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty());

    let due_on = body.due_on.map(Some);

    let task = update_task_for_user(
        &state.pool,
        user.id,
        id,
        title,
        description,
        status,
        due_on,
    )
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(task.into()))
}
