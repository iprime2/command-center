//! Helpers for integration tests (and local smoke).

use axum::Router;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use crate::state::AppState;

pub async fn test_pool() -> PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set for tests (postgres://...)");
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("failed to connect to DATABASE_URL")
}

pub async fn migrate(pool: &PgPool) {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .expect("migrations failed");
}

pub fn app_with_pool(pool: PgPool) -> Router {
    crate::app(AppState::new(
        pool,
        "http://127.0.0.1:5173".to_string(),
    ))
}

/// Hard-delete test user + sessions + projects + tasks + expenses by email (test cleanup only).
pub async fn cleanup_email(pool: &PgPool, email: &str) {
    let _ = sqlx::query(
        r#"
        DELETE FROM expenses
        WHERE user_id IN (SELECT id FROM users WHERE email = $1)
        "#,
    )
    .bind(email)
    .execute(pool)
    .await;

    let _ = sqlx::query(
        r#"
        DELETE FROM expense_categories
        WHERE user_id IN (SELECT id FROM users WHERE email = $1)
        "#,
    )
    .bind(email)
    .execute(pool)
    .await;

    let _ = sqlx::query(
        r#"
        DELETE FROM tasks
        WHERE user_id IN (SELECT id FROM users WHERE email = $1)
        "#,
    )
    .bind(email)
    .execute(pool)
    .await;

    let _ = sqlx::query(
        r#"
        DELETE FROM projects
        WHERE user_id IN (SELECT id FROM users WHERE email = $1)
        "#,
    )
    .bind(email)
    .execute(pool)
    .await;

    let _ = sqlx::query(
        r#"
        DELETE FROM sessions
        WHERE user_id IN (SELECT id FROM users WHERE email = $1)
        "#,
    )
    .bind(email)
    .execute(pool)
    .await;

    let _ = sqlx::query("DELETE FROM users WHERE email = $1")
        .bind(email)
        .execute(pool)
        .await;
}
