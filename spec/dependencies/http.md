# http Specification

**Status:** IMPLEMENTED
**Last Updated:** 2025-04-05

## Overview

The `http` crate provides a set of general-purpose HTTP types that are shared by HTTP clients and servers. It includes types for representing HTTP requests, responses, headers, methods, status codes, and versions.

## Usage in Marble

In the Marble project, the `http` crate provides:

- Core HTTP types for our WebDAV server
- Standardized header handling
- HTTP method and status code definitions
- Consistent HTTP request/response types

## Version and Features

- Current version: 1.3.1
- Required features: Default features are sufficient
- Version constraints: Compatible with Axum 0.8.x and Tower 0.4.x

## Configuration

The `http` crate doesn't require specific configuration but is used throughout our WebDAV code:

```rust
use http::{HeaderMap, Method, StatusCode, Uri};

// Used in handler functions
async fn handle_webdav(
    headers: HeaderMap,
    method: Method,
    uri: Uri,
    // ...
) -> impl IntoResponse {
    // ...
}
```

## Key APIs and Patterns

Important `http` types we use:

1. **HeaderMap** - For HTTP headers:
   ```rust
   let auth_header = headers
       .get(http::header::AUTHORIZATION)
       .and_then(|h| h.to_str().ok());
   ```

2. **Method** - For HTTP methods:
   ```rust
   match method.as_str() {
       "GET" => DavMethod::Get,
       "PUT" => DavMethod::Put,
       // ...
   }
   ```

3. **StatusCode** - For HTTP response codes:
   ```rust
   let status_code = match error {
       Error::Auth(_) => StatusCode::UNAUTHORIZED,
       Error::Storage(_) => StatusCode::NOT_FOUND,
       _ => StatusCode::INTERNAL_SERVER_ERROR,
   };
   ```

## Error Handling

- The `http` crate itself has minimal error types
- It's primarily used for type definitions
- Integration with response generation in Axum
- Proper status code selection for error responses

## Alternatives Considered

The `http` crate is a standard foundation for Rust HTTP libraries, so no alternatives were considered. It provides:

- A consistent API for HTTP concepts
- Shared types used by multiple HTTP libraries
- Well-designed and type-safe interfaces
- Wide adoption in the Rust ecosystem

## Performance Characteristics

- Efficient representation of HTTP constructs
- Minimal memory overhead for core types
- Optimized header storage and access patterns
- Fast conversion between string and typed representations

## Security Considerations

- Header validation is required to prevent injection attacks
- URI parsing should handle malicious input
- Method verification is important for proper authorization
- Header values should be sanitized before use

## Related Specifications

- [WebDAV Implementation](../handoffs/webdav_implementation.md)
- [Axum](axum.md)
- [Tower HTTP](tower_http.md)

## Future Considerations

- The `http` crate has a stable API with minimal breaking changes expected
- Future HTTP standards may be incorporated in later versions
- Integration with HTTP/2 and HTTP/3 features
- Potential expansion to support newer HTTP header standards
