-- Create files table
-- Tracks file metadata and content location

CREATE TABLE files (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id),
    path VARCHAR(1024) NOT NULL,
    content_hash VARCHAR(64) NOT NULL,
    content_type VARCHAR(255) NOT NULL,
    size INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    UNIQUE(user_id, path)
);

-- Create indexes for common queries
CREATE INDEX idx_files_user_path ON files(user_id, path);
CREATE INDEX idx_files_content_hash ON files(content_hash);
CREATE INDEX idx_files_user_deleted ON files(user_id, is_deleted);
