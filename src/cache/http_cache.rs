use crate::errors::Result;
use crate::fs::hash::sha256_bytes;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEnvelope {
    fetched_at: u64,
    body: String,
}

#[derive(Debug, Clone)]
pub struct HttpCache {
    root: PathBuf,
    ttl_secs: u64,
}

impl HttpCache {
    pub fn new(root: &Path, ttl_secs: u64) -> Result<Self> {
        fs::create_dir_all(root)?;
        Ok(Self {
            root: root.to_path_buf(),
            ttl_secs,
        })
    }

    pub fn get(&self, key: &str) -> Result<Option<String>> {
        let path = self.path_for(key);
        if !path.exists() {
            return Ok(None);
        }
        let raw = fs::read_to_string(path)?;
        let env: CacheEnvelope = serde_json::from_str(&raw)?;
        let now = now_epoch();
        if now.saturating_sub(env.fetched_at) > self.ttl_secs {
            return Ok(None);
        }
        Ok(Some(env.body))
    }

    pub fn put(&self, key: &str, body: &str) -> Result<()> {
        let env = CacheEnvelope {
            fetched_at: now_epoch(),
            body: body.to_string(),
        };
        let serialized = serde_json::to_string(&env)?;
        fs::write(self.path_for(key), serialized)?;
        Ok(())
    }

    pub fn key_for(source: &str, url: &str) -> String {
        sha256_bytes(format!("{source}|{url}").as_bytes())
    }

    fn path_for(&self, key: &str) -> PathBuf {
        self.root.join(format!("{key}.json"))
    }
}

fn now_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
