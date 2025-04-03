# Crate Specifications Index

**Last Updated:** 2025-04-03

This directory contains specifications for all Marble crates, detailing their purpose, responsibilities, and interfaces.

## Core Crates

- [marble_core](marble_core.md) - Shared types and utilities used throughout the system
- [marble_db](marble_db.md) - Database schema and operations
- [marble_storage](marble_storage.md) - Storage abstraction using OpenDAL

## Processing Crates

- [marble_write_processor](marble_write_processor.md) - Content analysis and metadata extraction
- [marble_read_processor](marble_read_processor.md) - Content transformation and publishing

## Server Crates

- [marble_webdav](marble_webdav.md) - WebDAV server implementation

## Templates

- [template](template.md) - Template for creating new crate specifications

## Crate Status Summary

| Crate Name | Status | Description | Key Dependencies |
|------------|--------|-------------|------------------|
| marble_core | DRAFT | Shared types and utilities | serde, thiserror |
| marble_db | DRAFT | Database operations | sqlx, tokio |
| marble_storage | DRAFT | Storage abstraction | opendal |
| marble_write_processor | DRAFT | Content analysis | gray_matter, marble_core, marble_storage |
| marble_read_processor | DRAFT | Content transformation | marble_core, marble_db, marble_storage |
| marble_webdav | DRAFT | WebDAV interface | dav-server-opendalfs, marble_storage |
