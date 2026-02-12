use crate::config::RetryConfig;
use crate::errors::{LitError, Result};
use std::future::Future;
use tokio::time::{Duration, sleep};

pub async fn retry_with_backoff<F, Fut, T>(cfg: &RetryConfig, mut task: F) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T>>,
{
    let mut attempt = 0usize;
    let mut delay = cfg.base_delay_ms;
    loop {
        attempt += 1;
        match task().await {
            Ok(v) => return Ok(v),
            Err(err) => {
                if attempt >= cfg.max_attempts || !is_retryable(&err) {
                    return Err(err);
                }
                let jitter = (attempt as u64 * 37) % 100;
                sleep(Duration::from_millis((delay + jitter).min(cfg.max_delay_ms))).await;
                delay = (delay * 2).min(cfg.max_delay_ms);
            }
        }
    }
}

fn is_retryable(err: &LitError) -> bool {
    match err {
        LitError::Http(inner) => {
            inner.is_timeout()
                || inner.is_connect()
                || inner.status().is_some_and(|s| {
                    s.as_u16() == 429 || (500..=599).contains(&s.as_u16())
                })
        }
        LitError::External(_) => true,
        _ => false,
    }
}
