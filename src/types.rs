use core::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use log::error;

pub const PAYMENTS_QUEUE: &str= "queue:payments";
pub const PAYMENT_DEFAULT_SORTED_SET: &str= "payments:default";
pub const PAYMENT_FALLBACK_SORTED_SET: &str= "payments:fallback";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PaymentRequest {
    #[serde(rename = "amount")]
    pub amount: f64,
    #[serde(rename = "correlationId")]
    pub correlation_id: String,
    #[serde(rename = "requestedAt", default="String::new")]
    pub requested_at: String,
}

impl PaymentRequest {
    pub fn new(amount: f64, correlation_id: String) -> Self {
        Self { amount, correlation_id, requested_at: String::new() }
    }

    pub fn update_requested_at(&mut self) {
        let now = Utc::now();
        self.requested_at = now.to_rfc3339_opts(chrono::SecondsFormat::Nanos, true);
    }

    pub fn get_requested_at(&self) -> u64 {
        match DateTime::parse_from_rfc3339(&self.requested_at) {
            Ok(dt) => dt.timestamp() as u64, // in seconds
            Err(_) => {
                error!("Failed to parse requested_at: {}", self.requested_at);
                0
            }
        }
    }
}

#[derive(Debug)]
pub enum PaymentError {
    UnavailableProcessor,
}

impl fmt::Display for PaymentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PaymentError::UnavailableProcessor => write!(f, "Payment processor is unavailable"),
        }
    }
}

impl std::error::Error for PaymentError {}

#[derive(Debug)]
pub enum RepositoryError {
    QueueEmpty,
}

impl fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RepositoryError::QueueEmpty => write!(f, "Queue is empty"),
        }
    }
}

impl std::error::Error for RepositoryError {}

pub enum PaymentProcessor {
    Default,
    Fallback,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentProcessorSummary {
    #[serde(rename = "totalRequests")]
    pub total_requests: u64,
    #[serde(rename = "totalAmount")]
    pub total_amount: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentsSummary {
    pub default: PaymentProcessorSummary,
    pub fallback: PaymentProcessorSummary,
}

#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub failing: bool,
    pub min_response_time: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    #[serde(rename = "failing")]
    pub failing: bool,
    #[serde(rename = "minResponseTime")]
    pub min_response_time: u64,
}