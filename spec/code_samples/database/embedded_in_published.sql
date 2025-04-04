CREATE TABLE embedded_in_published (
    id SERIAL PRIMARY KEY,
    published_content_id INTEGER NOT NULL REFERENCES published_content(id),
    embedded_file_id INTEGER NOT NULL REFERENCES files(id),
    fragment_id VARCHAR(255) NOT NULL,
    UNIQUE(published_content_id, embedded_file_id, fragment_id)
);

CREATE INDEX idx_embedded_published ON embedded_in_published(published_content_id);
CREATE INDEX idx_embedded_file ON embedded_in_published(embedded_file_id);