use crate::cli::QaArgs;
use crate::domain::evidence::EvidenceLedgerRow;
use crate::errors::{LitError, Result};
use crate::pipeline::app::App;
use crate::pipeline::io::{read_json, read_jsonl};
use crate::types::{QaReport, SearchOutput};

pub async fn run(app: &App, args: QaArgs) -> Result<()> {
    let docs = app.docs.list_docs()?;
    let unique_studies = docs.len();

    let search_total = if app.paths.search_output_path().exists() {
        let s: SearchOutput = read_json(&app.paths.search_output_path())?;
        s.results.len()
    } else {
        unique_studies
    };
    let duplicates_removed = search_total.saturating_sub(unique_studies);

    let oa_total = docs.iter().filter(|d| d.oa_url.is_some()).count();
    let oa_downloaded = docs
        .iter()
        .filter(|d| d.oa_url.is_some() && (d.local_pdf_path.is_some() || d.local_xml_path.is_some()))
        .count();
    let oa_retrieval_rate = if oa_total == 0 {
        0.0
    } else {
        oa_downloaded as f64 / oa_total as f64
    };

    let ledger: Vec<EvidenceLedgerRow> = read_jsonl(&app.paths.evidence_ledger_path())?;
    let total_claims = ledger.len();
    let unanchored = ledger
        .iter()
        .filter(|r| r.claim_text == "unknown" || r.errors.as_ref().is_some_and(|v| !v.is_empty()))
        .count();
    let extraction_success_rate = if total_claims == 0 {
        0.0
    } else {
        (total_claims - unanchored) as f64 / total_claims as f64
    };

    let report = QaReport {
        unique_studies,
        duplicates_removed,
        oa_retrieval_rate,
        extraction_success_rate,
        unanchored_claim_count: unanchored,
    };

    println!("{}", serde_json::to_string_pretty(&report)?);

    let strict = args.strict.unwrap_or(app.config.pipeline.strict_qa);
    if strict && unanchored > 0 {
        return Err(LitError::Pipeline(format!(
            "qa gate failed: unanchored claim count is {}",
            unanchored
        )));
    }

    Ok(())
}
