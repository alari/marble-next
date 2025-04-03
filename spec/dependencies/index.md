# External Dependencies Index

**Last Updated:** 2025-04-03

This directory contains specifications for external dependencies used in the Marble project, documenting their purpose, configuration, and usage patterns.

## Storage and Database

- [OpenDAL](opendal.md) - Storage abstraction library

## Future Dependency Documentation Needs

The following dependencies should be documented:

1. **dav-server-opendalfs** - WebDAV server integration with OpenDAL
2. **sqlx** - Database interaction
3. **gray_matter** - Frontmatter parsing
4. **tokio** - Async runtime
5. **thiserror/anyhow** - Error handling

## Templates

- [template](template.md) - Template for creating new dependency specifications

## Dependency Status Summary

| Dependency | Status | Version | Usage | Documentation Priority |
|------------|--------|---------|-------|------------------------|
| OpenDAL | DRAFT | 0.43.0 | S3 storage abstraction | HIGH |
| dav-server-opendalfs | PENDING | - | WebDAV server | HIGH |
| sqlx | PENDING | - | Database operations | MEDIUM |
| gray_matter | PENDING | - | Frontmatter parsing | MEDIUM |
| tokio | PENDING | - | Async runtime | LOW |
| serde | PENDING | - | Serialization | LOW |
