use core::fmt;

use chrono::Utc;
use serde::{Deserialize, Serialize};

pub const PAYMENTS_QUEUE: &str= "queue:payments";
pub const PAYMENT_DEFAULT_SORTED_SET: &str= "payments:default";
pub const PAYMENT_FALLBACK_SORTED_SET: &str= "payments:fallback";

#[derive(Serialize, Deserialize)]
pub struct PaymentRequest {
    pub amount: f64,
    pub correlation_id: String,
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

