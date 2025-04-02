# Marble Write Processor Specification

## Overview

The `marble-write-processor` crate is responsible for analyzing content in the raw storage, extracting metadata, and updating the database. It handles the input side of the processing pipeline, focusing on metadata extraction and dependency tracking.

## Responsibilities

- Monitor for new or updated content in raw storage
- Parse and analyze Obsidian markdown files
- Process Obsidian canvas files to extract references
- Extract frontmatter metadata from markdown files
- Identify Obsidian references (`[[...]]`) and embeds (`![[...]]`)
- Update the database with extracted metadata
- Track content dependencies for incremental processing

## Content Analysis

### Markdown Analysis
- Parse frontmatter using `gray_matter`
- Extract key frontmatter fields:
  - `publish`: Determines if content should be included in processed output
  - `permalink`: Defines output directory structure
  - `aliases`: Alternative names for reference resolution
  - `tags`: Content categorization
- Scan content for Obsidian references (`[[...]]`) and embeds (`![[...]]`)
- Store all extracted information in the database

### Canvas Analysis
- Canvas files don't contain frontmatter
- Parse canvas files as strings
- Extract Obsidian references and embeds from canvas content
- Store references and embeds in the database for dependency tracking

## Database Integration

The write processor updates several database tables:
- `files`: Records the current state of each file
- `file_versions`: Tracks historical versions
- `frontmatter`: Stores extracted frontmatter data
- `references`: Records links between files
- `embeds`: Tracks embedded content relationships

## Processing Flow

1. **Change Detection**:
   - Receive notification of new/modified content
   - Hash content for storage identification

2. **Content Analysis**:
   - Parse content based on file type
   - Extract relevant metadata

3. **Dependency Tracking**:
   - Identify references and embeds
   - Update dependency graph in database

4. **Queue Notification**:
   - Add entry to processing queue
   - Signal that content needs read-model processing

## Incremental Processing

- Processes only new or modified content
- Updates only affected metadata
- Uses database for efficient querying of files and relationships

## Integration Points

- Reads from raw storage (via `marble-storage`)
- Updates metadata in database (via `marble-db`)
- Triggers read model updates through processing queue

## Future Work

- Implement parallel processing for large batches
- Add support for additional Obsidian features
- Create plugins for custom content analysis
