use std::env;

#[derive(Clone)]
pub struct Config {
    pub redis_addr: String,
    pub socket_path: String,
    pub queue_name: String,
    pub sorted_set_default: String,
    pub sorted_set_fallback: String,
    pub workers_len: u32,
    pub payment_default_url: String,
    pub payment_fallback_url: String,
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            redis_addr: env::var("REDIS_ADDR").unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            socket_path: env::var("UNIX_SOCKET").unwrap_or_else(|_| "/tmp/hyperlocal.sock".to_string()),
            queue_name: "payments:queue".to_string(),
            sorted_set_default: "payments:list:default".to_string(),
            sorted_set_fallback: "payments:list:fallback".to_string(),
            workers_len: env::var("WORKERS").unwrap_or_else(|_| "1".to_string()).parse::<u32>().unwrap_or(1),
            payment_default_url: env::var("PAYMENT_PROCESSOR_URL_DEFAULT").unwrap_or_else(|_| "http://localhost:8001".to_string()),
            payment_fallback_url: env::var("PAYMENT_PROCESSOR_URL_FALLBACK").unwrap_or_else(|_| "http://localhost:8002".to_string()),
        })
    }
}
