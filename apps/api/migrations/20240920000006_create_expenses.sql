-- Expense categories and expenses (INR stored as integer paise). Soft-delete; sync-ready.

CREATE TABLE IF NOT EXISTS expense_categories (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users (id),
    name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ NULL
);

CREATE INDEX IF NOT EXISTS expense_categories_user_id_active_idx
    ON expense_categories (user_id)
    WHERE deleted_at IS NULL;

CREATE TABLE IF NOT EXISTS expenses (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users (id),
    category_id UUID NOT NULL REFERENCES expense_categories (id),
    amount_paise BIGINT NOT NULL CHECK (amount_paise > 0),
    currency TEXT NOT NULL DEFAULT 'INR',
    spent_on DATE NOT NULL,
    note TEXT NULL,
    project_id UUID NULL REFERENCES projects (id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ NULL
);

CREATE INDEX IF NOT EXISTS expenses_user_id_spent_on_active_idx
    ON expenses (user_id, spent_on)
    WHERE deleted_at IS NULL;
