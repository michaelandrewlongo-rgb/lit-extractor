use crate::domain::evidence::EvidenceLedgerRow;
use crate::domain::figure::FigureIndexRow;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BriefTakeaway {
    pub text: String,
    pub citation_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BriefCitation {
    pub claim_id: String,
    pub doc_id: String,
    pub doi: Option<String>,
    pub pmid: Option<String>,
    pub anchor_type: String,
    pub page_number: Option<u32>,
    pub section_heading: Option<String>,
    pub anchor_quote: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BriefKeyFigure {
    pub figure_id: String,
    pub doc_id: String,
    pub figure_path: String,
    pub caption: Option<String>,
    pub provenance: String,
    pub license: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BriefJson {
    pub slug: String,
    pub query: String,
    pub generated_at: DateTime<Utc>,
    pub takeaways: Vec<BriefTakeaway>,
    pub citations: Vec<BriefCitation>,
    pub key_figures: Vec<BriefKeyFigure>,
}

pub fn compose_brief(
    slug: String,
    query: String,
    ranked_claims: Vec<EvidenceLedgerRow>,
    figures: Vec<FigureIndexRow>,
    max_takeaways: usize,
    max_figures: usize,
) -> BriefJson {
    let claims = ranked_claims
        .into_iter()
        .filter(|c| c.claim_text != "unknown")
        .take(max_takeaways)
        .collect::<Vec<_>>();

    let citations = claims
        .iter()
        .map(|claim| BriefCitation {
            claim_id: claim.claim_id.clone(),
            doc_id: claim.doc_id.clone(),
            doi: claim.doi.clone(),
            pmid: claim.pmid.clone(),
            anchor_type: match claim.anchor_type {
                crate::domain::evidence::AnchorType::Pdf => "pdf".to_string(),
                crate::domain::evidence::AnchorType::Xml => "xml".to_string(),
            },
            page_number: claim.page_number,
            section_heading: claim.section_heading.clone(),
            anchor_quote: claim.anchor_quote.clone(),
        })
        .collect::<Vec<_>>();

    let takeaways = claims
        .iter()
        .map(|claim| BriefTakeaway {
            text: claim.claim_text.clone(),
            citation_ids: vec![claim.claim_id.clone()],
        })
        .collect::<Vec<_>>();

    let key_figures = figures
        .into_iter()
        .take(max_figures)
        .map(|fig| {
            let provenance = if let Some(page) = fig.page_number {
                format!(
                    "doi={:?} pmid={:?} page={} source={}",
                    fig.doi, fig.pmid, page, fig.source_type
                )
            } else {
                format!(
                    "doi={:?} pmid={:?} xml_fig_id={:?} source={}",
                    fig.doi, fig.pmid, fig.xml_fig_id, fig.source_type
                )
            };
            BriefKeyFigure {
                figure_id: fig.figure_id,
                doc_id: fig.doc_id,
                figure_path: fig.figure_path,
                caption: fig.caption,
                provenance,
                license: fig.license,
            }
        })
        .collect::<Vec<_>>();

    BriefJson {
        slug,
        query,
        generated_at: Utc::now(),
        takeaways,
        citations,
        key_figures,
    }
}
