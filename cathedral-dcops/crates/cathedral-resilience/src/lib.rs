use std::sync::atomic::{AtomicU8, Ordering};
use std::time::Duration;
use async_trait::async_trait;

pub struct CircuitBreaker {
    state: AtomicU8, // 0=Closed,1=Open,2=HalfOpen
    pub failure_threshold: usize,
    pub timeout: Duration,
}

pub struct RetryPolicy {
    pub max_attempts: usize,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub backoff_factor: f64,
}

impl RetryPolicy {
    pub fn with_jitter() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(2),
            backoff_factor: 2.0,
        }
    }
}

pub struct Response;
pub struct Error;

#[async_trait]
pub trait ResilientHttpClient {
    async fn get_with_retry(&self, url: &str) -> Result<Response, Error>;
}
