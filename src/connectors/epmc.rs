use crate::connectors::ApiClient;
use crate::errors::Result;
use crate::types::SearchResult;
use serde_json::Value;

pub async fn search(client: &ApiClient, query: &str, limit: usize, since: &str) -> Result<Vec<SearchResult>> {
    let q = if since.ends_with('d') {
        format!("{} FIRST_PDATE:[NOW-{} TO NOW]", query, since)
    } else {
        query.to_string()
    };
    let url = format!(
        "https://www.ebi.ac.uk/europepmc/webservices/rest/search?query={}&format=json&pageSize={}",
        urlencoding::encode(&q),
        limit.min(1000)
    );
    let payload: Value = client.get_json_cached("europepmc", &url).await?;
    let results = payload["resultList"]["result"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let mut out = Vec::new();

    for item in results {
        let doi = item["doi"].as_str().map(ToString::to_string);
        let pmid = item["pmid"].as_str().map(ToString::to_string);
        let epmc_id = item["id"].as_str().map(ToString::to_string);
        let is_oa = item["isOpenAccess"].as_str().unwrap_or("N") == "Y";
        let oa_url = if is_oa {
            epmc_id
                .clone()
                .map(|id| format!("https://www.ebi.ac.uk/europepmc/webservices/rest/{id}/fullTextXML"))
        } else {
            None
        };
        out.push(SearchResult {
            source: "europepmc".to_string(),
            doi,
            pmid,
            title: item["title"].as_str().unwrap_or("Untitled").to_string(),
            journal: item["journalTitle"].as_str().map(ToString::to_string),
            year: item["pubYear"].as_str().and_then(|y| y.parse::<i32>().ok()),
            authors: item["authorString"]
                .as_str()
                .map(|v| v.split(',').map(|x| x.trim().to_string()).collect())
                .unwrap_or_default(),
            abstract_text: item["abstractText"].as_str().map(ToString::to_string),
            oa_url,
            epmc_id,
            url: item["fullTextUrlList"]["fullTextUrl"]
                .as_array()
                .and_then(|arr| arr.first())
                .and_then(|x| x["url"].as_str())
                .map(ToString::to_string),
        });
    }

    Ok(out)
}

pub async fn get_jats_xml(client: &ApiClient, epmc_id: &str) -> Result<String> {
    let url = format!(
        "https://www.ebi.ac.uk/europepmc/webservices/rest/{}/fullTextXML",
        epmc_id
    );
    client.get_text_cached("europepmc", &url).await
}
