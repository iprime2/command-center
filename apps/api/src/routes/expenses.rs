use axum::extract::{Query, State};
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
use crate::expenses::{
    create_expense, list_expenses_for_month, list_or_seed_categories, resolve_year_month,
    rupees_to_paise, ExpenseCategoryRecord, ExpenseRecord,
};
use crate::state::AppState;

#[derive(Debug, Serialize, ToSchema)]
pub struct ExpenseCategoryDto {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ExpenseCategoryRecord> for ExpenseCategoryDto {
    fn from(c: ExpenseCategoryRecord) -> Self {
        Self {
            id: c.id,
            user_id: c.user_id,
            name: c.name,
            created_at: c.created_at,
            updated_at: c.updated_at,
        }
    }
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateExpenseRequest {
    /// Amount in INR rupees (decimals allowed); stored as integer paise.
    pub amount_rupees: f64,
    pub category_id: Uuid,
    /// Calendar date the money was spent (YYYY-MM-DD).
    pub spent_on: NaiveDate,
    #[validate(length(max = 2000))]
    pub note: Option<String>,
    pub project_id: Option<Uuid>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ExpenseDto {
    pub id: Uuid,
    pub user_id: Uuid,
    pub category_id: Uuid,
    pub amount_paise: i64,
    pub currency: String,
    pub spent_on: NaiveDate,
    pub note: Option<String>,
    pub project_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ExpenseRecord> for ExpenseDto {
    fn from(e: ExpenseRecord) -> Self {
        Self {
            id: e.id,
            user_id: e.user_id,
            category_id: e.category_id,
            amount_paise: e.amount_paise,
            currency: e.currency,
            spent_on: e.spent_on,
            note: e.note,
            project_id: e.project_id,
            created_at: e.created_at,
            updated_at: e.updated_at,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ListExpensesQuery {
    pub year: Option<i32>,
    pub month: Option<u32>,
}

/// List expense categories for the current user (seeds defaults if empty).
#[utoipa::path(
    get,
    path = "/expense-categories",
    tag = "expenses",
    responses(
        (status = 200, description = "Expense categories", body = Vec<ExpenseCategoryDto>),
        (status = 401, description = "Not authenticated", body = crate::error::ErrorBody)
    ),
    security(("cc_session" = []))
)]
pub async fn list_categories(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<Json<Vec<ExpenseCategoryDto>>, AppError> {
    let user = require_user(&state.pool, &jar).await?;
    let cats = list_or_seed_categories(&state.pool, user.id).await?;
    Ok(Json(cats.into_iter().map(ExpenseCategoryDto::from).collect()))
}

/// Create an expense owned by the current user.
#[utoipa::path(
    post,
    path = "/expenses",
    tag = "expenses",
    request_body = CreateExpenseRequest,
    responses(
        (status = 201, description = "Expense created", body = ExpenseDto),
        (status = 400, description = "Validation error", body = crate::error::ErrorBody),
        (status = 401, description = "Not authenticated", body = crate::error::ErrorBody),
        (status = 404, description = "Category or project not found", body = crate::error::ErrorBody)
    ),
    security(("cc_session" = []))
)]
pub async fn create(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(body): Json<CreateExpenseRequest>,
) -> Result<(StatusCode, Json<ExpenseDto>), AppError> {
    let user = require_user(&state.pool, &jar).await?;
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let amount_paise = rupees_to_paise(body.amount_rupees)?;

    let note = body
        .note
        .as_ref()
        .map(|n| n.trim())
        .filter(|n| !n.is_empty());

    let expense = create_expense(
        &state.pool,
        user.id,
        body.category_id,
        amount_paise,
        body.spent_on,
        note,
        body.project_id,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(expense.into())))
}

/// List expenses for a calendar month (defaults to current UTC month).
#[utoipa::path(
    get,
    path = "/expenses",
    tag = "expenses",
    params(
        ("year" = Option<i32>, Query, description = "Calendar year (UTC); omit with month for current month"),
        ("month" = Option<u32>, Query, description = "Month 1–12 (UTC); omit with year for current month")
    ),
    responses(
        (status = 200, description = "Expenses for the month", body = Vec<ExpenseDto>),
        (status = 400, description = "Validation error", body = crate::error::ErrorBody),
        (status = 401, description = "Not authenticated", body = crate::error::ErrorBody)
    ),
    security(("cc_session" = []))
)]
pub async fn list(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(query): Query<ListExpensesQuery>,
) -> Result<Json<Vec<ExpenseDto>>, AppError> {
    let user = require_user(&state.pool, &jar).await?;
    let (year, month) = resolve_year_month(query.year, query.month)?;
    let expenses = list_expenses_for_month(&state.pool, user.id, year, month).await?;
    Ok(Json(expenses.into_iter().map(ExpenseDto::from).collect()))
}
