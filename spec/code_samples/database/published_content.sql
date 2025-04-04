CREATE TABLE published_content (
    id SERIAL PRIMARY KEY,
    file_id INTEGER NOT NULL REFERENCES files(id),
    user_id INTEGER NOT NULL REFERENCES users(id),
    permalink VARCHAR(1024) NOT NULL,
    processed_hash VARCHAR(255) NOT NULL,
    published_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    invalidated BOOLEAN NOT NULL DEFAULT FALSE,
    UNIQUE(user_id, permalink)
);

CREATE INDEX idx_published_user ON published_content(user_id);
CREATE INDEX idx_published_file ON published_content(file_id);