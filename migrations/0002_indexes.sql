CREATE UNIQUE INDEX IF NOT EXISTS idx_docs_doi ON docs(doi) WHERE doi IS NOT NULL AND doi <> '';
CREATE UNIQUE INDEX IF NOT EXISTS idx_docs_pmid ON docs(pmid) WHERE pmid IS NOT NULL AND pmid <> '';
CREATE UNIQUE INDEX IF NOT EXISTS idx_docs_sha256 ON docs(sha256) WHERE sha256 IS NOT NULL AND sha256 <> '';
CREATE INDEX IF NOT EXISTS idx_docs_title_hash_year ON docs(title_hash, year);
CREATE INDEX IF NOT EXISTS idx_docs_access_needed ON docs(access_needed);
