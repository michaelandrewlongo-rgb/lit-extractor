use crate::connectors::ApiClient;
use crate::errors::Result;
use crate::types::SearchResult;
use serde_json::Value;

pub async fn search(client: &ApiClient, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
    let url = format!(
        "https://api.openalex.org/works?search={}&per-page={}",
        urlencoding::encode(query),
        limit.min(200)
    );
    let payload: Value = client.get_json_cached("openalex", &url).await?;
    let items = payload["results"].as_array().cloned().unwrap_or_default();
    let mut out = Vec::new();

    for item in items {
        let doi = item["doi"].as_str().map(|s| s.replace("https://doi.org/", ""));
        let year = item["publication_year"].as_i64().map(|v| v as i32);
        let authors = item["authorships"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|a| a["author"]["display_name"].as_str().map(ToString::to_string))
            .collect::<Vec<_>>();
        let oa_url = item["open_access"]["oa_url"].as_str().map(ToString::to_string);
        out.push(SearchResult {
            source: "openalex".to_string(),
            doi,
            pmid: None,
            title: item["display_name"]
                .as_str()
                .unwrap_or("Untitled")
                .to_string(),
            journal: item["primary_location"]["source"]["display_name"]
                .as_str()
                .map(ToString::to_string),
            year,
            authors,
            abstract_text: None,
            oa_url,
            epmc_id: None,
            url: item["id"].as_str().map(ToString::to_string),
        });
    }

    Ok(out)
}
