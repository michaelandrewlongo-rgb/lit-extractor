use regex::Regex;

pub fn limit_quote_words(input: &str, max_words: usize) -> String {
    input
        .split_whitespace()
        .take(max_words)
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn quote_word_count(input: &str) -> usize {
    input.split_whitespace().count()
}

pub fn normalize_for_match(input: &str) -> String {
    static WS: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let ws = WS.get_or_init(|| Regex::new(r"\s+").expect("regex compiles"));
    ws.replace_all(&input.to_lowercase(), " ").trim().to_string()
}

pub fn quote_exists(haystack: &str, quote: &str) -> bool {
    let h = normalize_for_match(haystack);
    let q = normalize_for_match(quote);
    !q.is_empty() && h.contains(&q)
}
