# axum Specification

**Status:** IMPLEMENTED
**Last Updated:** 2025-04-05

## Overview

Axum is a web application framework built on top of Tokio, Tower, and Hyper. It's focused on ergonomics and modularity, providing a robust foundation for building web services in Rust.

## Usage in Marble

In the Marble project, Axum serves as the HTTP server framework for our WebDAV interface:

- It handles all HTTP requests for the WebDAV endpoint
- It manages routing and middleware for the WebDAV server
- It integrates with our authentication system
- It provides the HTTP server implementation for the `marble-webdav` binary

## Version and Features

- Current version: 0.8.3
- Required features: Default features are sufficient for our core needs
- Version constraints: We need version 0.8.x or later for the current API

## Configuration

Basic Axum configuration for our WebDAV server:

```rust
// Create router with routes and middleware
let app = Router::new()
    .route("/*path", any(handle_webdav))
    .route("/", any(handle_webdav))
    .layer(TraceLayer::new_for_http())
    .with_state(state);

// Start server
axum::serve(listener, app).await?;
```

## Key APIs and Patterns

We use these key Axum patterns:

1. **State management** for sharing the WebDAV handler:
   ```rust
   async fn handle_webdav(
       State(state): State<Arc<WebDavState>>,
       // Other extractors...
   ) -> impl IntoResponse
   ```

2. **Routing with wildcards** to capture WebDAV paths:
   ```rust
   .route("/*path", any(handle_webdav))
   ```

3. **HTTP headers and method extraction** for WebDAV operations:
   ```rust
   async fn handle_webdav(
       // ...
       headers: HeaderMap,
       method: Method,
       uri: Uri,
       body: Bytes,
   ) -> impl IntoResponse
   ```

## Error Handling

Error handling approach:

- WebDAV errors are mapped to appropriate HTTP status codes
- Structured responses use Axum's `IntoResponse` trait
- Tracing provides detailed logs for debugging
- Internal errors are converted to standardized error responses

## Alternatives Considered

- **actix-web**: More mature but has a higher learning curve
- **warp**: Good but less active development than Axum
- **rocket**: More opinionated and requires nightly Rust for some features
- **hyper directly**: Too low-level for our needs

We chose Axum because:
1. It's built on solid foundations (Tokio, Tower, Hyper)
2. It has excellent ergonomics for handler functions
3. It has strong typing for request and response handling
4. It has active development and community support
5. It has a clean, modular middleware system

## Performance Characteristics

- Excellent performance for HTTP request handling
- Low overhead for routing and middleware
- Scales well with Tokio's async runtime
- Memory footprint grows with concurrent connections and middleware complexity

## Security Considerations

- Axum itself doesn't include authentication or authorization
- We implement these concerns ourselves using middleware or handler logic
- Headers are accessible for inspection and validation
- Proper request validation must be implemented separately

## Related Specifications

- [WebDAV Implementation](../handoffs/webdav_implementation.md)
- [Marble WebDAV](../crates/marble_webdav.md)
- [Authentication](../domain/authentication.md)

## Future Considerations

- Potential upgrade path is straightforward with Axum's semantic versioning
- API is stable with minimal breaking changes expected
- Expanding capabilities with additional Tower middleware as needed
- Monitoring performance with Axum tracing integration
