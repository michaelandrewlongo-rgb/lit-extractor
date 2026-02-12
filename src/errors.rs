use thiserror::Error;

pub type Result<T> = std::result::Result<T, LitError>;

#[derive(Debug, Error)]
pub enum LitError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("db error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("xml error: {0}")]
    Xml(#[from] roxmltree::Error),
    #[error("pdf error: {0}")]
    Pdf(#[from] lopdf::Error),
    #[error("config error: {0}")]
    Config(String),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("external service error: {0}")]
    External(String),
    #[error("pipeline error: {0}")]
    Pipeline(String),
}
