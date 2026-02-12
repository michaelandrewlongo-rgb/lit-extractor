use crate::brief::compose::BriefJson;
use crate::domain::evidence::{AnchorType, EvidenceLedgerRow};
use crate::domain::figure::FigureIndexRow;
use crate::errors::{LitError, Result};
use std::collections::HashMap;
use std::path::Path;

pub fn validate_brief(brief: &BriefJson, ledger_rows: &[EvidenceLedgerRow]) -> Result<()> {
    let ledger_map = ledger_rows
        .iter()
        .map(|r| (r.claim_id.clone(), r))
        .collect::<HashMap<_, _>>();
    let cite_map = brief
        .citations
        .iter()
        .map(|c| (c.claim_id.clone(), c))
        .collect::<HashMap<_, _>>();

    for takeaway in &brief.takeaways {
        if takeaway.citation_ids.is_empty() {
            return Err(LitError::Validation(
                "brief takeaway without citation".to_string(),
            ));
        }
        for cid in &takeaway.citation_ids {
            let ledger = ledger_map.get(cid).ok_or_else(|| {
                LitError::Validation(format!("citation {} not found in evidence ledger", cid))
            })?;
            let cite = cite_map.get(cid).ok_or_else(|| {
                LitError::Validation(format!("citation {} missing from brief citations", cid))
            })?;

            let anchor_matches = match ledger.anchor_type {
                AnchorType::Pdf => {
                    cite.anchor_type == "pdf"
                        && cite.page_number.is_some()
                        && ledger.page_number == cite.page_number
                }
                AnchorType::Xml => {
                    cite.anchor_type == "xml"
                        && cite.section_heading.is_some()
                        && ledger.section_heading == cite.section_heading
                }
            };
            if !anchor_matches {
                return Err(LitError::Validation(format!(
                    "citation {} anchor mismatch with ledger",
                    cid
                )));
            }
            if ledger.anchor_quote != cite.anchor_quote {
                return Err(LitError::Validation(format!(
                    "citation {} quote mismatch with ledger",
                    cid
                )));
            }
        }
    }

    Ok(())
}

pub fn validate_brief_figures(brief: &BriefJson, figures_index: &[FigureIndexRow]) -> Result<()> {
    let fig_map = figures_index
        .iter()
        .map(|f| (f.figure_id.clone(), f))
        .collect::<HashMap<_, _>>();

    for fig in &brief.key_figures {
        if !Path::new(&fig.figure_path).exists() {
            return Err(LitError::Validation(format!(
                "figure file missing on disk: {}",
                fig.figure_path
            )));
        }
        if !fig_map.contains_key(&fig.figure_id) {
            return Err(LitError::Validation(format!(
                "figure {} missing from figures index",
                fig.figure_id
            )));
        }
    }

    Ok(())
}
