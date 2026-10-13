# Command Center

Personal productivity OS (projects, expenses, fitness-lite, AI workflows). **Milestone A** is implemented: auth, projects/boards/tasks, Today, capture, INR expenses + budgets.

## Stack

| Layer | Tech |
|-------|------|
| API | Rust, Axum, SQLx, PostgreSQL, OpenAPI (utoipa) |
| Web | Vite, React, React Router, Tailwind |
| Auth | Email/password, **httpOnly cookie** `cc_session` + `sessions` table |
| CI | GitHub Actions — API tests, web build/lint, secret scan, `cargo audit`, `pnpm audit` |

Business rules live in the Rust API. The web app is a thin client.

## Prerequisites

- Rust (cargo), Node 22+, pnpm
- PostgreSQL reachable at `DATABASE_URL` — **Supabase (remote)** or **Docker Compose (local)**; see [Database](#database)
- A C compiler for crate build scripts. If system `gcc` is missing, use Zig wrappers already in `.tools/bin` (see below).

## Environment

```bash
cp .env.example .env
# Set DATABASE_URL — Supabase URI (remote) or local Docker URL (see Database)
```

Put `.env` at the **repo root**. The API loads it when you run `cargo run` from `apps/api`.

`.env` is gitignored. Never commit secrets.

## Database

The API only cares about **`DATABASE_URL`**. Migrations run automatically on API startup.

### Option A — Supabase (remote, free tier)

1. Create a project (e.g. region **South Asia (Mumbai)**).
2. **Project Settings → Database → Connection string → URI**
3. Prefer **Session pooler** or **Direct connection** (works best with SQLx migrations).
4. Append `?sslmode=require` if it is not already in the URI.
5. **Percent-encode** special characters in the password (`@`, `#`, `%`, etc.).
6. Paste into `.env`:

   `DATABASE_URL=postgres://postgres.[ref]:...@....pooler.supabase.com:5432/postgres?sslmode=require`

You do **not** need Supabase Data API for this app — the Rust API talks to Postgres directly. Leaving Data API enabled is fine.

### Option B — Docker Compose (local container)

Requires Docker (Docker Desktop with WSL integration on Windows).

```bash
docker compose up -d
```

In `.env`:

`DATABASE_URL=postgres://cc:cc@127.0.0.1:5432/command_center`

Optional overrides: `CC_POSTGRES_USER`, `CC_POSTGRES_PASSWORD`, `CC_POSTGRES_DB`, `CC_POSTGRES_PORT` (see `.env.example`).

Stop: `docker compose down` · Wipe data: `docker compose down -v`

### C compiler (local builds)

If system `gcc` is missing, Zig wrappers in `.tools/bin` may be used in this workspace:

`export PATH="$PWD/.tools/bin:$HOME/.cargo/bin:$PATH"`

### Auth model

- `POST /auth/register` and `POST /auth/login` set an **httpOnly** cookie named `cc_session` (opaque token; DB stores SHA-256 hash).
- `POST /auth/logout` soft-deletes the session and clears the cookie.
- `GET /auth/me` requires a valid session cookie (401 otherwise).
- Web calls use `credentials: 'include'`. CORS allows `WEB_ORIGIN` (default `http://127.0.0.1:5173`) with credentials.

## Run API

```bash
cd apps/api
cargo run
# listens on http://127.0.0.1:8080
# OpenAPI UI: http://127.0.0.1:8080/swagger-ui/
```

Migrations run automatically on startup.

```bash
cargo test   # needs DATABASE_URL
cargo build
```

## Run web

```bash
cd apps/web
cp .env.example .env   # optional; VITE_API_URL defaults to http://127.0.0.1:8080
pnpm install
pnpm dev               # http://127.0.0.1:5173
pnpm build
```

## CI

Pull requests and pushes to `dev` / `main` run `.github/workflows/ci.yml`:

- **API tests** — `cargo test` against Postgres 16 service
- **Web** — `pnpm lint`, `pnpm build`
- **Security** — gitleaks secret scan, `cargo audit`, `pnpm audit --audit-level=high`

## Repo layout

```
apps/api/              Rust Axum API
apps/web/              Vite React client
.github/workflows/     CI pipeline
.cursor/               Agent skills + rules
.env.example           Sample env (no secrets)
docker-compose.yml
```

## Branching

- `dev` — integration branch
- Feature work via PRs into `dev` (CI must pass)
- `main` — reserved for stable releases later
