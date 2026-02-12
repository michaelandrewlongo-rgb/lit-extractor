use crate::errors::Result;
use lopdf::Document;
use std::path::Path;

pub fn extract_pdf_pages(path: &Path) -> Result<Vec<(u32, String)>> {
    let doc = Document::load(path)?;
    let mut out = Vec::new();
    for (page_no, _obj_id) in doc.get_pages() {
        let text = doc.extract_text(&[page_no]).unwrap_or_default();
        if !text.trim().is_empty() {
            out.push((page_no, text));
        }
    }
    Ok(out)
}
