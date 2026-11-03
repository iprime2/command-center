---
name: feature-adapt
description: >-
  Turn a feature-recon brief into one Command Center vertical slice on the
  locked stack (Rust Axum SQLx, Vite React, INR paise, soft delete). Use after
  feature-recon, or when adapting another app's flow into this API and web
  client.
---

# Feature adapt

Read `docs/replications/<slug>/recon.md` and `features.csv`. If they are missing, run `feature-recon` first. Do not plan from memory of the other app.

This skill writes the slice plan. Implementation uses the existing skills (`lead-engineer`, then `tdd-workflow` with `rust-backend-engineer` and `frontend-engineer`).

## Locked choices

Use the stack already in this repo. Do not propose a second backend or a replacement architecture.

| Need | Use |
|------|-----|
| Behavior | Rust API only |
| HTTP | Axum + OpenAPI (`utoipa`) |
| Data | SQLx, parameterized queries, Postgres |
| Web | Vite + React + React Router + Tailwind, thin client |
| Identity | Existing email/password session cookie `cc_session` |
| Rows | UUID `id`, `created_at`, `updated_at`, `deleted_at` |
| Money | Integer paise, INR |
| Access | API authz by owner. Do not turn on Supabase RLS for this app |

Refuse Next.js, Prisma, Drizzle, Supabase Auth, Stripe, or Vercel as the plan for this slice. Supabase, when used, is Postgres behind `DATABASE_URL` only.

## Plan

Write `docs/replications/<slug>/slice.md`:

1. **Vertical slice.** The core loop end to end: one migration, the routes that flow needs, one web path. Must-haves only.
2. **Schema.** Tables and columns mapped from the recon entities. Soft delete included. Money columns are paise. State the owner column and the authz rule.
3. **API.** For each route: method, path, who can call it, input, output, which flow. Match `api-contract-design`.
4. **UI.** Routes, empty/loading/error/filled states, and which API fields the screen reads. No parallel DTO layer.
5. **Tests.** The failing test that proves the slice, then the browser flow.
6. **Later.** Should/could rows stay in `features.csv`. They are not this slice.

Risky behavior from the recon (races, time zones, double submit) becomes an acceptance criterion or an explicit non-goal.

## Exit

Show the slice in chat: tables, routes, web path, and the first test. Wait for the user to confirm before any implementation skill edits `apps/`.
