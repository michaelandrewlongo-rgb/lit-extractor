use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FigureIndexRow {
    pub figure_id: String,
    pub doc_id: String,
    pub doi: Option<String>,
    pub pmid: Option<String>,
    pub local_doc_path: String,
    pub figure_path: String,
    pub source_type: String,
    pub page_number: Option<u32>,
    pub xml_fig_id: Option<String>,
    pub figure_label: Option<String>,
    pub caption: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub sha256: Option<String>,
    pub license: Option<String>,
    pub retrieved_at: DateTime<Utc>,
}
