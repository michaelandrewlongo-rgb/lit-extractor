use crate::cli::ExtractArgs;
use crate::domain::doc::DocRecord;
use crate::domain::evidence::{AnchorType, ClaimType, EvidenceLedgerRow};
use crate::domain::figure::FigureIndexRow;
use crate::errors::Result;
use crate::extract::claim_candidate::generate_candidates;
use crate::extract::claim_verify::verify_candidate;
use crate::extract::figures_jats::extract_jats_figures;
use crate::extract::figures_pdf::extract_pdf_figures;
use crate::extract::numbers::parse_numbers;
use crate::extract::pdf_text::extract_pdf_pages;
use crate::extract::xml_text::extract_xml_sections;
use crate::pipeline::app::App;
use crate::pipeline::io::write_jsonl;
use chrono::Utc;
use jsonschema::JSONSchema;
use serde_json::Value;
use std::path::Path;
use uuid::Uuid;

pub async fn run(app: &App, args: ExtractArgs) -> Result<()> {
    if args.concurrency > 1 {
        tracing::info!(
            concurrency = args.concurrency,
            "extractor currently runs sequentially for deterministic ordering"
        );
    }

    let docs = app.docs.list_docs_for_extraction(args.doc_ids.as_deref())?;
    let evidence_schema = compile_schema(include_str!("../../schemas/evidence_ledger.schema.json"))?;
    let figure_schema = compile_schema(include_str!("../../schemas/figures_index.schema.json"))?;

    let mut ledger_rows = Vec::new();
    let mut figure_rows = Vec::new();

    for doc in docs {
        if let Some(pdf_path) = &doc.local_pdf_path {
            process_pdf(&doc, pdf_path, &mut ledger_rows)?;
            let out_dir = app.paths.local_doc_dir(&doc.doc_id).join("figures");
            let mut figs = extract_pdf_figures(&doc, Path::new(pdf_path), &out_dir)?;
            figure_rows.append(&mut figs);
        }
        if let Some(xml_path) = &doc.local_xml_path {
            process_xml(&doc, xml_path, &mut ledger_rows)?;
            let out_dir = app.paths.local_doc_dir(&doc.doc_id).join("figures");
            let mut figs = extract_jats_figures(&doc, Path::new(xml_path), &out_dir)?;
            figure_rows.append(&mut figs);
        }
    }

    enforce_schema_for_ledger(&mut ledger_rows, &evidence_schema)?;
    enforce_schema_for_figures(&figure_rows, &figure_schema)?;

    write_jsonl(&app.paths.evidence_ledger_path(), &ledger_rows)?;
    write_jsonl(&app.paths.figures_index_path(), &figure_rows)?;

    tracing::info!(
        evidence_rows = ledger_rows.len(),
        figure_rows = figure_rows.len(),
        "extraction artifacts written"
    );

    Ok(())
}

fn process_pdf(doc: &DocRecord, pdf_path: &str, out: &mut Vec<EvidenceLedgerRow>) -> Result<()> {
    let pages = extract_pdf_pages(Path::new(pdf_path))?;
    for (page_no, text) in pages {
        for candidate in generate_candidates(&text) {
            let verify = verify_candidate(&candidate, &text);
            let mut claim_text = candidate.sentence.clone();
            let mut errors = None;
            if !verify.verified {
                claim_text = "unknown".to_string();
                errors = Some(verify.errors.clone());
            }

            out.push(EvidenceLedgerRow {
                claim_id: format!("claim_{}", Uuid::new_v4()),
                doc_id: doc.doc_id.clone(),
                source_type: doc.added_via.clone(),
                doi: doc.doi.clone(),
                pmid: doc.pmid.clone(),
                url: doc.oa_url.clone(),
                local_path: pdf_path.to_string(),
                retrieved_at: Utc::now(),
                sha256: doc.sha256.clone(),
                anchor_type: AnchorType::Pdf,
                page_number: Some(page_no),
                section_heading: None,
                anchor_quote: if verify.anchor_quote.is_empty() {
                    candidate
                        .sentence
                        .split_whitespace()
                        .take(25)
                        .collect::<Vec<_>>()
                        .join(" ")
                } else {
                    verify.anchor_quote
                },
                claim_text,
                claim_type: ClaimType::classify(&candidate.sentence),
                numbers: parse_numbers(&candidate.sentence),
                errors,
            });
        }
    }
    Ok(())
}

fn process_xml(doc: &DocRecord, xml_path: &str, out: &mut Vec<EvidenceLedgerRow>) -> Result<()> {
    let sections = extract_xml_sections(Path::new(xml_path))?;
    for section in sections {
        for candidate in generate_candidates(&section.body) {
            let verify = verify_candidate(&candidate, &section.body);
            let mut claim_text = candidate.sentence.clone();
            let mut errors = None;
            if !verify.verified {
                claim_text = "unknown".to_string();
                errors = Some(verify.errors.clone());
            }

            out.push(EvidenceLedgerRow {
                claim_id: format!("claim_{}", Uuid::new_v4()),
                doc_id: doc.doc_id.clone(),
                source_type: doc.added_via.clone(),
                doi: doc.doi.clone(),
                pmid: doc.pmid.clone(),
                url: doc.oa_url.clone(),
                local_path: xml_path.to_string(),
                retrieved_at: Utc::now(),
                sha256: doc.sha256.clone(),
                anchor_type: AnchorType::Xml,
                page_number: None,
                section_heading: Some(section.heading.clone()),
                anchor_quote: if verify.anchor_quote.is_empty() {
                    candidate
                        .sentence
                        .split_whitespace()
                        .take(25)
                        .collect::<Vec<_>>()
                        .join(" ")
                } else {
                    verify.anchor_quote
                },
                claim_text,
                claim_type: ClaimType::classify(&candidate.sentence),
                numbers: parse_numbers(&candidate.sentence),
                errors,
            });
        }
    }
    Ok(())
}

fn compile_schema(raw: &str) -> Result<JSONSchema> {
    let v: Value = serde_json::from_str(raw)?;
    JSONSchema::compile(&v).map_err(|e| crate::errors::LitError::Validation(e.to_string()))
}

fn enforce_schema_for_ledger(rows: &mut [EvidenceLedgerRow], schema: &JSONSchema) -> Result<()> {
    for row in rows {
        let val = serde_json::to_value(&*row)?;
        if let Err(errors) = schema.validate(&val) {
            let mut err_messages = errors.map(|e| e.to_string()).collect::<Vec<_>>();
            row.claim_text = "unknown".to_string();
            match &mut row.errors {
                Some(existing) => existing.append(&mut err_messages),
                None => row.errors = Some(err_messages),
            }
        }
    }
    Ok(())
}

fn enforce_schema_for_figures(rows: &[FigureIndexRow], schema: &JSONSchema) -> Result<()> {
    for row in rows {
        let val = serde_json::to_value(row)?;
        if let Err(errors) = schema.validate(&val) {
            return Err(crate::errors::LitError::Validation(format!(
                "figure schema violation for {}: {}",
                row.figure_id,
                errors.map(|e| e.to_string()).collect::<Vec<_>>().join("; ")
            )));
        }
    }
    Ok(())
}
