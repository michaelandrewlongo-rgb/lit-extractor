pub mod clinicaltrials;
pub mod crossref;
pub mod epmc;
pub mod openalex;
pub mod pubmed;
pub mod unpaywall;

use crate::cache::http_cache::HttpCache;
use crate::config::{AppConfig, RetryConfig};
use crate::errors::{LitError, Result};
use crate::net::ratelimit::RateLimitRegistry;
use crate::net::retry::retry_with_backoff;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde::de::DeserializeOwned;

#[derive(Clone)]
pub struct ApiClient {
    http: reqwest::Client,
    cache: HttpCache,
    ratelimits: RateLimitRegistry,
    retry_cfg: RetryConfig,
}

impl ApiClient {
    pub fn new(cfg: &AppConfig) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_str(&cfg.network.user_agent)
                .unwrap_or_else(|_| HeaderValue::from_static("lit-harvester/0.1")),
        );
        let http = reqwest::Client::builder().default_headers(headers).build()?;
        let cache = HttpCache::new(&cfg.data.cache_dir, 60 * 60 * 24)?;
        Ok(Self {
            http,
            cache,
            ratelimits: RateLimitRegistry::from_config(&cfg.rate_limits),
            retry_cfg: cfg.retry.clone(),
        })
    }

    pub async fn get_text_cached(&self, source: &str, url: &str) -> Result<String> {
        let key = HttpCache::key_for(source, url);
        if let Some(hit) = self.cache.get(&key)? {
            return Ok(hit);
        }

        self.ratelimits.wait(source).await;
        let body = retry_with_backoff(&self.retry_cfg, || async {
            let resp = self.http.get(url).send().await?;
            if !resp.status().is_success() {
                return Err(LitError::External(format!(
                    "{source} returned status {} for {url}",
                    resp.status()
                )));
            }
            Ok(resp.text().await?)
        })
        .await?;

        self.cache.put(&key, &body)?;
        Ok(body)
    }

    pub async fn get_json_cached<T: DeserializeOwned>(&self, source: &str, url: &str) -> Result<T> {
        let text = self.get_text_cached(source, url).await?;
        serde_json::from_str(&text).map_err(|e| {
            LitError::External(format!("failed to decode {source} response as json: {e}"))
        })
    }

    pub async fn download_bytes(&self, source: &str, url: &str) -> Result<Vec<u8>> {
        self.ratelimits.wait(source).await;
        retry_with_backoff(&self.retry_cfg, || async {
            let resp = self.http.get(url).send().await?;
            if !resp.status().is_success() {
                return Err(LitError::External(format!(
                    "download failed from {source} with status {} ({url})",
                    resp.status()
                )));
            }
            Ok(resp.bytes().await?.to_vec())
        })
        .await
    }
}
