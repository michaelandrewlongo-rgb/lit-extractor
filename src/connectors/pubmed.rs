use crate::connectors::ApiClient;
use crate::errors::Result;
use crate::types::SearchResult;
use serde_json::Value;

pub async fn search(client: &ApiClient, query: &str, limit: usize, since: &str) -> Result<Vec<SearchResult>> {
    let term = format!("{} AND {}", query, since_to_pubmed_clause(since));
    let encoded_term = urlencoding::encode(&term);
    let esearch_url = format!(
        "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esearch.fcgi?db=pubmed&retmode=json&retmax={}&term={}",
        limit.min(200),
        encoded_term
    );

    let esearch: Value = client.get_json_cached("pubmed", &esearch_url).await?;
    let ids = esearch["esearchresult"]["idlist"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(ToString::to_string))
        .collect::<Vec<_>>();

    if ids.is_empty() {
        return Ok(Vec::new());
    }

    let esummary_url = format!(
        "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esummary.fcgi?db=pubmed&retmode=json&id={}",
        ids.join(",")
    );
    let summary: Value = client.get_json_cached("pubmed", &esummary_url).await?;
    let mut out = Vec::new();

    for id in ids {
        let item = &summary["result"][&id];
        if item.is_null() {
            continue;
        }

        let title = item["title"].as_str().unwrap_or("Untitled").trim().to_string();
        let journal = item["fulljournalname"].as_str().map(ToString::to_string);
        let year = item["pubdate"]
            .as_str()
            .and_then(|v| v.split_whitespace().next())
            .and_then(|y| y.parse::<i32>().ok());
        let authors = item["authors"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|a| a["name"].as_str().map(ToString::to_string))
            .collect::<Vec<_>>();

        out.push(SearchResult {
            source: "pubmed".to_string(),
            doi: item["elocationid"]
                .as_str()
                .and_then(|s| if s.starts_with("doi:") { Some(s[4..].trim().to_string()) } else { None }),
            pmid: Some(id.clone()),
            title,
            journal,
            year,
            authors,
            abstract_text: None,
            oa_url: None,
            epmc_id: None,
            url: Some(format!("https://pubmed.ncbi.nlm.nih.gov/{id}/")),
        });
    }

    Ok(out)
}

fn since_to_pubmed_clause(since: &str) -> String {
    if let Some(days) = since.strip_suffix('d').and_then(|d| d.parse::<i64>().ok()) {
        format!("last {} days[dp]", days.max(1))
    } else {
        "last 30 days[dp]".to_string()
    }
}
