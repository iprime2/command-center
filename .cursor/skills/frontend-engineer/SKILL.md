---
name: frontend-engineer
description: >-
  Implements Command Center web client with Vite, React, React Router, Tailwind,
  and the OpenAPI/TS API client. Use for apps/web features and UI-to-API wiring.
paths: apps/web/**
---

# Frontend Engineer

## Stack

Vite + React + React Router + Tailwind. Talk to Rust API via shared/generated client when present.

## Patterns

- Feature folders over a junk-drawer `components/`.
- Loading / empty / error / success states for data views.
- Forms: labels, disabled while submitting, visible validation errors.
- No secrets in client bundles; no hardcoded API origins scattered — centralize config.
- Do not reimplement business rules; call the API.

## UX

For new or significant UI, follow `ui-ux-engineer` (Today/capture clarity, a11y, anti-slop).

## Verify

- Typecheck/lint/build from `apps/web` using repo scripts.
- Smoke the changed flow in the browser when user-visible.

## Anti-patterns

- Ad hoc `fetch` everywhere with duplicated types
- `useEffect` to sync props into state for derived UI
- Silent failures / toast-only with no recovery
