use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::cookie::CookieJar;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::auth::require_user;
use crate::budgets::{budget_summary, upsert_budget, BudgetRecord};
use crate::error::AppError;
use crate::expenses::{resolve_year_month, rupees_to_paise};
use crate::state::AppState;

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpsertBudgetRequest {
    pub year: i32,
    /// Month 1–12 (UTC calendar).
    pub month: u32,
    /// Omit or null for overall monthly budget.
    pub category_id: Option<Uuid>,
    /// Amount in INR rupees (decimals allowed); stored as integer paise.
    pub amount_rupees: f64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BudgetDto {
    pub id: Uuid,
    pub user_id: Uuid,
    pub category_id: Option<Uuid>,
    pub year: i32,
    pub month: i32,
    pub amount_paise: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<BudgetRecord> for BudgetDto {
    fn from(b: BudgetRecord) -> Self {
        Self {
            id: b.id,
            user_id: b.user_id,
            category_id: b.category_id,
            year: b.year,
            month: b.month,
            amount_paise: b.amount_paise,
            created_at: b.created_at,
            updated_at: b.updated_at,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct BudgetSummaryQuery {
    pub year: Option<i32>,
    pub month: Option<u32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CategoryBudgetSummaryDto {
    pub category_id: Uuid,
    pub category_name: String,
    pub budget_paise: Option<i64>,
    pub spent_paise: i64,
    pub remaining_paise: Option<i64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BudgetSummaryDto {
    pub year: i32,
    pub month: u32,
    pub overall_budget_paise: Option<i64>,
    pub total_spent_paise: i64,
    pub remaining_paise: Option<i64>,
    pub by_category: Vec<CategoryBudgetSummaryDto>,
}

/// Upsert a monthly budget (overall when `category_id` is omitted).
#[utoipa::path(
    put,
    path = "/budgets",
    tag = "budgets",
    request_body = UpsertBudgetRequest,
    responses(
        (status = 200, description = "Budget upserted", body = BudgetDto),
        (status = 400, description = "Validation error", body = crate::error::ErrorBody),
        (status = 401, description = "Not authenticated", body = crate::error::ErrorBody),
        (status = 404, description = "Category not found", body = crate::error::ErrorBody)
    ),
    security(("cc_session" = []))
)]
pub async fn upsert(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(body): Json<UpsertBudgetRequest>,
) -> Result<(StatusCode, Json<BudgetDto>), AppError> {
    let user = require_user(&state.pool, &jar).await?;
    let amount_paise = rupees_to_paise(body.amount_rupees)?;
    let budget = upsert_budget(
        &state.pool,
        user.id,
        body.year,
        body.month,
        body.category_id,
        amount_paise,
    )
    .await?;
    Ok((StatusCode::OK, Json(budget.into())))
}

/// Budget vs spend summary for a calendar month (defaults to current UTC month).
#[utoipa::path(
    get,
    path = "/budgets/summary",
    tag = "budgets",
    params(
        ("year" = Option<i32>, Query, description = "Calendar year (UTC); omit with month for current month"),
        ("month" = Option<u32>, Query, description = "Month 1–12 (UTC); omit with year for current month")
    ),
    responses(
        (status = 200, description = "Budget vs spend summary", body = BudgetSummaryDto),
        (status = 400, description = "Validation error", body = crate::error::ErrorBody),
        (status = 401, description = "Not authenticated", body = crate::error::ErrorBody)
    ),
    security(("cc_session" = []))
)]
pub async fn summary(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(query): Query<BudgetSummaryQuery>,
) -> Result<Json<BudgetSummaryDto>, AppError> {
    let user = require_user(&state.pool, &jar).await?;
    let (year, month) = resolve_year_month(query.year, query.month)?;
    let summary = budget_summary(&state.pool, user.id, year, month).await?;
    Ok(Json(BudgetSummaryDto {
        year: summary.year,
        month: summary.month,
        overall_budget_paise: summary.overall_budget_paise,
        total_spent_paise: summary.total_spent_paise,
        remaining_paise: summary.remaining_paise,
        by_category: summary
            .by_category
            .into_iter()
            .map(|c| CategoryBudgetSummaryDto {
                category_id: c.category_id,
                category_name: c.category_name,
                budget_paise: c.budget_paise,
                spent_paise: c.spent_paise,
                remaining_paise: c.remaining_paise,
            })
            .collect(),
    }))
}
