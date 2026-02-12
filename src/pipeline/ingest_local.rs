use crate::cli::IngestLocalArgs;
use crate::errors::Result;
use crate::extract::pdf_text::extract_pdf_pages;
use crate::fs::hash::sha256_file;
use crate::pipeline::app::App;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub async fn run(app: &App, args: IngestLocalArgs) -> Result<()> {
    let entries = collect_files(&args.inbox, args.recursive);
    let mut ingested = 0usize;

    for path in entries {
        let ext = path
            .extension()
            .and_then(|x| x.to_str())
            .unwrap_or_default()
            .to_lowercase();
        if ext != "pdf" && ext != "xml" {
            continue;
        }

        let sha = sha256_file(&path)?;
        let text = if ext == "pdf" {
            extract_pdf_pages(&path)
                .unwrap_or_default()
                .into_iter()
                .take(2)
                .map(|(_, t)| t)
                .collect::<Vec<_>>()
                .join(" ")
        } else {
            fs::read_to_string(&path).unwrap_or_default()
        };

        let doi = detect_doi(&text);
        let pmid = detect_pmid(&text);
        let title = detect_title(&text).unwrap_or_else(|| {
            path.file_stem()
                .and_then(|x| x.to_str())
                .unwrap_or("local_document")
                .to_string()
        });

        let doc = app
            .docs
            .upsert_from_local(&title, doi, pmid, sha.clone(), None, None)?;

        let dir = app.paths.local_doc_dir(&doc.doc_id);
        fs::create_dir_all(&dir)?;
        let target = dir.join(
            path.file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| format!("source.{ext}")),
        );

        if args.mv {
            fs::rename(&path, &target)?;
        } else {
            fs::copy(&path, &target)?;
        }

        let pdf_path = if ext == "pdf" {
            Some(target.to_string_lossy().to_string())
        } else {
            None
        };
        let xml_path = if ext == "xml" {
            Some(target.to_string_lossy().to_string())
        } else {
            None
        };

        app.docs
            .update_local_paths(&doc.doc_id, pdf_path, xml_path, Some(sha))?;

        ingested += 1;
    }

    tracing::info!(ingested, "manual local ingest complete");
    Ok(())
}

fn collect_files(root: &Path, recursive: bool) -> Vec<PathBuf> {
    if recursive {
        WalkDir::new(root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|e| e.path().to_path_buf())
            .collect()
    } else {
        fs::read_dir(root)
            .ok()
            .into_iter()
            .flat_map(|it| it.filter_map(|e| e.ok()))
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .collect()
    }
}

fn detect_doi(text: &str) -> Option<String> {
    let re = Regex::new(r"10\.\d{4,9}/[-._;()/:A-Z0-9]+(?i)").ok()?;
    re.find(text).map(|m| m.as_str().to_lowercase())
}

fn detect_pmid(text: &str) -> Option<String> {
    let re = Regex::new(r"(?i)pmid\s*[: ]\s*(\d{5,9})").ok()?;
    re.captures(text)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
}

fn detect_title(text: &str) -> Option<String> {
    text.lines()
        .map(str::trim)
        .find(|line| line.len() > 15 && !line.to_lowercase().contains("copyright"))
        .map(ToString::to_string)
}

