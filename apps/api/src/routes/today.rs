use axum::extract::State;
use axum::Json;
use axum_extra::extract::cookie::CookieJar;
use chrono::{NaiveDate, Utc};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::auth::require_user;
use crate::error::AppError;
use crate::state::AppState;
use crate::tasks::{list_today_tasks, TodayTaskRecord};

#[derive(Debug, Serialize, ToSchema)]
pub struct TodayTaskDto {
    pub id: Uuid,
    pub project_id: Uuid,
    pub title: String,
    pub status: String,
    pub due_on: Option<NaiveDate>,
    pub project_name: String,
}

impl From<TodayTaskRecord> for TodayTaskDto {
    fn from(t: TodayTaskRecord) -> Self {
        Self {
            id: t.id,
            project_id: t.project_id,
            title: t.title,
            status: t.status,
            due_on: t.due_on,
            project_name: t.project_name,
        }
    }
}

/// List the current user's Today tasks: status `doing`, or `due_on` on/before today (UTC date).
#[utoipa::path(
    get,
    path = "/today",
    tag = "today",
    responses(
        (status = 200, description = "Today tasks (doing or due on/before UTC today)", body = Vec<TodayTaskDto>),
        (status = 401, description = "Not authenticated", body = crate::error::ErrorBody)
    ),
    security(("cc_session" = []))
)]
pub async fn list(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<Json<Vec<TodayTaskDto>>, AppError> {
    let user = require_user(&state.pool, &jar).await?;
    let today = Utc::now().date_naive();
    let tasks = list_today_tasks(&state.pool, user.id, today).await?;
    Ok(Json(tasks.into_iter().map(TodayTaskDto::from).collect()))
}
