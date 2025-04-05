# Marble WebDAV Server Specification

## Overview

The `marble-webdav` binary provides a WebDAV interface for clients (particularly Obsidian) to interact with the Marble system. It acts as the primary entry point for content creation and consumption, handling authentication, routing, and enabling tenant-isolated access to the storage system.

## Responsibilities

- Provide a WebDAV interface compatible with Obsidian and OS-level WebDAV clients
- Implement authentication and authorization for users
- Map users to tenant IDs for all storage operations
- Enforce tenant isolation for all content access
- Manage WebDAV locking for concurrent modifications
- Translate WebDAV operations to `TenantStorage` API calls

## Architecture

The WebDAV server consists of the following components:

1. **Server Framework**:
   - Built using the standard `dav-server` crate (not OpenDAL variant)
   - Handles WebDAV methods (GET, PUT, PROPFIND, etc.)
   - Manages WebDAV locking for concurrent access
   - Integrates with Axum for HTTP serving

2. **Authentication System**:
   - Validates user credentials against the database
   - Maps usernames to tenant UUIDs
   - Isolates tenant content to prevent cross-tenant access
   - Implements `AuthService` trait for credential verification

3. **TenantStorage Integration**:
   - Uses the `TenantStorage` API for all storage operations
   - Ensures tenant isolation by passing tenant ID to all operations
   - Translates WebDAV paths to storage paths

## Endpoints

The server exposes a single WebDAV endpoint for authenticated access:

1. **WebDAV Root Endpoint** (`/`):
   - Authenticated access only
   - Read-write permission
   - Direct mapping to tenant's content
   - Used for synchronizing Obsidian vaults
   - Each user's credentials map to their tenant UUID

## Implementation Details

### WebDAV Integration

The server uses `dav-server` to handle WebDAV protocol specifics:
- Implements the WebDAV HTTP extension methods
- Manages WebDAV properties
- Provides locking functionality for concurrent access
- Handles file system-like operations over HTTP

### TenantStorage Integration

The WebDAV server directly integrates with the `TenantStorage` API:
- Creates a `MarbleDavHandler` that wraps `TenantStorage` operations
- Maps WebDAV methods to corresponding `TenantStorage` calls
- Passes tenant ID (from authentication) to all storage operations
- Normalizes paths between WebDAV and storage formats

### Authentication Flow

1. Client connects to WebDAV endpoint
2. Server extracts Basic Auth credentials from the request
3. Credentials verified against the user database via `AuthService`
4. Username mapped to tenant UUID for storage operations
5. All operations scoped to authenticated tenant's context

## Connection with Obsidian

The WebDAV server is specifically designed to work with Obsidian:
- Compatible with Obsidian's WebDAV sync feature
- Supports all Obsidian-specific file operations
- Handles .obsidian directory and settings files
- Optimized for performance with large vaults

## Configuration

The WebDAV server is configurable through environment variables or a configuration file:
- Database connection information
- Server binding address and port
- TLS/certificate settings
- Authentication parameters
- Lock management settings

## Implementation Phases

The implementation follows a phased approach:

1. **Phase 1**: Core infrastructure, authentication, and path handling
2. **Phase 2**: Basic WebDAV methods (GET, PUT, PROPFIND, MKCOL, DELETE)
3. **Phase 3**: Advanced methods (COPY, MOVE, LOCK, UNLOCK)
4. **Phase 4**: Obsidian-specific optimizations and advanced features
