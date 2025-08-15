use std::sync::Arc;
use hyper::{Request, Response};
use hyper::body::{Incoming, Body};
use http_body_util::BodyExt;
use crate::repository::PaymentRepository;

pub struct PaymentHandler {
    repo: Arc<PaymentRepository>,
}

impl PaymentHandler {
    pub fn new(repo: Arc<PaymentRepository>) -> Self{
        Self {repo}
    }

    pub async fn handle(&self, request: Request<Incoming>) -> Result<Response<String>, hyper::Error> {
        match (request.method(), request.uri().path()) {
            (&hyper::Method::POST, "/payments") => self.handle_payment(request).await,
            (&hyper::Method::GET, "/payments-summary") => self.handle_summary(request).await,
            (&hyper::Method::POST, "/purge-payments") => self.handle_purge(request).await,
            (&hyper::Method::GET, _) => self.handle_index().await,
            _ => Ok(Response::builder()
                .status(405)
                .body("Method not allowed".to_string())
                .unwrap())
        }
    }

    pub async fn handle_payment(&self, request: Request<Incoming>)-> Result<Response<String>, hyper::Error>{        
        let upper = request.body().size_hint().upper().unwrap_or(u64::MAX);
        if upper > 1024 * 64 {
            return Ok(Response::builder()
                .status(413)
                .body("Body too big".to_string())
                .unwrap());
        }

        let body = match request.into_body().collect().await {
            Ok(collected) => collected.to_bytes(),
            Err(e) => {
                return Ok(Response::builder()
                    .status(400)
                    .body(format!("Failed to read request body: {}", e))
                    .unwrap());
            }
        };

        match self.repo.enqueue(&body[..]).await {
            Ok(_) => {
                Ok(Response::builder()
                    .status(202)
                    .body("".to_string())
                    .unwrap())
            }
            Err(e) => {
                Ok(Response::builder()
                    .status(500)
                    .body(format!("Failed to save to Redis: {}", e))
                    .unwrap())
            }
        }
    }
    
    pub async fn handle_summary(&self, _request: Request<Incoming>)-> Result<Response<String>, hyper::Error>{
        Ok(Response::builder()
            .status(501)
            .body("Not implemented".to_string())
            .unwrap())
    }
    
    pub async fn handle_purge(&self, _request: Request<Incoming>)-> Result<Response<String>, hyper::Error>{
        match self.repo.purge().await {
            Ok(_) => {
                return Ok(Response::builder()
                .status(200)
                .body("".to_string())
                .unwrap())
            }
            Err(_) => {
                return Ok(Response::builder()
                .status(500)
                .body("".to_string())
                .unwrap())
            }
        };
    }
    
    pub async fn handle_index(&self) -> Result<Response<String>, hyper::Error>{
        let response = Response::new("Not found".to_string());
        Ok(response)
    }
}
