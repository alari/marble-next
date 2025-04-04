# Marble Database 

**Status: [PARTIALLY IMPLEMENTED]**

## Overview

The `marble-db` crate provides the PostgreSQL database layer for Marble, managing metadata storage, relationship tracking, and authentication. It complements the S3-based content storage system.

This document serves as an index to the detailed specifications for the database layer.

## Documentation Structure

- [Database Overview](marble_db_overview.md) - Core concepts and responsibilities
- [Database API Design](marble_db_api.md) - Repository interfaces and implementation
- [Implementation Status](marble_db_implementation.md) - Current progress and next steps

## Schema Design

The database schema is split into several components:
- Core tables (users, folders, files)
- Content analysis tables (frontmatter, document_links)
- Processing tables (processing_queue, published_content)
- Version control (file_versions)

For a complete schema description:
- [Database Schema Specification](../domain/database_schema.md) - Full design
- [Current Database Schema](../domain/database_schema_current.md) - Implemented tables

## Implementation Status Summary

### Implemented ✅
- Core database configuration and connection management
- Migration system with version control
- Models for users, folders, and files
- Repository pattern with trait-based interfaces
- CRUD operations for core tables
- Transaction support
- Testing infrastructure

### Pending ⏳
- Content analysis tables and repositories
- Processing queue and publication tracking
- Versioning and history tracking
- Advanced queries and batch operations

## Key Design Decisions

1. **Repository Pattern**: Separating interfaces from implementations
2. **Trait-based API**: Enabling dependency injection and testing
3. **Transaction Support**: Ensuring data consistency
4. **Soft Deletion**: Preserving data with tombstone flags
5. **Path-based Organization**: Organizing content hierarchically

## Integration Points

- **Storage Layer**: Maps paths to content hashes
- **Processor**: Queries for changed files and dependencies
- **WebDAV Server**: Uses database for authentication and paths

## Related Handoffs

- [Database Schema Implementation](../handoffs/database_schema_implementation.md) - Core schema implementation
- [Database Models Implementation](../handoffs/database_models_implementation.md) - Model implementations
- [Database Repositories Implementation](../handoffs/database_repositories_implementation.md) - Repository implementations
- [Database Testing Fixes](../handoffs/database_testing_fixes.md) - Test reliability improvements

## Next Steps

See [Implementation Status](marble_db_implementation.md) for detailed next steps.
