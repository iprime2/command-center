use axum_extra::extract::cookie::{Cookie, SameSite};
use chrono::{Duration, Utc};
use rand::RngCore;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use time::Duration as TimeDuration;
use uuid::Uuid;

use crate::auth::user::UserRecord;
use crate::error::AppError;

pub const SESSION_COOKIE: &str = "cc_session";
const SESSION_TTL_DAYS: i64 = 30;

pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

fn generate_token() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

pub async fn create_session(pool: &PgPool, user_id: Uuid) -> Result<String, AppError> {
    let token = generate_token();
    let token_hash = hash_token(&token);
    let id = Uuid::new_v4();
    let now = Utc::now();
    let expires_at = now + Duration::days(SESSION_TTL_DAYS);

    sqlx::query(
        r#"
        INSERT INTO sessions (id, user_id, token_hash, expires_at, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $5)
        "#,
    )
    .bind(id)
    .bind(user_id)
    .bind(token_hash)
    .bind(expires_at)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(token)
}

pub async fn destroy_session(pool: &PgPool, raw_token: &str) -> Result<(), AppError> {
    let token_hash = hash_token(raw_token);
    let now = Utc::now();
    sqlx::query(
        r#"
        UPDATE sessions
        SET deleted_at = $1, updated_at = $1
        WHERE token_hash = $2 AND deleted_at IS NULL
        "#,
    )
    .bind(now)
    .bind(token_hash)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn lookup_user_by_session_token(
    pool: &PgPool,
    raw_token: &str,
) -> Result<Option<UserRecord>, AppError> {
    let token_hash = hash_token(raw_token);
    let now = Utc::now();

    let user = sqlx::query_as::<_, UserRecord>(
        r#"
        SELECT u.id, u.email, u.password_hash, u.created_at, u.updated_at
        FROM sessions s
        JOIN users u ON u.id = s.user_id
        WHERE s.token_hash = $1
          AND s.deleted_at IS NULL
          AND s.expires_at > $2
          AND u.deleted_at IS NULL
        "#,
    )
    .bind(token_hash)
    .bind(now)
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

pub async fn require_user(
    pool: &PgPool,
    jar: &axum_extra::extract::cookie::CookieJar,
) -> Result<UserRecord, AppError> {
    let cookie = jar.get(SESSION_COOKIE).ok_or(AppError::Unauthorized)?;
    lookup_user_by_session_token(pool, cookie.value())
        .await?
        .ok_or(AppError::Unauthorized)
}

pub fn issue_session_cookie(token: String) -> Cookie<'static> {
    Cookie::build((SESSION_COOKIE, token))
        .http_only(true)
        .same_site(SameSite::Lax)
        .path("/")
        .max_age(TimeDuration::days(SESSION_TTL_DAYS))
        .build()
}

pub fn clear_session_cookie() -> Cookie<'static> {
    Cookie::build((SESSION_COOKIE, ""))
        .http_only(true)
        .same_site(SameSite::Lax)
        .path("/")
        .max_age(TimeDuration::seconds(0))
        .build()
}
