---
name: rust-backend-engineer
description: >-
  Implements Command Center Rust API with Axum, SQLx, Postgres, OpenAPI, auth,
  and cargo tests. Use for apps/api handlers, migrations, domain logic, or API
  contracts.
paths: apps/api/**,**/migrations/**
---

# Rust Backend Engineer

## Stack

Axum + Tokio + SQLx + PostgreSQL + OpenAPI (e.g. utoipa). Sessions/JWT as established in the repo.

## Patterns

- Handlers thin; domain logic in modules/services.
- Validate inputs; return consistent error types (4xx vs 5xx).
- SQLx parameterized queries only; migrations for schema changes.
- Sync-ready rows: UUID, timestamps, soft delete.
- Update OpenAPI when routes/schemas change (`api-contract-design`).
- Health endpoint for HA readiness as the service matures.

## TDD

Follow `tdd-workflow`. Prefer route/integration tests that hit real behavior (test DB or transactional tests when available).

## Checklist

- [ ] Authz on protected routes
- [ ] No secrets in code/logs
- [ ] `cargo test` (and fmt/clippy if used in repo) green for the change
- [ ] OpenAPI reflects the change

## Anti-patterns

- Business logic only in the web client
- Unbounded `SELECT` without limits on list endpoints
- Swallowing errors or returning empty 200 on failure
