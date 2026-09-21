use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::cookie::CookieJar;
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::auth::require_user;
use crate::error::AppError;
use crate::projects::{find_or_create_inbox, find_project_for_user};
use crate::routes::tasks::TaskDto;
use crate::state::AppState;
use crate::tasks::create_task;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CaptureRequest {
    #[validate(length(min = 1, max = 500))]
    pub title: String,
    pub project_id: Option<Uuid>,
}

/// Quick-capture a backlog task into a project, or the personal Inbox when omitted.
#[utoipa::path(
    post,
    path = "/capture",
    tag = "capture",
    request_body = CaptureRequest,
    responses(
        (status = 201, description = "Task created from capture", body = TaskDto),
        (status = 400, description = "Validation error", body = crate::error::ErrorBody),
        (status = 401, description = "Not authenticated", body = crate::error::ErrorBody),
        (status = 404, description = "Project not found", body = crate::error::ErrorBody)
    ),
    security(("cc_session" = []))
)]
pub async fn capture(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(body): Json<CaptureRequest>,
) -> Result<(StatusCode, Json<TaskDto>), AppError> {
    let user = require_user(&state.pool, &jar).await?;
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let title = body.title.trim();
    if title.is_empty() {
        return Err(AppError::Validation("title must not be empty".to_string()));
    }

    let project_id = match body.project_id {
        Some(id) => {
            find_project_for_user(&state.pool, user.id, id)
                .await?
                .ok_or(AppError::NotFound)?;
            id
        }
        None => find_or_create_inbox(&state.pool, user.id).await?.id,
    };

    let task = create_task(
        &state.pool,
        user.id,
        project_id,
        title,
        None,
        "backlog",
        None,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(task.into())))
}
