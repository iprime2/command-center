use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::cookie::CookieJar;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::auth::require_user;
use crate::error::AppError;
use crate::projects::{create_project, find_project_for_user, list_projects_for_user, ProjectRecord};
use crate::state::AppState;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateProjectRequest {
    #[validate(length(min = 1, max = 200))]
    pub name: String,
    #[validate(length(max = 2000))]
    pub description: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ProjectDto {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ProjectRecord> for ProjectDto {
    fn from(p: ProjectRecord) -> Self {
        Self {
            id: p.id,
            user_id: p.user_id,
            name: p.name,
            description: p.description,
            status: p.status,
            created_at: p.created_at,
            updated_at: p.updated_at,
        }
    }
}

/// Create a project owned by the current user.
#[utoipa::path(
    post,
    path = "/projects",
    tag = "projects",
    request_body = CreateProjectRequest,
    responses(
        (status = 201, description = "Project created", body = ProjectDto),
        (status = 400, description = "Validation error", body = crate::error::ErrorBody),
        (status = 401, description = "Not authenticated", body = crate::error::ErrorBody)
    ),
    security(("cc_session" = []))
)]
pub async fn create(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(body): Json<CreateProjectRequest>,
) -> Result<(StatusCode, Json<ProjectDto>), AppError> {
    let user = require_user(&state.pool, &jar).await?;
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let name = body.name.trim();
    if name.is_empty() {
        return Err(AppError::Validation("name must not be empty".to_string()));
    }

    let description = body
        .description
        .as_ref()
        .map(|d| d.trim())
        .filter(|d| !d.is_empty());

    let project = create_project(&state.pool, user.id, name, description).await?;
    Ok((StatusCode::CREATED, Json(project.into())))
}

/// List non-deleted projects for the current user.
#[utoipa::path(
    get,
    path = "/projects",
    tag = "projects",
    responses(
        (status = 200, description = "Current user's projects", body = Vec<ProjectDto>),
        (status = 401, description = "Not authenticated", body = crate::error::ErrorBody)
    ),
    security(("cc_session" = []))
)]
pub async fn list(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<Json<Vec<ProjectDto>>, AppError> {
    let user = require_user(&state.pool, &jar).await?;
    let projects = list_projects_for_user(&state.pool, user.id).await?;
    Ok(Json(projects.into_iter().map(ProjectDto::from).collect()))
}

/// Get a single project owned by the current user (404 if missing or not owned).
#[utoipa::path(
    get,
    path = "/projects/{id}",
    tag = "projects",
    params(
        ("id" = Uuid, Path, description = "Project id")
    ),
    responses(
        (status = 200, description = "Project", body = ProjectDto),
        (status = 401, description = "Not authenticated", body = crate::error::ErrorBody),
        (status = 404, description = "Not found", body = crate::error::ErrorBody)
    ),
    security(("cc_session" = []))
)]
pub async fn get(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(id): Path<Uuid>,
) -> Result<Json<ProjectDto>, AppError> {
    let user = require_user(&state.pool, &jar).await?;
    let project = find_project_for_user(&state.pool, user.id, id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(project.into()))
}
