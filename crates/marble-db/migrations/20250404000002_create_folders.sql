-- Create folders table
-- Tracks folder structure for each user

CREATE TABLE folders (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id),
    path VARCHAR(1024) NOT NULL,
    parent_id INTEGER REFERENCES folders(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    UNIQUE(user_id, path)
);

-- Create indexes for common queries
CREATE INDEX idx_folders_user_path ON folders(user_id, path);
CREATE INDEX idx_folders_parent ON folders(parent_id);
CREATE INDEX idx_folders_user_deleted ON folders(user_id, is_deleted);
