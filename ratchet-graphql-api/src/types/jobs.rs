//! GraphQL types for jobs

use super::scalars::GraphQLApiId;
use async_graphql::{InputObject, SimpleObject};
use chrono::{DateTime, Utc};
use ratchet_api_types::{JobPriority, JobStatus, UnifiedJob};

/// GraphQL Job type with additional fields for GraphQL API
#[derive(SimpleObject, Clone, Debug)]
#[graphql(rename_fields = "camelCase")]
pub struct Job {
    pub id: GraphQLApiId,
    pub task_id: GraphQLApiId,
    pub priority: JobPriorityGraphQL,
    pub status: JobStatusGraphQL,
    pub retry_count: i32,
    pub max_retries: i32,
    pub queued_at: DateTime<Utc>,
    pub scheduled_for: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub output_destinations: Option<Vec<OutputDestination>>,
}

impl From<UnifiedJob> for Job {
    fn from(job: UnifiedJob) -> Self {
        // Convert output destinations from UnifiedOutputDestination to GraphQL types
        let output_destinations = job.output_destinations.map(|dests| {
            dests
                .into_iter()
                .map(|dest| {
                    let destination_type = match dest.destination_type.as_str() {
                        "webhook" => OutputDestinationType::Webhook,
                        "file" => OutputDestinationType::File,
                        "database" => OutputDestinationType::Database,
                        _ => OutputDestinationType::Webhook, // Default fallback
                    };
                    OutputDestination { destination_type }
                })
                .collect()
        });

        Self {
            id: job.id.into(),
            task_id: job.task_id.into(),
            priority: job.priority,
            status: job.status,
            retry_count: job.retry_count,
            max_retries: job.max_retries,
            queued_at: job.queued_at,
            scheduled_for: job.scheduled_for,
            error_message: job.error_message,
            output_destinations,
        }
    }
}

/// GraphQL JobStatus - using unified JobStatus directly
pub type JobStatusGraphQL = JobStatus;

/// GraphQL JobPriority - using unified JobPriority directly
pub type JobPriorityGraphQL = JobPriority;

/// Input type for creating jobs
#[derive(InputObject)]
#[graphql(rename_fields = "camelCase")]
pub struct CreateJobInput {
    pub task_id: GraphQLApiId,
    pub priority: Option<JobPriorityGraphQL>,
    pub scheduled_for: Option<DateTime<Utc>>,
    pub max_retries: Option<i32>,
}

/// Input type for updating jobs
#[derive(InputObject)]
#[graphql(rename_fields = "camelCase")]
pub struct UpdateJobInput {
    pub priority: Option<JobPriorityGraphQL>,
    pub status: Option<JobStatusGraphQL>,
    pub scheduled_for: Option<DateTime<Utc>>,
    pub max_retries: Option<i32>,
    pub error_message: Option<String>,
}

/// Input type for job filtering
#[derive(InputObject)]
#[graphql(rename_fields = "camelCase")]
pub struct JobFiltersInput {
    // ID filtering
    pub task_id: Option<GraphQLApiId>,
    pub task_id_in: Option<Vec<GraphQLApiId>>,
    pub id_in: Option<Vec<GraphQLApiId>>,

    // Status filtering
    pub status: Option<JobStatusGraphQL>,
    pub status_in: Option<Vec<JobStatusGraphQL>>,
    pub status_not: Option<JobStatusGraphQL>,

    // Priority filtering
    pub priority: Option<JobPriorityGraphQL>,
    pub priority_in: Option<Vec<JobPriorityGraphQL>>,
    pub priority_min: Option<JobPriorityGraphQL>,

    // Date range filtering
    pub queued_after: Option<DateTime<Utc>>,
    pub queued_before: Option<DateTime<Utc>>,
    pub scheduled_after: Option<DateTime<Utc>>,
    pub scheduled_before: Option<DateTime<Utc>>,

    // Retry filtering
    pub retry_count_min: Option<i32>,
    pub retry_count_max: Option<i32>,
    pub max_retries_min: Option<i32>,
    pub max_retries_max: Option<i32>,
    pub has_retries_remaining: Option<bool>,

    // Error filtering
    pub has_error: Option<bool>,
    pub error_message_contains: Option<String>,

    // Scheduling filtering
    pub is_scheduled: Option<bool>,
    pub due_now: Option<bool>, // scheduled_for <= now
}

/// Job statistics
#[derive(SimpleObject)]
#[graphql(rename_fields = "camelCase")]
pub struct JobStats {
    pub total_jobs: i64,
    pub pending_jobs: i64,
    pub running_jobs: i64,
    pub completed_jobs: i64,
    pub failed_jobs: i64,
    pub cancelled_jobs: i64,
    pub average_processing_time_ms: Option<f64>,
}

/// Input type for executing tasks (creating jobs with execution)
#[derive(InputObject)]
#[graphql(rename_fields = "camelCase")]
pub struct ExecuteTaskInput {
    pub task_id: GraphQLApiId,
    pub input_data: serde_json::Value,
    pub priority: Option<JobPriorityGraphQL>,
    pub output_destinations: Option<Vec<OutputDestinationInput>>,
    pub max_retries: Option<i32>,
}

/// Output destination configuration
#[derive(InputObject)]
#[graphql(rename_fields = "camelCase")]
pub struct OutputDestinationInput {
    pub destination_type: OutputDestinationType,
    pub webhook: Option<WebhookDestinationInput>,
}

/// Webhook destination configuration
#[derive(InputObject)]
#[graphql(rename_fields = "camelCase")]
pub struct WebhookDestinationInput {
    pub url: String,
    pub method: String,
    pub content_type: String,
    pub retry_policy: Option<RetryPolicyInput>,
}

/// Retry policy configuration
#[derive(InputObject)]
#[graphql(rename_fields = "camelCase")]
pub struct RetryPolicyInput {
    pub max_attempts: i32,
    pub initial_delay_ms: i32,
    pub max_delay_ms: i32,
    pub backoff_multiplier: f64,
}

/// Output destination type
#[derive(async_graphql::Enum, Copy, Clone, Debug, Eq, PartialEq)]
pub enum OutputDestinationType {
    #[graphql(name = "WEBHOOK")]
    Webhook,
    #[graphql(name = "FILE")]
    File,
    #[graphql(name = "DATABASE")]
    Database,
}

/// Output destination info for responses
#[derive(SimpleObject, Clone, Debug)]
#[graphql(rename_fields = "camelCase")]
pub struct OutputDestination {
    pub destination_type: OutputDestinationType,
}
