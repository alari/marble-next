# tower-http Specification

**Status:** IMPLEMENTED
**Last Updated:** 2025-04-05

## Overview

Tower-HTTP is a collection of HTTP-specific middleware and utilities built on top of the Tower service abstraction. It provides common functionality needed for HTTP services, such as compression, tracing, authentication, CORS, and request/response transformation.

## Usage in Marble

In the Marble project, Tower-HTTP provides:

- Tracing middleware for HTTP request/response logging
- Authentication handling for the WebDAV server
- Request validation and transformation

## Version and Features

- Current version: 0.5.2
- Required features: 
  - `trace` - For HTTP request/response tracing
  - `auth` - For authentication middleware
- Version constraints: Compatible with Tower 0.4.x and Axum 0.8.x

## Configuration

Example configuration in our WebDAV server:

```rust
// Add HTTP-specific middleware
let app = Router::new()
    .route("/*path", any(handle_webdav))
    // Add tracing middleware
    .layer(TraceLayer::new_for_http()
        .make_span_with(|request: &Request<_>| {
            tracing::info_span!(
                "http_request",
                method = %request.method(),
                uri = %request.uri(),
                version = ?request.version(),
            )
        })
        .on_request(|request: &Request<_>, _span: &Span| {
            tracing::debug!("processing request");
        })
        .on_response(|response: &Response, latency: Duration, _span: &Span| {
            tracing::debug!("response generated in {:?}", latency);
        })
    )
    .with_state(state);
```

## Key APIs and Patterns

Key Tower-HTTP components we use:

1. **TraceLayer** - For request/response tracing:
   ```rust
   .layer(TraceLayer::new_for_http())
   ```

2. **AuthLayer** - For basic authentication (when implemented):
   ```rust
   .layer(AuthLayer::new(auth_service))
   ```

3. **Request/Response transformation** - For customizing HTTP behavior

## Error Handling

- Middleware errors are propagated through the service stack
- TraceLayer captures error information in spans
- Authentication middleware can reject requests with appropriate HTTP status codes
- Centralized error handling at the Axum handler level

## Alternatives Considered

Tower-HTTP is the standard HTTP middleware collection for Tower and Axum. Alternatives would involve:

- Writing custom middleware (more maintenance burden)
- Using framework-specific middleware (less portable)
- Handling cross-cutting concerns in application code (poor separation of concerns)

We chose Tower-HTTP because:
1. It integrates seamlessly with Axum and Tower
2. It provides high-quality implementations of common HTTP middleware
3. It follows best practices for middleware design
4. It's actively maintained

## Performance Characteristics

- Minimal overhead for middleware processing
- Tracing has configurable verbosity for performance tuning
- Memory usage correlates with tracing data and request complexity
- Scales well with Tokio's async runtime

## Security Considerations

- Authentication middleware must be properly configured
- Middleware ordering affects security guarantees
- Tracing middleware should be configured to avoid logging sensitive information
- HTTP headers should be validated to prevent injection attacks

## Related Specifications

- [WebDAV Implementation](../handoffs/webdav_implementation.md)
- [Axum](axum.md)
- [Tower](tower.md)
- [Authentication](../domain/authentication.md)

## Future Considerations

- Adding more middleware as project requirements evolve
- Tuning tracing verbosity based on production experience
- Implementing custom middleware for WebDAV-specific concerns
- Monitoring middleware performance in production
