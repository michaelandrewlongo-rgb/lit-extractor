use crate::config::AppConfig;
use crate::connectors::ApiClient;
use crate::db::docs_repo::DocsRepo;
use crate::db::Db;
use crate::errors::Result;
use crate::fs::layout::Paths;

#[derive(Clone)]
pub struct App {
    pub config: AppConfig,
    pub paths: Paths,
    pub db: Db,
    pub docs: DocsRepo,
    pub api: ApiClient,
}

impl App {
    pub fn new(config: AppConfig) -> Result<Self> {
        let paths = Paths::new(&config);
        let db = Db::open(&config.data.sqlite_path)?;
        let docs = DocsRepo::new(db.clone());
        let api = ApiClient::new(&config)?;
        Ok(Self {
            config,
            paths,
            db,
            docs,
            api,
        })
    }
}
