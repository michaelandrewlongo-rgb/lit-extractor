use crate::extract::anchor::{limit_quote_words, quote_exists, quote_word_count};
use crate::extract::claim_candidate::ClaimCandidate;

#[derive(Debug, Clone)]
pub struct ClaimVerification {
    pub verified: bool,
    pub anchor_quote: String,
    pub errors: Vec<String>,
}

pub fn verify_candidate(candidate: &ClaimCandidate, source_text: &str) -> ClaimVerification {
    let mut quote = limit_quote_words(&candidate.sentence, 25);
    if quote_word_count(&quote) == 0 {
        return ClaimVerification {
            verified: false,
            anchor_quote: String::new(),
            errors: vec!["empty_quote".to_string()],
        };
    }
    if quote_word_count(&quote) > 25 {
        quote = limit_quote_words(&quote, 25);
    }

    let exists = quote_exists(source_text, &quote);
    let mut errors = Vec::new();
    if !exists {
        errors.push("anchor_quote_not_found".to_string());
    }

    ClaimVerification {
        verified: exists,
        anchor_quote: quote,
        errors,
    }
}
