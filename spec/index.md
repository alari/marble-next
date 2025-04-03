# Marble Specification

**Last Updated:** 2025-04-03

## Project Overview

Marble is a multi-tenant notes and knowledge management platform with publishing capabilities. This project is a rewrite of an existing implementation, aimed at improving the architecture and functionality.

## Domain Context

Marble manages notes and knowledge content across multiple tenants. The system allows for content creation, modification, and publishing through a structured interface.

Key aspects:
- Multi-tenant design for content isolation
- Content management and organization
- Publishing capabilities for knowledge sharing

## Architecture

Marble is built on a hybrid storage architecture using S3 for content and PostgreSQL for metadata, with several components handling different responsibilities:

1. **Core Components (`marble-core`)**:
   - Defines shared data models and types
   - Provides authentication interfaces
   - Implements common utilities
   - Defines frontmatter structure and parsing

2. **Database Layer (`marble-db`)**:
   - Manages PostgreSQL schema and operations
   - Stores file metadata, relationships, and user data
   - Provides APIs for tracking and querying content

3. **Storage Abstraction (`marble-storage`)**:
   - Implements OpenDAL backends for different storage needs
   - Uses S3 for content storage (hash-based)
   - Integrates with marble-db for metadata
   - Provides tenant isolation through path management
   - Handles raw data storage (original Obsidian files)
   - Manages processed data storage (transformed content)

4. **Write Model Processor (`marble-write-processor`)**:
   - Analyzes new/updated content
   - Extracts frontmatter, references, and embeds
   - Updates database metadata
   - Handles dependency tracking

5. **Read Model Generator (`marble-read-processor`)**:
   - Determines what content is published
   - Transforms content for publishing
   - Manages the reading structure (permalinks, etc.)
   - Converts Obsidian links to standard markdown
   - Generates the processed content cache

6. **WebDAV Server (`bin/marble-webdav`)**:
   - Provides WebDAV interface using dav-server-opendalfs
   - Handles authentication and authorization
   - Injects OpenDAL backends via dependency injection
   - Exposes both raw (read-write) and processed (read-only) data

7. **Read Side** (future):
   - Fetches data from the processed WebDAV API
   - Uses hostname to identify tenant (username)
   - Generates styled HTML output with Handlebars templates and markdown-it
   - Implements caching for performance

## Current Handoffs

- [Database Schema](handoffs/database_schema.md) - Database schema specification refinement in progress
- [Dependencies Update](handoffs/dependencies_update.md) - Added key dependencies for core functionality

## Component Index

### Domain Concepts
- [Glossary](domain/glossary.md) - Definitions of key terms and concepts
- [Architecture Diagram](domain/architecture_diagram.md) - Visual overview of system architecture
- [Crate Dependencies](domain/crate_dependencies.md) - Relationship between crates
- [Storage Architecture](domain/storage_architecture.md) - Content storage design
- [Database Schema](domain/database_schema.md) - PostgreSQL schema specification
- [Write Side](domain/write_side.md) - Content creation and management process
- [Read Side](domain/read_side.md) - Content publishing process

### Crate Specifications
- [Marble Core](crates/marble_core.md) - Shared types and utilities
- [Marble Database](crates/marble_db.md) - Database operations
- [Marble Storage](crates/marble_storage.md) - Storage abstraction
- [Marble Write Processor](crates/marble_write_processor.md) - Content analysis
- [Marble Read Processor](crates/marble_read_processor.md) - Content transformation
- [Marble WebDAV Server](crates/marble_webdav.md) - WebDAV interface

### External Dependencies
- [OpenDAL](dependencies/opendal.md) - Storage abstraction library
- [SQLx](dependencies/sqlx.md) - Database interaction
- [dotenv](dependencies/dotenv.md) - Environment variable loading
- [tracing](dependencies/tracing.md) - Structured logging
- [serde](dependencies/serde.md) - Serialization framework
- [chrono](dependencies/chrono.md) - Date and time handling
- [Dependencies Template](dependencies/template.md) - Template for documenting dependencies

### Templates
- [Crate Specification Template](crates/template.md) - Template for new crate specs
- [Domain Concept Template](domain/template.md) - Template for new domain concepts
- [Handoff Template](handoffs/template.md) - Template for work handoffs

## Documentation Standards

### Status Indicators
Each specification document should include a status indicator:
- **[DRAFT]** - Initial documentation, may be incomplete or change significantly
- **[REVIEW]** - Ready for review, seeking feedback
- **[STABLE]** - Approved specification, only minor changes expected
- **[IMPLEMENTED]** - Specification has been implemented in code

### File Naming Conventions
- All filenames should use underscores instead of hyphens (e.g., `marble_core.md` not `marble-core.md`)
- Template files should be named `template.md` in each directory
- Domain concept files should be descriptive of the concept (e.g., `authentication.md`)

### Cross-References
- All specs should include a "Related Specifications" section
- Links should use relative paths (e.g., `../domain/concept.md`)
- Link text should include the document title

### Diagrams
- Use Mermaid diagrams for visualizations
- Include diagrams directly in Markdown using triple-backtick syntax
- Provide a text description of the diagram for accessibility

## Data Flow

1. Users connect to the WebDAV endpoint using OS-level WebDAV clients or Obsidian
2. Users authenticate with username/password credentials
3. WebDAV server validates credentials and establishes session
4. Raw content operations:
   - User uploads/modifies Obsidian vault content through WebDAV
   - WebDAV server routes to raw storage backend
   - Content is stored in user-specific isolated storage
5. Processing pipeline:
   - Raw content is analyzed to extract metadata:
     - Canvas files are processed to extract markdown snippets
     - Markdown files are analyzed for frontmatter, references (`[[...]]`), and embeds (`![[...]]`)
     - Only published content (and its embeds) is processed for the output
   - Content is transformed:
     - Published markdown files are collected with all their embedded content
     - Files are reorganized according to permalink structure
     - Obsidian links are converted to standard markdown links
     - Embedded content gets linked with anchors
   - Processed content is stored in the processed backend
6. Read-only access:
   - Processed content is accessible through WebDAV with username-prefixed paths
   - Read Side accesses this processed content
   - Read Side generates HTML output for publishing

## Next Steps

1. **Database Implementation**:
   - Implement the PostgreSQL schema as defined in the database specification
   - Create SQLx queries and operations for common database interactions
   - Build migration system for schema evolution

2. **WebDAV Server Implementation**:
   - Integrate dav-server-opendalfs with OpenDAL backends
   - Implement authentication and routing logic
   - Create both raw and processed endpoints

3. **Storage and Processing Pipeline**:
   - Implement S3 content storage with hash-based addressing
   - Build the incremental processing system
   - Create the cache invalidation mechanism

4. **Testing and Validation**:
   - Develop comprehensive unit and integration tests
   - Validate against Obsidian compatibility requirements
   - Ensure performance with large vaults (1000+ notes)

## Spec Improvement Roadmap

1. **Documentation Completeness**:
   - Create spec files for all external dependencies
   - Update crate specs with more detailed API interfaces
   - Add more visual diagrams to clarify architecture

2. **Standardization**:
   - Standardize all filename conventions
   - Add status indicators to all spec files
   - Ensure consistent formatting across all documents

3. **Traceability**:
   - Improve cross-references between related specifications
   - Add requirement IDs for key functionality
   - Link requirements to implementation components

4. **Validation**:
   - Review and validate all spec documents
   - Ensure alignment with actual implementation
   - Identify and resolve inconsistencies between specs
