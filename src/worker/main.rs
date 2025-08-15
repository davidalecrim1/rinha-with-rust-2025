use std::sync::Arc;
use std::{error::Error};
use redis::{Client};
use log::{info};

use rinha_with_rust_2025::repository;
use rinha_with_rust_2025::config::Config;
use rinha_with_rust_2025::worker::PaymentWorkerPool;
use rinha_with_rust_2025::gateway;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let config = Arc::new(Config::load().unwrap());

    let redis_client = Arc::new(
        Client::open(config.redis_addr.clone()).expect("Connection to redis failed"),
    );

    let payment_repo = Arc::new(
        repository::PaymentRepository::new(redis_client.clone()),
    );

    let http_client = Arc::new(
        reqwest::Client::new(),
    );

    let payment_gateway = Arc::new(
        gateway::PaymentGateway::new(http_client.clone(), config.clone()),
    );

    let payment_worker_pool = PaymentWorkerPool::new(
        config.workers_len,
        payment_repo.clone(),
        payment_gateway.clone(),
    );

    info!("Starting payment worker pool");
    payment_gateway.start_health_check();
    payment_repo.monitor_queue().await;
    payment_worker_pool.run().await;
    info!("Payment worker pool stopped");

    Ok(())

}

