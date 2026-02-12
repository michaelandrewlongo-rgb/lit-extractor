use crate::errors::{LitError, Result};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub data: DataConfig,
    pub network: NetworkConfig,
    pub rate_limits: RateLimitConfig,
    pub retry: RetryConfig,
    pub pipeline: PipelineConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DataConfig {
    pub root: PathBuf,
    pub sqlite_path: PathBuf,
    pub cache_dir: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NetworkConfig {
    pub user_agent: String,
    pub unpaywall_email: String,
    pub pubmed_api_key: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitConfig {
    pub pubmed_per_sec: u32,
    pub europepmc_per_sec: u32,
    pub crossref_per_sec: u32,
    pub openalex_per_sec: u32,
    pub unpaywall_per_sec: u32,
    pub clinicaltrials_per_sec: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: usize,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PipelineConfig {
    pub default_limit: usize,
    pub default_since: String,
    pub strict_qa: bool,
    pub top_k_sources: usize,
    pub max_key_figures: usize,
}

impl AppConfig {
    pub fn load(path: Option<&Path>, data_dir: Option<&Path>) -> Result<Self> {
        let default_toml = include_str!("../config/default.toml");
        let mut cfg: AppConfig = toml::from_str(default_toml)
            .map_err(|e| LitError::Config(format!("failed to parse bundled config: {e}")))?;

        if let Some(path) = path {
            let custom_raw = fs::read_to_string(path)?;
            let custom: AppConfig = toml::from_str(&custom_raw)
                .map_err(|e| LitError::Config(format!("failed to parse {}: {e}", path.display())))?;
            cfg = custom;
        }

        if let Some(dir) = data_dir {
            cfg.data.root = dir.to_path_buf();
            cfg.data.sqlite_path = dir.join("lit.db");
            cfg.data.cache_dir = dir.join("cache");
        }

        if cfg.network.unpaywall_email.is_empty() {
            if let Ok(v) = std::env::var("UNPAYWALL_EMAIL") {
                cfg.network.unpaywall_email = v;
            }
        }
        if cfg.network.pubmed_api_key.is_empty() {
            if let Ok(v) = std::env::var("PUBMED_API_KEY") {
                cfg.network.pubmed_api_key = v;
            }
        }

        Ok(cfg)
    }

    pub fn ensure_layout(&self) -> Result<()> {
        fs::create_dir_all(&self.data.root)?;
        fs::create_dir_all(self.data.root.join("oa"))?;
        fs::create_dir_all(self.data.root.join("inbox"))?;
        fs::create_dir_all(self.data.root.join("docs"))?;
        fs::create_dir_all(self.data.root.join("artifacts"))?;
        fs::create_dir_all(self.data.root.join("briefs"))?;
        fs::create_dir_all(&self.data.cache_dir)?;
        Ok(())
    }
}
