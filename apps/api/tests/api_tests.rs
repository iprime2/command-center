//! Integration tests for health, auth, projects, tasks, today, capture, expenses, and budgets.
//! Requires DATABASE_URL pointing at a Postgres instance.

use axum::body::Body;
use axum::http::{header, Method, Request, StatusCode};
use command_center_api::test_support::{app_with_pool, cleanup_email, migrate, test_pool};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;

fn unique_email(prefix: &str) -> String {
    format!("{prefix}-{}@example.com", uuid::Uuid::new_v4())
}

async fn json_body(response: axum::response::Response) -> Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap_or_else(|_| json!({}))
}

fn session_cookie(response: &axum::response::Response) -> Option<String> {
    response
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .find(|c| c.starts_with("cc_session="))
        .map(|c| c.split(';').next().unwrap().to_string())
}

#[tokio::test]
async fn health_returns_ok() {
    let pool = test_pool().await;
    migrate(&pool).await;
    let app = app_with_pool(pool);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn register_login_me_and_logout_flow() {
    let pool = test_pool().await;
    migrate(&pool).await;
    let email = unique_email("flow");
    cleanup_email(&pool, &email).await;

    let app = app_with_pool(pool.clone());

    // Register
    let register = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/auth/register")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({ "email": email, "password": "password123" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(register.status(), StatusCode::CREATED);
    let cookie = session_cookie(&register).expect("register should set cc_session cookie");
    let register_body = json_body(register).await;
    assert_eq!(register_body["user"]["email"], email);
    assert!(register_body["user"]["id"].is_string());

    // Me with session
    let me = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/auth/me")
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(me.status(), StatusCode::OK);
    let me_body = json_body(me).await;
    assert_eq!(me_body["email"], email);

    // Logout
    let logout = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/auth/logout")
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(logout.status(), StatusCode::NO_CONTENT);

    // Me after logout → 401
    let me_after = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/auth/me")
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(me_after.status(), StatusCode::UNAUTHORIZED);

    // Login again
    let login = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({ "email": email, "password": "password123" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(login.status(), StatusCode::OK);
    let login_cookie = session_cookie(&login).expect("login should set cookie");
    let login_body = json_body(login).await;
    assert_eq!(login_body["user"]["email"], email);

    let me_again = app
        .oneshot(
            Request::builder()
                .uri("/auth/me")
                .header(header::COOKIE, &login_cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(me_again.status(), StatusCode::OK);

    cleanup_email(&pool, &email).await;
}

#[tokio::test]
async fn me_without_session_is_unauthorized() {
    let pool = test_pool().await;
    migrate(&pool).await;
    let app = app_with_pool(pool);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/auth/me")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn login_with_wrong_password_fails() {
    let pool = test_pool().await;
    migrate(&pool).await;
    let email = unique_email("badpw");
    cleanup_email(&pool, &email).await;
    let app = app_with_pool(pool.clone());

    let register = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/auth/register")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({ "email": email, "password": "password123" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(register.status(), StatusCode::CREATED);

    let login = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({ "email": email, "password": "wrong-password" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(login.status(), StatusCode::UNAUTHORIZED);

    cleanup_email(&pool, &email).await;
}

#[tokio::test]
async fn duplicate_register_conflicts() {
    let pool = test_pool().await;
    migrate(&pool).await;
    let email = unique_email("dup");
    cleanup_email(&pool, &email).await;
    let app = app_with_pool(pool.clone());

    let body = json!({ "email": email, "password": "password123" }).to_string();

    let first = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/auth/register")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body.clone()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(first.status(), StatusCode::CREATED);

    let second = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/auth/register")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(second.status(), StatusCode::CONFLICT);

    cleanup_email(&pool, &email).await;
}

async fn register_and_cookie(app: &axum::Router, email: &str) -> String {
    let register = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/auth/register")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({ "email": email, "password": "password123" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(register.status(), StatusCode::CREATED);
    session_cookie(&register).expect("register should set cc_session cookie")
}

#[tokio::test]
async fn create_list_and_get_project() {
    let pool = test_pool().await;
    migrate(&pool).await;
    let email = unique_email("proj");
    cleanup_email(&pool, &email).await;
    let app = app_with_pool(pool.clone());
    let cookie = register_and_cookie(&app, &email).await;

    let create = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/projects")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, &cookie)
                .body(Body::from(
                    json!({ "name": "Alpha", "description": "First project" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create.status(), StatusCode::CREATED);
    let created = json_body(create).await;
    assert_eq!(created["name"], "Alpha");
    assert_eq!(created["description"], "First project");
    assert_eq!(created["status"], "active");
    assert!(created["id"].is_string());
    let project_id = created["id"].as_str().unwrap().to_string();

    let list = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/projects")
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list.status(), StatusCode::OK);
    let listed = json_body(list).await;
    let projects = listed.as_array().expect("list should be a JSON array");
    assert!(
        projects
            .iter()
            .any(|p| p["id"] == project_id && p["name"] == "Alpha"),
        "created project should appear in list"
    );

    let get = app
        .oneshot(
            Request::builder()
                .uri(format!("/projects/{project_id}"))
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get.status(), StatusCode::OK);
    let got = json_body(get).await;
    assert_eq!(got["id"], project_id);
    assert_eq!(got["name"], "Alpha");
    assert_eq!(got["description"], "First project");

    cleanup_email(&pool, &email).await;
}

#[tokio::test]
async fn projects_require_auth() {
    let pool = test_pool().await;
    migrate(&pool).await;
    let app = app_with_pool(pool);

    let list = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/projects")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list.status(), StatusCode::UNAUTHORIZED);

    let create = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/projects")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!({ "name": "Nope" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create.status(), StatusCode::UNAUTHORIZED);

    let get = app
        .oneshot(
            Request::builder()
                .uri(format!("/projects/{}", uuid::Uuid::new_v4()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn get_other_users_project_is_not_found() {
    let pool = test_pool().await;
    migrate(&pool).await;
    let email_a = unique_email("owner");
    let email_b = unique_email("other");
    cleanup_email(&pool, &email_a).await;
    cleanup_email(&pool, &email_b).await;
    let app = app_with_pool(pool.clone());

    let cookie_a = register_and_cookie(&app, &email_a).await;
    let cookie_b = register_and_cookie(&app, &email_b).await;

    let create = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/projects")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, &cookie_a)
                .body(Body::from(json!({ "name": "Private" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create.status(), StatusCode::CREATED);
    let project_id = json_body(create).await["id"].as_str().unwrap().to_string();

    let get = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/projects/{project_id}"))
                .header(header::COOKIE, &cookie_b)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get.status(), StatusCode::NOT_FOUND);

    let list = app
        .oneshot(
            Request::builder()
                .uri("/projects")
                .header(header::COOKIE, &cookie_b)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list.status(), StatusCode::OK);
    let projects = json_body(list).await;
    let arr = projects.as_array().expect("list should be a JSON array");
    assert!(
        !arr.iter().any(|p| p["id"] == project_id),
        "other user's project must not appear in list"
    );

    cleanup_email(&pool, &email_a).await;
    cleanup_email(&pool, &email_b).await;
}

#[tokio::test]
async fn create_list_and_patch_task_status() {
    let pool = test_pool().await;
    migrate(&pool).await;
    let email = unique_email("task");
    cleanup_email(&pool, &email).await;
    let app = app_with_pool(pool.clone());
    let cookie = register_and_cookie(&app, &email).await;

    let create_project = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/projects")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, &cookie)
                .body(Body::from(json!({ "name": "Board" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_project.status(), StatusCode::CREATED);
    let project_id = json_body(create_project).await["id"]
        .as_str()
        .unwrap()
        .to_string();

    let create_task = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/projects/{project_id}/tasks"))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, &cookie)
                .body(Body::from(
                    json!({
                        "title": "Write tests",
                        "description": "Red then green"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_task.status(), StatusCode::CREATED);
    let created = json_body(create_task).await;
    assert_eq!(created["title"], "Write tests");
    assert_eq!(created["description"], "Red then green");
    assert_eq!(created["status"], "backlog");
    assert_eq!(created["project_id"], project_id);
    assert!(created["id"].is_string());
    let task_id = created["id"].as_str().unwrap().to_string();

    let list = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/projects/{project_id}/tasks"))
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list.status(), StatusCode::OK);
    let listed = json_body(list).await;
    let tasks = listed.as_array().expect("list should be a JSON array");
    assert!(
        tasks
            .iter()
            .any(|t| t["id"] == task_id && t["title"] == "Write tests" && t["status"] == "backlog"),
        "created task should appear in list"
    );

    let patch_doing = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri(format!("/tasks/{task_id}"))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, &cookie)
                .body(Body::from(json!({ "status": "doing" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(patch_doing.status(), StatusCode::OK);
    let doing = json_body(patch_doing).await;
    assert_eq!(doing["status"], "doing");
    assert_eq!(doing["title"], "Write tests");

    let patch_done = app
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri(format!("/tasks/{task_id}"))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, &cookie)
                .body(Body::from(json!({ "status": "done" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(patch_done.status(), StatusCode::OK);
    assert_eq!(json_body(patch_done).await["status"], "done");

    cleanup_email(&pool, &email).await;
}

#[tokio::test]
async fn tasks_require_auth() {
    let pool = test_pool().await;
    migrate(&pool).await;
    let app = app_with_pool(pool);
    let project_id = uuid::Uuid::new_v4();
    let task_id = uuid::Uuid::new_v4();

    let list = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/projects/{project_id}/tasks"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list.status(), StatusCode::UNAUTHORIZED);

    let create = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/projects/{project_id}/tasks"))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!({ "title": "Nope" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create.status(), StatusCode::UNAUTHORIZED);

    let patch = app
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri(format!("/tasks/{task_id}"))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!({ "status": "doing" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(patch.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn other_users_task_is_not_found() {
    let pool = test_pool().await;
    migrate(&pool).await;
    let email_a = unique_email("task-owner");
    let email_b = unique_email("task-other");
    cleanup_email(&pool, &email_a).await;
    cleanup_email(&pool, &email_b).await;
    let app = app_with_pool(pool.clone());

    let cookie_a = register_and_cookie(&app, &email_a).await;
    let cookie_b = register_and_cookie(&app, &email_b).await;

    let create_project = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/projects")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, &cookie_a)
                .body(Body::from(json!({ "name": "Private board" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_project.status(), StatusCode::CREATED);
    let project_id = json_body(create_project).await["id"]
        .as_str()
        .unwrap()
        .to_string();

    let create_task = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/projects/{project_id}/tasks"))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, &cookie_a)
                .body(Body::from(json!({ "title": "Secret" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_task.status(), StatusCode::CREATED);
    let task_id = json_body(create_task).await["id"]
        .as_str()
        .unwrap()
        .to_string();

    let list = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/projects/{project_id}/tasks"))
                .header(header::COOKIE, &cookie_b)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list.status(), StatusCode::NOT_FOUND);

    let create_on_foreign = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/projects/{project_id}/tasks"))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, &cookie_b)
                .body(Body::from(json!({ "title": "Hijack" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_on_foreign.status(), StatusCode::NOT_FOUND);

    let patch = app
        .oneshot(
            Request::builder()
                .method(Method::PATCH)
                .uri(format!("/tasks/{task_id}"))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, &cookie_b)
                .body(Body::from(json!({ "status": "done" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(patch.status(), StatusCode::NOT_FOUND);

    cleanup_email(&pool, &email_a).await;
    cleanup_email(&pool, &email_b).await;
}

#[tokio::test]
async fn today_requires_auth() {
    let pool = test_pool().await;
    migrate(&pool).await;
    let app = app_with_pool(pool);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/today")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn today_empty_when_no_matching_tasks() {
    let pool = test_pool().await;
    migrate(&pool).await;
    let email = unique_email("today-empty");
    cleanup_email(&pool, &email).await;
    let app = app_with_pool(pool.clone());
    let cookie = register_and_cookie(&app, &email).await;

    let create_project = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/projects")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, &cookie)
                .body(Body::from(json!({ "name": "Quiet" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_project.status(), StatusCode::CREATED);
    let project_id = json_body(create_project).await["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Backlog with no due date should not appear on Today.
    let create_task = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/projects/{project_id}/tasks"))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, &cookie)
                .body(Body::from(json!({ "title": "Later" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_task.status(), StatusCode::CREATED);

    let today = app
        .oneshot(
            Request::builder()
                .uri("/today")
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(today.status(), StatusCode::OK);
    let body = json_body(today).await;
    let tasks = body.as_array().expect("today should be a JSON array");
    assert!(tasks.is_empty(), "expected empty today list, got {tasks:?}");

    cleanup_email(&pool, &email).await;
}

#[tokio::test]
async fn today_includes_doing_and_due_tasks() {
    let pool = test_pool().await;
    migrate(&pool).await;
    let email = unique_email("today-match");
    cleanup_email(&pool, &email).await;
    let app = app_with_pool(pool.clone());
    let cookie = register_and_cookie(&app, &email).await;

    let create_project = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/projects")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, &cookie)
                .body(Body::from(json!({ "name": "Focus" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_project.status(), StatusCode::CREATED);
    let project_id = json_body(create_project).await["id"]
        .as_str()
        .unwrap()
        .to_string();

    let create_doing = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/projects/{project_id}/tasks"))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, &cookie)
                .body(Body::from(
                    json!({ "title": "In progress", "status": "doing" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_doing.status(), StatusCode::CREATED);
    let doing_id = json_body(create_doing).await["id"]
        .as_str()
        .unwrap()
        .to_string();

    let create_due = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/projects/{project_id}/tasks"))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, &cookie)
                .body(Body::from(
                    json!({
                        "title": "Due today",
                        "status": "backlog",
                        "due_on": "2020-01-01"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_due.status(), StatusCode::CREATED);
    let due_id = json_body(create_due).await["id"].as_str().unwrap().to_string();

    // Future due date, not doing — should not appear.
    let create_future = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/projects/{project_id}/tasks"))
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, &cookie)
                .body(Body::from(
                    json!({
                        "title": "Someday",
                        "status": "backlog",
                        "due_on": "2099-12-31"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_future.status(), StatusCode::CREATED);

    let today = app
        .oneshot(
            Request::builder()
                .uri("/today")
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(today.status(), StatusCode::OK);
    let body = json_body(today).await;
    let tasks = body.as_array().expect("today should be a JSON array");
    assert_eq!(tasks.len(), 2, "expected doing + due tasks, got {tasks:?}");

    let doing = tasks
        .iter()
        .find(|t| t["id"] == doing_id)
        .expect("doing task should appear");
    assert_eq!(doing["title"], "In progress");
    assert_eq!(doing["status"], "doing");
    assert_eq!(doing["project_id"], project_id);
    assert_eq!(doing["project_name"], "Focus");

    let due = tasks
        .iter()
        .find(|t| t["id"] == due_id)
        .expect("due task should appear");
    assert_eq!(due["title"], "Due today");
    assert_eq!(due["status"], "backlog");
    assert_eq!(due["due_on"], "2020-01-01");
    assert_eq!(due["project_name"], "Focus");

    cleanup_email(&pool, &email).await;
}

#[tokio::test]
async fn capture_requires_auth() {
    let pool = test_pool().await;
    migrate(&pool).await;
    let app = app_with_pool(pool);

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/capture")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!({ "title": "Nope" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn capture_with_project_id_creates_backlog_task() {
    let pool = test_pool().await;
    migrate(&pool).await;
    let email = unique_email("capture-proj");
    cleanup_email(&pool, &email).await;
    let app = app_with_pool(pool.clone());
    let cookie = register_and_cookie(&app, &email).await;

    let create_project = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/projects")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, &cookie)
                .body(Body::from(json!({ "name": "Work" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_project.status(), StatusCode::CREATED);
    let project_id = json_body(create_project).await["id"]
        .as_str()
        .unwrap()
        .to_string();

    let capture = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/capture")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, &cookie)
                .body(Body::from(
                    json!({
                        "title": "Call dentist",
                        "project_id": project_id
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(capture.status(), StatusCode::CREATED);
    let task = json_body(capture).await;
    assert_eq!(task["title"], "Call dentist");
    assert_eq!(task["status"], "backlog");
    assert_eq!(task["project_id"], project_id);
    assert!(task["id"].is_string());

    let list = app
        .oneshot(
            Request::builder()
                .uri(format!("/projects/{project_id}/tasks"))
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list.status(), StatusCode::OK);
    let tasks = json_body(list).await;
    let arr = tasks.as_array().expect("list should be a JSON array");
    assert!(
        arr.iter()
            .any(|t| t["id"] == task["id"] && t["status"] == "backlog"),
        "captured task should appear on the project board"
    );

    cleanup_email(&pool, &email).await;
}

#[tokio::test]
async fn capture_without_project_id_uses_or_creates_inbox() {
    let pool = test_pool().await;
    migrate(&pool).await;
    let email = unique_email("capture-inbox");
    cleanup_email(&pool, &email).await;
    let app = app_with_pool(pool.clone());
    let cookie = register_and_cookie(&app, &email).await;

    let first = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/capture")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, &cookie)
                .body(Body::from(json!({ "title": "Buy milk" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(first.status(), StatusCode::CREATED);
    let first_task = json_body(first).await;
    assert_eq!(first_task["title"], "Buy milk");
    assert_eq!(first_task["status"], "backlog");
    let inbox_id = first_task["project_id"].as_str().unwrap().to_string();

    let projects = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/projects")
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(projects.status(), StatusCode::OK);
    let listed = json_body(projects).await;
    let arr = listed.as_array().expect("projects list");
    let inbox = arr
        .iter()
        .find(|p| p["id"] == inbox_id)
        .expect("Inbox project should exist");
    assert_eq!(inbox["name"], "Inbox");

    let second = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/capture")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, &cookie)
                .body(Body::from(json!({ "title": "Pay rent" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(second.status(), StatusCode::CREATED);
    let second_task = json_body(second).await;
    assert_eq!(second_task["project_id"], inbox_id);
    assert_eq!(second_task["status"], "backlog");

    cleanup_email(&pool, &email).await;
}

#[tokio::test]
async fn capture_bad_project_id_returns_404() {
    let pool = test_pool().await;
    migrate(&pool).await;
    let email = unique_email("capture-404");
    cleanup_email(&pool, &email).await;
    let app = app_with_pool(pool.clone());
    let cookie = register_and_cookie(&app, &email).await;
    let missing = uuid::Uuid::new_v4();

    let capture = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/capture")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, &cookie)
                .body(Body::from(
                    json!({
                        "title": "Orphan",
                        "project_id": missing
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(capture.status(), StatusCode::NOT_FOUND);

    cleanup_email(&pool, &email).await;
}

#[tokio::test]
async fn expense_categories_require_auth() {
    let pool = test_pool().await;
    migrate(&pool).await;
    let app = app_with_pool(pool);

    let list = app
        .oneshot(
            Request::builder()
                .uri("/expense-categories")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn expenses_require_auth() {
    let pool = test_pool().await;
    migrate(&pool).await;
    let app = app_with_pool(pool);

    let list = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/expenses")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list.status(), StatusCode::UNAUTHORIZED);

    let create = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/expenses")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "amount_rupees": 10,
                        "category_id": uuid::Uuid::new_v4(),
                        "spent_on": "2024-09-15"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn expense_categories_seed_defaults_when_empty() {
    let pool = test_pool().await;
    migrate(&pool).await;
    let email = unique_email("exp-cat");
    cleanup_email(&pool, &email).await;
    let app = app_with_pool(pool.clone());
    let cookie = register_and_cookie(&app, &email).await;

    let list = app
        .oneshot(
            Request::builder()
                .uri("/expense-categories")
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list.status(), StatusCode::OK);
    let body = json_body(list).await;
    let cats = body.as_array().expect("categories array");
    let names: Vec<&str> = cats
        .iter()
        .filter_map(|c| c["name"].as_str())
        .collect();
    for expected in ["Food", "Transport", "Rent", "Utilities", "Other"] {
        assert!(
            names.contains(&expected),
            "missing default category {expected}: {names:?}"
        );
    }
    assert_eq!(cats.len(), 5);

    cleanup_email(&pool, &email).await;
}

#[tokio::test]
async fn create_and_list_expenses_for_month() {
    let pool = test_pool().await;
    migrate(&pool).await;
    let email = unique_email("exp-crud");
    cleanup_email(&pool, &email).await;
    let app = app_with_pool(pool.clone());
    let cookie = register_and_cookie(&app, &email).await;

    let cats = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/expense-categories")
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(cats.status(), StatusCode::OK);
    let cats_body = json_body(cats).await;
    let food_id = cats_body
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "Food")
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let create = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/expenses")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, &cookie)
                .body(Body::from(
                    json!({
                        "amount_rupees": 123.45,
                        "category_id": food_id,
                        "spent_on": "2024-09-15",
                        "note": "Lunch"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create.status(), StatusCode::CREATED);
    let created = json_body(create).await;
    assert_eq!(created["amount_paise"], 12345);
    assert_eq!(created["currency"], "INR");
    assert_eq!(created["spent_on"], "2024-09-15");
    assert_eq!(created["note"], "Lunch");
    assert_eq!(created["category_id"], food_id);
    let expense_id = created["id"].as_str().unwrap().to_string();

    let list_sep = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/expenses?year=2024&month=9")
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_sep.status(), StatusCode::OK);
    let listed = json_body(list_sep).await;
    let expenses = listed.as_array().expect("expenses array");
    assert!(
        expenses
            .iter()
            .any(|e| e["id"] == expense_id && e["amount_paise"] == 12345),
        "created expense should appear in September list"
    );

    let list_aug = app
        .oneshot(
            Request::builder()
                .uri("/expenses?year=2024&month=8")
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_aug.status(), StatusCode::OK);
    let august = json_body(list_aug).await;
    let august_list = august.as_array().unwrap();
    assert!(
        !august_list.iter().any(|e| e["id"] == expense_id),
        "September expense must not appear in August"
    );

    cleanup_email(&pool, &email).await;
}

#[tokio::test]
async fn create_expense_rejects_non_positive_amount() {
    let pool = test_pool().await;
    migrate(&pool).await;
    let email = unique_email("exp-val");
    cleanup_email(&pool, &email).await;
    let app = app_with_pool(pool.clone());
    let cookie = register_and_cookie(&app, &email).await;

    let cats = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/expense-categories")
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let food_id = json_body(cats).await.as_array().unwrap()[0]["id"]
        .as_str()
        .unwrap()
        .to_string();

    for amount in [json!(0), json!(-5), json!(0.0)] {
        let create = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/expenses")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header(header::COOKIE, &cookie)
                    .body(Body::from(
                        json!({
                            "amount_rupees": amount,
                            "category_id": food_id,
                            "spent_on": "2024-09-15"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            create.status(),
            StatusCode::BAD_REQUEST,
            "amount {amount} should be rejected"
        );
        let body = json_body(create).await;
        assert_eq!(body["error"]["code"], "validation_error");
    }

    cleanup_email(&pool, &email).await;
}

#[tokio::test]
async fn budgets_require_auth() {
    let pool = test_pool().await;
    migrate(&pool).await;
    let app = app_with_pool(pool);

    let summary = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/budgets/summary")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(summary.status(), StatusCode::UNAUTHORIZED);

    let upsert = app
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri("/budgets")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "year": 2024,
                        "month": 9,
                        "amount_rupees": 10000
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(upsert.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn upsert_budget_and_summary_math() {
    let pool = test_pool().await;
    migrate(&pool).await;
    let email = unique_email("budget");
    cleanup_email(&pool, &email).await;
    let app = app_with_pool(pool.clone());
    let cookie = register_and_cookie(&app, &email).await;

    let cats = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/expense-categories")
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(cats.status(), StatusCode::OK);
    let cats_body = json_body(cats).await;
    let food_id = cats_body
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "Food")
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    let transport_id = cats_body
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "Transport")
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Overall budget: ₹10,000 → 1_000_000 paise
    let overall = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri("/budgets")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, &cookie)
                .body(Body::from(
                    json!({
                        "year": 2024,
                        "month": 9,
                        "amount_rupees": 10000
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(overall.status(), StatusCode::OK);
    let overall_body = json_body(overall).await;
    assert_eq!(overall_body["amount_paise"], 1_000_000);
    assert!(overall_body["category_id"].is_null());
    assert_eq!(overall_body["year"], 2024);
    assert_eq!(overall_body["month"], 9);
    let overall_id = overall_body["id"].as_str().unwrap().to_string();

    // Upsert same overall key → updates amount, keeps id
    let overall_update = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri("/budgets")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, &cookie)
                .body(Body::from(
                    json!({
                        "year": 2024,
                        "month": 9,
                        "amount_rupees": 12000
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(overall_update.status(), StatusCode::OK);
    let overall_updated = json_body(overall_update).await;
    assert_eq!(overall_updated["id"], overall_id);
    assert_eq!(overall_updated["amount_paise"], 1_200_000);

    // Per-category Food budget: ₹2,000
    let food_budget = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri("/budgets")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::COOKIE, &cookie)
                .body(Body::from(
                    json!({
                        "year": 2024,
                        "month": 9,
                        "category_id": food_id,
                        "amount_rupees": 2000
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(food_budget.status(), StatusCode::OK);
    let food_budget_body = json_body(food_budget).await;
    assert_eq!(food_budget_body["amount_paise"], 200_000);
    assert_eq!(food_budget_body["category_id"], food_id);

    // Expenses: Food ₹500 + Transport ₹300 = ₹800 total
    for (cat_id, amount) in [(&food_id, 500.0), (&transport_id, 300.0)] {
        let create = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/expenses")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header(header::COOKIE, &cookie)
                    .body(Body::from(
                        json!({
                            "amount_rupees": amount,
                            "category_id": cat_id,
                            "spent_on": "2024-09-10"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(create.status(), StatusCode::CREATED);
    }

    let summary = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/budgets/summary?year=2024&month=9")
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(summary.status(), StatusCode::OK);
    let s = json_body(summary).await;
    assert_eq!(s["year"], 2024);
    assert_eq!(s["month"], 9);
    assert_eq!(s["overall_budget_paise"], 1_200_000);
    assert_eq!(s["total_spent_paise"], 80_000);
    assert_eq!(s["remaining_paise"], 1_120_000);

    let by_cat = s["by_category"].as_array().expect("by_category array");
    let food = by_cat
        .iter()
        .find(|c| c["category_id"] == food_id)
        .expect("Food row");
    assert_eq!(food["category_name"], "Food");
    assert_eq!(food["budget_paise"], 200_000);
    assert_eq!(food["spent_paise"], 50_000);
    assert_eq!(food["remaining_paise"], 150_000);

    let transport = by_cat
        .iter()
        .find(|c| c["category_id"] == transport_id)
        .expect("Transport row");
    assert_eq!(transport["spent_paise"], 30_000);
    assert!(transport["budget_paise"].is_null());
    assert!(transport["remaining_paise"].is_null());

    // Empty month: spend 0, no budgets set for August
    let empty = app
        .oneshot(
            Request::builder()
                .uri("/budgets/summary?year=2024&month=8")
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(empty.status(), StatusCode::OK);
    let empty_body = json_body(empty).await;
    assert_eq!(empty_body["total_spent_paise"], 0);
    assert!(empty_body["overall_budget_paise"].is_null());
    assert!(empty_body["remaining_paise"].is_null());
    assert_eq!(empty_body["by_category"].as_array().unwrap().len(), 0);

    cleanup_email(&pool, &email).await;
}

