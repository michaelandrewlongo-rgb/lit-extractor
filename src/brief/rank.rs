use crate::domain::doc::DocRecord;
use crate::domain::evidence::EvidenceLedgerRow;
use chrono::{Datelike, Utc};

pub fn score_claim(claim: &EvidenceLedgerRow, doc: Option<&DocRecord>, query: &str) -> f64 {
    if claim.claim_text == "unknown" {
        return 0.0;
    }

    let mut score = 1.0;
    let text = claim.claim_text.to_lowercase();

    if text.contains("guideline") {
        score += 3.0;
    }
    if text.contains("randomized") || text.contains("rct") {
        score += 2.5;
    }
    if text.contains("prospective") {
        score += 1.5;
    }
    if text.contains("retrospective") {
        score += 0.8;
    }
    if text.contains("case series") {
        score += 0.3;
    }

    for token in query.to_lowercase().split_whitespace() {
        if text.contains(token) {
            score += 0.2;
        }
    }

    if let Some(doc) = doc {
        if let Some(year) = doc.year {
            let years_old = (Utc::now().year() - year).max(0) as f64;
            score += (10.0 - (years_old / 2.0)).max(0.0) * 0.1;
        }
    }

    score
}
