# Database Schema Questions and Considerations

**Status: [DRAFT]**

This document captures open questions and considerations about the database schema that need resolution.

## Open Questions

1. **Permalink Generation**
   - Should we enforce unique permalinks per user, or allow duplicates with a resolution strategy?
   - How should we handle permalink conflicts between different users?
   - Should we generate permalinks automatically from titles or require explicit specification?

2. **Frontmatter Storage**
   - Is the current structure sufficient, or should we normalize some of the frontmatter fields?
   - How flexible should we be with unknown frontmatter fields?
   - Should we support structured frontmatter with nested data?

3. **Content Hashing**
   - How should we handle conflicts if two different files produce the same content hash?
   - What hash algorithm provides the best balance of performance and collision resistance?
   - Should we use content-dependent or content-independent identifiers?

4. **Versioning Strategy**
   - How many versions should we keep per file?
   - Should there be a cleanup/archival process for old versions?
   - Should versions be created automatically or on explicit save points?

5. **Processing Queue**
   - Should we implement a separate table for processing results, or keep everything in the queue table?
   - How should we handle retries and error cases?
   - What priority mechanism should be used for processing items?

6. **Performance**
   - Are there any specific query patterns that might need denormalization or additional indexes?
   - How should we optimize for large vaults (1000+ notes)?
   - Should we consider partitioning for multi-tenant scalability?

7. **Error Handling**
   - How should we track and manage processing errors?
   - Should errors be stored directly in the queue table or in a separate error log?
   - What error recovery mechanisms should be implemented?

8. **Concurrency**
   - How should we handle concurrent updates to the same files or related metadata?
   - What locking strategy should be used to prevent conflicts?
   - How should we implement optimistic vs. pessimistic concurrency control?

9. **Link Navigation**
   - How should we optimize queries for finding next/previous published notes?
   - Should we pre-calculate navigation relationships or determine them dynamically?
   - How do we handle navigation across different sections or categories?

10. **Recursive Embeds**
    - What is the most efficient way to query and display recursively embedded content?
    - How do we prevent circular references in embedded content?
    - Should we impose limits on embedding depth?

## Technical Considerations

### Database Scaling

- How should we handle database scaling for multiple tenants?
- Would horizontal partitioning (sharding) be appropriate for tenant isolation?
- What metrics should we track to monitor database performance?

### Migration Strategy

- How do we handle schema evolution over time?
- What backward compatibility guarantees should we make?
- How should we test migrations thoroughly?

### Query Performance

- Which queries will be most common and need optimization?
- Should we implement materialized views for complex reporting queries?
- How can we minimize query complexity for common operations?

## Implementation Priorities

1. Core tables for basic functionality (users, folders, files)
2. Content relationship tracking (document_links)
3. Publishing infrastructure (frontmatter, published_content)
4. Advanced features (versioning, processing queue)
5. Performance optimizations and scaling

## Related Documents

- [Database Schema](database_schema.md) - Complete schema specification
- [Current Database Schema](database_schema_current.md) - Implemented tables
- [Marble Database Overview](../crates/marble_db_overview.md)
