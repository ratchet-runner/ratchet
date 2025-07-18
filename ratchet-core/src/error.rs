//! Core error types for Ratchet

use thiserror::Error;
use std::time::Duration;

pub mod standardized;
pub use standardized::{StandardizedError, ErrorCategory, ErrorMetadata, ToApiError};

/// Core error type for all Ratchet errors
#[derive(Debug, Error)]
pub enum RatchetError {
    /// Task-related errors
    #[error("Task error: {0}")]
    Task(#[from] TaskError),

    /// Execution-related errors
    #[error("Execution error: {0}")]
    Execution(#[from] ExecutionError),

    /// Simple execution error message
    #[error("Execution error: {0}")]
    ExecutionError(String),

    /// Storage-related errors
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    /// Validation errors
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),

    /// Service-related errors
    #[error("Service error: {0}")]
    Service(#[from] ServiceError),

    /// Plugin-related errors
    #[error("Plugin error: {0}")]
    Plugin(#[from] PluginError),

    /// Network/HTTP errors
    #[error("Network error: {0}")]
    Network(String),

    /// I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization/deserialization errors
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Timeout errors
    #[error("Operation timed out: {0}")]
    Timeout(String),

    /// Generic errors
    #[error("{0}")]
    Other(String),
}

/// Result type alias for Ratchet
pub type Result<T> = std::result::Result<T, RatchetError>;

/// Task-related errors
#[derive(Debug, Error)]
pub enum TaskError {
    #[error("Task not found: {0}")]
    NotFound(String),

    #[error("Task validation failed: {0}")]
    ValidationFailed(String),

    #[error("Task is disabled: {0}")]
    Disabled(String),

    #[error("Task version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: String, actual: String },

    #[error("Invalid task source: {0}")]
    InvalidSource(String),

    #[error("Task is deprecated: {0}")]
    Deprecated(String),
}

/// Execution-related errors
#[derive(Debug, Error)]
pub enum ExecutionError {
    #[error("Execution not found: {0}")]
    NotFound(String),

    #[error("Execution failed: {0}")]
    Failed(String),

    #[error("Execution cancelled")]
    Cancelled,

    #[error("Execution timed out after {0} seconds")]
    Timeout(u64),

    #[error("Invalid execution state: {0}")]
    InvalidState(String),

    #[error("Worker error: {0}")]
    WorkerError(String),
}

/// Storage-related errors
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Database connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Query failed: {0}")]
    QueryFailed(String),

    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    #[error("Migration failed: {0}")]
    MigrationFailed(String),

    #[error("Entity not found")]
    NotFound,

    #[error("Duplicate key: {0}")]
    DuplicateKey(String),
}

/// Configuration errors
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Missing required configuration: {0}")]
    MissingRequired(String),

    #[error("Invalid configuration value: {0}")]
    InvalidValue(String),

    #[error("Configuration file not found: {0}")]
    FileNotFound(String),

    #[error("Failed to parse configuration: {0}")]
    ParseError(String),

    #[error("Environment variable not set: {0}")]
    MissingEnvVar(String),
}

/// Validation errors
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Input validation failed: {0}")]
    InputValidation(String),

    #[error("Output validation failed: {0}")]
    OutputValidation(String),

    #[error("Schema validation failed: {0}")]
    SchemaValidation(String),

    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    #[error("Required field missing: {0}")]
    RequiredFieldMissing(String),
}

/// Service-related errors
#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Service not found: {0}")]
    NotFound(String),

    #[error("Service unavailable: {0}")]
    Unavailable(String),

    #[error("Service initialization failed: {0}")]
    InitializationFailed(String),

    #[error("Dependency injection failed: {0}")]
    DependencyInjectionFailed(String),
}

/// Plugin-related errors
#[derive(Debug, Error)]
pub enum PluginError {
    #[error("Plugin not found: {0}")]
    NotFound(String),

    #[error("Plugin load failed: {0}")]
    LoadFailed(String),

    #[error("Plugin initialization failed: {0}")]
    InitializationFailed(String),

    #[error("Plugin API version mismatch: expected {expected}, got {actual}")]
    ApiVersionMismatch { expected: String, actual: String },

    #[error("Plugin capability not supported: {0}")]
    CapabilityNotSupported(String),
}

/// Error context for debugging
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub operation: String,
    pub details: std::collections::HashMap<String, String>,
    pub source_location: Option<SourceLocation>,
}

/// Source location information
#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub file: String,
    pub line: u32,
    pub column: Option<u32>,
}

/// Extension trait for adding context to errors
pub trait ErrorExt<T> {
    /// Add a simple string context
    fn context(self, ctx: &str) -> Result<T>;

    /// Add detailed error context
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> ErrorContext;
}

impl<T> ErrorExt<T> for Result<T> {
    fn context(self, ctx: &str) -> Result<T> {
        self.map_err(|e| RatchetError::Other(format!("{}: {}", ctx, e)))
    }

    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> ErrorContext,
    {
        self.map_err(|e| {
            let ctx = f();
            log::error!("Error in {}: {} (details: {:?})", ctx.operation, e, ctx.details);
            e
        })
    }
}

/// Helper macro for creating errors with context
#[macro_export]
macro_rules! ratchet_error {
    ($err_type:ident :: $variant:ident, $msg:expr) => {
        $crate::error::RatchetError::from($crate::error::$err_type::$variant($msg.to_string()))
    };
    ($err_type:ident :: $variant:ident { $($field:ident : $value:expr),* }) => {
        $crate::error::RatchetError::from($crate::error::$err_type::$variant {
            $($field: $value.to_string()),*
        })
    };
}

// Implement the standardized error trait for RatchetError
impl StandardizedError for RatchetError {
    fn metadata(&self) -> ErrorMetadata {
        use std::collections::HashMap;

        let (code, category, retryable, retry_delay, http_status) = match self {
            RatchetError::Task(TaskError::NotFound(_)) => (
                "TASK_NOT_FOUND", ErrorCategory::NotFound, false, None, 404
            ),
            RatchetError::Task(TaskError::ValidationFailed(_)) => (
                "TASK_VALIDATION_FAILED", ErrorCategory::Validation, false, None, 400
            ),
            RatchetError::Task(TaskError::Disabled(_)) => (
                "TASK_DISABLED", ErrorCategory::Client, false, None, 403
            ),
            RatchetError::Task(TaskError::Deprecated(_)) => (
                "TASK_DEPRECATED", ErrorCategory::Client, false, None, 410
            ),
            RatchetError::Task(TaskError::VersionMismatch { .. }) => (
                "TASK_VERSION_MISMATCH", ErrorCategory::Client, false, None, 400
            ),
            RatchetError::Task(TaskError::InvalidSource(_)) => (
                "TASK_INVALID_SOURCE", ErrorCategory::Validation, false, None, 400
            ),
            RatchetError::Execution(ExecutionError::NotFound(_)) => (
                "EXECUTION_NOT_FOUND", ErrorCategory::NotFound, false, None, 404
            ),
            RatchetError::Execution(ExecutionError::Failed(_)) => (
                "EXECUTION_FAILED", ErrorCategory::Server, false, None, 500
            ),
            RatchetError::Execution(ExecutionError::Cancelled) => (
                "EXECUTION_CANCELLED", ErrorCategory::Client, false, None, 400
            ),
            RatchetError::Execution(ExecutionError::Timeout(_)) => (
                "EXECUTION_TIMEOUT", ErrorCategory::Network, true, Some(Duration::from_secs(2)), 408
            ),
            RatchetError::Execution(ExecutionError::InvalidState(_)) => (
                "EXECUTION_INVALID_STATE", ErrorCategory::Client, false, None, 400
            ),
            RatchetError::Execution(ExecutionError::WorkerError(_)) => (
                "EXECUTION_WORKER_ERROR", ErrorCategory::Server, true, Some(Duration::from_secs(1)), 500
            ),
            RatchetError::ExecutionError(_) => (
                "EXECUTION_ERROR", ErrorCategory::Server, false, None, 500
            ),
            RatchetError::Storage(StorageError::NotFound) => (
                "ENTITY_NOT_FOUND", ErrorCategory::NotFound, false, None, 404
            ),
            RatchetError::Storage(StorageError::ConnectionFailed(_)) => (
                "STORAGE_CONNECTION_FAILED", ErrorCategory::Network, true, Some(Duration::from_secs(1)), 503
            ),
            RatchetError::Storage(StorageError::QueryFailed(_)) => (
                "STORAGE_QUERY_FAILED", ErrorCategory::Server, false, None, 500
            ),
            RatchetError::Storage(StorageError::TransactionFailed(_)) => (
                "STORAGE_TRANSACTION_FAILED", ErrorCategory::Server, true, Some(Duration::from_millis(500)), 500
            ),
            RatchetError::Storage(StorageError::MigrationFailed(_)) => (
                "STORAGE_MIGRATION_FAILED", ErrorCategory::Configuration, false, None, 500
            ),
            RatchetError::Storage(StorageError::DuplicateKey(_)) => (
                "STORAGE_DUPLICATE_KEY", ErrorCategory::Client, false, None, 409
            ),
            RatchetError::Config(_) => (
                "CONFIG_ERROR", ErrorCategory::Configuration, false, None, 500
            ),
            RatchetError::Validation(_) => (
                "VALIDATION_ERROR", ErrorCategory::Validation, false, None, 400
            ),
            RatchetError::Service(ServiceError::NotFound(_)) => (
                "SERVICE_NOT_FOUND", ErrorCategory::NotFound, false, None, 404
            ),
            RatchetError::Service(ServiceError::Unavailable(_)) => (
                "SERVICE_UNAVAILABLE", ErrorCategory::Network, true, Some(Duration::from_secs(5)), 503
            ),
            RatchetError::Service(ServiceError::InitializationFailed(_)) => (
                "SERVICE_INITIALIZATION_FAILED", ErrorCategory::Configuration, false, None, 500
            ),
            RatchetError::Service(ServiceError::DependencyInjectionFailed(_)) => (
                "SERVICE_DEPENDENCY_INJECTION_FAILED", ErrorCategory::Configuration, false, None, 500
            ),
            RatchetError::Plugin(PluginError::NotFound(_)) => (
                "PLUGIN_NOT_FOUND", ErrorCategory::NotFound, false, None, 404
            ),
            RatchetError::Plugin(PluginError::LoadFailed(_)) => (
                "PLUGIN_LOAD_FAILED", ErrorCategory::Configuration, false, None, 500
            ),
            RatchetError::Plugin(PluginError::InitializationFailed(_)) => (
                "PLUGIN_INITIALIZATION_FAILED", ErrorCategory::Configuration, false, None, 500
            ),
            RatchetError::Plugin(PluginError::ApiVersionMismatch { .. }) => (
                "PLUGIN_API_VERSION_MISMATCH", ErrorCategory::Configuration, false, None, 500
            ),
            RatchetError::Plugin(PluginError::CapabilityNotSupported(_)) => (
                "PLUGIN_CAPABILITY_NOT_SUPPORTED", ErrorCategory::Client, false, None, 400
            ),
            RatchetError::Network(_) => (
                "NETWORK_ERROR", ErrorCategory::Network, true, Some(Duration::from_secs(1)), 503
            ),
            RatchetError::Io(_) => (
                "IO_ERROR", ErrorCategory::Server, true, Some(Duration::from_millis(500)), 500
            ),
            RatchetError::Serialization(_) => (
                "SERIALIZATION_ERROR", ErrorCategory::Client, false, None, 400
            ),
            RatchetError::Timeout(_) => (
                "TIMEOUT", ErrorCategory::Network, true, Some(Duration::from_secs(2)), 408
            ),
            RatchetError::Other(_) => (
                "INTERNAL_ERROR", ErrorCategory::Server, false, None, 500
            ),
        };

        ErrorMetadata {
            code: code.to_string(),
            http_status,
            retryable,
            retry_delay,
            category,
            context: HashMap::new(),
        }
    }
}

impl RatchetError {
    /// Check if this error is retryable (backward compatibility)
    pub fn is_retryable(&self) -> bool {
        StandardizedError::is_retryable(self)
    }

    /// Get the error code for API responses (backward compatibility)
    pub fn error_code(&self) -> &str {
        // We need to return a &str, but StandardizedError returns String
        // For backward compatibility, we'll keep the original implementation
        match self {
            RatchetError::Task(TaskError::NotFound(_)) => "TASK_NOT_FOUND",
            RatchetError::Execution(ExecutionError::NotFound(_)) => "EXECUTION_NOT_FOUND",
            RatchetError::Storage(StorageError::NotFound) => "ENTITY_NOT_FOUND",
            RatchetError::Task(TaskError::Disabled(_)) => "TASK_DISABLED",
            RatchetError::Task(TaskError::Deprecated(_)) => "TASK_DEPRECATED",
            RatchetError::Validation(_) => "VALIDATION_ERROR",
            RatchetError::Config(_) => "CONFIG_ERROR",
            RatchetError::Timeout(_) => "TIMEOUT",
            RatchetError::Network(_) => "NETWORK_ERROR",
            RatchetError::Service(ServiceError::Unavailable(_)) => "SERVICE_UNAVAILABLE",
            _ => "INTERNAL_ERROR",
        }
    }

    /// Get the HTTP status code for this error (backward compatibility)
    pub fn status_code(&self) -> u16 {
        StandardizedError::http_status(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_retryable() {
        assert!(RatchetError::Network("timeout".to_string()).is_retryable());
        assert!(RatchetError::Timeout("30s".to_string()).is_retryable());
        assert!(!RatchetError::Validation(ValidationError::InvalidFormat("json".to_string())).is_retryable());
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(
            RatchetError::Task(TaskError::NotFound("123".to_string())).error_code(),
            "TASK_NOT_FOUND"
        );
        assert_eq!(
            RatchetError::Validation(ValidationError::InvalidFormat("json".to_string())).error_code(),
            "VALIDATION_ERROR"
        );
    }

    #[test]
    fn test_status_codes() {
        assert_eq!(
            RatchetError::Task(TaskError::NotFound("123".to_string())).status_code(),
            404
        );
        assert_eq!(
            RatchetError::Validation(ValidationError::InvalidFormat("json".to_string())).status_code(),
            400
        );
        assert_eq!(RatchetError::Timeout("30s".to_string()).status_code(), 408);
    }

    #[test]
    fn test_error_macro() {
        let err = ratchet_error!(TaskError::NotFound, "test-task");
        match err {
            RatchetError::Task(TaskError::NotFound(msg)) => {
                assert_eq!(msg, "test-task");
            }
            _ => panic!("Wrong error type"),
        }
    }
}
