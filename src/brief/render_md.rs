use crate::brief::compose::BriefJson;

pub fn render_markdown(brief: &BriefJson) -> String {
    let mut out = String::new();
    out.push_str(&format!("# Neurosurgery Brief: {}\n\n", brief.query));
    out.push_str(&format!("Generated: {}\n\n", brief.generated_at.to_rfc3339()));

    out.push_str("## Key Takeaways\n\n");
    for (idx, takeaway) in brief.takeaways.iter().enumerate() {
        let cites = takeaway
            .citation_ids
            .iter()
            .map(|id| format!("`{id}`"))
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!("{}. {} [{}]\n", idx + 1, takeaway.text, cites));
    }

    out.push_str("\n## Citations\n\n");
    for citation in &brief.citations {
        let anchor = if citation.anchor_type == "pdf" {
            format!("page {}", citation.page_number.unwrap_or_default())
        } else {
            citation
                .section_heading
                .clone()
                .unwrap_or_else(|| "Unknown section".to_string())
        };
        out.push_str(&format!(
            "- `{}` doc={} doi={:?} pmid={:?} anchor={} quote=\"{}\"\n",
            citation.claim_id,
            citation.doc_id,
            citation.doi,
            citation.pmid,
            anchor,
            citation.anchor_quote
        ));
    }

    out.push_str("\n## Key Figures\n\n");
    for fig in &brief.key_figures {
        out.push_str(&format!(
            "- `{}` {}\n  - caption: {}\n  - provenance: {}\n  - license: {}\n",
            fig.figure_id,
            fig.figure_path,
            fig.caption.clone().unwrap_or_else(|| "N/A".to_string()),
            fig.provenance,
            fig.license.clone().unwrap_or_else(|| "unknown".to_string())
        ));
    }

    out
}
