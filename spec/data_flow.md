# Marble Data Flow

## Overview

This document describes the high-level data flow through the Marble system, from content creation to publishing.

## Content Creation and Management

1. Users connect to the WebDAV endpoint using OS-level WebDAV clients or Obsidian
2. Users authenticate with username/password credentials
3. WebDAV server validates credentials and establishes session
4. Raw content operations:
   - User uploads/modifies Obsidian vault content through WebDAV
   - WebDAV server routes to raw storage backend
   - Content is stored in user-specific isolated storage

## Content Processing Pipeline

5. Raw content is analyzed to extract metadata:
   - Canvas files are processed to extract markdown snippets
   - Markdown files are analyzed for frontmatter, references (`[[...]]`), and embeds (`![[...]]`)
   - Only published content (and its embeds) is processed for the output
6. Content is transformed:
   - Published markdown files are collected with all their embedded content
   - Files are reorganized according to permalink structure
   - Obsidian links are converted to standard markdown links
   - Embedded content gets linked with anchors
7. Processed content is stored in the processed backend

## Content Delivery

8. Read-only access:
   - Processed content is accessible through WebDAV with username-prefixed paths
   - Read Side accesses this processed content
   - Read Side generates HTML output for publishing

## Data Storage Components

- **S3 Storage**: Content-addressable storage for file contents
- **PostgreSQL**: Metadata storage for paths, relationships, and processing status
- **Processed Cache**: Transformed content ready for publishing

## Related Documentation

- [Storage Architecture](domain/storage_architecture.md)
- [Database Schema](domain/database_schema.md)
- [Write Side](domain/write_side.md)
- [Read Side](domain/read_side.md)
