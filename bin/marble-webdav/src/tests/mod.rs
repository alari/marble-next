pub mod mock_storage;
pub mod mock_auth;
pub mod mock_lock;
pub mod basic_operations;
pub mod copy_operations;
pub mod move_operations;
pub mod lock_tests;

// Re-export the mocks for use in tests
pub use mock_storage::MockTenantStorage;
pub use mock_auth::MockAuthService;
pub use mock_lock::MockLockManager;
