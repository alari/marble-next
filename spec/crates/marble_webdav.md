# Marble WebDAV Server Specification

## Overview

The `marble-webdav` binary provides a WebDAV interface for clients (particularly Obsidian) to interact with the Marble system. It acts as the primary entry point for content creation and consumption, handling authentication, routing, and storage backend integration.

## Responsibilities

- Provide a WebDAV interface compatible with Obsidian and OS-level WebDAV clients
- Implement authentication and authorization for users
- Route requests to appropriate storage backends (raw or processed)
- Expose both read-write and read-only interfaces
- Manage WebDAV locking for concurrent modifications

## Architecture

The WebDAV server consists of the following components:

1. **Server Framework**:
   - Built using dav-server-opendalfs for WebDAV protocol implementation
   - Handles WebDAV methods (GET, PUT, PROPFIND, etc.)
   - Manages WebDAV locking for concurrent access

2. **Authentication System**:
   - Validates user credentials against the database
   - Manages sessions and authorization
   - Isolates user content to prevent cross-tenant access

3. **Backend Integration**:
   - Injects OpenDAL backends via dependency injection
   - Connects to different storage implementations based on path/endpoint

## Endpoints

The server exposes two primary WebDAV endpoints:

1. **Raw Content Endpoint** (`/raw/`):
   - Authenticated access only
   - Read-write permission
   - Direct mapping to user's original content
   - Used for synchronizing Obsidian vaults

2. **Processed Content Endpoint** (`/processed/`):
   - Public access with username in path (`/processed/{username}/...`)
   - Read-only permission
   - Contains only published content
   - Reorganized according to permalink structure
   - Used by the Read Side for generating HTML

## Implementation Details

### WebDAV Integration

The server uses dav-server-opendalfs to handle WebDAV protocol specifics:
- Implements the WebDAV HTTP extension methods
- Manages WebDAV properties
- Provides locking functionality for concurrent access
- Handles file system-like operations over HTTP

### OpenDAL Backend Integration

The WebDAV server injects OpenDAL backends for storage operations:
- Creates appropriate backends for raw and processed endpoints
- Adds user-specific context to operations
- Maps WebDAV operations to OpenDAL operations

### Authentication Flow

1. Client connects to WebDAV endpoint
2. Server prompts for authentication if needed
3. Credentials verified against the user database
4. Session established for authenticated user
5. All operations scoped to authenticated user's context

## Configuration

The WebDAV server is configurable through environment variables or a configuration file:
- Database connection information
- S3/storage configuration
- Server binding address and port
- TLS/certificate settings
- Authentication parameters

## Future Considerations

- WebDAV property storage and retrieval
- Additional authentication methods
- Performance optimization for large files
- Extended locking capabilities
- Custom property extensions for Obsidian
