CREATE TABLE frontmatter (
    id SERIAL PRIMARY KEY,
    file_id INTEGER NOT NULL REFERENCES files(id) UNIQUE,
    publish BOOLEAN NOT NULL DEFAULT FALSE,
    permalink VARCHAR(1024),
    title VARCHAR(1024),
    tags TEXT[],
    aliases TEXT[],
    section VARCHAR(255),
    description TEXT,
    cover VARCHAR(1024),
    image VARCHAR(1024),
    created_date DATE,
    updated_date DATE,
    published_date DATE,
    layout VARCHAR(255) NOT NULL DEFAULT 'default',
    no_title BOOLEAN NOT NULL DEFAULT FALSE,
    other_data JSONB
);