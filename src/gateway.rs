use crate::types::{PaymentError, PaymentRequest};
use std::sync::Arc;
use tokio::sync::RwLock; // ✅ Use tokio's async RwLock
use reqwest::{Client, Response};
use crate::config::Config;
use log::{error, debug};
use hyper::http::StatusCode;
use tokio::time::{timeout, Duration};
use crate::types::{HealthStatus, HealthCheckResponse};

pub struct PaymentGateway {
    http_client: Arc<Client>,
    config: Arc<Config>,
    health_status_default: Arc<RwLock<HealthStatus>>
}

impl PaymentGateway {
    pub fn new(client: Arc<Client>, config: Arc<Config>) -> Self {
        Self {
            http_client: client,
            config,
            health_status_default: Arc::new(RwLock::new(HealthStatus {
                failing: false,
                min_response_time: 0,
            })),
        }
    }

    pub async fn get_health_status(&self) -> HealthStatus {
        self.health_status_default.read().await.clone()
    }

    pub async fn is_failing(&self) -> bool {
        self.health_status_default.read().await.failing
    }

    pub async fn process_default(&self, payment: &mut PaymentRequest) -> Result<(), Box<dyn std::error::Error + Send>> {
        payment.update_requested_at();

        debug!("Sending payment to default processor: {:?}", payment);

        if self.is_failing().await {
            debug!("Default processor is failing, skipping payment");
            return Err(Box::new(PaymentError::UnavailableProcessor));
        }

        if self.get_health_status().await.min_response_time > 100 {
            debug!("Default processor is slow, skipping payment");
            return Err(Box::new(PaymentError::UnavailableProcessor));
        }

        let url = self.config.payment_default_url.clone() + "/payments";
        let response: Response = match self.http_client.post(url)
            .json(&payment)
            .send()
            .await {
                Ok(r) => r,
                Err(e) => {
                    error!("Failed to send payment: {}", e);
                    return Err(Box::new(PaymentError::UnavailableProcessor));
                }
            };

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

    pub fn  start_health_check(&self) -> tokio::task::JoinHandle<()> {
        let config = self.config.clone();
        let http_client = self.http_client.clone();
        let health_status_default = self.health_status_default.clone();

        tokio::task::spawn(async move {
            loop {
                debug!("Running health check...");
            
                let url = config.payment_default_url.clone() + "/payments/service-health";
                let result = timeout(
                    Duration::from_secs(1),
                    http_client.get(url).send()
                ).await;

                let mut status = health_status_default.write().await;
                
                match result {
                    Ok(Ok(response)) => {
                        if response.status().is_success() {
                            match response.json::<HealthCheckResponse>().await {
                                Ok(data) => {
                                    status.failing = data.failing;
                                    status.min_response_time = data.min_response_time;
                                }
                                Err(e) => {
                                    error!("Failed to parse health check response: {}", e);
                                    status.failing = true;
                                    status.min_response_time = 0;
                                }
                            }
                        } else {
                            error!("Health check failed: {}", response.status());
                            status.failing = true;
                            status.min_response_time = 0;
                        }
                    }
                    Ok(Err(e)) => {
                        error!("Health check request error: {}", e);
                        status.failing = true;
                        status.min_response_time = 0;
                    }
                    Err(_) => {
                        debug!("Health check timed out after 1 second");
                    }
                }
                
                drop(status);
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        })
    }
}