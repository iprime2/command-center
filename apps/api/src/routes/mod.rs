use axum::routing::{get, patch, post, put};
use axum::Router;

use crate::state::AppState;

pub mod auth;
pub mod budgets;
pub mod capture;
pub mod expenses;
pub mod health;
pub mod projects;
pub mod tasks;
pub mod today;

pub use auth::{AuthCredentials, AuthResponse, PublicUserDto};
pub use budgets::{
    BudgetDto, BudgetSummaryDto, CategoryBudgetSummaryDto, UpsertBudgetRequest,
};
pub use capture::CaptureRequest;
pub use expenses::{CreateExpenseRequest, ExpenseCategoryDto, ExpenseDto};
pub use health::HealthResponse;
pub use projects::{CreateProjectRequest, ProjectDto};
pub use tasks::{CreateTaskRequest, PatchTaskRequest, TaskDto};
pub use today::TodayTaskDto;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health::health))
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/auth/logout", post(auth::logout))
        .route("/auth/me", get(auth::me))
        .route("/projects", get(projects::list).post(projects::create))
        .route("/projects/{id}", get(projects::get))
        .route(
            "/projects/{id}/tasks",
            get(tasks::list).post(tasks::create),
        )
        .route("/tasks/{id}", patch(tasks::patch))
        .route("/today", get(today::list))
        .route("/capture", post(capture::capture))
        .route("/expense-categories", get(expenses::list_categories))
        .route("/expenses", get(expenses::list).post(expenses::create))
        .route("/budgets", put(budgets::upsert))
        .route("/budgets/summary", get(budgets::summary))
}
