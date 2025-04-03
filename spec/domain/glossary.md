# Marble Glossary

**Status:** DRAFT
**Last Updated:** 2025-04-03

This document defines key terms and concepts used throughout the Marble project specifications. It serves as a reference to ensure consistent terminology across all documentation.

## Core Concepts

### Tenant
An individual user with isolated data space. In Marble, each user is a separate tenant with their own content vault.

### Vault
A collection of documents and media belonging to a single tenant. Similar to an Obsidian vault.

### Raw Content
The original files as stored by the user, including Obsidian-specific formatting and syntax.

### Processed Content
Content that has been transformed for publishing, with resolved references and converted links.

### WebDAV
Web Distributed Authoring and Versioning - the protocol used to provide file system-like access to Marble content.

### Write Side
The component of Marble that handles user content creation and modification, primarily through WebDAV.

### Read Side
The component of Marble that handles content delivery and presentation for published content.

## Content Types

### Markdown Document
A text file using Markdown syntax, potentially with Obsidian-specific extensions and frontmatter.

### Canvas Document
An Obsidian-specific visualization file that contains nodes with content and connections between them.

### Media Object
Any non-text file such as images, PDFs, or other binary content.

## Obsidian-Specific Terms

### Frontmatter
YAML metadata at the beginning of a Markdown file, enclosed between `---` lines.

### Reference
A link to another note using the syntax `[[note-name]]` or `[[note-name|display text]]`.

### Embed
An inclusion of another note's content using the syntax `![[note-name]]` or `![[note-name#section]]`.

### Permalink
A URL path specified in frontmatter that determines where the content will be published.

## Database and Storage

### Content Hash
A unique identifier for file content, used for content-addressable storage.

### File Version
A historical record of a file at a specific point in time.

### Processing Queue
A list of files that need to be analyzed and processed due to changes.

### Cache Invalidation
The process of marking processed content as outdated when dependencies change.

## Processing Terminology

### Publish Status
Whether a document is marked for public visibility through the `publish: true` frontmatter property.

### Reference Resolution
The process of converting Obsidian references to their corresponding files.

### Dependency Graph
The network of relationships between documents based on references and embeds.

### Link Navigation
The ability to move between related documents using resolved links.

## Technical Components

### OpenDAL
Open Data Access Layer - the abstraction used for accessing storage backends.

### S3 Backend
The storage service used for content storage.

### PostgreSQL
The database used for storing metadata and relationships.

### SQLx
The Rust library used for type-safe SQL queries.
