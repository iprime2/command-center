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
- PostgreSQL reachable at `DATABASE_URL` (local micromamba Postgres on `127.0.0.1:54329` is fine; or `docker compose up -d` for Docker Postgres on `5432`)
- A C compiler for crate build scripts. If system `gcc` is missing, use Zig wrappers already in `.tools/bin` (see below).

## Environment

```bash
cp .env.example .env
# edit DATABASE_URL / ports as needed

# Shell setup used in this workspace:
export PATH="$PWD/.tools/bin:$HOME/micromamba-root/envs/cc-pg/bin:$HOME/.cargo/bin:$HOME/.local/bin:$PATH"
export DATABASE_URL="postgres://cc@127.0.0.1:54329/command_center"
```

`.env` is gitignored. Never commit secrets.

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

## Optional Docker Postgres

```bash
docker compose up -d
# then set DATABASE_URL=postgres://cc:cc@127.0.0.1:5432/command_center
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
