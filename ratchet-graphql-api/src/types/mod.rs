//! GraphQL type definitions

use async_graphql::SimpleObject;
use ratchet_api_types::pagination::PaginationMeta;

pub mod executions;
pub mod jobs;
pub mod scalars;
pub mod schedules;
pub mod tasks;
pub mod workers;

// Re-export all types
pub use executions::*;
pub use jobs::*;
pub use scalars::*;
pub use schedules::*;
pub use tasks::*;
pub use workers::*;

/// Pagination metadata for GraphQL responses - using unified PaginationMeta directly
pub type PaginationMetaGraphQL = PaginationMeta;

/// Paginated task response
#[derive(SimpleObject)]
pub struct TaskList {
    pub items: Vec<Task>,
    pub meta: PaginationMetaGraphQL,
}

/// Paginated execution response
#[derive(SimpleObject)]
pub struct ExecutionList {
    pub items: Vec<Execution>,
    pub meta: PaginationMetaGraphQL,
}

/// Paginated job response
#[derive(SimpleObject)]
pub struct JobList {
    pub items: Vec<Job>,
    pub meta: PaginationMetaGraphQL,
}

/// Paginated schedule response
#[derive(SimpleObject)]
pub struct ScheduleList {
    pub items: Vec<Schedule>,
    pub meta: PaginationMetaGraphQL,
}

/// Paginated worker response
#[derive(SimpleObject)]
pub struct WorkerList {
    pub items: Vec<Worker>,
    pub meta: PaginationMetaGraphQL,
}

/// System health status
#[derive(SimpleObject)]
pub struct HealthStatus {
    pub database: bool,
    pub message: String,
}
