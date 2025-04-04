CREATE TABLE document_links (
    id SERIAL PRIMARY KEY,
    source_file_id INTEGER NOT NULL REFERENCES files(id),
    target_name VARCHAR(1024) NOT NULL,
    display_text VARCHAR(1024),
    is_embed BOOLEAN NOT NULL,
    target_file_id INTEGER REFERENCES files(id),
    position INTEGER NOT NULL,
    fragment VARCHAR(255)
);

CREATE INDEX idx_links_source ON document_links(source_file_id, position);
CREATE INDEX idx_links_target ON document_links(target_file_id);
CREATE INDEX idx_links_unresolved ON document_links(target_name) WHERE target_file_id IS NULL;