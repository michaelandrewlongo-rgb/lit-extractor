use crate::config::AppConfig;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Paths {
    pub root: PathBuf,
}

impl Paths {
    pub fn new(cfg: &AppConfig) -> Self {
        Self {
            root: cfg.data.root.clone(),
        }
    }

    pub fn sqlite_path(&self) -> PathBuf {
        self.root.join("lit.db")
    }

    pub fn oa_doc_dir(&self, doc_id: &str) -> PathBuf {
        self.root.join("oa").join(doc_id)
    }

    pub fn local_doc_dir(&self, doc_id: &str) -> PathBuf {
        self.root.join("docs").join(doc_id)
    }

    pub fn inbox_dir(&self) -> PathBuf {
        self.root.join("inbox")
    }

    pub fn artifacts_dir(&self) -> PathBuf {
        self.root.join("artifacts")
    }

    pub fn evidence_ledger_path(&self) -> PathBuf {
        self.artifacts_dir().join("evidence_ledger.jsonl")
    }

    pub fn figures_index_path(&self) -> PathBuf {
        self.artifacts_dir().join("figures_index.jsonl")
    }

    pub fn digest_path(&self) -> PathBuf {
        self.artifacts_dir().join("digest.md")
    }

    pub fn stubs_path(&self) -> PathBuf {
        self.artifacts_dir().join("access_needed_stubs.json")
    }

    pub fn search_output_path(&self) -> PathBuf {
        self.artifacts_dir().join("search_results.json")
    }

    pub fn briefs_root(&self) -> PathBuf {
        self.root.join("briefs")
    }

    pub fn brief_dir(&self, slug: &str) -> PathBuf {
        self.briefs_root().join(slug)
    }
}
