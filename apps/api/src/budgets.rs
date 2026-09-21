use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::expenses::{
    find_category_for_user, list_or_seed_categories, month_range, resolve_year_month,
};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct BudgetRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub category_id: Option<Uuid>,
    pub year: i32,
    pub month: i32,
    pub amount_paise: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct CategoryBudgetSummary {
    pub category_id: Uuid,
    pub category_name: String,
    pub budget_paise: Option<i64>,
    pub spent_paise: i64,
    pub remaining_paise: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct BudgetSummary {
    pub year: i32,
    pub month: u32,
    pub overall_budget_paise: Option<i64>,
    pub total_spent_paise: i64,
    pub remaining_paise: Option<i64>,
    pub by_category: Vec<CategoryBudgetSummary>,
}

fn remaining(budget: i64, spent: i64) -> i64 {
    budget - spent
}

pub async fn upsert_budget(
    pool: &PgPool,
    user_id: Uuid,
    year: i32,
    month: u32,
    category_id: Option<Uuid>,
    amount_paise: i64,
) -> Result<BudgetRecord, AppError> {
    if !(1..=12).contains(&month) {
        return Err(AppError::Validation(
            "month must be between 1 and 12".to_string(),
        ));
    }
    if amount_paise <= 0 {
        return Err(AppError::Validation(
            "amount_rupees must be a positive number".to_string(),
        ));
    }

    if let Some(cid) = category_id {
        find_category_for_user(pool, user_id, cid)
            .await?
            .ok_or(AppError::NotFound)?;
    }

    let month_i32 = month as i32;
    let now = Utc::now();

    // Soft-deleted rows are ignored by the partial unique indexes; update an
    // active row if present, otherwise insert (or revive a soft-deleted twin).
    let existing = find_active_budget(pool, user_id, year, month_i32, category_id).await?;

    if let Some(row) = existing {
        let updated = sqlx::query_as::<_, BudgetRecord>(
            r#"
            UPDATE budgets
            SET amount_paise = $1, updated_at = $2
            WHERE id = $3 AND user_id = $4 AND deleted_at IS NULL
            RETURNING id, user_id, category_id, year, month, amount_paise, created_at, updated_at
            "#,
        )
        .bind(amount_paise)
        .bind(now)
        .bind(row.id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;
        return Ok(updated);
    }

    let revived =
        revive_soft_deleted(pool, user_id, year, month_i32, category_id, amount_paise, now).await?;
    if let Some(row) = revived {
        return Ok(row);
    }

    let id = Uuid::new_v4();
    let inserted = sqlx::query_as::<_, BudgetRecord>(
        r#"
        INSERT INTO budgets (
            id, user_id, category_id, year, month, amount_paise, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $7)
        RETURNING id, user_id, category_id, year, month, amount_paise, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(user_id)
    .bind(category_id)
    .bind(year)
    .bind(month_i32)
    .bind(amount_paise)
    .bind(now)
    .fetch_one(pool)
    .await?;

    Ok(inserted)
}

async fn find_active_budget(
    pool: &PgPool,
    user_id: Uuid,
    year: i32,
    month: i32,
    category_id: Option<Uuid>,
) -> Result<Option<BudgetRecord>, AppError> {
    let row = match category_id {
        Some(cid) => {
            sqlx::query_as::<_, BudgetRecord>(
                r#"
                SELECT id, user_id, category_id, year, month, amount_paise, created_at, updated_at
                FROM budgets
                WHERE user_id = $1
                  AND category_id = $2
                  AND year = $3
                  AND month = $4
                  AND deleted_at IS NULL
                "#,
            )
            .bind(user_id)
            .bind(cid)
            .bind(year)
            .bind(month)
            .fetch_optional(pool)
            .await?
        }
        None => {
            sqlx::query_as::<_, BudgetRecord>(
                r#"
                SELECT id, user_id, category_id, year, month, amount_paise, created_at, updated_at
                FROM budgets
                WHERE user_id = $1
                  AND category_id IS NULL
                  AND year = $2
                  AND month = $3
                  AND deleted_at IS NULL
                "#,
            )
            .bind(user_id)
            .bind(year)
            .bind(month)
            .fetch_optional(pool)
            .await?
        }
    };
    Ok(row)
}

async fn revive_soft_deleted(
    pool: &PgPool,
    user_id: Uuid,
    year: i32,
    month: i32,
    category_id: Option<Uuid>,
    amount_paise: i64,
    now: DateTime<Utc>,
) -> Result<Option<BudgetRecord>, AppError> {
    let row = match category_id {
        Some(cid) => {
            sqlx::query_as::<_, BudgetRecord>(
                r#"
                UPDATE budgets
                SET amount_paise = $1, updated_at = $2, deleted_at = NULL
                WHERE id = (
                    SELECT id FROM budgets
                    WHERE user_id = $3
                      AND category_id = $4
                      AND year = $5
                      AND month = $6
                      AND deleted_at IS NOT NULL
                    ORDER BY updated_at DESC
                    LIMIT 1
                )
                RETURNING id, user_id, category_id, year, month, amount_paise, created_at, updated_at
                "#,
            )
            .bind(amount_paise)
            .bind(now)
            .bind(user_id)
            .bind(cid)
            .bind(year)
            .bind(month)
            .fetch_optional(pool)
            .await?
        }
        None => {
            sqlx::query_as::<_, BudgetRecord>(
                r#"
                UPDATE budgets
                SET amount_paise = $1, updated_at = $2, deleted_at = NULL
                WHERE id = (
                    SELECT id FROM budgets
                    WHERE user_id = $3
                      AND category_id IS NULL
                      AND year = $4
                      AND month = $5
                      AND deleted_at IS NOT NULL
                    ORDER BY updated_at DESC
                    LIMIT 1
                )
                RETURNING id, user_id, category_id, year, month, amount_paise, created_at, updated_at
                "#,
            )
            .bind(amount_paise)
            .bind(now)
            .bind(user_id)
            .bind(year)
            .bind(month)
            .fetch_optional(pool)
            .await?
        }
    };
    Ok(row)
}

pub async fn budget_summary(
    pool: &PgPool,
    user_id: Uuid,
    year: i32,
    month: u32,
) -> Result<BudgetSummary, AppError> {
    let _ = resolve_year_month(Some(year), Some(month))?;
    let (start, end) = month_range(year, month)?;
    let month_i32 = month as i32;

    let categories = list_or_seed_categories(pool, user_id).await?;

    let budgets = sqlx::query_as::<_, BudgetRecord>(
        r#"
        SELECT id, user_id, category_id, year, month, amount_paise, created_at, updated_at
        FROM budgets
        WHERE user_id = $1
          AND year = $2
          AND month = $3
          AND deleted_at IS NULL
        "#,
    )
    .bind(user_id)
    .bind(year)
    .bind(month_i32)
    .fetch_all(pool)
    .await?;

    let overall_budget_paise = budgets
        .iter()
        .find(|b| b.category_id.is_none())
        .map(|b| b.amount_paise);

    #[derive(sqlx::FromRow)]
    struct SpentRow {
        category_id: Uuid,
        spent_paise: i64,
    }

    let spent_rows = sqlx::query_as::<_, SpentRow>(
        r#"
        SELECT category_id, COALESCE(SUM(amount_paise), 0)::BIGINT AS spent_paise
        FROM expenses
        WHERE user_id = $1
          AND deleted_at IS NULL
          AND spent_on >= $2
          AND spent_on < $3
        GROUP BY category_id
        "#,
    )
    .bind(user_id)
    .bind(start)
    .bind(end)
    .fetch_all(pool)
    .await?;

    let total_spent_paise: i64 = spent_rows.iter().map(|r| r.spent_paise).sum();

    let remaining_paise = overall_budget_paise.map(|b| remaining(b, total_spent_paise));

    let mut by_category: Vec<CategoryBudgetSummary> = categories
        .into_iter()
        .map(|cat| {
            let budget_paise = budgets
                .iter()
                .find(|b| b.category_id == Some(cat.id))
                .map(|b| b.amount_paise);
            let spent_paise = spent_rows
                .iter()
                .find(|r| r.category_id == cat.id)
                .map(|r| r.spent_paise)
                .unwrap_or(0);
            let remaining_paise = budget_paise.map(|b| remaining(b, spent_paise));
            CategoryBudgetSummary {
                category_id: cat.id,
                category_name: cat.name,
                budget_paise,
                spent_paise,
                remaining_paise,
            }
        })
        .filter(|c| c.budget_paise.is_some() || c.spent_paise > 0)
        .collect();

    by_category.sort_by(|a, b| a.category_name.cmp(&b.category_name));

    Ok(BudgetSummary {
        year,
        month,
        overall_budget_paise,
        total_spent_paise,
        remaining_paise,
        by_category,
    })
}
