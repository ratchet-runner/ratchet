//! Common types for REST API requests and responses

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Standard API response wrapper
pub use ratchet_web::ApiResponse;

pub use ratchet_web::extractors::{ListQuery, PaginationParams};
/// Query parameter types
pub use ratchet_web::{FilterQuery, PaginationQuery, QueryParams, SortQuery};

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub version: Option<String>,
    pub checks: Option<std::collections::HashMap<String, HealthCheckResult>>,
}

/// Individual health check result
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct HealthCheckResult {
    pub status: HealthStatus,
    pub message: Option<String>,
    pub duration_ms: Option<u64>,
}

/// Health check status
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Unhealthy,
    Degraded,
}

impl HealthResponse {
    pub fn healthy() -> Self {
        Self {
            status: "healthy".to_string(),
            timestamp: chrono::Utc::now(),
            version: env!("CARGO_PKG_VERSION").parse().ok(),
            checks: None,
        }
    }

    pub fn with_checks(mut self, checks: std::collections::HashMap<String, HealthCheckResult>) -> Self {
        // Update overall status based on individual checks
        let has_unhealthy = checks
            .values()
            .any(|check| matches!(check.status, HealthStatus::Unhealthy));
        let has_degraded = checks
            .values()
            .any(|check| matches!(check.status, HealthStatus::Degraded));

        self.checks = Some(checks);

        self.status = if has_unhealthy {
            "unhealthy"
        } else if has_degraded {
            "degraded"
        } else {
            "healthy"
        }
        .to_string();

        self
    }
}

/// Statistics response wrapper
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StatsResponse<T> {
    pub stats: T,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub period: Option<String>,
}

impl<T> StatsResponse<T> {
    pub fn new(stats: T) -> Self {
        Self {
            stats,
            timestamp: chrono::Utc::now(),
            period: None,
        }
    }

    pub fn with_period(mut self, period: String) -> Self {
        self.period = Some(period);
        self
    }
}
