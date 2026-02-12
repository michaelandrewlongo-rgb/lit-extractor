use crate::db::schema::{normalize_doi, normalize_pmid, title_hash};
use crate::domain::doc::{DocIdentity, DocRecord, OaStatus};
use crate::errors::{LitError, Result};
use chrono::Utc;
use rusqlite::{params, OptionalExtension};
use uuid::Uuid;

#[derive(Clone)]
pub struct DocsRepo {
    db: crate::db::Db,
}

impl DocsRepo {
    pub fn new(db: crate::db::Db) -> Self {
        Self { db }
    }

    pub fn upsert_from_search(&self, item: &crate::types::SearchResult) -> Result<DocRecord> {
        let th = title_hash(&item.title);
        let identity = DocIdentity {
            doi: item.doi.clone().map(|x| normalize_doi(&x)),
            pmid: item.pmid.clone().map(|x| normalize_pmid(&x)),
            title_hash: th.clone(),
            year: item.year,
            sha256: None,
        };

        if let Some(existing) = self.find_existing(&identity)? {
            return self.merge(existing, item);
        }

        let now = Utc::now();
        let doc = DocRecord {
            doc_id: format!("doc_{}", Uuid::new_v4()),
            doi: identity.doi.clone(),
            pmid: identity.pmid.clone(),
            title: item.title.clone(),
            journal: item.journal.clone(),
            year: item.year,
            authors: item.authors.clone(),
            abstract_text: item.abstract_text.clone(),
            oa_status: OaStatus::from_oa_url(item.oa_url.as_deref()),
            oa_url: item.oa_url.clone(),
            epmc_id: item.epmc_id.clone(),
            local_pdf_path: None,
            local_xml_path: None,
            sha256: None,
            added_via: "oa".to_string(),
            access_needed: item.oa_url.is_none(),
            title_hash: th,
            created_at: now,
            updated_at: now,
        };
        self.insert_doc(&doc)?;
        Ok(doc)
    }

    pub fn upsert_from_local(
        &self,
        title: &str,
        doi: Option<String>,
        pmid: Option<String>,
        sha256: String,
        local_pdf_path: Option<String>,
        local_xml_path: Option<String>,
    ) -> Result<DocRecord> {
        let th = title_hash(title);
        let identity = DocIdentity {
            doi: doi.clone().map(|d| normalize_doi(&d)),
            pmid: pmid.clone().map(|p| normalize_pmid(&p)),
            title_hash: th.clone(),
            year: None,
            sha256: Some(sha256.clone()),
        };
        if let Some(existing) = self.find_existing(&identity)? {
            {
                let conn = self.db.conn();
                let conn = conn.lock().expect("db mutex poisoned");
                conn.execute(
                    "UPDATE docs SET local_pdf_path = COALESCE(?, local_pdf_path), local_xml_path = COALESCE(?, local_xml_path), sha256 = COALESCE(?, sha256), added_via = 'inbox', updated_at = ? WHERE doc_id = ?",
                    params![local_pdf_path, local_xml_path, sha256, Utc::now().to_rfc3339(), existing.doc_id],
                )?;
            }
            return self.get_doc(&existing.doc_id)?.ok_or_else(|| LitError::NotFound(existing.doc_id));
        }

        let now = Utc::now();
        let doc = DocRecord {
            doc_id: format!("doc_{}", Uuid::new_v4()),
            doi: doi.map(|d| normalize_doi(&d)),
            pmid: pmid.map(|p| normalize_pmid(&p)),
            title: title.to_string(),
            journal: None,
            year: None,
            authors: vec![],
            abstract_text: None,
            oa_status: OaStatus::Unknown,
            oa_url: None,
            epmc_id: None,
            local_pdf_path,
            local_xml_path,
            sha256: Some(sha256),
            added_via: "inbox".to_string(),
            access_needed: false,
            title_hash: th,
            created_at: now,
            updated_at: now,
        };
        self.insert_doc(&doc)?;
        Ok(doc)
    }

    pub fn get_doc(&self, doc_id: &str) -> Result<Option<DocRecord>> {
        let conn = self.db.conn();
        let conn = conn.lock().expect("db mutex poisoned");
        let mut stmt = conn.prepare(
            "SELECT doc_id,doi,pmid,title,journal,year,authors,abstract,oa_status,oa_url,epmc_id,local_pdf_path,local_xml_path,sha256,added_via,access_needed,title_hash,created_at,updated_at FROM docs WHERE doc_id = ?",
        )?;
        let row = stmt
            .query_row(params![doc_id], |r| map_doc_row(r))
            .optional()?;
        Ok(row)
    }

    pub fn list_docs(&self) -> Result<Vec<DocRecord>> {
        let conn = self.db.conn();
        let conn = conn.lock().expect("db mutex poisoned");
        let mut stmt = conn.prepare("SELECT doc_id,doi,pmid,title,journal,year,authors,abstract,oa_status,oa_url,epmc_id,local_pdf_path,local_xml_path,sha256,added_via,access_needed,title_hash,created_at,updated_at FROM docs ORDER BY created_at DESC")?;
        let mut rows = stmt.query([])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            out.push(map_doc_row(row)?);
        }
        Ok(out)
    }

    pub fn list_docs_needing_oa(&self, max: Option<usize>) -> Result<Vec<DocRecord>> {
        let docs = self
            .list_docs()?
            .into_iter()
            .filter(|d| d.local_pdf_path.is_none() && d.local_xml_path.is_none() && (d.oa_url.is_some() || d.epmc_id.is_some()))
            .collect::<Vec<_>>();
        Ok(if let Some(m) = max { docs.into_iter().take(m).collect() } else { docs })
    }

    pub fn list_docs_for_extraction(&self, doc_ids: Option<&str>) -> Result<Vec<DocRecord>> {
        let ids = doc_ids.map(|csv| {
            csv.split(',')
                .map(|x| x.trim().to_string())
                .filter(|x| !x.is_empty())
                .collect::<Vec<_>>()
        });
        let docs = self.list_docs()?;
        let filtered = docs
            .into_iter()
            .filter(|d| d.local_pdf_path.is_some() || d.local_xml_path.is_some())
            .filter(|d| {
                if let Some(ids) = &ids {
                    ids.contains(&d.doc_id)
                } else {
                    true
                }
            })
            .collect();
        Ok(filtered)
    }

    pub fn update_local_paths(
        &self,
        doc_id: &str,
        pdf: Option<String>,
        xml: Option<String>,
        sha256: Option<String>,
    ) -> Result<()> {
        let conn = self.db.conn();
        let conn = conn.lock().expect("db mutex poisoned");
        conn.execute(
            "UPDATE docs SET local_pdf_path = COALESCE(?, local_pdf_path), local_xml_path = COALESCE(?, local_xml_path), sha256 = COALESCE(?, sha256), access_needed = CASE WHEN COALESCE(?, local_pdf_path) IS NULL AND COALESCE(?, local_xml_path) IS NULL THEN 1 ELSE 0 END, updated_at = ? WHERE doc_id = ?",
            params![pdf, xml, sha256, pdf, xml, Utc::now().to_rfc3339(), doc_id],
        )?;
        Ok(())
    }

    pub fn count_docs(&self) -> Result<usize> {
        let conn = self.db.conn();
        let conn = conn.lock().expect("db mutex poisoned");
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM docs", [], |r| r.get(0))?;
        Ok(count as usize)
    }

    pub fn count_access_needed(&self) -> Result<usize> {
        let conn = self.db.conn();
        let conn = conn.lock().expect("db mutex poisoned");
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM docs WHERE access_needed = 1", [], |r| r.get(0))?;
        Ok(count as usize)
    }

    pub fn find_existing(&self, identity: &DocIdentity) -> Result<Option<DocRecord>> {
        let conn = self.db.conn();
        let conn = conn.lock().expect("db mutex poisoned");

        if let Some(doi) = &identity.doi {
            if let Some(doc) = conn
                .query_row(
                    "SELECT doc_id,doi,pmid,title,journal,year,authors,abstract,oa_status,oa_url,epmc_id,local_pdf_path,local_xml_path,sha256,added_via,access_needed,title_hash,created_at,updated_at FROM docs WHERE doi = ?",
                    params![doi],
                    map_doc_row,
                )
                .optional()?
            {
                return Ok(Some(doc));
            }
        }

        if let Some(pmid) = &identity.pmid {
            if let Some(doc) = conn
                .query_row(
                    "SELECT doc_id,doi,pmid,title,journal,year,authors,abstract,oa_status,oa_url,epmc_id,local_pdf_path,local_xml_path,sha256,added_via,access_needed,title_hash,created_at,updated_at FROM docs WHERE pmid = ?",
                    params![pmid],
                    map_doc_row,
                )
                .optional()?
            {
                return Ok(Some(doc));
            }
        }

        if let Some(sha) = &identity.sha256 {
            if let Some(doc) = conn
                .query_row(
                    "SELECT doc_id,doi,pmid,title,journal,year,authors,abstract,oa_status,oa_url,epmc_id,local_pdf_path,local_xml_path,sha256,added_via,access_needed,title_hash,created_at,updated_at FROM docs WHERE sha256 = ?",
                    params![sha],
                    map_doc_row,
                )
                .optional()?
            {
                return Ok(Some(doc));
            }
        }

        let mut stmt = conn.prepare(
            "SELECT doc_id,doi,pmid,title,journal,year,authors,abstract,oa_status,oa_url,epmc_id,local_pdf_path,local_xml_path,sha256,added_via,access_needed,title_hash,created_at,updated_at FROM docs WHERE title_hash = ?",
        )?;
        let mut rows = stmt.query(params![identity.title_hash])?;
        while let Some(row) = rows.next()? {
            let doc = map_doc_row(row)?;
            if identity.year.is_none() || identity.year == doc.year {
                return Ok(Some(doc));
            }
        }

        Ok(None)
    }

    fn merge(&self, existing: DocRecord, item: &crate::types::SearchResult) -> Result<DocRecord> {
        let merged_oa = existing.oa_url.clone().or(item.oa_url.clone());
        let access_needed = merged_oa.is_none()
            && existing.local_pdf_path.is_none()
            && existing.local_xml_path.is_none();

        {
            let conn = self.db.conn();
            let conn = conn.lock().expect("db mutex poisoned");
            conn.execute(
                "UPDATE docs SET title = COALESCE(NULLIF(?, ''), title), journal = COALESCE(?, journal), year = COALESCE(?, year), authors = COALESCE(?, authors), abstract = COALESCE(?, abstract), oa_url = COALESCE(?, oa_url), epmc_id = COALESCE(?, epmc_id), oa_status = ?, access_needed = ?, updated_at = ? WHERE doc_id = ?",
                params![
                    item.title,
                    item.journal,
                    item.year,
                    serde_json::to_string(&item.authors).unwrap_or_else(|_| "[]".to_string()),
                    item.abstract_text,
                    item.oa_url,
                    item.epmc_id,
                    OaStatus::from_oa_url(merged_oa.as_deref()).as_str(),
                    access_needed as i32,
                    Utc::now().to_rfc3339(),
                    existing.doc_id,
                ],
            )?;
        }
        self.get_doc(&existing.doc_id)?
            .ok_or_else(|| LitError::NotFound(existing.doc_id))
    }

    fn insert_doc(&self, doc: &DocRecord) -> Result<()> {
        let conn = self.db.conn();
        let conn = conn.lock().expect("db mutex poisoned");
        conn.execute(
            "INSERT INTO docs (doc_id,doi,pmid,title,journal,year,authors,abstract,oa_status,oa_url,epmc_id,local_pdf_path,local_xml_path,sha256,added_via,access_needed,title_hash,created_at,updated_at)
             VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
            params![
                doc.doc_id,
                doc.doi,
                doc.pmid,
                doc.title,
                doc.journal,
                doc.year,
                serde_json::to_string(&doc.authors).unwrap_or_else(|_| "[]".to_string()),
                doc.abstract_text,
                doc.oa_status.as_str(),
                doc.oa_url,
                doc.epmc_id,
                doc.local_pdf_path,
                doc.local_xml_path,
                doc.sha256,
                doc.added_via,
                doc.access_needed as i32,
                doc.title_hash,
                doc.created_at.to_rfc3339(),
                doc.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }
}

fn map_doc_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<DocRecord> {
    let authors_raw: String = row.get(6)?;
    let authors = serde_json::from_str(&authors_raw).unwrap_or_default();

    let created_at_raw: String = row.get(17)?;
    let updated_at_raw: String = row.get(18)?;
    Ok(DocRecord {
        doc_id: row.get(0)?,
        doi: row.get(1)?,
        pmid: row.get(2)?,
        title: row.get(3)?,
        journal: row.get(4)?,
        year: row.get(5)?,
        authors,
        abstract_text: row.get(7)?,
        oa_status: OaStatus::parse(row.get::<_, String>(8)?.as_str()),
        oa_url: row.get(9)?,
        epmc_id: row.get(10)?,
        local_pdf_path: row.get(11)?,
        local_xml_path: row.get(12)?,
        sha256: row.get(13)?,
        added_via: row.get(14)?,
        access_needed: row.get::<_, i32>(15)? != 0,
        title_hash: row.get(16)?,
        created_at: chrono::DateTime::parse_from_rfc3339(&created_at_raw)
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(17, rusqlite::types::Type::Text, Box::new(e)))?
            .with_timezone(&Utc),
        updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at_raw)
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(18, rusqlite::types::Type::Text, Box::new(e)))?
            .with_timezone(&Utc),
    })
}
