CREATE TABLE folders (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id),
    path VARCHAR(4096) NOT NULL,
    parent_id INTEGER REFERENCES folders(id),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    UNIQUE(user_id, path)
);

CREATE INDEX idx_folders_user_path ON folders(user_id, path);
CREATE INDEX idx_folders_parent ON folders(parent_id);
CREATE INDEX idx_folders_user_deleted ON folders(user_id, is_deleted);