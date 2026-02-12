use crate::cli::SearchArgs;
use crate::connectors::{clinicaltrials, crossref, epmc, openalex, pubmed};
use crate::db::schema::title_hash;
use crate::errors::Result;
use crate::pipeline::app::App;
use crate::pipeline::io::write_json;
use crate::types::{SearchOutput, SearchResult};
use chrono::Utc;
use std::collections::HashSet;

pub async fn run(app: &App, args: SearchArgs) -> Result<()> {
    let out_path = args.out.unwrap_or_else(|| app.paths.search_output_path());
    let per_source_limit = (args.limit / args.sources.len().max(1)).max(10);
    let mut all = Vec::new();

    for source in &args.sources {
        let mut rows = match source.as_str() {
            "pubmed" => pubmed::search(&app.api, &args.query, per_source_limit, &args.since).await?,
            "europepmc" => epmc::search(&app.api, &args.query, per_source_limit, &args.since).await?,
            "crossref" => crossref::search(&app.api, &args.query, per_source_limit).await?,
            "openalex" => openalex::search(&app.api, &args.query, per_source_limit).await?,
            "clinicaltrials" => {
                clinicaltrials::search(&app.api, &args.query, per_source_limit).await?
            }
            _ => Vec::new(),
        };
        all.append(&mut rows);
    }

    let deduped = dedupe_results(all);
    let output = SearchOutput {
        query: args.query,
        generated_at: Utc::now(),
        results: deduped.into_iter().take(args.limit).collect(),
    };
    write_json(&out_path, &output)?;

    tracing::info!(
        path = %out_path.display(),
        count = output.results.len(),
        "search results written"
    );

    Ok(())
}

fn dedupe_results(results: Vec<SearchResult>) -> Vec<SearchResult> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();

    for item in results {
        let key = if let Some(doi) = &item.doi {
            format!("doi:{}", doi.to_lowercase())
        } else if let Some(pmid) = &item.pmid {
            format!("pmid:{}", pmid)
        } else {
            format!("title:{}", title_hash(&item.title))
        };

        if seen.insert(key) {
            out.push(item);
        }
    }

    out
}
