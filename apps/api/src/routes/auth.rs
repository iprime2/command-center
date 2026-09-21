use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use axum_extra::extract::cookie::CookieJar;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::auth::{
    clear_session_cookie, create_session, create_user, destroy_session, find_active_user_by_email,
    hash_password, issue_session_cookie, lookup_user_by_session_token, verify_password,
    SESSION_COOKIE,
};
use crate::auth::user::PublicUser;
use crate::error::AppError;
use crate::state::AppState;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct AuthCredentials {
    #[validate(length(min = 3, max = 255))]
    pub email: String,
    #[validate(length(min = 8, max = 128))]
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PublicUserDto {
    pub id: Uuid,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<PublicUser> for PublicUserDto {
    fn from(u: PublicUser) -> Self {
        Self {
            id: u.id,
            email: u.email,
            created_at: u.created_at,
            updated_at: u.updated_at,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponse {
    pub user: PublicUserDto,
}

fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

fn validate_credentials(creds: &AuthCredentials) -> Result<(), AppError> {
    creds
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    let email = normalize_email(&creds.email);
    if !email.contains('@') || email.starts_with('@') || email.ends_with('@') {
        return Err(AppError::Validation("invalid email".to_string()));
    }
    Ok(())
}

/// Register a new user and establish an httpOnly session cookie (`cc_session`).
#[utoipa::path(
    post,
    path = "/auth/register",
    tag = "auth",
    request_body = AuthCredentials,
    responses(
        (status = 201, description = "User created; session cookie set", body = AuthResponse),
        (status = 400, description = "Validation error", body = crate::error::ErrorBody),
        (status = 409, description = "Email already registered", body = crate::error::ErrorBody)
    )
)]
pub async fn register(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(body): Json<AuthCredentials>,
) -> Result<(StatusCode, CookieJar, Json<AuthResponse>), AppError> {
    validate_credentials(&body)?;
    let email = normalize_email(&body.email);

    if find_active_user_by_email(&state.pool, &email)
        .await?
        .is_some()
    {
        return Err(AppError::Conflict("email already registered".to_string()));
    }

    let password_hash = hash_password(&body.password)?;
    let user = create_user(&state.pool, &email, &password_hash).await?;
    let token = create_session(&state.pool, user.id).await?;
    let jar = jar.add(issue_session_cookie(token));

    Ok((
        StatusCode::CREATED,
        jar,
        Json(AuthResponse {
            user: PublicUser::from(user).into(),
        }),
    ))
}

/// Log in and establish an httpOnly session cookie (`cc_session`).
#[utoipa::path(
    post,
    path = "/auth/login",
    tag = "auth",
    request_body = AuthCredentials,
    responses(
        (status = 200, description = "Authenticated; session cookie set", body = AuthResponse),
        (status = 400, description = "Validation error", body = crate::error::ErrorBody),
        (status = 401, description = "Invalid credentials", body = crate::error::ErrorBody)
    )
)]
pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(body): Json<AuthCredentials>,
) -> Result<(CookieJar, Json<AuthResponse>), AppError> {
    validate_credentials(&body)?;
    let email = normalize_email(&body.email);

    let user = find_active_user_by_email(&state.pool, &email)
        .await?
        .ok_or(AppError::Unauthorized)?;

    if !verify_password(&body.password, &user.password_hash)? {
        return Err(AppError::Unauthorized);
    }

    let token = create_session(&state.pool, user.id).await?;
    let jar = jar.add(issue_session_cookie(token));

    Ok((
        jar,
        Json(AuthResponse {
            user: PublicUser::from(user).into(),
        }),
    ))
}

/// Clear the current session (cookie + soft-delete session row).
#[utoipa::path(
    post,
    path = "/auth/logout",
    tag = "auth",
    responses(
        (status = 204, description = "Session cleared")
    )
)]
pub async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<impl IntoResponse, AppError> {
    if let Some(cookie) = jar.get(SESSION_COOKIE) {
        destroy_session(&state.pool, cookie.value()).await?;
    }
    let jar = jar.add(clear_session_cookie());
    Ok((StatusCode::NO_CONTENT, jar))
}

/// Current authenticated user.
#[utoipa::path(
    get,
    path = "/auth/me",
    tag = "auth",
    responses(
        (status = 200, description = "Current user", body = PublicUserDto),
        (status = 401, description = "Not authenticated", body = crate::error::ErrorBody)
    )
)]
pub async fn me(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<Json<PublicUserDto>, AppError> {
    let cookie = jar.get(SESSION_COOKIE).ok_or(AppError::Unauthorized)?;
    let user = lookup_user_by_session_token(&state.pool, cookie.value())
        .await?
        .ok_or(AppError::Unauthorized)?;

    Ok(Json(PublicUser::from(user).into()))
}
