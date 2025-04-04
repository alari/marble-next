CREATE TABLE file_versions (
    id SERIAL PRIMARY KEY,
    file_id INTEGER NOT NULL REFERENCES files(id),
    content_hash VARCHAR(255) NOT NULL,
    version_number INTEGER NOT NULL,
    size BIGINT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    comment TEXT,
    UNIQUE(file_id, version_number)
);