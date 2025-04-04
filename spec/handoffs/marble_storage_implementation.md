# Marble Storage Implementation Handoff

**Last Updated: 2025-04-04**

## Current Status
We've made significant progress on Phase 3 (tenant isolation through database metadata) and started Phase 4 (OpenDAL integration). The `RawStorageBackend` has been implemented with database integration to map file paths to content hashes, enforcing tenant isolation through user IDs. The code now successfully compiles, though the OpenDAL adapter is currently a placeholder that needs to be completed.

## User Identification and Authentication

### Dual ID System
Marble uses a dual approach to user identification:

1. **Internal Database IDs (i32)**:
   - Used as primary keys in the database
   - Used for database relationships and foreign keys
   - Used internally by repositories

2. **UUIDs (Universally Unique Identifiers)**:
   - Used for external-facing user identification
   - Used in the `MarbleStorage` API
   - Provides security by not exposing internal database IDs

The `uuid_to_db_id` function bridges these two systems, looking up the internal ID from the UUID.

### Authentication
- The `username` field is used for authentication in both write and read sides
- The WebDAV interface uses username/password authentication
- Passwords are stored as hashes in the `password_hash` field
- Authentication happens before storage operations

### Path Structure
- Usernames are used in path structures for processed content
- All processed paths are prefixed with username: `/{username}/...`
- This ensures tenant isolation at the path level

## Key Insights from Implementation

### OpenDAL API Usage
- OpenDAL's API requires a two-step process to create an operator:
  1. Create an operator builder with `Operator::new(builder)`
  2. Finish the builder with `.finish()` to get the actual operator
- Custom layers need special handling and are not implemented yet
- The borrowing checker requires `Vec<u8>` instead of `&[u8]` when writing content in async functions

### OpenDAL Custom Adapters
- Creating a custom OpenDAL adapter requires implementing the RawAccessor trait
- The adapter needs to implement various Rp* traits for read, write, delete, and list operations
- Async operations are particularly complex and require implementing custom AsyncRead and AsyncWrite wrappers
- The adapter needs to integrate with our backend to provide tenant isolation

### Database Integration
- File paths are mapped to content hashes in the database
- User ID scoping enforces tenant isolation
- File operations check for the user_id to ensure proper authorization
- The database maintains metadata like file paths, deletion status, and content types

### Error Handling
- Database errors must be properly mapped to storage errors
- Different repositories may use different error types
- Match expressions often provide cleaner error handling than map_err chains

## Implementation Plan

We will focus exclusively on the write side storage implementation in this phase, leaving the read side for a future project. The plan is structured in phases:

### Phase 1: Setup and Dependencies ✅
1. Add OpenDAL with S3 support to dependencies
2. Define a consistent error handling strategy with a dedicated `StorageError` type
3. Implement a configuration system for storage backends to support different environments

### Phase 2: Raw Storage Implementation ✅
1. Create content-addressable hashed storage for raw data
   - Implement storage with `/.hash/{hash}` addressing scheme
   - Cover with unit tests using OpenDAL's file backend
   - Ensure proper error handling and validation
   - This storage will be shared across all tenants since content is addressed by hash

### Phase 3: Tenant Isolation through Metadata ✅
1. Implement tenant isolation primarily through database metadata ✅
   - Store user_id with all file/path metadata in the database
   - Ensure all queries are scoped to the specific user_id
   - No need for tenant-specific partitioning in the hash-based raw storage
   - Use proper authentication and authorization checks before operations
2. Create utilities for user ID conversion and lookup ✅
   - Support conversion between UUID and database user ID

### Phase 4: OpenDAL Integration 🔄
1. Create OpenDAL backend that integrates with marble-db for metadata 🔄
   - Implement adapter skeleton ✅
   - Implement the `raw_storage()` method in MarbleStorageImpl ✅
   - Add database connection support to MarbleStorageImpl ✅
   - Complete the OpenDAL adapter implementation ⏳
   - Implement proper read/write/list/delete operations in the adapter ⏳

### Phase 5: Testing and Validation ⏳
1. Develop comprehensive integration tests
   - Verify isolation between different user contexts
   - Test concurrent operations from different users
   - Validate that users cannot access others' content

### Phase 6: Documentation and Finalization ⏳
1. Update marble-storage specification to match implementation
2. Create usage examples and API documentation
3. Implement any remaining features for the write side

## Progress Update
We've implemented the following components:

1. **RawStorageBackend**: A backend that enforces tenant isolation through database integration
   - File operations (read, write, delete, list) scoped to specific user_id
   - Integration with ContentHasher for deduplication
   - Proper error handling for database operations

2. **User ID Conversion**: Utilities to convert between UUID and database user ID
   - `uuid_to_db_id()` function for lookup and conversion
   - Security checks for user existence
   - Error handling for database lookup failures

3. **OpenDAL Adapter Skeleton**: Started implementing the adapter
   - Basic structure defined
   - Placeholder for full implementation
   - Error handling for unsupported operations

4. **MarbleStorageImpl Updates**: Enhanced to support database connections
   - Added `new_with_db()` method to create with database connection
   - Updated `raw_storage()` method to integrate with RawStorageBackend
   - Added database pool validation and user ID conversion

The code now compiles successfully, though there are some expected warnings about unused code since we're implementing in phases.

## Next Steps
1. Complete the OpenDAL adapter implementation:
   - Implement custom OpenDAL layers for raw storage operations
   - Support read/write/delete/list operations properly
   - Ensure proper error handling and tenant isolation

2. Add comprehensive tests for the OpenDAL adapter
   - Verify that tenant isolation works correctly
   - Test all operations with proper user contexts

3. Update documentation and examples
   - Update marble-storage specification
   - Create usage examples

## References
- [Storage Architecture](../domain/storage_architecture.md)
- [Marble Storage Specification](../crates/marble_storage.md)
- [Database Schema](../domain/database_schema.md)
