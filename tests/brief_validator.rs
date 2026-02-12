use chrono::Utc;
use lit::brief::compose::{BriefCitation, BriefJson, BriefTakeaway};
use lit::brief::validate::validate_brief;
use lit::domain::evidence::{AnchorType, ClaimType, EvidenceLedgerRow};

fn ledger_row() -> EvidenceLedgerRow {
    EvidenceLedgerRow {
        claim_id: "claim_1".into(),
        doc_id: "doc_1".into(),
        source_type: "oa".into(),
        doi: Some("10.1000/z".into()),
        pmid: Some("111".into()),
        url: None,
        local_path: "doc.pdf".into(),
        retrieved_at: Utc::now(),
        sha256: None,
        anchor_type: AnchorType::Pdf,
        page_number: Some(5),
        section_heading: None,
        anchor_quote: "The randomized trial showed lower complication rates".into(),
        claim_text: "The randomized trial showed lower complication rates".into(),
        claim_type: ClaimType::Outcome,
        numbers: None,
        errors: None,
    }
}

#[test]
fn brief_validator_accepts_valid_citations() {
    let brief = BriefJson {
        slug: "x".into(),
        query: "neurosurgery".into(),
        generated_at: Utc::now(),
        takeaways: vec![BriefTakeaway {
            text: "Lower complications were observed".into(),
            citation_ids: vec!["claim_1".into()],
        }],
        citations: vec![BriefCitation {
            claim_id: "claim_1".into(),
            doc_id: "doc_1".into(),
            doi: Some("10.1000/z".into()),
            pmid: Some("111".into()),
            anchor_type: "pdf".into(),
            page_number: Some(5),
            section_heading: None,
            anchor_quote: "The randomized trial showed lower complication rates".into(),
        }],
        key_figures: vec![],
    };

    let res = validate_brief(&brief, &[ledger_row()]);
    assert!(res.is_ok());
}

#[test]
fn brief_validator_rejects_anchor_mismatch() {
    let brief = BriefJson {
        slug: "x".into(),
        query: "neurosurgery".into(),
        generated_at: Utc::now(),
        takeaways: vec![BriefTakeaway {
            text: "Lower complications were observed".into(),
            citation_ids: vec!["claim_1".into()],
        }],
        citations: vec![BriefCitation {
            claim_id: "claim_1".into(),
            doc_id: "doc_1".into(),
            doi: Some("10.1000/z".into()),
            pmid: Some("111".into()),
            anchor_type: "pdf".into(),
            page_number: Some(99),
            section_heading: None,
            anchor_quote: "Wrong quote".into(),
        }],
        key_figures: vec![],
    };

    let res = validate_brief(&brief, &[ledger_row()]);
    assert!(res.is_err());
}
