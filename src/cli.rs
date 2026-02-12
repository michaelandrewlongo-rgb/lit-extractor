use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "lit", version, about = "Biomedical literature harvester")]
pub struct Cli {
    #[arg(long)]
    pub config: Option<PathBuf>,
    #[arg(long)]
    pub data_dir: Option<PathBuf>,
    #[arg(long, default_value = "info")]
    pub log_level: String,
    #[arg(long, default_value_t = false)]
    pub no_color: bool,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Search(SearchArgs),
    Fetch(FetchArgs),
    #[command(name = "download-oa")]
    DownloadOa(DownloadOaArgs),
    #[command(name = "ingest-local")]
    IngestLocal(IngestLocalArgs),
    Extract(ExtractArgs),
    #[command(name = "build-digest")]
    BuildDigest(BuildDigestArgs),
    Brief(BriefArgs),
    Qa(QaArgs),
    Run(RunArgs),
}

#[derive(Debug, Args, Clone)]
pub struct SearchArgs {
    #[arg(long)]
    pub query: String,
    #[arg(long, default_value = "30d")]
    pub since: String,
    #[arg(long, default_value_t = 500)]
    pub limit: usize,
    #[arg(long, value_delimiter = ',', default_value = "pubmed,europepmc,crossref,openalex,clinicaltrials")]
    pub sources: Vec<String>,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Debug, Args, Clone)]
pub struct FetchArgs {
    #[arg(long)]
    pub input: PathBuf,
    #[arg(long, default_value_t = true)]
    pub enrich: bool,
}

#[derive(Debug, Args, Clone)]
pub struct DownloadOaArgs {
    #[arg(long)]
    pub doc_ids: Option<String>,
    #[arg(long)]
    pub max: Option<usize>,
    #[arg(long, default_value_t = 4)]
    pub concurrency: usize,
}

#[derive(Debug, Args, Clone)]
pub struct IngestLocalArgs {
    #[arg(long, default_value = "data/inbox")]
    pub inbox: PathBuf,
    #[arg(long, default_value_t = false)]
    pub recursive: bool,
    #[arg(long, default_value_t = false)]
    pub mv: bool,
}

#[derive(Debug, Args, Clone)]
pub struct ExtractArgs {
    #[arg(long)]
    pub doc_ids: Option<String>,
    #[arg(long, default_value_t = 2)]
    pub concurrency: usize,
}

#[derive(Debug, Args, Clone)]
pub struct BuildDigestArgs {
    #[arg(long)]
    pub query: String,
    #[arg(long)]
    pub brief_slug: Option<String>,
}

#[derive(Debug, Args, Clone)]
pub struct BriefArgs {
    #[arg(long)]
    pub brief_slug: String,
    #[arg(long, default_value_t = false)]
    pub with_pdf: bool,
    #[arg(long, default_value_t = 6)]
    pub figures: usize,
}

#[derive(Debug, Args, Clone)]
pub struct QaArgs {
    #[arg(long)]
    pub strict: Option<bool>,
}

#[derive(Debug, Args, Clone)]
pub struct RunArgs {
    #[arg(long)]
    pub query: String,
    #[arg(long, default_value = "30d")]
    pub since: String,
    #[arg(long, default_value_t = 500)]
    pub limit: usize,
    #[arg(long, default_value_t = false)]
    pub with_pdf: bool,
}
