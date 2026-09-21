-- At most one active Inbox project per user (soft-delete aware).
CREATE UNIQUE INDEX IF NOT EXISTS projects_user_inbox_active_uidx
    ON projects (user_id)
    WHERE deleted_at IS NULL AND name = 'Inbox';
