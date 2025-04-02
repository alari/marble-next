# Marble

Marble is a multi-tenant notes and knowledge management platform with publishing capabilities.

## Overview

Marble provides a system for creating, managing, and publishing knowledge content across multiple tenants. It uses a split architecture:

- **Write Side**: Provides WebDAV interface for content management using Axum and OpenDAL
- **Read Side**: Generates styled HTML content from processed data

## Getting Started

```bash
cargo build
```

## Project Structure

The project is organized as a Rust workspace with the following components:

- `crates/marble_core`: Shared types, frontmatter definitions, and utilities
- `crates/marble-db`: Database schema and operations with PostgreSQL
- `crates/marble-storage`: OpenDAL backends for raw and processed data
- `crates/marble-write-processor`: Content analysis and metadata extraction
- `crates/marble-read-processor`: Content transformation and read model generation
- `bin/marble-webdav`: WebDAV server binary with authentication and routing
- Read Side crates (future implementation)

The system uses a hybrid storage architecture:
- S3 for content storage (hash-based approach)
- PostgreSQL for metadata, paths, and relationships

Key features:
- Multi-tenant architecture with isolated user data spaces
- Real-time incremental processing for large content vaults
- Content-addressable storage for efficient deduplication
- WebDAV interface for seamless Obsidian integration
- Publishing controls via frontmatter metadata
- Optimized for large vaults (1000+ notes)

## Documentation

- See the `spec/` directory for detailed specifications
- `TASKS.md` tracks current development priorities

## License

*[License information will be added here]*
