# Command Center — Agent North Star

Personal productivity OS (projects, expenses, fitness-lite, AI workflows). Personal use first; SaaS later. FitSage stays a separate app.

## Stack (locked)

- **API:** Rust + Axum + SQLx + PostgreSQL + OpenAPI
- **Web:** Vite + React + React Router + Tailwind (thin client)
- **Desktop later:** Electron
- **Mobile later:** Expo
- **Contracts:** OpenAPI is the bridge; business logic lives in the Rust API only

## Non-negotiables

1. Surface assumptions before building; wrong silent assumptions are the top failure mode.
2. Stop and ask when requirements conflict — do not guess.
3. Prefer the boring, obvious solution; cleverness is expensive.
4. Touch only what you were asked to touch.
5. **Evidence before done** — tests, build, and UI flow proof. “Seems right” never closes.

## How to work here

- Read `.cursor/rules/` (always-on) and load skills from `.cursor/skills/` via `using-cc-skills`.
- Non-trivial work: Lead Engineer plans vertical slices → Senior/Rust/Frontend implement → review → independent QA.
- Git: never pull/push/commit unless the user explicitly asks in the conversation.

## Verify (discover from repo once bootstrap exists)

Prefer repo scripts/README. Typical targets once present:

- API: `cargo test` (and health endpoint)
- Web: lint/typecheck/build from `apps/web`
- UI flows: exercise in browser, not a single screenshot
