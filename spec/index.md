# Marble Specification

**Last Updated:** 2025-04-05

## Project Overview

Marble is a multi-tenant notes and knowledge management platform with publishing capabilities, built on a hybrid storage architecture using S3 for content and PostgreSQL for metadata.

## Key Documentation

- [Architecture Overview](architecture.md) - System architecture and components
- [Data Flow](data_flow.md) - Content lifecycle from creation to publishing
- [Documentation Standards](standards.md) - Guidelines for specifications

## Component Index

### Domain Concepts
- [Glossary](domain/glossary.md) - Key terms and concepts
- [Architecture Diagram](domain/architecture_diagram.md) - Visual system overview
- [Storage Architecture](domain/storage_architecture.md) - Content storage design
- [Database Schema](domain/database_schema.md) - Database design and tables
  - [Concise Schema](domain/database_schema_concise.md) - Compact table descriptions
  - [Current Schema](domain/database_schema_current.md) - **[IMPLEMENTED]** tables
  - [Schema Questions](domain/database_schema_questions.md) - Open questions
- [Write Side](domain/write_side.md) - Content creation and management
- [Read Side](domain/read_side.md) - Content publishing process

### Crate Specifications
- [Marble Core](crates/marble_core.md) - Shared types and utilities
- [Marble Database](crates/marble_db.md) - **[PARTIALLY IMPLEMENTED]** Database operations
  - [Database Overview](crates/marble_db_overview.md) - Core concepts
  - [Database API](crates/marble_db_api.md) - Repository interfaces
  - [Implementation Status](crates/marble_db_implementation.md) - Current progress
- [Marble Storage](crates/marble_storage.md) - Storage abstraction
- [Marble Write Processor](crates/marble_write_processor.md) - Content analysis
- [Marble Read Processor](crates/marble_read_processor.md) - Content transformation
- [Marble WebDAV Server](crates/marble_webdav.md) - WebDAV interface

### External Dependencies
- [Dependencies Index](dependencies/index.md) - External library documentation

## Current Handoffs

- [Database Schema](handoffs/database_schema.md) - Schema refinement
- [Database Schema Implementation](handoffs/database_schema_implementation.md) - **[IMPLEMENTED]** Core schema
- [Database Models Implementation](handoffs/database_models_implementation.md) - **[IMPLEMENTED]** Rust models
- [Database Repositories Implementation](handoffs/database_repositories_implementation.md) - **[IMPLEMENTED]** Repositories
- [Database Testing Fixes](handoffs/database_testing_fixes.md) - **[IMPLEMENTED]** Test reliability
- [Documentation Restructuring](handoffs/documentation_restructuring.md) - **[IMPLEMENTED]** Doc reorganization
- [Marble Storage Implementation](handoffs/marble_storage_implementation.md) - **[PARTIALLY IMPLEMENTED]** Storage implementation with tenant isolation
- [OpenDAL Integration Research](handoffs/opendal_adapter_implementation.md) - **[COMPLETE]** Findings and strategic pivot to Unified Storage API
- [All Handoffs](handoffs/index.md) - Complete list of work handoffs

## Implementation Status

### Completed
- ✅ Core database schema with users, folders, and files tables
- ✅ Database models with helper methods and utility functions
- ✅ Repository pattern implementation with SQLx
- ✅ Database connection and transaction management
- ✅ Testing infrastructure with PostgreSQL 17

### In Progress
- 🔄 Advanced database schema tables (content analysis, processing)
- 🔄 Unified Tenant Storage API implementation
- 🔄 WebDAV server integration

### Planned
- ⏳ Content processing and analysis pipeline
- ⏳ Publishing system implementation
- ⏳ Read-side generation

## Next Steps

1. **Database Implementation**: Implement remaining tables for content analysis
2. **WebDAV Server**: Integrate with database and storage
3. **Processing Pipeline**: Build incremental analysis system
4. **Testing**: Develop comprehensive unit and integration tests
