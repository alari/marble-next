# External Dependencies Index

**Last Updated:** 2025-04-03

This directory contains specifications for external dependencies used in the Marble project, documenting their purpose, configuration, and usage patterns.

## Storage and Database

- [OpenDAL](opendal.md) - Storage abstraction library
- [SQLx](sqlx.md) - Database interaction

## Configuration and Environment

- [dotenv](dotenv.md) - Environment variable loading from .env files

## Logging and Instrumentation

- [tracing](tracing.md) - Structured logging and instrumentation

## Serialization and Data Handling

- [serde](serde.md) - Serialization framework
- [chrono](chrono.md) - Date and time handling

## Async Runtime

- tokio - Async runtime
- tokio-stream - Stream utilities for tokio
- tokio-util - Additional utilities for tokio
- async-trait - Trait support for async methods

## Content Processing

- gray_matter - Frontmatter parsing
- base64 - Base64 encoding/decoding
- mime - MIME type handling
- mime_guess - MIME type detection from file extensions

## Future Dependency Documentation Needs

The following dependencies should be documented:

1. **dav-server-opendalfs** - WebDAV server integration with OpenDAL
2. **gray_matter** - Frontmatter parsing
3. **tokio ecosystem** - Async runtime and utilities
4. **thiserror/anyhow** - Error handling

## Templates

- [template](template.md) - Template for creating new dependency specifications

## Dependency Status Summary

| Dependency | Status | Version | Usage | Documentation Priority |
|------------|--------|---------|-------|------------------------|
| OpenDAL | DRAFT | 0.52.0 | S3 storage abstraction | HIGH |
| SQLx | DRAFT | 0.8.3 | Database operations | HIGH |
| dotenv | DRAFT | 0.15.0 | Configuration management | MEDIUM |
| tracing | DRAFT | 0.1.41 | Logging and instrumentation | MEDIUM |
| serde | DRAFT | 1.0.219 | Serialization | MEDIUM |
| serde_json | DRAFT | 1.0.140 | JSON handling | MEDIUM |
| chrono | DRAFT | 0.4.40 | Date and time | MEDIUM |
| tokio | PENDING | 1.44.1 | Async runtime | MEDIUM |
| tokio-stream | PENDING | 0.1.17 | Async stream utilities | LOW |
| tokio-util | PENDING | 0.7.14 | Async utilities | LOW |
| async-trait | PENDING | 0.1.88 | Async trait support | LOW |
| gray_matter | PENDING | 0.2.8 | Frontmatter parsing | HIGH |
| base64 | PENDING | 0.22.1 | Encoding/decoding | LOW |
| mime | PENDING | 0.3.17 | MIME type handling | LOW |
| mime_guess | PENDING | 2.0.5 | MIME type detection | LOW |
| dav-server-opendalfs | PENDING | - | WebDAV server | HIGH |
