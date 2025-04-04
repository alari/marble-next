# Marble Database Schema

**Status: [STABLE]**

## Overview

This document serves as an index to the database schema documentation. The complete schema has been split into multiple documents for better organization and readability.

## Main Documentation

- [Concise Database Schema](database_schema_concise.md) - Core tables and relationships in a compact format
- [Current Database Schema](database_schema_current.md) - Implemented tables and their Rust models
- [Database Schema Questions](database_schema_questions.md) - Open questions and considerations

## SQL Definitions

All SQL table definitions are available in the code_samples directory:

- [users.sql](../code_samples/database/users.sql)
- [folders.sql](../code_samples/database/folders.sql)
- [files.sql](../code_samples/database/files.sql)
- [file_versions.sql](../code_samples/database/file_versions.sql)
- [frontmatter.sql](../code_samples/database/frontmatter.sql)
- [aliases.sql](../code_samples/database/aliases.sql)
- [document_links.sql](../code_samples/database/document_links.sql)
- [processing_queue.sql](../code_samples/database/processing_queue.sql)
- [published_content.sql](../code_samples/database/published_content.sql)
- [embedded_in_published.sql](../code_samples/database/embedded_in_published.sql)
- [cache_invalidations.sql](../code_samples/database/cache_invalidations.sql)

## Schema Organization

The database schema is organized into logical groups:

1. **Core Tables**: users, folders, files
2. **Content Analysis**: frontmatter, aliases, document_links
3. **Processing**: processing_queue, published_content
4. **Version Control**: file_versions
5. **Cache Management**: cache_invalidations, embedded_in_published

## Implementation Status

Currently, the following tables have been implemented:
- users
- folders
- files

The remaining tables are part of the planned future implementation.

## Related Documentation

- [Marble Database Overview](../crates/marble_db_overview.md)
- [API Design](../crates/marble_db_api.md)
- [Implementation Status](../crates/marble_db_implementation.md)
