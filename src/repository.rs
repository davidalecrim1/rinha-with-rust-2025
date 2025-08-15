use chrono::DateTime;
use redis::{Client};
use std::{error::Error, sync::Arc};
use log::{error, info};
use redis::AsyncCommands;

use crate::types::{PaymentProcessor, PaymentProcessorSummary, PaymentRequest, PaymentsSummary, RepositoryError, PAYMENTS_QUEUE, PAYMENT_DEFAULT_SORTED_SET, PAYMENT_FALLBACK_SORTED_SET};

pub struct PaymentRepository {
    redis_client: Arc<Client>
}

impl PaymentRepository {
    pub fn new(redis_client: Arc<Client>) -> Self{
        Self {redis_client}
    }

    pub async fn purge(&self) -> Result<(), Box<dyn Error + Send>> {
        let mut conn = match self.redis_client.get_multiplexed_async_connection().await {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to get Redis connection: {}", e);
                return Err(Box::new(e));
            }
        };

        if let Err(e) = conn.del::<_, ()>(PAYMENT_DEFAULT_SORTED_SET).await {
            error!("Failed to purge Redis keys: {}", e);
            return Err(Box::new(e));
        }

        if let Err(e) = conn.del::<_, ()>(PAYMENT_FALLBACK_SORTED_SET).await {
            error!("Failed to purge Redis keys: {}", e);
            return Err(Box::new(e));
        }

        if let Err(e) = conn.del::<_, ()>(PAYMENTS_QUEUE).await {
            error!("Failed to purge Redis keys: {}", e);
            return Err(Box::new(e));
        }

        Ok(())
    }

    pub async fn enqueue(&self, raw_payment: &[u8]) -> Result<(), Box<dyn Error + Send>> {
        let mut conn = match self.redis_client.get_multiplexed_async_connection().await {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to get Redis connection: {}", e);
                return Err(Box::new(e));
            }
        };

        if let Err(e) = conn.lpush::<_, _, ()>(PAYMENTS_QUEUE, raw_payment).await {
            error!("Failed to enqueue payment: {}", e);
            return Err(Box::new(e));
        }

        Ok(())
    }

    pub async fn dequeue(&self) -> Result<Vec<u8>, Box<dyn Error + Send>> {
        let mut conn = match self.redis_client.get_multiplexed_async_connection().await {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to get Redis connection: {}", e);
                return Err(Box::new(e));
            }
        };

        let raw_payment = match conn.rpop::<_, Option<Vec<u8>>>(PAYMENTS_QUEUE, None).await {
            Ok(Some(p)) => p,
            Ok(None) => return Err(Box::new(RepositoryError::QueueEmpty)),
            Err(e) => {
                error!("Failed to dequeue payment: {}", e);
                return Err(Box::new(e));
            }
        };

        Ok(raw_payment)
    }

    pub async fn add(&self, payment: PaymentRequest, processor: PaymentProcessor) -> Result<(), Box<dyn Error + Send>> {
        let mut conn = match self.redis_client.get_multiplexed_async_connection().await {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to get Redis connection: {}", e);
                return Err(Box::new(e));
            }
        };

        match processor {
            PaymentProcessor::Default => {
                let raw_payment = serde_json::to_string(&payment).unwrap();

                if let Err(e) = conn.zadd::<_, _, _, ()>(PAYMENT_DEFAULT_SORTED_SET, raw_payment, payment.get_requested_at()).await {
                    error!("Failed to add payment to default sorted set: {}", e);
                    return Err(Box::new(e));
                }
            }
            PaymentProcessor::Fallback => {
                let raw_payment = serde_json::to_string(&payment).unwrap();

                if let Err(e) = conn.zadd::<_, _, _, ()>(PAYMENT_FALLBACK_SORTED_SET, raw_payment, payment.get_requested_at()).await {
                    error!("Failed to add payment to fallback sorted set: {}", e);
                    return Err(Box::new(e));
                }
            }
        }

        Ok(())
    }

    pub async fn get_summary(&self, from: &str, to: &str) -> Result<PaymentsSummary, Box<dyn Error + Send>> {
        let mut from_unix = i32::MIN;
        let mut to_unix = i32::MAX;

        if !from.is_empty() {
            from_unix = match DateTime::parse_from_rfc3339(from) {
                Ok(dt) => dt.timestamp() as i32,
                Err(e) => {
                    error!("Failed to parse from: {}, from: {}", e, from);
                    return Err(Box::new(e));
                }
            };
        }
        
        if !to.is_empty() {
            to_unix = match DateTime::parse_from_rfc3339(to) {
                Ok(dt) => dt.timestamp() as i32,
                Err(e) => {
                    error!("Failed to parse to: {}, to: {}", e, to);
                    return Err(Box::new(e));
                }
            };

            to_unix -=1; // this is because the range is exclusive in the end
        }


        let (default_summary, fallback_summary) = tokio::join!(
            self.get_summary_from_processor(PAYMENT_DEFAULT_SORTED_SET, from_unix, to_unix),
            self.get_summary_from_processor(PAYMENT_FALLBACK_SORTED_SET, from_unix, to_unix),
        );

        Ok(PaymentsSummary{
            default: default_summary.unwrap_or_else(|_| PaymentProcessorSummary {
                total_requests: 0,
                total_amount: 0.0,
            }),
            fallback: fallback_summary.unwrap_or_else(|_| PaymentProcessorSummary {
                total_requests: 0,
                total_amount: 0.0,
            }),
        })
    }

    async fn get_summary_from_processor(&self, sorted_set: &str, from: i32, to: i32) -> Result<PaymentProcessorSummary, Box<dyn Error + Send>> {
        let mut conn = match self.redis_client.get_multiplexed_async_connection().await {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to get Redis connection: {}", e);
                return Err(Box::new(e));
            }
        };

        let results = match conn.zrangebyscore::<_, _, _, Vec<String>>(sorted_set, from, to).await {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to get summary: {}, from: {}, to: {}", e, from, to);
                return Err(Box::new(e));
            }
        };

        let mut summary = PaymentProcessorSummary {
            total_requests: 0,
            total_amount: 0.0,
        };

        for payment in results {
            let payment: PaymentRequest = serde_json::from_str(&payment).unwrap();
            summary.total_requests += 1;
            summary.total_amount += payment.amount;
        }

        summary.total_amount = (summary.total_amount * 100.0).round() / 100.0;
        Ok(summary)
    }

    pub async fn monitor_queue(&self){
        let mut conn = match self.redis_client.get_multiplexed_async_connection().await {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to get Redis connection: {}", e);
                return;
            }
        };

        tokio::task::spawn(async move {
            loop {
                let queue_length = match conn.llen::<_, i64>(PAYMENTS_QUEUE).await {
                    Ok(l) => l,
                    Err(e) => {
                        error!("Failed to get queue length: {}", e);
                        continue;
                    }
                };
    
                info!("Queue length: {}", queue_length);
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            }
        });
    }
}