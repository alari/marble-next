# Write Side Specification

## Overview

The Write Side of Marble handles content creation, modification, and storage through a WebDAV interface. It's designed to work seamlessly with Obsidian, allowing users to synchronize their vaults through standard WebDAV clients.

## Content Types

The system handles several types of content:
1. **Obsidian Markdown Documents** - Text files with Obsidian-specific markdown syntax
2. **Obsidian Canvas Documents** - Canvas visualization files used by Obsidian
3. **Media Objects** - Images and potentially other media files

## Tenant Model

In Marble:
- A tenant is a user/password pair
- Each tenant corresponds to an Obsidian vault
- Tenant isolation is enforced at the authentication level
- No multi-user access to a single vault is supported
- Each user can only access their own data

## Architecture Components

1. **WebDAV Server Binary (`marble-webdav`)**:
   - Placed in `bin/` directory as it's an executable, not a library
   - Provides WebDAV interface for Obsidian clients
   - Handles authentication and authorization
   - Routes requests to appropriate storage backend (raw or processed)
   - Manages user sessions and authentication
   - Uses dav-server-opendalfs for WebDAV implementation including lock system
   - Minimal logic - primarily imports and configures backends
   - Exposes two WebDAV endpoints:
     - Authenticated endpoint for raw content (read-write)
     - Public endpoint for processed content (read-only)

2. **Storage Abstraction Layer (`marble-storage`)**:
   - Provides two OpenDAL backends:
     - Raw data backend (read-write access)
     - Processed data backend (read-only access)
   - Handles tenant-specific data isolation
   - Manages data organization and access patterns

3. **Database Layer (`marble-db`)**:
   - Manages PostgreSQL schema and operations
   - Tracks file metadata, paths, and relationships
   - Provides historical versioning information

4. **User Management API**:
   - Interface for user administration
   - Default implementation uses configuration-based user/password pairs
   - Potential for alternative authentication providers

## Data Flow

1. User connects to WebDAV endpoint using OS/Obsidian client
2. User authenticates with username/password
3. `marble-webdav` validates credentials and establishes session
4. User performs WebDAV operations on their vault
5. Raw operations are routed to the raw storage backend
6. Data processing occurs between raw and processed storage
7. Processed data is accessible through the read-only WebDAV interface

## API Paths

- `/raw/{file_path}` - Access to raw Obsidian files (read-write)
- `/processed/{username}/{file_path}` - Access to processed content (read-only)

## Implementation Notes

- Authentication and authorization handled at the WebDAV layer
- Strict tenant isolation enforced through path prefixing and access control
- Processed WebDAV paths must include username prefix
- Raw data is directly from user's Obsidian vault
- No cross-tenant data access is permitted

## Future Work

- Define exact WebDAV feature requirements for Obsidian compatibility
- Establish data processing pipeline between raw and processed storage
- Define user management API details
