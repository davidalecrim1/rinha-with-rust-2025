use std::sync::Arc;
use std::{error::Error, fs};
use std::os::unix::fs::PermissionsExt;

use hyperlocal::UnixListenerExt;
use redis::{Client};
use tokio::net::{UnixListener};
use log::{info};

use rinha_with_rust_2025::handler;
use rinha_with_rust_2025::repository;
use rinha_with_rust_2025::config::Config;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    env_logger::init();

    let config = Arc::new(Config::load()?);
    let _ = fs::remove_file(&config.socket_path);

    let listener = UnixListener::bind(&config.socket_path)?;
    info!("Listening on {}", config.socket_path);

    fs::set_permissions(&config.socket_path, fs::Permissions::from_mode(0o666))?;

    let redis_client = Arc::new(
        Client::open(config.redis_addr.clone()).expect("Connection to redis failed"),
    );

    let payment_repo = Arc::new(
        repository::PaymentRepository::new(redis_client.clone()),
    );
    let payment_handler = handler::PaymentHandler::new(
        payment_repo.clone(),
    );

    listener
        .serve(|| {
            |request| async {
                payment_handler.handle(request).await
            }
        })
        .await?;

    Ok(())

}

