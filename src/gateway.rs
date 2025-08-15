use crate::types::{PaymentError, PaymentRequest};
use std::sync::Arc;
use reqwest::{Client, Response};
use crate::config::Config;
use log::{error, debug};
use serde::{Deserialize, Serialize};
use hyper::http::StatusCode;


pub struct PaymentGateway {
    http_client: Arc<Client>,
    config: Arc<Config>,
}

impl PaymentGateway {
    pub fn new(client: Arc<Client>, config: Arc<Config>) -> Self {
        Self {
            http_client: client,
            config,
        }
    }

    pub async fn process_default(&self, mut payment: PaymentRequest) -> Result<(), Box<dyn std::error::Error>> {
        payment.update_requested_at();

        let response: Response = self.http_client.post(self.config.payment_default_url.clone())
            .json(&payment)
            .send()
            .await?;

        if response.status() == StatusCode::UNPROCESSABLE_ENTITY {
            error!("Payment processor payment already exists");
            return Ok(());
        }

        if !response.status().is_success() {
            debug!("Payment processor unavailable: {}", response.status());
            return Err(Box::new(PaymentError::UnavailableProcessor));
        }

        Ok(())
    }

}