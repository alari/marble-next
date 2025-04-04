# Marble Architecture

## Overview

Marble is built on a hybrid storage architecture with several components handling different responsibilities:

## Core Components

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

## Component Relationships

![Architecture Diagram](domain/architecture_diagram.md)

## Related Documentation

- [Data Flow](data_flow.md) - Content lifecycle from creation to publishing
- [Database Schema](domain/database_schema.md) - Database design
- [Storage Architecture](domain/storage_architecture.md) - Content storage design
