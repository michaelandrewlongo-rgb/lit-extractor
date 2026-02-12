use sha2::{Digest, Sha256};

pub fn normalize_doi(doi: &str) -> String {
    doi.trim().to_lowercase()
}

pub fn normalize_pmid(pmid: &str) -> String {
    pmid.trim().to_string()
}

pub fn title_hash(title: &str) -> String {
    let normalized = title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() || c.is_whitespace() { c } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let mut hasher = Sha256::new();
    hasher.update(normalized.as_bytes());
    format!("{:x}", hasher.finalize())
}
