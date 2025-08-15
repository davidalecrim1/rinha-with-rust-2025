
use crate::repository::PaymentRepository;
use crate::gateway::PaymentGateway;
use crate::types::{PaymentError, PaymentProcessor, PaymentRequest, RepositoryError};
use std::sync::Arc;
use std::time::Duration;
use log::{error, debug};
use serde_json;

pub struct PaymentWorkerPool {  
    workers_len: u32,
    repo: Arc<PaymentRepository>,
    gateway: Arc<PaymentGateway>,
}

impl PaymentWorkerPool {
    pub fn new(workers_len: u32, repo: Arc<PaymentRepository>, gateway: Arc<PaymentGateway>) -> Self {
        Self { workers_len, repo, gateway }
    }

    pub async fn run(&self) {
        let mut running_workers = Vec::new();
        
        for _ in 0..self.workers_len {
            let repo = self.repo.clone();
            let gateway = self.gateway.clone();
            
            let worker = tokio::task::spawn(async move {
                loop {
                    let raw_payment = match repo.dequeue().await {
                        Ok(p) => p,
                        Err(e) => {
                            if let Some(e) = e.downcast_ref::<RepositoryError>() {
                                debug!("Received repository error: {}", e);
                                tokio::time::sleep(Duration::from_millis(200)).await;
                                continue;
                            }
                            error!("Failed to dequeue payment: {}", e);
                            continue;
                        }
                    };

                    debug!("Processing payment: {:?}", String::from_utf8(raw_payment.clone()).unwrap());

                    let mut payment = match serde_json::from_slice::<PaymentRequest>(&raw_payment) {
                        Ok(p) => p,
                        Err(e) => {
                            error!("Failed to parse payment: {}", e);
                            continue;
                        }
                    };

                    match gateway.process_default(&mut payment).await {
                        Ok(_) => {},
                        Err(e) => {
                            if let Some(e) = e.downcast_ref::<PaymentError>() {
                                debug!("Payment processor is unavailable, skipping payment: {}", e);
                            } else {
                                error!("Failed to process payment: {}", e);
                            }

                            match repo.enqueue(&raw_payment).await {
                                Ok(_) => {},
                                Err(e) => {
                                    error!("Failed to enqueue payment: {}", e);
                                }
                            }
                            continue;
                        }
                    }

                    match repo.add(payment.clone(), PaymentProcessor::Default).await {
                        Ok(_) => {},
                        Err(e) => {
                            error!("Failed to add payment: {}", e);
                        }
                    }
                }
            });
            
            running_workers.push(worker);
        }
        
        // Wait for all worker tasks to complete (they run forever, so this will block indefinitely)
        for worker in running_workers {
            let _ = worker.await;
        }
    }
}