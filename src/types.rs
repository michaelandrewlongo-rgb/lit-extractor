use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub source: String,
    pub doi: Option<String>,
    pub pmid: Option<String>,
    pub title: String,
    pub journal: Option<String>,
    pub year: Option<i32>,
    pub authors: Vec<String>,
    pub abstract_text: Option<String>,
    pub oa_url: Option<String>,
    pub epmc_id: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOutput {
    pub query: String,
    pub generated_at: DateTime<Utc>,
    pub results: Vec<SearchResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QaReport {
    pub unique_studies: usize,
    pub duplicates_removed: usize,
    pub oa_retrieval_rate: f64,
    pub extraction_success_rate: f64,
    pub unanchored_claim_count: usize,
}
