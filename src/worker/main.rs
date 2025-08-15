use std::sync::Arc;
use std::{error::Error};
use redis::{Client};
use std::env;


use rinha_with_rust_2025::handler;
use rinha_with_rust_2025::repository;
use rinha_with_rust_2025::config::Config;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    env_logger::init();

    let config = Arc::new(Config::load()?);

    let redis_client = Arc::new(
        Client::open(config.redis_addr.clone()).expect("Connection to redis failed"),
    );

    let payment_repo = Arc::new(
        repository::PaymentRepository::new(redis_client.clone()),
    );
    let payment_handler = handler::PaymentHandler::new(
        payment_repo.clone(),
    );

    let workers_len = env::var("WORKERS").expect("WORKERS is not set");



    Ok(())

}

