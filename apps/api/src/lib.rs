//! Command Center API library (Axum + SQLx).

pub mod auth;
pub mod budgets;
pub mod config;
pub mod env_file;
pub mod error;
pub mod expenses;
pub mod openapi;
pub mod projects;
pub mod routes;
pub mod state;
pub mod tasks;
pub mod test_support;

use axum::Router;
use state::AppState;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::openapi::ApiDoc;

/// Build the HTTP router (used by `main` and integration tests).
pub fn app(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::exact(
            state
                .web_origin
                .parse()
                .expect("WEB_ORIGIN must be a valid Origin header value"),
        ))
        .allow_credentials(true)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::PATCH,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::ACCEPT,
            axum::http::header::COOKIE,
        ]);

    Router::new()
        .merge(routes::router())
        .merge(
            SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()),
        )
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
