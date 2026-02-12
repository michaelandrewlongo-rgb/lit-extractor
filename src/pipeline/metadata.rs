use crate::cli::FetchArgs;
use crate::connectors::unpaywall;
use crate::errors::Result;
use crate::pipeline::app::App;
use crate::pipeline::io::read_json;
use crate::types::SearchOutput;

pub async fn run(app: &App, args: FetchArgs) -> Result<()> {
    let mut payload: SearchOutput = read_json(&args.input)?;
    let mut stored = 0usize;
    let mut enriched = 0usize;

    for item in &mut payload.results {
        if args.enrich && item.oa_url.is_none() {
            if let Some(doi) = &item.doi {
                if !app.config.network.unpaywall_email.trim().is_empty() {
                    if let Ok(oa) = unpaywall::resolve_oa_url(
                        &app.api,
                        doi,
                        &app.config.network.unpaywall_email,
                    )
                    .await
                    {
                        if oa.is_some() {
                            enriched += 1;
                            item.oa_url = oa;
                        }
                    }
                }
            }
        }

        let _ = app.docs.upsert_from_search(item)?;
        stored += 1;
    }

    tracing::info!(stored, enriched, "metadata fetch complete");
    Ok(())
}
