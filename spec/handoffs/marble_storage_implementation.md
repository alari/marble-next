# Marble Storage Implementation Handoff

**Last Updated: 2025-04-04**

## Current Status
Phase 2 of the implementation plan is complete. We've implemented the content-addressable hashed storage with the `/.hash/{hash}` scheme, including support for both filesystem and S3 backends. A `ContentHasher` service manages the content hashing and storage operations.

## Implementation Plan

We will focus exclusively on the write side storage implementation in this phase, leaving the read side for a future project. The plan is structured in phases:

### Phase 1: Setup and Dependencies
1. Add OpenDAL with S3 support to dependencies
2. Define a consistent error handling strategy with a dedicated `StorageError` type
3. Implement a configuration system for storage backends to support different environments

### Phase 2: Raw Storage Implementation
1. Create content-addressable hashed storage for raw data
   - Implement storage with `/.hash/{hash}` addressing scheme
   - Cover with unit tests using OpenDAL's file backend
   - Ensure proper error handling and validation
   - This storage will be shared across all tenants since content is addressed by hash

### Phase 3: Tenant Isolation through Metadata
1. Implement tenant isolation primarily through database metadata
   - Store user_id with all file/path metadata in the database
   - Ensure all queries are scoped to the specific user_id
   - No need for tenant-specific partitioning in the hash-based raw storage
   - Use proper authentication and authorization checks before operations

### Phase 4: Metadata Integration
1. Create OpenDAL backend that integrates with marble-db for metadata
   - Implement path-to-hash lookup via database (scoped to user_id)
   - Track file versions and modifications
   - Support folder structure operations
   - Ensure all operations maintain user_id scope for isolation

### Phase 5: Testing and Validation
1. Develop comprehensive integration tests
   - Verify isolation between different user contexts
   - Test concurrent operations from different users
   - Validate that users cannot access others' content

### Phase 6: Documentation and Finalization
1. Update marble-storage specification to match implementation
2. Create usage examples and API documentation
3. Implement any remaining features for the write side

## Key Insights
- Tenant isolation is primarily enforced through user_id in database metadata
- Content in the hash-based raw storage can be shared across all tenants (deduplicated)
- The read side implementation (with path-based tenant isolation) is a separate future project

## Next Steps
1. Begin Phase 3 by implementing tenant isolation through database metadata
2. Integrate with marble-db to store and retrieve file path to hash mappings
3. Implement user_id scoping for all database queries to ensure tenant isolation

## References
- [Storage Architecture](../domain/storage_architecture.md)
- [Marble Storage Specification](../crates/marble_storage.md)
- [Database Schema](../domain/database_schema.md)
