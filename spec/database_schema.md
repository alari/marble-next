# Marble Database Schema Specification

## Overview

This document describes the PostgreSQL database schema for Marble. The database serves as the metadata layer for the system, tracking file paths, content hashes, relationships, and processing status.

## Design Principles

1. **Clear Separation of Concerns**:
   - User data is isolated by tenant
   - Content metadata is separate from relationship data
   - Processing state is tracked independently

2. **Efficient Querying**:
   - Optimized for common operations (path lookup, reference finding)
   - Appropriate indexes for performance
   - Denormalization where beneficial for query patterns

3. **Versioning Support**:
   - Track history of changes
   - Support potential "time machine" features
   - Enable efficient content updates

4. **Incremental Processing**:
   - Track changes for minimal reprocessing
   - Maintain dependency graph for impact analysis
   - Queue management for processing operations

## Schema Design

### Core Tables

#### `users`
Stores user authentication information.

| Column          | Type           | Description                                |
|-----------------|----------------|--------------------------------------------|
| id              | SERIAL         | Primary key                                |
| username        | VARCHAR(255)   | Unique username                            |
| password_hash   | VARCHAR(255)   | Securely stored password hash              |
| created_at      | TIMESTAMP      | When the user was created                  |
| updated_at      | TIMESTAMP      | When the user was last updated             |
| last_login      | TIMESTAMP      | When the user last logged in               |
| active          | BOOLEAN        | Whether the user is active                 |

```sql
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    last_login TIMESTAMP WITH TIME ZONE,
    active BOOLEAN NOT NULL DEFAULT TRUE
);
```

#### `files`
Tracks current state of each file.

| Column          | Type           | Description                                |
|-----------------|----------------|--------------------------------------------|
| id              | SERIAL         | Primary key                                |
| user_id         | INTEGER        | Foreign key to users                       |
| path            | VARCHAR(4096)  | File path in the vault                     |
| content_hash    | VARCHAR(255)   | Current content hash (links to S3)         |
| content_type    | VARCHAR(255)   | MIME type or file format                   |
| size            | BIGINT         | File size in bytes                         |
| created_at      | TIMESTAMP      | When the file was first created            |
| updated_at      | TIMESTAMP      | When the file was last updated             |
| last_processed  | TIMESTAMP      | When the file was last processed           |
| is_deleted      | BOOLEAN        | Tombstone flag                             |

```sql
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
```

#### `file_versions`
Historical record of file changes.

| Column          | Type           | Description                                |
|-----------------|----------------|--------------------------------------------|
| id              | SERIAL         | Primary key                                |
| file_id         | INTEGER        | Foreign key to files                       |
| content_hash    | VARCHAR(255)   | Content hash for this version              |
| version_number  | INTEGER        | Sequential version number                  |
| size            | BIGINT         | File size in bytes                         |
| created_at      | TIMESTAMP      | When this version was created              |
| comment         | TEXT           | Optional comment/metadata                  |

```sql
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
```

#### `folders`
Tracks folder structure.

| Column          | Type           | Description                                |
|-----------------|----------------|--------------------------------------------|
| id              | SERIAL         | Primary key                                |
| user_id         | INTEGER        | Foreign key to users                       |
| path            | VARCHAR(4096)  | Folder path                                |
| parent_id       | INTEGER        | Foreign key to parent folder               |
| created_at      | TIMESTAMP      | When the folder was created                |
| updated_at      | TIMESTAMP      | When the folder was last updated           |
| is_deleted      | BOOLEAN        | Tombstone flag                             |

```sql
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
```

### Content Analysis Tables

#### `frontmatter`
Extracted frontmatter data.

| Column          | Type           | Description                                |
|-----------------|----------------|--------------------------------------------|
| id              | SERIAL         | Primary key                                |
| file_id         | INTEGER        | Foreign key to files                       |
| publish         | BOOLEAN        | Whether content should be published        |
| permalink       | VARCHAR(1024)  | Custom URL path for published content      |
| title           | VARCHAR(1024)  | Content title                              |
| tags            | TEXT[]         | Array of tags                              |
| aliases         | TEXT[]         | Array of alternative names                 |
| section         | VARCHAR(255)   | Section information                        |
| description     | TEXT           | Content description                        |
| cover           | VARCHAR(1024)  | Cover image reference                      |
| image           | VARCHAR(1024)  | Image URL                                  |
| created_date    | DATE           | Creation date from frontmatter             |
| updated_date    | DATE           | Update date from frontmatter               |
| published_date  | DATE           | Publication date from frontmatter          |
| layout          | VARCHAR(255)   | Layout type                                |
| no_title        | BOOLEAN        | Whether to hide title                      |
| other_data      | JSONB          | Additional frontmatter fields              |

```sql
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
```

#### `aliases`
Normalized list of aliases for efficient lookup.

| Column          | Type           | Description                                |
|-----------------|----------------|--------------------------------------------|
| id              | SERIAL         | Primary key                                |
| file_id         | INTEGER        | Foreign key to files                       |
| alias           | VARCHAR(1024)  | Alias value                                |
| is_primary      | BOOLEAN        | Whether this is the primary name           |

```sql
CREATE TABLE aliases (
    id SERIAL PRIMARY KEY,
    file_id INTEGER NOT NULL REFERENCES files(id),
    alias VARCHAR(1024) NOT NULL,
    is_primary BOOLEAN NOT NULL DEFAULT FALSE,
    UNIQUE(file_id, alias)
);

CREATE INDEX idx_aliases_lookup ON aliases(alias);
```

#### `references`
Links between files (Obsidian references).

| Column          | Type           | Description                                |
|-----------------|----------------|--------------------------------------------|
| id              | SERIAL         | Primary key                                |
| source_file_id  | INTEGER        | Foreign key to source file                 |
| target_path     | VARCHAR(1024)  | Referenced path                            |
| display_text    | VARCHAR(1024)  | Text displayed for the reference           |
| original_syntax | VARCHAR(1024)  | Original Obsidian reference syntax         |
| target_file_id  | INTEGER        | Foreign key to target file (if resolved)   |
| resolved        | BOOLEAN        | Whether the reference has been resolved    |

```sql
CREATE TABLE references (
    id SERIAL PRIMARY KEY,
    source_file_id INTEGER NOT NULL REFERENCES files(id),
    target_path VARCHAR(1024) NOT NULL,
    display_text VARCHAR(1024),
    original_syntax VARCHAR(1024) NOT NULL,
    target_file_id INTEGER REFERENCES files(id),
    resolved BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE INDEX idx_references_source ON references(source_file_id);
CREATE INDEX idx_references_target ON references(target_file_id);
```

#### `embeds`
Embeds between files (Obsidian embeds).

| Column          | Type           | Description                                |
|-----------------|----------------|--------------------------------------------|
| id              | SERIAL         | Primary key                                |
| source_file_id  | INTEGER        | Foreign key to source file                 |
| target_path     | VARCHAR(1024)  | Embedded path                              |
| fragment        | VARCHAR(255)   | Section fragment if any                    |
| original_syntax | VARCHAR(1024)  | Original Obsidian embed syntax             |
| target_file_id  | INTEGER        | Foreign key to target file (if resolved)   |
| resolved        | BOOLEAN        | Whether the embed has been resolved        |

```sql
CREATE TABLE embeds (
    id SERIAL PRIMARY KEY,
    source_file_id INTEGER NOT NULL REFERENCES files(id),
    target_path VARCHAR(1024) NOT NULL,
    fragment VARCHAR(255),
    original_syntax VARCHAR(1024) NOT NULL,
    target_file_id INTEGER REFERENCES files(id),
    resolved BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE INDEX idx_embeds_source ON embeds(source_file_id);
CREATE INDEX idx_embeds_target ON embeds(target_file_id);
```

### Processing Tables

#### `processing_queue`
Tracks files needing processing.

| Column          | Type           | Description                                |
|-----------------|----------------|--------------------------------------------|
| id              | SERIAL         | Primary key                                |
| file_id         | INTEGER        | Foreign key to files                       |
| operation       | VARCHAR(50)    | Type of change (create, update, delete)    |
| enqueued_at     | TIMESTAMP      | When it was added to queue                 |
| priority        | INTEGER        | Processing priority                        |
| status          | VARCHAR(50)    | Current status (pending, processing, etc.) |
| last_attempt    | TIMESTAMP      | Timestamp of last processing attempt       |
| attempts        | INTEGER        | Number of processing attempts              |
| error           | TEXT           | Error message if processing failed         |

```sql
CREATE TABLE processing_queue (
    id SERIAL PRIMARY KEY,
    file_id INTEGER NOT NULL REFERENCES files(id),
    operation VARCHAR(50) NOT NULL,
    enqueued_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    priority INTEGER NOT NULL DEFAULT 0,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    last_attempt TIMESTAMP WITH TIME ZONE,
    attempts INTEGER NOT NULL DEFAULT 0,
    error TEXT
);

CREATE INDEX idx_queue_status ON processing_queue(status, priority, enqueued_at);
```

#### `published_content`
Tracks what content is published.

| Column          | Type           | Description                                |
|-----------------|----------------|--------------------------------------------|
| id              | SERIAL         | Primary key                                |
| file_id         | INTEGER        | Foreign key to source file                 |
| user_id         | INTEGER        | Foreign key to users                       |
| permalink       | VARCHAR(1024)  | Published path                             |
| processed_hash  | VARCHAR(255)   | Hash of processed content                  |
| published_at    | TIMESTAMP      | When it was published                      |
| invalidated     | BOOLEAN        | Whether it needs reprocessing              |

```sql
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
```

#### `embedded_in_published`
Tracks which published content embeds which files.

| Column                 | Type           | Description                                |
|------------------------|----------------|--------------------------------------------|
| id                     | SERIAL         | Primary key                                |
| published_content_id   | INTEGER        | Foreign key to published content           |
| embedded_file_id       | INTEGER        | Foreign key to embedded file               |
| fragment_id            | VARCHAR(255)   | Fragment identifier in the published content|

```sql
CREATE TABLE embedded_in_published (
    id SERIAL PRIMARY KEY,
    published_content_id INTEGER NOT NULL REFERENCES published_content(id),
    embedded_file_id INTEGER NOT NULL REFERENCES files(id),
    fragment_id VARCHAR(255) NOT NULL,
    UNIQUE(published_content_id, embedded_file_id, fragment_id)
);

CREATE INDEX idx_embedded_published ON embedded_in_published(published_content_id);
CREATE INDEX idx_embedded_file ON embedded_in_published(embedded_file_id);
```

#### `cache_invalidations`
Tracks what needs to be reprocessed.

| Column          | Type           | Description                                |
|-----------------|----------------|--------------------------------------------|
| id              | SERIAL         | Primary key                                |
| user_id         | INTEGER        | Foreign key to users                       |
| path_pattern    | VARCHAR(4096)  | Path pattern to invalidate                 |
| created_at      | TIMESTAMP      | When invalidation was created              |
| processed       | BOOLEAN        | Whether it has been processed              |

```sql
CREATE TABLE cache_invalidations (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id),
    path_pattern VARCHAR(4096) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    processed BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE INDEX idx_invalidations_pending ON cache_invalidations(processed, created_at);
```

## Indexes and Performance

Key indexes have been included in the table creation statements above. These are designed to optimize the most common query patterns:

1. **Path Lookup**: Finding files by path within a user's vault
2. **Reference Resolution**: Finding files that match a reference or alias
3. **Dependency Tracking**: Finding files that reference or embed a specific file
4. **Processing Queue**: Finding pending items for processing

Additional indexes may be needed based on observed query patterns during development.

## Migrations

Database migrations will be managed using SQLx migrations. Initial schema creation will be the first migration, with subsequent changes applied through additional migrations.

## Questions and Considerations

1. **Permalink Generation**: Should we enforce unique permalinks per user, or allow duplicates with a resolution strategy?

2. **Frontmatter Storage**: Is the current structure sufficient, or should we normalize some of the frontmatter fields into separate tables?

3. **Content Hashing**: How should we handle conflicts if two different files produce the same content hash?

4. **Versioning Strategy**: How many versions should we keep per file? Should there be a cleanup/archival process?

5. **Processing Queue**: Should we implement a separate table for processing results, or keep everything in the queue table?

6. **Performance**: Are there any specific query patterns that might need denormalization or additional indexes?

7. **Error Handling**: How should we track and manage processing errors? Should errors be stored directly in the queue table or in a separate error log?

8. **Concurrency**: How should we handle concurrent updates to the same files or related metadata?
