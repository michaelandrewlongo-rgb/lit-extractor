use crate::domain::doc::DocRecord;
use crate::domain::figure::FigureIndexRow;
use crate::errors::Result;
use crate::fs::hash::sha256_bytes;
use chrono::Utc;
use lopdf::Document;
use std::fs;
use std::path::Path;
use uuid::Uuid;

pub fn extract_pdf_figures(doc: &DocRecord, pdf_path: &Path, out_dir: &Path) -> Result<Vec<FigureIndexRow>> {
    fs::create_dir_all(out_dir)?;
    let pdf = Document::load(pdf_path)?;
    let mut rows = Vec::new();

    for (page_number, page_id) in pdf.get_pages() {
        let images = match pdf.get_page_images(page_id) {
            Ok(v) => v,
            Err(_) => continue,
        };

        for (idx, image) in images.into_iter().enumerate() {
            let ext = pick_extension(image.filters.as_ref());
            let file_name = format!(
                "{}_p{}_obj{}_{}_{}.{}",
                doc.doc_id,
                page_number,
                image.id.0,
                image.id.1,
                idx + 1,
                ext
            );
            let figure_path = out_dir.join(file_name);
            fs::write(&figure_path, image.content)?;

            rows.push(FigureIndexRow {
                figure_id: format!("fig_{}", Uuid::new_v4()),
                doc_id: doc.doc_id.clone(),
                doi: doc.doi.clone(),
                pmid: doc.pmid.clone(),
                local_doc_path: pdf_path.to_string_lossy().to_string(),
                figure_path: figure_path.to_string_lossy().to_string(),
                source_type: "pdf".to_string(),
                page_number: Some(page_number),
                xml_fig_id: None,
                figure_label: Some(format!("PDF image {}", idx + 1)),
                caption: None,
                width: to_u32(image.width),
                height: to_u32(image.height),
                sha256: Some(sha256_bytes(image.content)),
                license: None,
                retrieved_at: Utc::now(),
            });
        }
    }

    Ok(rows)
}

fn to_u32(value: i64) -> Option<u32> {
    if value < 0 || value > u32::MAX as i64 {
        None
    } else {
        Some(value as u32)
    }
}

fn pick_extension(filters: Option<&Vec<String>>) -> &'static str {
    let Some(filters) = filters else {
        return "bin";
    };
    if filters.iter().any(|f| f.contains("DCTDecode")) {
        "jpg"
    } else if filters.iter().any(|f| f.contains("JPXDecode")) {
        "jp2"
    } else if filters.iter().any(|f| f.contains("JBIG2Decode")) {
        "jb2"
    } else if filters.iter().any(|f| f.contains("CCITTFaxDecode")) {
        "tiff"
    } else {
        "bin"
    }
}

#[cfg(test)]
mod tests {
    use super::pick_extension;

    #[test]
    fn picks_extension_from_filters() {
        let jpeg = Some(vec!["DCTDecode".to_string()]);
        let jp2 = Some(vec!["FlateDecode".to_string(), "JPXDecode".to_string()]);
        let none: Option<Vec<String>> = None;

        assert_eq!(pick_extension(jpeg.as_ref()), "jpg");
        assert_eq!(pick_extension(jp2.as_ref()), "jp2");
        assert_eq!(pick_extension(none.as_ref()), "bin");
    }
}
