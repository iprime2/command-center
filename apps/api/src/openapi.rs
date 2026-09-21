use utoipa::OpenApi;

use crate::error::{ErrorBody, ErrorDetail};
use crate::routes::{
    AuthCredentials, AuthResponse, BudgetDto, BudgetSummaryDto, CaptureRequest,
    CategoryBudgetSummaryDto, CreateExpenseRequest, CreateProjectRequest, CreateTaskRequest,
    ExpenseCategoryDto, ExpenseDto, HealthResponse, PatchTaskRequest, ProjectDto, PublicUserDto,
    TaskDto, TodayTaskDto, UpsertBudgetRequest,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::health::health,
        crate::routes::auth::register,
        crate::routes::auth::login,
        crate::routes::auth::logout,
        crate::routes::auth::me,
        crate::routes::projects::create,
        crate::routes::projects::list,
        crate::routes::projects::get,
        crate::routes::tasks::list,
        crate::routes::tasks::create,
        crate::routes::tasks::patch,
        crate::routes::today::list,
        crate::routes::capture::capture,
        crate::routes::expenses::list_categories,
        crate::routes::expenses::create,
        crate::routes::expenses::list,
        crate::routes::budgets::upsert,
        crate::routes::budgets::summary,
    ),
    components(schemas(
        HealthResponse,
        AuthCredentials,
        AuthResponse,
        PublicUserDto,
        CreateProjectRequest,
        ProjectDto,
        CreateTaskRequest,
        PatchTaskRequest,
        TaskDto,
        TodayTaskDto,
        CaptureRequest,
        CreateExpenseRequest,
        ExpenseCategoryDto,
        ExpenseDto,
        UpsertBudgetRequest,
        BudgetDto,
        BudgetSummaryDto,
        CategoryBudgetSummaryDto,
        ErrorBody,
        ErrorDetail,
    )),
    tags(
        (name = "health", description = "Liveness / readiness"),
        (name = "auth", description = "Email/password session auth"),
        (name = "projects", description = "User-owned projects"),
        (name = "tasks", description = "Project board tasks"),
        (
            name = "today",
            description = "Today view: doing tasks and tasks due on/before the current UTC date"
        ),
        (
            name = "capture",
            description = "Quick capture into a project or the personal Inbox"
        ),
        (
            name = "expenses",
            description = "INR expenses (amounts stored as integer paise) and categories"
        ),
        (
            name = "budgets",
            description = "Monthly INR budgets (overall or per category) vs spend"
        )
    ),
    modifiers(&SecurityAddon),
    info(
        title = "Command Center API",
        version = "0.1.0",
        description = "Personal productivity OS API. Auth uses httpOnly cookie `cc_session` (server-side sessions). Today due-date comparisons use the UTC calendar date. Expense amounts are INR paise (rupees × 100)."
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "cc_session",
                utoipa::openapi::security::SecurityScheme::ApiKey(
                    utoipa::openapi::security::ApiKey::Cookie(
                        utoipa::openapi::security::ApiKeyValue::new("cc_session"),
                    ),
                ),
            )
        }
    }
}
