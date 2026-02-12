use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::{Value, json};

static NUM_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?x)
        (?P<value>[-+]?\d+(?:\.\d+)?)
        \s*(?P<unit>%|mm|cm|ml|mg|kg|years?|days?|months?)?
    ")
    .expect("regex compiles")
});

pub fn parse_numbers(input: &str) -> Option<Value> {
    let mut items = Vec::new();
    for cap in NUM_RE.captures_iter(input) {
        items.push(json!({
            "value": cap.name("value").map(|m| m.as_str()).unwrap_or_default(),
            "unit": cap.name("unit").map(|m| m.as_str()),
        }));
    }
    if items.is_empty() {
        None
    } else {
        Some(json!({"values": items}))
    }
}
