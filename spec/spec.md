# Marble Specification

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

1. **Core Components (`marble_core`)**:
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

## Component Index

- [Write Side](./write_side.md)
- [Read Side](./read_side.md)
- [Marble Core](./marble_core.md)
- [Marble Database](./marble_db.md)
- [Marble Storage](./marble_storage.md)
- [Marble Write Processor](./marble_write_processor.md)
- [Marble Read Processor](./marble_read_processor.md)
- [Marble WebDAV Server](./marble_webdav.md)
- [Storage Architecture](./storage_architecture.md)
- [Database Schema](./database_schema.md)

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
