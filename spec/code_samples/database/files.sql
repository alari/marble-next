CREATE TABLE files (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id),
    path VARCHAR(4096) NOT NULL,
    content_hash VARCHAR(255) NOT NULL,
    content_type VARCHAR(255) NOT NULL,
    size BIGINT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    last_processed TIMESTAMP WITH TIME ZONE,
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    UNIQUE(user_id, path)
);

CREATE INDEX idx_files_user_path ON files(user_id, path);
CREATE INDEX idx_files_content_hash ON files(content_hash);
CREATE INDEX idx_files_user_deleted ON files(user_id, is_deleted);