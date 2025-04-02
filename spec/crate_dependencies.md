# Crate Dependencies

This document outlines the dependencies between the various crates in the Marble project.

## Dependency Graph

```mermaid
graph TD
    Core[marble-core]
    DB[marble-db]
    Storage[marble-storage]
    WriteProc[marble-write-processor]
    ReadProc[marble-read-processor]
    WebDAV[marble-webdav]
    
    DB -->|depends on| Core
    Storage -->|depends on| Core
    Storage -->|depends on| DB
    
    WriteProc -->|depends on| Core
    WriteProc -->|depends on| DB
    WriteProc -->|depends on| Storage
    
    ReadProc -->|depends on| Core
    ReadProc -->|depends on| DB
    ReadProc -->|depends on| Storage
    
    WebDAV -->|depends on| Core
    WebDAV -->|depends on| DB
    WebDAV -->|depends on| Storage
    
    classDef core fill:#f9f,stroke:#333,stroke-width:2px;
    classDef infra fill:#bbf,stroke:#333,stroke-width:2px;
    classDef proc fill:#bfb,stroke:#333,stroke-width:2px;
    classDef bin fill:#ff9,stroke:#333,stroke-width:2px;
    
    class Core core;
    class DB,Storage infra;
    class WriteProc,ReadProc proc;
    class WebDAV bin;
```

## Crate Descriptions

### Core Crates

- **marble-core**: Fundamental types and interfaces used across the system
  - Frontmatter definitions
  - Authentication interfaces
  - Common utilities

### Infrastructure Crates

- **marble-db**: Database schema and operations
  - PostgreSQL schema definitions
  - SQLx queries and operations
  - Migrations

- **marble-storage**: Storage abstraction
  - OpenDAL backends
  - S3 integration
  - Path mapping

### Processing Crates

- **marble-write-processor**: Content analysis and metadata extraction
  - File parsing
  - Frontmatter extraction
  - Reference identification
  - Database updates

- **marble-read-processor**: Content transformation for publishing
  - Publishing logic
  - Link transformation
  - Permalink structure generation
  - Cache generation

### Binary Crates

- **marble-webdav**: WebDAV server for client interaction
  - WebDAV protocol implementation
  - Authentication handling
  - Request routing
  - Backend integration

## Implementation Order

The suggested implementation order is:

1. **marble-core**: Establishes shared types and interfaces
2. **marble-db**: Creates database foundation
3. **marble-storage**: Implements storage layer
4. **marble-webdav**: Provides client interface
5. **marble-write-processor**: Handles content analysis
6. **marble-read-processor**: Generates published output
