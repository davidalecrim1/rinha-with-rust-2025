use redis::{Client};
use std::{error::Error, sync::Arc};
use log::error;
use redis::AsyncCommands;

use crate::types::{PAYMENT_DEFAULT_SORTED_SET, PAYMENT_FALLBACK_SORTED_SET, PAYMENTS_QUEUE};

pub struct PaymentRepository {
    redis_client: Arc<Client>
}

impl PaymentRepository {
    pub fn new(redis_client: Arc<Client>) -> Self{
        Self {redis_client}
    }

    pub async fn purge(&self) -> Result<(), Box<dyn Error>> {
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

    pub async fn enqueue(&self, raw_payment: &[u8]) -> Result<(), Box<dyn Error>> {
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

    pub async fn dequeue(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut conn = match self.redis_client.get_multiplexed_async_connection().await {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to get Redis connection: {}", e);
                return Err(Box::new(e));
            }
        };

        let raw_payment = match conn.rpop::<_, Option<Vec<u8>>>(PAYMENTS_QUEUE, None).await {
            Ok(Some(p)) => p,
            Ok(None) => return Err("Queue is empty".into()),
            Err(e) => {
                error!("Failed to dequeue payment: {}", e);
                return Err(Box::new(e));
            }
        };

        Ok(raw_payment)
    }
}