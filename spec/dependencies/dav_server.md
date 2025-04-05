# dav-server Specification

**Status:** IMPLEMENTED
**Last Updated:** 2025-04-05

## Overview

`dav-server` is a Rust library that implements the WebDAV protocol (RFC 4918), providing a framework for building WebDAV servers. It supports the core WebDAV methods (GET, PUT, PROPFIND, etc.) and features like property storage and lock management.

## Usage in Marble

Marble uses `dav-server` to provide a WebDAV interface that allows direct integration with Obsidian and other WebDAV clients. This enables users to synchronize their notes seamlessly with the Marble platform.

- The `marble-webdav` crate implements a WebDAV server using this library
- It integrates with our `TenantStorage` API for tenant-isolated file operations
- It provides the primary interface for write-side operations in our system

## Version and Features

- Current version: 0.7.0 (previously considered 0.5.9)
- Required features: Default features are sufficient for our needs
- Version constraints: We're using the latest version as of April 2025

## Integration Approach

We've decided to use a layered integration approach where:

1. `dav-server` handles WebDAV protocol specifics
2. Our custom `MarbleDavHandler` connects WebDAV operations to `TenantStorage`
3. Authentication extracts tenant IDs for proper data isolation

This approach lets us leverage the WebDAV protocol implementation while maintaining our multi-tenant architecture.

## Key APIs and Patterns

The primary integration point is through the `DavHandler` trait:

```rust
// Implementing the DavHandler trait for our custom handler
impl DavHandler for MarbleDavHandler {
    async fn handle(
        &self,
        method: DavMethod,
        path: &str,
        headers: HeaderMap,
        body: Body,
    ) -> Result<DavResponse, Error> {
        // Extract tenant ID from authentication
        // Normalize path for storage operations
        // Dispatch to appropriate method handler
    }
}
```

## Error Handling

Errors from `dav-server` are converted to our application-specific error types:

- WebDAV protocol errors are mapped to appropriate HTTP status codes
- Authentication failures return 401 Unauthorized
- Storage errors are mapped to 404 Not Found or 500 Internal Server Error as appropriate
- We maintain detailed error context for diagnostic purposes

## Alternatives Considered

- **dav-server-opendalfs**: Initially considered to leverage OpenDAL integration, but the implementation complexity outweighed benefits
- **Custom WebDAV implementation**: Considered but rejected due to protocol complexity and maintenance burden
- **webdav-handler**: Older library with less active maintenance

We chose `dav-server` because:
1. It has active maintenance
2. It supports all required WebDAV features
3. It has a clean API that integrates well with our architecture
4. It doesn't force us into a specific storage model

## Performance Characteristics

- Handles concurrent WebDAV operations efficiently
- Lock management can be a bottleneck under high contention
- PROPFIND operations on large directories may require optimization
- Memory usage scales primarily with concurrent connection count

## Security Considerations

- Authentication is handled separately from the WebDAV protocol
- Tenant isolation is enforced through our custom handler
- Path traversal attacks are prevented by normalizing paths
- Lock tokens must be properly validated to prevent unauthorized access

## Related Specifications

- [WebDAV Implementation](../handoffs/webdav_implementation.md)
- [Marble Storage](../crates/marble_storage.md)
- [Authentication](../domain/authentication.md)

## Future Considerations

- Potential future upgrade to newer versions as they're released
- Supporting additional WebDAV extensions as needed for clients
- Performance optimizations for large directories
- Adding support for WebDAV access control extensions if multi-user tenants are implemented
