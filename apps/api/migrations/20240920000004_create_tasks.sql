-- Project tasks (soft-delete; board columns via status).
CREATE TABLE IF NOT EXISTS tasks (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL REFERENCES projects (id),
    user_id UUID NOT NULL REFERENCES users (id),
    title TEXT NOT NULL,
    description TEXT NULL,
    status TEXT NOT NULL DEFAULT 'backlog'
        CHECK (status IN ('backlog', 'doing', 'done')),
    due_on DATE NULL,
    position INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ NULL
);

CREATE INDEX IF NOT EXISTS tasks_project_id_active_idx
    ON tasks (project_id)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS tasks_user_id_active_idx
    ON tasks (user_id)
    WHERE deleted_at IS NULL;
