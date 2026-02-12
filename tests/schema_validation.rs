use chrono::Utc;
use jsonschema::JSONSchema;
use lit::domain::evidence::{AnchorType, ClaimType, EvidenceLedgerRow};
use lit::domain::figure::FigureIndexRow;
use serde_json::Value;

#[test]
fn evidence_ledger_schema_validates_required_shape() {
    let schema: Value = serde_json::from_str(include_str!("../schemas/evidence_ledger.schema.json"))
        .expect("schema parse");
    let compiled = JSONSchema::compile(&schema).expect("compile");

    let row = EvidenceLedgerRow {
        claim_id: "claim_1".into(),
        doc_id: "doc_1".into(),
        source_type: "oa".into(),
        doi: Some("10.1000/x".into()),
        pmid: Some("123".into()),
        url: Some("https://example.org".into()),
        local_path: "data/docs/doc_1/file.pdf".into(),
        retrieved_at: Utc::now(),
        sha256: Some("abcd".into()),
        anchor_type: AnchorType::Pdf,
        page_number: Some(3),
        section_heading: None,
        anchor_quote: "Outcome improved at 12 months with 30% reduction".into(),
        claim_text: "Outcome improved at 12 months with 30% reduction".into(),
        claim_type: ClaimType::Outcome,
        numbers: None,
        errors: None,
    };

    let value = serde_json::to_value(row).expect("to value");
    assert!(compiled.is_valid(&value));
}

#[test]
fn figures_index_schema_validates_required_shape() {
    let schema: Value = serde_json::from_str(include_str!("../schemas/figures_index.schema.json"))
        .expect("schema parse");
    let compiled = JSONSchema::compile(&schema).expect("compile");

    let row = FigureIndexRow {
        figure_id: "fig_1".into(),
        doc_id: "doc_1".into(),
        doi: Some("10.1000/y".into()),
        pmid: Some("789".into()),
        local_doc_path: "data/docs/doc_1/doc.xml".into(),
        figure_path: "data/docs/doc_1/figures/fig_1.bin".into(),
        source_type: "jats".into(),
        page_number: None,
        xml_fig_id: Some("F1".into()),
        figure_label: Some("Figure 1".into()),
        caption: Some("Operative field".into()),
        width: Some(800),
        height: Some(600),
        sha256: Some("beef".into()),
        license: Some("CC-BY".into()),
        retrieved_at: Utc::now(),
    };

    let value = serde_json::to_value(row).expect("to value");
    assert!(compiled.is_valid(&value));
}
