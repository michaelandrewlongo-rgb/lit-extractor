use chrono::Utc;
use lit::brief::compose::{BriefJson, BriefKeyFigure};
use lit::brief::validate::validate_brief_figures;
use lit::domain::figure::FigureIndexRow;
use tempfile::tempdir;

#[test]
fn figure_validator_requires_file_and_index_row() {
    let dir = tempdir().expect("tempdir");
    let fig_path = dir.path().join("fig.bin");
    std::fs::write(&fig_path, b"figure-bytes").expect("write fig");

    let brief = BriefJson {
        slug: "slug".into(),
        query: "query".into(),
        generated_at: Utc::now(),
        takeaways: vec![],
        citations: vec![],
        key_figures: vec![BriefKeyFigure {
            figure_id: "fig_1".into(),
            doc_id: "doc_1".into(),
            figure_path: fig_path.to_string_lossy().to_string(),
            caption: Some("caption".into()),
            provenance: "pmid=1 page=1".into(),
            license: None,
        }],
    };

    let index = vec![FigureIndexRow {
        figure_id: "fig_1".into(),
        doc_id: "doc_1".into(),
        doi: None,
        pmid: None,
        local_doc_path: "doc.pdf".into(),
        figure_path: fig_path.to_string_lossy().to_string(),
        source_type: "pdf".into(),
        page_number: Some(1),
        xml_fig_id: None,
        figure_label: None,
        caption: Some("caption".into()),
        width: None,
        height: None,
        sha256: None,
        license: None,
        retrieved_at: Utc::now(),
    }];

    assert!(validate_brief_figures(&brief, &index).is_ok());

    let bad = BriefJson {
        key_figures: vec![BriefKeyFigure {
            figure_id: "fig_missing".into(),
            doc_id: "doc_1".into(),
            figure_path: dir.path().join("missing.bin").to_string_lossy().to_string(),
            caption: None,
            provenance: "x".into(),
            license: None,
        }],
        ..brief
    };

    assert!(validate_brief_figures(&bad, &index).is_err());
}
