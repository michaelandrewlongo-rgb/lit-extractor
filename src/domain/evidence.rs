use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ClaimType {
    Outcome,
    Method,
    Population,
    Complication,
    Anatomy,
    Technique,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AnchorType {
    Pdf,
    Xml,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceLedgerRow {
    pub claim_id: String,
    pub doc_id: String,
    pub source_type: String,
    pub doi: Option<String>,
    pub pmid: Option<String>,
    pub url: Option<String>,
    pub local_path: String,
    pub retrieved_at: DateTime<Utc>,
    pub sha256: Option<String>,
    pub anchor_type: AnchorType,
    pub page_number: Option<u32>,
    pub section_heading: Option<String>,
    pub anchor_quote: String,
    pub claim_text: String,
    pub claim_type: ClaimType,
    pub numbers: Option<Value>,
    pub errors: Option<Vec<String>>,
}

impl ClaimType {
    pub fn classify(sentence: &str) -> Self {
        let s = sentence.to_lowercase();
        if s.contains("complication") || s.contains("adverse") {
            Self::Complication
        } else if s.contains("anatom") {
            Self::Anatomy
        } else if s.contains("technique") || s.contains("operative") || s.contains("surgical") {
            Self::Technique
        } else if s.contains("cohort") || s.contains("patient") || s.contains("population") {
            Self::Population
        } else if s.contains("method") || s.contains("protocol") {
            Self::Method
        } else if s.contains("outcome") || s.contains("improved") || s.contains("reduced") {
            Self::Outcome
        } else {
            Self::Other
        }
    }
}
