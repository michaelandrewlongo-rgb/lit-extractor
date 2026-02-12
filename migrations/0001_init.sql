CREATE TABLE IF NOT EXISTS docs (
    doc_id TEXT PRIMARY KEY,
    doi TEXT,
    pmid TEXT,
    title TEXT NOT NULL,
    journal TEXT,
    year INTEGER,
    authors TEXT,
    abstract TEXT,
    oa_status TEXT NOT NULL,
    oa_url TEXT,
    epmc_id TEXT,
    local_pdf_path TEXT,
    local_xml_path TEXT,
    sha256 TEXT,
    added_via TEXT NOT NULL,
    access_needed INTEGER NOT NULL DEFAULT 0,
    title_hash TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS runs (
    run_id TEXT PRIMARY KEY,
    command TEXT NOT NULL,
    created_at TEXT NOT NULL
);
