---
name: api-contract-design
description: >-
  OpenAPI-first HTTP contract design for Command Center Rust API: versioning,
  errors, validation, resource shapes. Use when adding or changing endpoints or
  shared client types.
---

# API Contract Design

## Principles

- **Contract first:** define request/response shapes; implement Axum handlers to match; regenerate TS client when the pipeline exists.
- Every observable response field may be depended on later — don’t casually rename/remove (Hyrum’s Law).
- One clear versioning story (URL or document version); don’t silently break clients.
- Consistent error body (code, message, fields) across modules.
- List endpoints: pagination/limits; never unbounded dumps by default.
- Auth: document which routes are public vs protected.

## Checklist for a new endpoint

- [ ] Path + method named for the resource
- [ ] Input validation rules documented and enforced
- [ ] Success + error responses specified
- [ ] OpenAPI updated
- [ ] Ownership/authz rule stated
- [ ] Soft-delete / sync fields considered if entity is user data

## Anti-patterns

- Duplicating divergent types in web and API with no OpenAPI source of truth
- Returning raw DB rows with internal columns
- Breaking JSON shapes without a migration path for clients
