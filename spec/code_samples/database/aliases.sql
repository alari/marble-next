CREATE TABLE aliases (
    id SERIAL PRIMARY KEY,
    file_id INTEGER NOT NULL REFERENCES files(id),
    alias VARCHAR(1024) NOT NULL,
    is_primary BOOLEAN NOT NULL DEFAULT FALSE,
    UNIQUE(file_id, alias)
);

CREATE INDEX idx_aliases_lookup ON aliases(alias);