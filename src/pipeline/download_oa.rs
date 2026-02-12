use crate::cli::DownloadOaArgs;
use crate::errors::Result;
use crate::fs::hash::sha256_bytes;
use crate::pipeline::app::App;
use std::fs;

pub async fn run(app: &App, args: DownloadOaArgs) -> Result<()> {
    if args.concurrency > 1 {
        tracing::info!(
            concurrency = args.concurrency,
            "downloader currently runs sequentially for deterministic file writes"
        );
    }

    let requested_ids = args.doc_ids.as_ref().map(|csv| {
        csv.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
    });

    let mut docs = app.docs.list_docs_needing_oa(args.max)?;
    if let Some(ids) = requested_ids {
        docs.retain(|d| ids.contains(&d.doc_id));
    }

    let mut downloaded = 0usize;
    for doc in docs {
        let url = if let Some(oa) = &doc.oa_url {
            oa.clone()
        } else if let Some(epmc_id) = &doc.epmc_id {
            format!(
                "https://www.ebi.ac.uk/europepmc/webservices/rest/{}/fullTextXML",
                epmc_id
            )
        } else {
            continue;
        };

        let source = source_for_url(&url);
        let bytes = app.api.download_bytes(source, &url).await?;

        let ext = if looks_like_xml(&bytes) { "xml" } else { "pdf" };
        let dir = app.paths.oa_doc_dir(&doc.doc_id);
        fs::create_dir_all(&dir)?;
        let file_path = dir.join(format!("document.{ext}"));
        fs::write(&file_path, &bytes)?;

        let sha = sha256_bytes(&bytes);
        let pdf_path = if ext == "pdf" {
            Some(file_path.to_string_lossy().to_string())
        } else {
            None
        };
        let xml_path = if ext == "xml" {
            Some(file_path.to_string_lossy().to_string())
        } else {
            None
        };

        app.docs
            .update_local_paths(&doc.doc_id, pdf_path, xml_path, Some(sha))?;
        downloaded += 1;
    }

    tracing::info!(downloaded, "OA download stage complete");
    Ok(())
}

fn looks_like_xml(bytes: &[u8]) -> bool {
    let preview = String::from_utf8_lossy(&bytes[..bytes.len().min(200)]).to_lowercase();
    preview.contains("<?xml") || preview.contains("<article")
}

fn source_for_url(url: &str) -> &'static str {
    if url.contains("europepmc") {
        "europepmc"
    } else if url.contains("crossref") {
        "crossref"
    } else if url.contains("openalex") {
        "openalex"
    } else {
        "unpaywall"
    }
}
