use crate::connectors::ApiClient;
use crate::errors::Result;
use crate::types::SearchResult;
use serde_json::Value;

pub async fn search(client: &ApiClient, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
    let url = format!(
        "https://clinicaltrials.gov/api/v2/studies?query.term={}&pageSize={}",
        urlencoding::encode(query),
        limit.min(100)
    );
    let payload: Value = client.get_json_cached("clinicaltrials", &url).await?;
    let studies = payload["studies"].as_array().cloned().unwrap_or_default();
    let mut out = Vec::new();

    for st in studies {
        let nct = st["protocolSection"]["identificationModule"]["nctId"]
            .as_str()
            .unwrap_or("unknown");
        let title = st["protocolSection"]["identificationModule"]["briefTitle"]
            .as_str()
            .unwrap_or("Clinical trial")
            .to_string();
        out.push(SearchResult {
            source: "clinicaltrials".to_string(),
            doi: None,
            pmid: None,
            title,
            journal: Some("ClinicalTrials.gov".to_string()),
            year: None,
            authors: vec![],
            abstract_text: st["protocolSection"]["descriptionModule"]["briefSummary"]
                .as_str()
                .map(ToString::to_string),
            oa_url: None,
            epmc_id: None,
            url: Some(format!("https://clinicaltrials.gov/study/{nct}")),
        });
    }

    Ok(out)
}
