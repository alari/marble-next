# Marble Database Schema

**Status: [STABLE]**

## Overview

This document describes the PostgreSQL database schema for Marble. The database serves as the metadata layer for the system, tracking file paths, content hashes, relationships, and processing status.

## Design Principles

1. **Clear Separation of Concerns**: Tenant isolation, content vs. relationship data
2. **Efficient Querying**: Optimized for common operations
3. **Versioning Support**: Track history of changes
4. **Incremental Processing**: Track changes for minimal reprocessing

## Schema Design

### Core Tables

#### `users`
| Key Columns     | Description                        |
|-----------------|------------------------------------|
| id              | Primary key                        |
| username        | Unique username                    |
| password_hash   | Securely stored password hash      |
| last_login      | When the user last logged in       |

[Full SQL Definition](../code_samples/database/users.sql)

#### `files`
| Key Columns     | Description                        |
|-----------------|------------------------------------|
| id              | Primary key                        |
| user_id         | Foreign key to users               |
| path            | File path in the vault             |
| content_hash    | Current content hash (links to S3) |
| content_type    | MIME type or file format           |
| is_deleted      | Tombstone flag                     |

[Full SQL Definition](../code_samples/database/files.sql)

#### `file_versions`
| Key Columns     | Description                        |
|-----------------|------------------------------------|
| id              | Primary key                        |
| file_id         | Foreign key to files               |
| content_hash    | Content hash for this version      |
| version_number  | Sequential version number          |

[Full SQL Definition](../code_samples/database/file_versions.sql)

#### `folders`
| Key Columns     | Description                        |
|-----------------|------------------------------------|
| id              | Primary key                        |
| user_id         | Foreign key to users               |
| path            | Folder path                        |
| parent_id       | Foreign key to parent folder       |
| is_deleted      | Tombstone flag                     |

[Full SQL Definition](../code_samples/database/folders.sql)

### Content Analysis Tables

#### `frontmatter`
| Key Columns     | Description                        |
|-----------------|------------------------------------|
| id              | Primary key                        |
| file_id         | Foreign key to files               |
| publish         | Whether content should be published|
| permalink       | Custom URL path for publishing     |
| tags            | Array of tags                      |
| aliases         | Array of alternative names         |

[Full SQL Definition](../code_samples/database/frontmatter.sql)

#### `aliases`
| Key Columns     | Description                        |
|-----------------|------------------------------------|
| id              | Primary key                        |
| file_id         | Foreign key to files               |
| alias           | Alias value                        |
| is_primary      | Whether this is the primary name   |

[Full SQL Definition](../code_samples/database/aliases.sql)

#### `document_links`
| Key Columns     | Description                        |
|-----------------|------------------------------------|
| id              | Primary key                        |
| source_file_id  | Foreign key to source file         |
| target_name     | Referenced note title/name         |
| is_embed        | true for embeds, false for refs    |
| target_file_id  | Foreign key to target (if resolved)|
| position        | Position in document for ordering  |

[Full SQL Definition](../code_samples/database/document_links.sql)

### Processing Tables

#### `processing_queue`
| Key Columns     | Description                        |
|-----------------|------------------------------------|
| id              | Primary key                        |
| file_id         | Foreign key to files               |
| operation       | Type of change                     |
| status          | Current status                     |
| attempts        | Number of processing attempts      |

[Full SQL Definition](../code_samples/database/processing_queue.sql)

#### `published_content`
| Key Columns     | Description                        |
|-----------------|------------------------------------|
| id              | Primary key                        |
| file_id         | Foreign key to source file         |
| permalink       | Published path                     |
| processed_hash  | Hash of processed content          |
| invalidated     | Whether it needs reprocessing      |

[Full SQL Definition](../code_samples/database/published_content.sql)

#### `embedded_in_published`
| Key Columns         | Description                     |
|---------------------|---------------------------------|
| id                  | Primary key                     |
| published_content_id| Foreign key to published content|
| embedded_file_id    | Foreign key to embedded file    |
| fragment_id         | Fragment identifier             |

[Full SQL Definition](../code_samples/database/embedded_in_published.sql)

#### `cache_invalidations`
| Key Columns     | Description                        |
|-----------------|------------------------------------|
| id              | Primary key                        |
| user_id         | Foreign key to users               |
| path_pattern    | Path pattern to invalidate         |
| processed       | Whether it has been processed      |

[Full SQL Definition](../code_samples/database/cache_invalidations.sql)

## Indexes and Performance

Key indexes have been included in the table creation statements above. These are designed to optimize the most common query patterns:

1. **Path Lookup**: Finding files by path within a user's vault
2. **Reference Resolution**: Finding files that match a reference or alias
3. **Dependency Tracking**: Finding files that reference or embed a specific file
4. **Processing Queue**: Finding pending items for processing

## Implementation Status

- **Implemented**: users, folders, files tables
- **Pending**: All other tables

For current implementation details, see [Current Database Schema](database_schema_current.md).

## Related Documentation

- [Full Database Schema](database_schema.md) - Complete details with all SQL
- [Marble Database Implementation](../crates/marble_db_implementation.md)
- [Database Schema Questions](database_schema_questions.md)
