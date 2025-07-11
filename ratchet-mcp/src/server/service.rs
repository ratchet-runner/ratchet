//! MCP service implementation for integration with Ratchet's service architecture

use async_trait::async_trait;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tracing::{error, info};

use ratchet_execution::ProcessTaskExecutor;
use ratchet_interfaces::{Service, ServiceHealth, ServiceMetrics, TaskService};
use ratchet_storage::seaorm::repositories::{
    execution_repository::ExecutionRepository,
};

use crate::security::{AuditLogger, McpAuthManager, SecurityConfig};
use crate::server::{McpServer, McpServerConfig, McpServerTransport, RatchetMcpAdapter, RatchetToolRegistry};
use crate::{McpAuth, McpError, McpResult};

/// MCP service configuration
#[derive(Debug, Clone)]
pub struct McpServiceConfig {
    /// Server configuration
    pub server_config: McpServerConfig,
    /// Optional log file path for enhanced logging
    pub log_file_path: Option<std::path::PathBuf>,
}

impl Default for McpServiceConfig {
    fn default() -> Self {
        Self {
            server_config: McpServerConfig {
                transport: McpServerTransport::Stdio,
                security: SecurityConfig::default(),
                bind_address: None,
            },
            log_file_path: None,
        }
    }
}

/// MCP service that can be integrated into Ratchet's service architecture
pub struct McpService {
    /// MCP server instance
    server: Arc<McpServer>,
    /// Server task handle (for SSE transport)
    server_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    /// Service configuration
    config: McpServiceConfig,
    /// Service metrics
    metrics: Arc<Mutex<ServiceMetrics>>,
    /// Service start time
    start_time: std::time::Instant,
}

impl McpService {
    /// Create a new MCP service with repositories and executor
    pub async fn new(
        config: McpServiceConfig,
        task_executor: Arc<ProcessTaskExecutor>,
        task_service: Arc<dyn ratchet_interfaces::TaskService>,
        execution_repository: Arc<ExecutionRepository>,
    ) -> McpResult<Self> {
        // Create the MCP adapter
        let adapter = if let Some(log_path) = &config.log_file_path {
            RatchetMcpAdapter::with_log_file(task_executor, task_service, execution_repository, log_path.clone())
        } else {
            RatchetMcpAdapter::new(task_executor, task_service, execution_repository)
        };

        // Create tool registry with the adapter
        let mut tool_registry = RatchetToolRegistry::new();
        tool_registry.set_executor(Arc::new(adapter));

        // Create security components from configuration
        let auth_manager = Arc::new(McpAuthManager::new(McpAuth::None)); // TODO: Phase 3 - Add auth config to SecurityConfig
        let audit_logger = Arc::new(AuditLogger::new(config.server_config.security.audit_log_enabled));

        // Create MCP server
        let server = McpServer::new(
            config.server_config.clone(),
            Arc::new(tool_registry),
            auth_manager,
            audit_logger,
        );

        Ok(Self {
            server: Arc::new(server),
            server_handle: Arc::new(Mutex::new(None)),
            config,
            metrics: Arc::new(Mutex::new(ServiceMetrics::default())),
            start_time: std::time::Instant::now(),
        })
    }

    /// Start the MCP server
    pub async fn start(&self) -> McpResult<()> {
        match &self.config.server_config.transport {
            McpServerTransport::Stdio => {
                // For stdio transport, we run in the current task
                // The server will block until shutdown
                info!("Starting MCP server with STDIO transport");
                self.server.start().await?;
            }
            McpServerTransport::Sse { host, port, .. } => {
                // For SSE transport, spawn a background task
                info!("Starting MCP server with SSE transport on {}:{}", host, port);

                let server = self.server.clone();

                let handle = tokio::spawn(async move {
                    if let Err(e) = server.start().await {
                        error!("MCP server error: {}", e);
                    }
                });

                let mut server_handle = self.server_handle.lock().await;
                *server_handle = Some(handle);
            }
        }

        Ok(())
    }

    /// Stop the MCP server
    pub async fn stop(&self) -> McpResult<()> {
        info!("Stopping MCP server");

        // Cancel the server task if running
        let mut handle_guard = self.server_handle.lock().await;
        if let Some(handle) = handle_guard.take() {
            handle.abort();
            // Wait for task to finish (with timeout)
            let _ = tokio::time::timeout(std::time::Duration::from_secs(5), handle).await;
        }

        Ok(())
    }

    /// Check if the server is running
    pub async fn is_running(&self) -> bool {
        let handle_guard = self.server_handle.lock().await;
        if let Some(handle) = &*handle_guard {
            !handle.is_finished()
        } else {
            // For stdio transport, we can't easily check
            // Assume it's running if we haven't explicitly stopped it
            matches!(self.config.server_config.transport, McpServerTransport::Stdio)
        }
    }

    /// Get server address (for SSE transport)
    pub fn server_address(&self) -> Option<SocketAddr> {
        match &self.config.server_config.transport {
            McpServerTransport::Sse { host, port, .. } => format!("{}:{}", host, port).parse().ok(),
            _ => None,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum McpServiceError {
    #[error("MCP error: {0}")]
    McpError(#[from] McpError),

    #[error("Service configuration error: {0}")]
    ConfigError(String),

    #[error("Service initialization failed: {0}")]
    InitializationFailed(String),
}

#[async_trait]
impl Service for McpService {
    type Error = McpServiceError;
    type Config = McpServiceConfig;

    async fn initialize(_config: Self::Config) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        // This would need the repositories and executor to be passed somehow
        // For now, return an error as we can't create a complete service without dependencies
        Err(McpServiceError::InitializationFailed(
            "McpService requires repositories and executor - use McpService::new instead".to_string(),
        ))
    }

    fn name(&self) -> &'static str {
        "mcp-server"
    }

    async fn health_check(&self) -> Result<ServiceHealth, Self::Error> {
        let is_running = self.is_running().await;

        let mut health = if is_running {
            ServiceHealth::healthy().with_message("MCP server is running")
        } else {
            ServiceHealth::unhealthy("MCP server is not running")
        };

        // Add transport info
        health = health.with_metadata(
            "transport",
            match &self.config.server_config.transport {
                McpServerTransport::Stdio => "stdio",
                McpServerTransport::Sse { .. } => "sse",
            },
        );

        // Add server address for SSE
        if let Some(addr) = self.server_address() {
            health = health.with_metadata("address", addr.to_string());
        }

        // Add metrics
        let metrics = self.metrics.lock().await;
        health = health
            .with_metadata("requests_total", metrics.requests_total)
            .with_metadata("requests_failed", metrics.requests_failed)
            .with_metadata("uptime_seconds", self.start_time.elapsed().as_secs());

        Ok(health)
    }

    async fn shutdown(&self) -> Result<(), Self::Error> {
        self.stop().await?;
        Ok(())
    }

    fn metrics(&self) -> ServiceMetrics {
        // Return a clone of the current metrics
        // In a real implementation, this would be updated by the MCP server
        let metrics = self.metrics.blocking_lock();
        metrics.clone()
    }

    fn config(&self) -> Option<&Self::Config> {
        Some(&self.config)
    }
}

/// Builder for creating MCP service with all dependencies
pub struct McpServiceBuilder {
    config: McpServiceConfig,
    task_executor: Option<Arc<ProcessTaskExecutor>>,
    task_service: Option<Arc<dyn TaskService>>,
    execution_repository: Option<Arc<ExecutionRepository>>,
}

impl McpServiceBuilder {
    /// Create a new builder with default config
    pub fn new() -> Self {
        Self {
            config: McpServiceConfig::default(),
            task_executor: None,
            task_service: None,
            execution_repository: None,
        }
    }

    /// Set the service configuration
    pub fn with_config(mut self, config: McpServiceConfig) -> Self {
        self.config = config;
        self
    }

    /// Set the task executor
    pub fn with_task_executor(mut self, executor: Arc<ProcessTaskExecutor>) -> Self {
        self.task_executor = Some(executor);
        self
    }

    /// Set the task service
    pub fn with_task_service(mut self, service: Arc<dyn TaskService>) -> Self {
        self.task_service = Some(service);
        self
    }

    /// Set the execution repository
    pub fn with_execution_repository(mut self, repo: Arc<ExecutionRepository>) -> Self {
        self.execution_repository = Some(repo);
        self
    }

    /// Build the MCP service
    pub async fn build(self) -> Result<McpService, McpServiceError> {
        let task_executor = self
            .task_executor
            .ok_or_else(|| McpServiceError::InitializationFailed("Task executor is required".to_string()))?;

        let task_service = self
            .task_service
            .ok_or_else(|| McpServiceError::InitializationFailed("Task service is required".to_string()))?;

        let execution_repository = self
            .execution_repository
            .ok_or_else(|| McpServiceError::InitializationFailed("Execution repository is required".to_string()))?;

        McpService::new(self.config, task_executor, task_service, execution_repository)
            .await
            .map_err(Into::into)
    }
}

impl Default for McpServiceBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Integration helper to create MCP service from Ratchet config
impl McpService {
    /// Create from Ratchet's modular MCP configuration
    pub async fn from_ratchet_config(
        mcp_config: &ratchet_config::domains::mcp::McpConfig,
        task_executor: Arc<ProcessTaskExecutor>,
        task_service: Arc<dyn TaskService>,
        execution_repository: Arc<ExecutionRepository>,
        log_file_path: Option<std::path::PathBuf>,
    ) -> McpResult<Self> {
        // Use the new convenience method from McpServerConfig
        let server_config = crate::server::config::McpServerConfig::from_ratchet_config(mcp_config);

        let config = McpServiceConfig {
            server_config,
            log_file_path,
        };

        Self::new(config, task_executor, task_service, execution_repository).await
    }

    /// Create from Ratchet's legacy MCP configuration (for backward compatibility)
    /// TODO: Re-enable in Phase 3 when configuration is unified
    #[allow(dead_code)]
    async fn from_legacy_ratchet_config_disabled(
        _mcp_config: &std::marker::PhantomData<()>, // Placeholder for future MCP config
        _task_executor: Arc<ProcessTaskExecutor>,
        _task_service: Arc<dyn TaskService>,
        _execution_repository: Arc<ExecutionRepository>,
        _log_file_path: Option<std::path::PathBuf>,
    ) -> McpResult<Self> {
        /*
        // TODO: Re-enable in Phase 3 - Convert Ratchet config to MCP service config
        let transport = match mcp_config.transport.as_str() {
            "stdio" => McpServerTransport::Stdio,
            "sse" => McpServerTransport::Sse {
                host: mcp_config.host.clone(),
                port: mcp_config.port,
                tls: false, // TODO: Make configurable
                cors: crate::server::config::CorsConfig {
                    allowed_origins: vec!["*".to_string()], // Default CORS
                    allowed_methods: vec![
                        "GET".to_string(),
                        "POST".to_string(),
                        "OPTIONS".to_string(),
                    ],
                    allowed_headers: vec!["Content-Type".to_string(), "Authorization".to_string()],
                    allow_credentials: false,
                },
                timeout: std::time::Duration::from_secs(300), // Default 5 minutes
            },
            _ => {
                return Err(McpError::Configuration {
                    message: format!("Unknown transport: {}", mcp_config.transport),
                })
            }
        };

        let security = SecurityConfig {
            max_execution_time: std::time::Duration::from_secs(300), // Default 5 minutes
            max_log_entries: 1000,                                   // Default limit
            allow_dangerous_tasks: false, // Default disabled for security
            audit_log_enabled: true,      // Default enabled
            input_sanitization: true,     // Default enabled
            max_request_size: 1048576,    // Default 1MB
            max_response_size: 10485760,  // Default 10MB
            session_timeout: std::time::Duration::from_secs(3600), // Default 1 hour
            require_encryption: false,    // Default disabled
        };

        let server_config = McpServerConfig {
            transport,
            security,
            bind_address: Some(format!("{}:{}", mcp_config.host, mcp_config.port)),
        };

        let config = McpServiceConfig {
            server_config,
            log_file_path,
        };

        Self::new(config, task_executor, task_repository, execution_repository).await
        */
        Err(McpError::Configuration {
            message: "Legacy config support is disabled in this version. Please use the new modular configuration format. Legacy support will be re-enabled in Phase 3.".to_string(),
        })
    }
}
