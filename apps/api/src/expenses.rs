use chrono::{DateTime, Datelike, NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::projects::find_project_for_user;

pub const DEFAULT_CATEGORY_NAMES: &[&str] = &["Food", "Transport", "Rent", "Utilities", "Other"];

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ExpenseCategoryRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ExpenseRecord {
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

/// Convert decimal rupees to integer paise (rupees × 100, rounded to nearest paise).
pub fn rupees_to_paise(amount_rupees: f64) -> Result<i64, AppError> {
    if !amount_rupees.is_finite() || amount_rupees <= 0.0 {
        return Err(AppError::Validation(
            "amount_rupees must be a positive number".to_string(),
        ));
    }
    let paise = (amount_rupees * 100.0).round();
    if paise < 1.0 || paise > i64::MAX as f64 {
        return Err(AppError::Validation(
            "amount_rupees must be a positive number".to_string(),
        ));
    }
    Ok(paise as i64)
}

async fn list_categories_raw(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<ExpenseCategoryRecord>, AppError> {
    let cats = sqlx::query_as::<_, ExpenseCategoryRecord>(
        r#"
        SELECT id, user_id, name, created_at, updated_at
        FROM expense_categories
        WHERE user_id = $1 AND deleted_at IS NULL
        ORDER BY name ASC
        LIMIT 100
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(cats)
}

async fn insert_category(
    pool: &PgPool,
    user_id: Uuid,
    name: &str,
) -> Result<ExpenseCategoryRecord, AppError> {
    let id = Uuid::new_v4();
    let now = Utc::now();
    let cat = sqlx::query_as::<_, ExpenseCategoryRecord>(
        r#"
        INSERT INTO expense_categories (id, user_id, name, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $4)
        RETURNING id, user_id, name, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(user_id)
    .bind(name)
    .bind(now)
    .fetch_one(pool)
    .await?;
    Ok(cat)
}

/// List categories for the user, seeding defaults when none exist.
pub async fn list_or_seed_categories(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<ExpenseCategoryRecord>, AppError> {
    let existing = list_categories_raw(pool, user_id).await?;
    if !existing.is_empty() {
        return Ok(existing);
    }

    for name in DEFAULT_CATEGORY_NAMES {
        insert_category(pool, user_id, name).await?;
    }

    list_categories_raw(pool, user_id).await
}

pub async fn find_category_for_user(
    pool: &PgPool,
    user_id: Uuid,
    category_id: Uuid,
) -> Result<Option<ExpenseCategoryRecord>, AppError> {
    let cat = sqlx::query_as::<_, ExpenseCategoryRecord>(
        r#"
        SELECT id, user_id, name, created_at, updated_at
        FROM expense_categories
        WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL
        "#,
    )
    .bind(category_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    Ok(cat)
}

pub async fn create_expense(
    pool: &PgPool,
    user_id: Uuid,
    category_id: Uuid,
    amount_paise: i64,
    spent_on: NaiveDate,
    note: Option<&str>,
    project_id: Option<Uuid>,
) -> Result<ExpenseRecord, AppError> {
    if amount_paise <= 0 {
        return Err(AppError::Validation(
            "amount_rupees must be a positive number".to_string(),
        ));
    }

    find_category_for_user(pool, user_id, category_id)
        .await?
        .ok_or(AppError::NotFound)?;

    if let Some(pid) = project_id {
        find_project_for_user(pool, user_id, pid)
            .await?
            .ok_or(AppError::NotFound)?;
    }

    let id = Uuid::new_v4();
    let now = Utc::now();

    let expense = sqlx::query_as::<_, ExpenseRecord>(
        r#"
        INSERT INTO expenses (
            id, user_id, category_id, amount_paise, currency, spent_on, note, project_id,
            created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, 'INR', $5, $6, $7, $8, $8)
        RETURNING
            id, user_id, category_id, amount_paise, currency, spent_on, note, project_id,
            created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(user_id)
    .bind(category_id)
    .bind(amount_paise)
    .bind(spent_on)
    .bind(note)
    .bind(project_id)
    .bind(now)
    .fetch_one(pool)
    .await?;

    Ok(expense)
}

/// Resolve year/month for listing; defaults to current UTC month when both unset.
pub fn resolve_year_month(
    year: Option<i32>,
    month: Option<u32>,
) -> Result<(i32, u32), AppError> {
    match (year, month) {
        (None, None) => {
            let now = Utc::now().date_naive();
            Ok((now.year(), now.month()))
        }
        (Some(y), Some(m)) => {
            if !(1..=12).contains(&m) {
                return Err(AppError::Validation(
                    "month must be between 1 and 12".to_string(),
                ));
            }
            Ok((y, m))
        }
        _ => Err(AppError::Validation(
            "year and month must both be provided or both omitted".to_string(),
        )),
    }
}

pub fn month_range(year: i32, month: u32) -> Result<(NaiveDate, NaiveDate), AppError> {
    let start = NaiveDate::from_ymd_opt(year, month, 1).ok_or_else(|| {
        AppError::Validation("invalid year/month".to_string())
    })?;
    let end = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .ok_or_else(|| AppError::Validation("invalid year/month".to_string()))?;
    Ok((start, end))
}

pub async fn list_expenses_for_month(
    pool: &PgPool,
    user_id: Uuid,
    year: i32,
    month: u32,
) -> Result<Vec<ExpenseRecord>, AppError> {
    let (start, end) = month_range(year, month)?;

    let expenses = sqlx::query_as::<_, ExpenseRecord>(
        r#"
        SELECT
            id, user_id, category_id, amount_paise, currency, spent_on, note, project_id,
            created_at, updated_at
        FROM expenses
        WHERE user_id = $1
          AND deleted_at IS NULL
          AND spent_on >= $2
          AND spent_on < $3
        ORDER BY spent_on DESC, created_at DESC
        LIMIT 500
        "#,
    )
    .bind(user_id)
    .bind(start)
    .bind(end)
    .fetch_all(pool)
    .await?;

    Ok(expenses)
}
