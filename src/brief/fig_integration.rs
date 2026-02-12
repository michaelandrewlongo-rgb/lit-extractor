use crate::brief::compose::{BriefJson, BriefKeyFigure};
use crate::domain::doc::DocRecord;
use crate::domain::figure::FigureIndexRow;
use crate::errors::Result;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn integrate_figures_and_sources(
    brief: &mut BriefJson,
    docs: &[DocRecord],
    figures: &[FigureIndexRow],
    brief_dir: &Path,
    top_k_sources: usize,
    max_figures: usize,
) -> Result<()> {
    let mut contribution: HashMap<String, usize> = HashMap::new();
    for c in &brief.citations {
        *contribution.entry(c.doc_id.clone()).or_insert(0) += 1;
    }

    let mut ranked_docs = contribution.into_iter().collect::<Vec<_>>();
    ranked_docs.sort_by(|a, b| b.1.cmp(&a.1));

    let sources_dir = brief_dir.join("sources");
    let figs_dir = brief_dir.join("figures");
    fs::create_dir_all(&sources_dir)?;
    fs::create_dir_all(&figs_dir)?;

    let selected_doc_ids = ranked_docs
        .into_iter()
        .take(top_k_sources)
        .map(|(doc_id, _)| doc_id)
        .collect::<Vec<_>>();

    for doc_id in &selected_doc_ids {
        if let Some(doc) = docs.iter().find(|d| &d.doc_id == doc_id) {
            if let Some(pdf) = &doc.local_pdf_path {
                let src = Path::new(pdf);
                if src.exists() {
                    let dst = sources_dir.join(src.file_name().unwrap_or_default());
                    let _ = fs::copy(src, dst);
                }
            }
            if let Some(xml) = &doc.local_xml_path {
                let src = Path::new(xml);
                if src.exists() {
                    let dst = sources_dir.join(src.file_name().unwrap_or_default());
                    let _ = fs::copy(src, dst);
                }
            }
        }
    }

    let mut selected_figs = Vec::new();
    for fig in figures
        .iter()
        .filter(|f| selected_doc_ids.contains(&f.doc_id))
        .take(max_figures)
    {
        let src = Path::new(&fig.figure_path);
        if src.exists() {
            let dst = figs_dir.join(src.file_name().unwrap_or_default());
            fs::copy(src, &dst)?;
            selected_figs.push(BriefKeyFigure {
                figure_id: fig.figure_id.clone(),
                doc_id: fig.doc_id.clone(),
                figure_path: dst.to_string_lossy().to_string(),
                caption: fig.caption.clone(),
                provenance: if let Some(page) = fig.page_number {
                    format!("doi={:?} pmid={:?} page={} license={:?}", fig.doi, fig.pmid, page, fig.license)
                } else {
                    format!(
                        "doi={:?} pmid={:?} xml_fig_id={:?} license={:?}",
                        fig.doi, fig.pmid, fig.xml_fig_id, fig.license
                    )
                },
                license: fig.license.clone(),
            });
        }
    }

    brief.key_figures = selected_figs;
    Ok(())
}
