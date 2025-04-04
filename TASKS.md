# Marble Project Tasks

## Project Setup

- [DONE] Initialize project structure
- [DONE] Create initial workspace Cargo.toml
- [DONE] Create initial README.md
- [DONE] Set up specs directory

## Specification Development

- [WIP] Define project purpose and scope
  - Next steps: Refine project goals based on recent information
  - Completion criteria: Documented in spec/spec.md with clear objectives

- [WIP] Identify core domain concepts
  - Next steps: Gather information on content types, tenant model, and data structures
  - Completion criteria: Domain concepts documented with relationships

- [WIP] Design initial architecture
  - Next steps: Refine Write Side and Read Side specifications
  - Completion criteria: Architecture diagram and component descriptions

## Domain Questions to Address

- [DONE] Content types and format specifications
  - Identified: Obsidian markdown, Obsidian canvas, and media objects (images)
  - Documented frontmatter structure with publishing controls
  - Identified key Obsidian-specific syntax (references and embeds)

- [DONE] Multi-tenant design
  - Each user is a tenant with username/password authentication
  - Complete isolation between tenants; no multi-user vault access

- [DONE] Storage architecture
  - Raw data: Original Obsidian files (read-write)
  - Processed data: Transformed and published content (read-only)
  - Processed content organized by permalink

- [DONE] Processing pipeline
  - Analyzes markdown and canvas files for references, embeds, and frontmatter
  - Publishes only content with `publish: true` and its dependencies
  - Restructures content according to permalink values
  - Transforms Obsidian links to standard markdown

- [WIP] WebDAV implementation details
  - WebDAV chosen for direct Obsidian sync compatibility
  - [TODO] What specific WebDAV features are required for Obsidian compatibility?
  - [TODO] How should WebDAV properties be handled?

- [TODO] User management API
  - What operations should the user management API support?
  - What authentication methods beyond configuration files might be needed?

- [DONE] Processing triggers and execution
  - Real-time, incremental processing approach
  - Changes trigger selective reprocessing of affected content
  - Metadata database tracks dependencies for efficient updates

- [DONE] Underlying storage implementation
  - Hybrid architecture: S3 for content, PostgreSQL for metadata
  - Content stored using hash-based approach for deduplication
  - Metadata database tracks paths, versions, and relationships
  - Optimized for large vaults (1000+ notes)

- [DONE] Database schema design
  - Outlined primary tables for users, files, versions, folders
  - Defined content analysis tables for frontmatter, references, embeds
  - Created processing-related tables for queue and invalidation
  - Designed basic API interface for database operations

- [DONE] Caching strategy
  - Processed content cached in S3 (serves as read model)
  - Cache invalidation via processing queue with 5-second buffer
  - Changes batch-processed after sync operations complete

- [DONE] Content processing details
  - Canvas files processed as strings, extracting references without frontmatter
  - References resolved using database lookup of matching names/aliases
  - Obsidian links converted to standard markdown format: `[{title}]({permalink}{#anchor})`
  - Links to unpublished content replaced with link text only

- [DONE] User management approach
  - Minimal user management through configuration
  - No public API for user creation/management
  - Manual user creation through database/configuration
  - Future expansion possible if needed

- [DONE] Read Side architecture
  - Uses Handlebars templates for HTML generation
  - markdown-it for markdown-to-HTML conversion
  - Accesses processed content via public WebDAV API
  - Uses hostname as username prefix for tenant isolation
  - Initial implementation with fixed templates
  - Future support for custom templates from vaults

- [TODO] WebDAV implementation specifics
  - Integration approach for dav-server-opendalfs with OpenDAL backends
  - Implementation of public endpoint for processed content

## Implementation Planning

- [DONE] Define crate structure
  - Crates identified:
    - `marble-core`: Shared types, frontmatter definitions, and utilities
    - `marble-db`: Database schema and operations with PostgreSQL
    - `marble-storage`: OpenDAL backend implementations for raw and processed data
    - `marble-write-processor`: Content analysis and metadata extraction
    - `marble-read-processor`: Content transformation and read model generation
    - `bin/marble-webdav`: WebDAV server binary with authentication
    - Read side crates (to be implemented)
  - Interfaces between crates defined in specs
  - Clear responsibilities documented for each component

- [WIP] Select dependencies
  - Core dependencies identified:
    - `dav-server-opendalfs` for WebDAV implementation including locking
    - `opendal` for storage abstraction
    - `sqlx` for PostgreSQL interaction
    - `gray_matter` for frontmatter parsing
    - `blake2b_simd` for content hashing
    - `base64` for hash encoding (URL_SAFE_NO_PAD)
    - `serde`/`serde_json` for serialization
    - `aws-sdk-s3` or equivalent for S3 interaction (possibly through OpenDAL)
    - Authentication libraries (to be determined)
  - Next steps: Explore integration of dav-server-opendalfs with OpenDAL
  - Completion criteria: Documented dependency choices with rationale

## Implementation Plan

The implementation will follow these steps, with each component being individually testable:

1. **Database Schema (marble-db)**
   - [WIP] Set up SQLx with migration support
   - Implement PostgreSQL schema for users, files, and metadata
   - Create SQLx operations for common queries
   - Build tests using a test database
   - This provides the foundation for all other components

2. **WebDAV Server Framework (bin/marble-webdav)**
   - Implement basic server structure using dav-server-opendalfs
   - Create interfaces for storage backends (to be injected)
   - Implement authentication mechanisms
   - Create mock backends for testing
   - This allows testing the WebDAV interface independently

3. **Storage Implementation (marble-storage)**
   - Implement S3 content storage with hash-based addressing
   - Create OpenDAL backends that integrate with marble-db
   - Build both raw and processed storage interfaces
   - This connects the database to the content storage

4. **Write Model Processor (marble-write-processor)**
   - Implement content analysis for new/updated files
   - Create metadata extraction pipeline
   - Build dependency tracking system
   - This handles the input side of the system

5. **Read Model Generator (marble-read-processor)**
   - Implement publishing rules and content transformation
   - Create permalink structure generation
   - Build reference resolution system
   - This generates the processed output

6. **Integration and Testing**
   - Connect all components
   - Implement end-to-end tests
   - Verify that the acceptance criteria are met:
     - Connect to WebDAV twice (with/without auth)
     - Copy vault to authenticated endpoint
     - Verify processed content in public endpoint

7. **Read Side Implementation** (future)
   - Implement HTML generation with templates
   - Create the public-facing website system
