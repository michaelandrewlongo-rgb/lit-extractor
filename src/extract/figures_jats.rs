use crate::domain::doc::DocRecord;
use crate::domain::figure::FigureIndexRow;
use crate::errors::Result;
use crate::fs::hash::{sha256_bytes, sha256_file};
use chrono::Utc;
use roxmltree::Document;
use std::fs;
use std::path::Path;
use uuid::Uuid;

pub fn extract_jats_figures(doc: &DocRecord, xml_path: &Path, out_dir: &Path) -> Result<Vec<FigureIndexRow>> {
    let raw = fs::read_to_string(xml_path)?;
    let xml = Document::parse(&raw)?;
    fs::create_dir_all(out_dir)?;

    let mut rows = Vec::new();
    for fig in xml.descendants().filter(|n| n.has_tag_name("fig")) {
        let fig_id = fig.attribute("id").unwrap_or("unknown").to_string();
        let label = fig
            .children()
            .find(|n| n.has_tag_name("label"))
            .and_then(|n| n.text())
            .map(|s| s.trim().to_string());

        let caption = fig
            .descendants()
            .find(|n| n.has_tag_name("caption"))
            .and_then(|n| n.text())
            .map(|s| s.trim().to_string());

        let graphic_href = fig
            .descendants()
            .find(|n| n.has_tag_name("graphic"))
            .and_then(|n| n.attribute(("http://www.w3.org/1999/xlink", "href")))
            .map(ToString::to_string);

        let target_name = format!("{}_{}.bin", doc.doc_id, fig_id);
        let target_path = out_dir.join(target_name);

        if let Some(href) = graphic_href {
            let src = xml_path.parent().unwrap_or_else(|| Path::new(".")).join(href);
            if src.exists() {
                let bytes = fs::read(&src)?;
                fs::write(&target_path, &bytes)?;
            } else {
                let fallback = caption.clone().unwrap_or_else(|| "missing graphic asset".to_string());
                fs::write(&target_path, fallback.as_bytes())?;
            }
        } else {
            let fallback = caption.clone().unwrap_or_else(|| "figure caption unavailable".to_string());
            fs::write(&target_path, fallback.as_bytes())?;
        }

        let sha = if target_path.exists() {
            Some(sha256_file(&target_path)?)
        } else {
            Some(sha256_bytes(fig_id.as_bytes()))
        };

        rows.push(FigureIndexRow {
            figure_id: format!("fig_{}", Uuid::new_v4()),
            doc_id: doc.doc_id.clone(),
            doi: doc.doi.clone(),
            pmid: doc.pmid.clone(),
            local_doc_path: xml_path.to_string_lossy().to_string(),
            figure_path: target_path.to_string_lossy().to_string(),
            source_type: "jats".to_string(),
            page_number: None,
            xml_fig_id: Some(fig_id),
            figure_label: label,
            caption,
            width: None,
            height: None,
            sha256: sha,
            license: None,
            retrieved_at: Utc::now(),
        });
    }

    Ok(rows)
}
