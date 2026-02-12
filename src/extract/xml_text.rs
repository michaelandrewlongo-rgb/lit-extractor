use crate::errors::Result;
use roxmltree::Document;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct XmlSection {
    pub heading: String,
    pub body: String,
}

pub fn extract_xml_sections(path: &Path) -> Result<Vec<XmlSection>> {
    let raw = fs::read_to_string(path)?;
    extract_xml_sections_from_str(&raw)
}

pub fn extract_xml_sections_from_str(raw: &str) -> Result<Vec<XmlSection>> {
    let doc = Document::parse(raw)?;
    let mut out = Vec::new();
    for sec in doc.descendants().filter(|n| n.has_tag_name("sec")) {
        let heading = sec
            .children()
            .find(|n| n.has_tag_name("title"))
            .and_then(|n| n.text())
            .unwrap_or("Unknown Section")
            .trim()
            .to_string();
        let mut body = String::new();
        for p in sec.children().filter(|n| n.has_tag_name("p")) {
            if let Some(t) = p.text() {
                body.push_str(t);
                body.push(' ');
            }
        }
        if !body.trim().is_empty() {
            out.push(XmlSection { heading, body });
        }
    }
    Ok(out)
}
