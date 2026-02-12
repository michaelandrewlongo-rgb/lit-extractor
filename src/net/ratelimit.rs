use crate::config::RateLimitConfig;
use governor::clock::DefaultClock;
use governor::state::InMemoryState;
use governor::state::direct::NotKeyed;
use governor::{Quota, RateLimiter};
use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::Arc;

#[derive(Clone)]
pub struct RateLimitRegistry {
    map: Arc<HashMap<String, Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>>>,
}

impl RateLimitRegistry {
    pub fn from_config(cfg: &RateLimitConfig) -> Self {
        let mut map = HashMap::new();
        map.insert("pubmed".to_string(), new_limiter(cfg.pubmed_per_sec));
        map.insert("europepmc".to_string(), new_limiter(cfg.europepmc_per_sec));
        map.insert("crossref".to_string(), new_limiter(cfg.crossref_per_sec));
        map.insert("openalex".to_string(), new_limiter(cfg.openalex_per_sec));
        map.insert("unpaywall".to_string(), new_limiter(cfg.unpaywall_per_sec));
        map.insert("clinicaltrials".to_string(), new_limiter(cfg.clinicaltrials_per_sec));
        Self { map: Arc::new(map) }
    }

    pub async fn wait(&self, source: &str) {
        if let Some(limiter) = self.map.get(source) {
            let _ = limiter.until_ready().await;
        }
    }
}

fn new_limiter(per_sec: u32) -> Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>> {
    let nz = NonZeroU32::new(per_sec.max(1)).expect("nonzero");
    Arc::new(RateLimiter::direct(Quota::per_second(nz)))
}
