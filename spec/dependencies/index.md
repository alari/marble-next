# External Dependencies Index

**Last Updated:** 2025-04-03

This directory contains specifications for external dependencies used in the Marble project, documenting their purpose, configuration, and usage patterns.

## Storage and Database

- [OpenDAL](opendal.md) - Storage abstraction library
- [SQLx](sqlx.md) - Database interaction

## Configuration and Environment

- [dotenv](dotenv.md) - Environment variable loading from .env files

## Future Dependency Documentation Needs

The following dependencies should be documented:

1. **dav-server-opendalfs** - WebDAV server integration with OpenDAL
2. **gray_matter** - Frontmatter parsing
3. **tokio** - Async runtime
4. **thiserror/anyhow** - Error handling

## Templates

- [template](template.md) - Template for creating new dependency specifications

## Dependency Status Summary

| Dependency | Status | Version | Usage | Documentation Priority |
|------------|--------|---------|-------|------------------------|
| OpenDAL | DRAFT | 0.52.0 | S3 storage abstraction | HIGH |
| SQLx | DRAFT | 0.8.3 | Database operations | HIGH |
| dotenv | DRAFT | 0.15.0 | Configuration management | MEDIUM |
| dav-server-opendalfs | PENDING | - | WebDAV server | HIGH |
| gray_matter | PENDING | - | Frontmatter parsing | MEDIUM |
| tokio | PENDING | 1.36.0 | Async runtime | LOW |
| serde | PENDING | - | Serialization | LOW |
