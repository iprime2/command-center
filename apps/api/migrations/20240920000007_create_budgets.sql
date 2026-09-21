-- Monthly budgets (INR stored as integer paise). Soft-delete; sync-ready.
-- category_id NULL = overall monthly budget for the user.
-- Unique active (user_id, category_id, year, month): PostgreSQL UNIQUE treats
-- NULLs as distinct, so we use two partial unique indexes (not a single
-- UNIQUE including nullable category_id).

CREATE TABLE IF NOT EXISTS budgets (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users (id),
    category_id UUID NULL REFERENCES expense_categories (id),
    year INT NOT NULL,
    month INT NOT NULL CHECK (month >= 1 AND month <= 12),
    amount_paise BIGINT NOT NULL CHECK (amount_paise > 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ NULL
);

-- Overall budgets (category_id IS NULL)
CREATE UNIQUE INDEX IF NOT EXISTS budgets_user_overall_month_active_uidx
    ON budgets (user_id, year, month)
    WHERE deleted_at IS NULL AND category_id IS NULL;

-- Per-category budgets
CREATE UNIQUE INDEX IF NOT EXISTS budgets_user_category_month_active_uidx
    ON budgets (user_id, category_id, year, month)
    WHERE deleted_at IS NULL AND category_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS budgets_user_year_month_active_idx
    ON budgets (user_id, year, month)
    WHERE deleted_at IS NULL;
