use clap::Parser;
use lit::cli::{Cli, Commands};
use lit::config::AppConfig;
use lit::errors::Result;
use lit::pipeline;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = AppConfig::load(cli.config.as_deref(), cli.data_dir.as_deref())?;
    config.ensure_layout()?;
    tracing_subscriber::fmt()
        .with_env_filter(cli.log_level.clone())
        .with_target(false)
        .compact()
        .init();

    let app = pipeline::app::App::new(config)?;

    match cli.command {
        Commands::Search(args) => pipeline::search::run(&app, args).await?,
        Commands::Fetch(args) => pipeline::metadata::run(&app, args).await?,
        Commands::DownloadOa(args) => pipeline::download_oa::run(&app, args).await?,
        Commands::IngestLocal(args) => pipeline::ingest_local::run(&app, args).await?,
        Commands::Extract(args) => pipeline::extract::run(&app, args).await?,
        Commands::BuildDigest(args) => pipeline::synthesis::run_digest(&app, args).await?,
        Commands::Brief(args) => pipeline::synthesis::run_brief(&app, args).await?,
        Commands::Qa(args) => pipeline::qa::run(&app, args).await?,
        Commands::Run(args) => pipeline::run::run(&app, args).await?,
    }

    Ok(())
}
