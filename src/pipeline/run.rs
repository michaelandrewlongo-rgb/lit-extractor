use crate::cli::{
    BriefArgs, BuildDigestArgs, DownloadOaArgs, ExtractArgs, FetchArgs, QaArgs, RunArgs, SearchArgs,
};
use crate::errors::Result;
use crate::pipeline::app::App;

pub async fn run(app: &App, args: RunArgs) -> Result<()> {
    let search_args = SearchArgs {
        query: args.query.clone(),
        since: args.since,
        limit: args.limit,
        sources: vec![
            "pubmed".to_string(),
            "europepmc".to_string(),
            "crossref".to_string(),
            "openalex".to_string(),
            "clinicaltrials".to_string(),
        ],
        out: Some(app.paths.search_output_path()),
    };
    super::search::run(app, search_args).await?;

    let fetch_args = FetchArgs {
        input: app.paths.search_output_path(),
        enrich: true,
    };
    super::metadata::run(app, fetch_args).await?;

    super::download_oa::run(
        app,
        DownloadOaArgs {
            doc_ids: None,
            max: Some(args.limit),
            concurrency: 4,
        },
    )
    .await?;

    super::extract::run(
        app,
        ExtractArgs {
            doc_ids: None,
            concurrency: 2,
        },
    )
    .await?;

    let slug = slugify(&args.query);
    super::synthesis::run_digest(
        app,
        BuildDigestArgs {
            query: args.query,
            brief_slug: Some(slug.clone()),
        },
    )
    .await?;

    super::synthesis::run_brief(
        app,
        BriefArgs {
            brief_slug: slug,
            with_pdf: args.with_pdf,
            figures: app.config.pipeline.max_key_figures,
        },
    )
    .await?;

    super::qa::run(app, QaArgs { strict: None }).await?;
    Ok(())
}

fn slugify(input: &str) -> String {
    input
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
