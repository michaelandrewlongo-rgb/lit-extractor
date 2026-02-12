use crate::connectors::ApiClient;
use crate::errors::{LitError, Result};
use serde_json::Value;

pub async fn resolve_oa_url(client: &ApiClient, doi: &str, email: &str) -> Result<Option<String>> {
    if email.trim().is_empty() {
        return Err(LitError::Config(
            "UNPAYWALL_EMAIL is required for Unpaywall lookups".to_string(),
        ));
    }
    let url = format!(
        "https://api.unpaywall.org/v2/{}?email={}",
        urlencoding::encode(doi),
        urlencoding::encode(email)
    );
    let payload: Value = client.get_json_cached("unpaywall", &url).await?;

    if let Some(best) = payload["best_oa_location"]["url_for_pdf"].as_str() {
        return Ok(Some(best.to_string()));
    }
    if let Some(best) = payload["best_oa_location"]["url"].as_str() {
        return Ok(Some(best.to_string()));
    }

    Ok(None)
}
