# Read Side Specification

## Overview

The Read Side of Marble focuses on content consumption and presentation. It fetches processed data from the Write Side's WebDAV API and transforms it into styled HTML output for end users.

## Responsibilities

- Connect to the processed data WebDAV API
- Retrieve and cache content as needed
- Transform content into HTML with appropriate styling
- Handle user requests for published content
- Manage content navigation and discovery

## Architecture

The Read Side consists of the following components:

1. **WebDAV Client**:
   - Connects to the public processed data WebDAV API
   - Uses hostname as username for path prefixing
   - Retrieves content and metadata without authentication

2. **Content Processor**:
   - Transforms markdown into HTML using markdown-it
   - Applies Handlebars templates for layout and styling
   - Generates navigation and content relationships

3. **Web Server**:
   - Serves HTML content to end users
   - Handles user navigation and requests
   - Potentially generates manifest.json

## Data Flow

1. User requests content through the Read Side
2. Read Side identifies the tenant (via hostname)
3. Read Side fetches required data from the WebDAV API using tenant prefix
4. Content is transformed into HTML using Handlebars templates
5. Styled content is served to the user

## Implementation Plan

### Initial Version
- Basic HTML generation using markdown-it and Handlebars
- Fixed template structure for all sites
- Local caching for performance

### Future Enhancements
- Custom templates stored in user vaults
- Configurable layouts and styling
- Advanced navigation and search capabilities

## API

The Read Side exposes:
- Web interface for content browsing and reading
- Potentially manifest.json for site metadata

## Integration

- Connects to processed WebDAV API provided by Write Side
- No authentication required for processed content
- Uses hostname-to-username mapping for tenant isolation
