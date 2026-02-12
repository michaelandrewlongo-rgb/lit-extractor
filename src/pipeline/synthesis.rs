use crate::brief::compose::{BriefJson, compose_brief};
use crate::brief::fig_integration::integrate_figures_and_sources;
use crate::brief::rank::score_claim;
use crate::brief::render_md::render_markdown;
use crate::brief::render_pdf::render_pdf;
use crate::brief::validate::{validate_brief, validate_brief_figures};
use crate::cli::{BriefArgs, BuildDigestArgs};
use crate::domain::evidence::EvidenceLedgerRow;
use crate::domain::figure::FigureIndexRow;
use crate::errors::Result;
use crate::pipeline::app::App;
use crate::pipeline::io::{read_json, read_jsonl, write_json};
use std::collections::HashMap;
use std::fs;

pub async fn run_digest(app: &App, args: BuildDigestArgs) -> Result<()> {
    let ledger: Vec<EvidenceLedgerRow> = read_jsonl(&app.paths.evidence_ledger_path())?;
    let figures: Vec<FigureIndexRow> = read_jsonl(&app.paths.figures_index_path())?;
    let docs = app.docs.list_docs()?;
    let doc_map = docs
        .iter()
        .map(|d| (d.doc_id.clone(), d.clone()))
        .collect::<HashMap<_, _>>();

    let mut ranked = ledger.clone();
    ranked.sort_by(|a, b| {
        let sa = score_claim(a, doc_map.get(&a.doc_id), &args.query);
        let sb = score_claim(b, doc_map.get(&b.doc_id), &args.query);
        sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
    });

    let digest = build_digest_markdown(&args.query, &ranked);
    fs::write(app.paths.digest_path(), digest)?;

    let slug = args
        .brief_slug
        .unwrap_or_else(|| slugify(&args.query));
    let brief_dir = app.paths.brief_dir(&slug);
    fs::create_dir_all(&brief_dir)?;

    let brief = compose_brief(
        slug.clone(),
        args.query,
        ranked,
        figures,
        8,
        app.config.pipeline.max_key_figures,
    );
    write_json(&brief_dir.join("brief.json"), &brief)?;

    let stubs = docs
        .into_iter()
        .filter(|d| d.access_needed)
        .collect::<Vec<_>>();
    write_json(&app.paths.stubs_path(), &stubs)?;

    tracing::info!(
        digest_path = %app.paths.digest_path().display(),
        brief_slug = %slug,
        access_needed = stubs.len(),
        "digest + brief json created"
    );

    Ok(())
}

pub async fn run_brief(app: &App, args: BriefArgs) -> Result<()> {
    let brief_dir = app.paths.brief_dir(&args.brief_slug);
    let brief_json_path = brief_dir.join("brief.json");
    let mut brief: BriefJson = read_json(&brief_json_path)?;

    let figures: Vec<FigureIndexRow> = read_jsonl(&app.paths.figures_index_path())?;
    let ledger: Vec<EvidenceLedgerRow> = read_jsonl(&app.paths.evidence_ledger_path())?;
    let docs = app.docs.list_docs()?;

    integrate_figures_and_sources(
        &mut brief,
        &docs,
        &figures,
        &brief_dir,
        app.config.pipeline.top_k_sources,
        args.figures,
    )?;

    validate_brief(&brief, &ledger)?;
    validate_brief_figures(&brief, &figures)?;

    let md = render_markdown(&brief);
    fs::write(brief_dir.join("brief.md"), md)?;

    if args.with_pdf {
        render_pdf(&brief, &brief_dir.join("brief.pdf"))?;
    }

    write_json(&brief_json_path, &brief)?;

    tracing::info!(slug = %args.brief_slug, with_pdf = args.with_pdf, "brief rendered");
    Ok(())
}

fn build_digest_markdown(query: &str, ranked: &[EvidenceLedgerRow]) -> String {
    let mut out = String::new();
    out.push_str(&format!("# Digest for query: {}\n\n", query));
    out.push_str("## Top Evidence\n\n");
    for (idx, claim) in ranked.iter().filter(|c| c.claim_text != "unknown").take(20).enumerate() {
        let anchor = match claim.anchor_type {
            crate::domain::evidence::AnchorType::Pdf => {
                format!("page {}", claim.page_number.unwrap_or_default())
            }
            crate::domain::evidence::AnchorType::Xml => claim
                .section_heading
                .clone()
                .unwrap_or_else(|| "Unknown section".to_string()),
        };
        out.push_str(&format!(
            "{}. {}\n   - claim_id: `{}`\n   - doc_id: `{}` doi={:?} pmid={:?}\n   - anchor: {}\n   - quote: \"{}\"\n",
            idx + 1,
            claim.claim_text,
            claim.claim_id,
            claim.doc_id,
            claim.doi,
            claim.pmid,
            anchor,
            claim.anchor_quote
        ));
    }
    out
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
