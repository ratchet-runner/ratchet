//! Testing utilities for ratchet-storage
//!
//! This module provides comprehensive testing infrastructure including:
//! - Database test utilities with automatic cleanup
//! - Mock repository implementations using mockall
//! - Test data builders with the builder pattern
//! - File-based testing fixtures for tasks and configurations
//!
//! ## Feature Requirements
//!
//! The testing module requires both the `testing` and `seaorm` features to be enabled.
//! Most testing utilities depend on SeaORM entities and database functionality.
//!
//! ```toml
//! [dependencies]
//! ratchet-storage = { features = ["testing"] }  # Automatically includes seaorm
//! ```
//!
//! ## Available Testing Utilities
//!
//! - `TestDatabase` - Creates isolated test databases with automatic cleanup
//! - `MockFactory` - Generates mock repository implementations with configurable behavior
//! - Builder patterns - `TaskBuilder`, `ExecutionBuilder`, etc. for creating test data
//! - Test fixtures - File-based testing utilities for tasks and configurations

pub mod builders;
pub mod database;
pub mod mocks;

#[cfg(feature = "testing")]
pub mod fixtures;

// Re-export commonly used testing utilities
#[cfg(all(feature = "testing", feature = "seaorm"))]
pub use builders::{factories, DeliveryResultBuilder, ExecutionBuilder, JobBuilder, ScheduleBuilder, TaskBuilder};
#[cfg(all(feature = "testing", feature = "seaorm"))]
pub use database::{SharedTestDatabase, TestDatabase, TestDatabaseError};
#[cfg(all(feature = "testing", feature = "seaorm"))]
pub use mocks::{mock_errors, MockExecutionRepo, MockFactory, MockJobRepo, MockScheduleRepo, MockTaskRepo};

#[cfg(feature = "testing")]
pub use fixtures::{FixtureBuilder, TestFixtures};

// Re-export testing macros
#[cfg(all(feature = "testing", feature = "seaorm"))]
pub use crate::{test_with_db, test_with_seeded_db};
