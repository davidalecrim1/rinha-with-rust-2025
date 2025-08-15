
use crate::repository::PaymentRepository;
use crate::gateway::PaymentGateway;
use std::sync::Arc;

struct PaymentWorkerPool {
    workers_len: u32,
    repo: Arc<PaymentRepository>,
    gateway: Arc<PaymentGateway>,
}

impl PaymentWorkerPool {
    pub fn new(workers_len: u32, repo: Arc<PaymentRepository>, gateway: Arc<PaymentGateway>) -> Self {
        Self { workers_len, repo, gateway }
    }

    pub async fn run(&self) {
        // for _ in 0..self.workers_len {
        //     let repo = self.repo.clone();
        //     let gateway = self.gateway.clone();
            
        //     tokio::spawn(async move {
        //         loop {

        //         }


        //     })
        // }
    }
}