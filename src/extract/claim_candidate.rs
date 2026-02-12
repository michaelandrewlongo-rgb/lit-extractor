use regex::Regex;

#[derive(Debug, Clone)]
pub struct ClaimCandidate {
    pub sentence: String,
    pub has_number: bool,
}

pub fn generate_candidates(text: &str) -> Vec<ClaimCandidate> {
    static SENTENCE_SPLIT: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    static DIGITS: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();

    let splitter = SENTENCE_SPLIT.get_or_init(|| Regex::new(r"(?m)(?<=[.!?])\s+").expect("regex"));
    let digits = DIGITS.get_or_init(|| Regex::new(r"\d").expect("regex"));

    splitter
        .split(text)
        .map(str::trim)
        .filter(|s| s.len() > 24)
        .filter(|s| {
            let lower = s.to_lowercase();
            digits.is_match(s)
                || lower.contains("conclusion")
                || lower.contains("outcome")
                || lower.contains("improve")
                || lower.contains("significant")
                || lower.contains("complication")
                || lower.contains("anatom")
                || lower.contains("operative")
        })
        .map(|sentence| ClaimCandidate {
            sentence: sentence.to_string(),
            has_number: digits.is_match(sentence),
        })
        .collect()
}
