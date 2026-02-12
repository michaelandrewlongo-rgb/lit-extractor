use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OaStatus {
    Open,
    Closed,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocRecord {
    pub doc_id: String,
    pub doi: Option<String>,
    pub pmid: Option<String>,
    pub title: String,
    pub journal: Option<String>,
    pub year: Option<i32>,
    pub authors: Vec<String>,
    pub abstract_text: Option<String>,
    pub oa_status: OaStatus,
    pub oa_url: Option<String>,
    pub epmc_id: Option<String>,
    pub local_pdf_path: Option<String>,
    pub local_xml_path: Option<String>,
    pub sha256: Option<String>,
    pub added_via: String,
    pub access_needed: bool,
    pub title_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct DocIdentity {
    pub doi: Option<String>,
    pub pmid: Option<String>,
    pub title_hash: String,
    pub year: Option<i32>,
    pub sha256: Option<String>,
}

impl OaStatus {
    pub fn from_oa_url(oa_url: Option<&str>) -> Self {
        match oa_url {
            Some(v) if !v.trim().is_empty() => Self::Open,
            _ => Self::Unknown,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            OaStatus::Open => "open",
            OaStatus::Closed => "closed",
            OaStatus::Unknown => "unknown",
        }
    }

    pub fn parse(raw: &str) -> Self {
        match raw {
            "open" => Self::Open,
            "closed" => Self::Closed,
            _ => Self::Unknown,
        }
    }
}
