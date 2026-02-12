use crate::connectors::ApiClient;
use crate::errors::Result;
use crate::types::SearchResult;
use serde_json::Value;

pub async fn search(client: &ApiClient, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
    let url = format!(
        "https://api.crossref.org/works?query.bibliographic={}&rows={}",
        urlencoding::encode(query),
        limit.min(200)
    );
    let payload: Value = client.get_json_cached("crossref", &url).await?;
    let items = payload["message"]["items"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let mut out = Vec::new();
    for item in items {
        let title = item["title"]
            .as_array()
            .and_then(|a| a.first())
            .and_then(|x| x.as_str())
            .unwrap_or("Untitled")
            .to_string();
        let journal = item["container-title"]
            .as_array()
            .and_then(|a| a.first())
            .and_then(|x| x.as_str())
            .map(ToString::to_string);
        let year = item["published-print"]["date-parts"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|a| a.as_array())
            .and_then(|a| a.first())
            .and_then(|x| x.as_i64())
            .map(|v| v as i32);
        let authors = item["author"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|a| {
                format!(
                    "{} {}",
                    a["given"].as_str().unwrap_or_default(),
                    a["family"].as_str().unwrap_or_default()
                )
                .trim()
                .to_string()
            })
            .filter(|v| !v.is_empty())
            .collect::<Vec<_>>();
        let doi = item["DOI"].as_str().map(ToString::to_string);

        out.push(SearchResult {
            source: "crossref".to_string(),
            doi,
            pmid: None,
            title,
            journal,
            year,
            authors,
            abstract_text: item["abstract"].as_str().map(ToString::to_string),
            oa_url: None,
            epmc_id: None,
            url: item["URL"].as_str().map(ToString::to_string),
        });
    }

    Ok(out)
}
