# Marble Read Processor Specification

## Overview

The `marble-read-processor` crate is responsible for generating the processed content that is exposed through the read-only WebDAV interface. It consumes the metadata stored in the database, applies publishing rules, transforms content, and generates the read model.

## Responsibilities

- Determine what content should be published (based on `publish: true` flag)
- Resolve references between content items
- Transform content according to publishing rules
- Restructure content based on permalink values
- Convert Obsidian-specific syntax to standard markdown
- Generate the processed content cache
- Handle incremental updates based on the processing queue

## Processing Rules

1. **Publication Filtering**:
   - Only content with `publish: true` in frontmatter (and its embeds) is included in processed output
   - Embedded content is included even if not explicitly marked as published

2. **Structure Transformation**:
   - Content is reorganized based on permalink values
   - Published markdown files become index files in permalink-named directories
   - Embedded content is included within the appropriate directories

3. **Link Transformation**:
   - Link resolution process:
     1. Query database for matching document name or alias (within same vault)
     2. For published content, create link to permalink with optional fragment anchor
     3. For unpublished or non-existent content, replace with link text only
   - Standard markdown format: `[{ref title}]({permalink of published page}{optional: #anchor})`
   - Links are only transformed in the read model, not in raw storage

## Cache Generation

- Processed content is stored in S3 as a read model cache
- Content is organized according to the permalink structure
- All paths are prefixed with username for tenant isolation
- Cache serves as the source for the read-only WebDAV endpoint

## Incremental Processing

1. **Queue Monitoring**:
   - Monitor processing queue for content that needs updating
   - Use buffer period (5 seconds) after last change for batch processing

2. **Impact Analysis**:
   - Determine what content is affected by a change
   - Identify content that references changed content

3. **Selective Regeneration**:
   - Regenerate only affected content
   - Update cache entries selectively

## Integration Points

- Reads metadata from database (via `marble-db`)
- Retrieves content from raw storage (via `marble-storage`)
- Writes processed content to S3 cache

## Future Work

- Implement more advanced reference resolution
- Add support for additional publishing features
- Create plugins for custom content transformations
