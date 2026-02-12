use chrono::Utc;
use lit::db::docs_repo::DocsRepo;
use lit::db::Db;
use lit::types::SearchResult;
use tempfile::tempdir;

fn mk_repo() -> DocsRepo {
    let dir = tempdir().expect("tempdir");
    let db = Db::open(&dir.path().join("lit.db")).expect("db open");
    DocsRepo::new(db)
}

#[test]
fn dedupe_by_doi() {
    let repo = mk_repo();
    let a = SearchResult {
        source: "pubmed".into(),
        doi: Some("10.1000/test".into()),
        pmid: Some("123".into()),
        title: "Surgical Outcomes in Neurosurgery".into(),
        journal: Some("JNS".into()),
        year: Some(2024),
        authors: vec!["A".into()],
        abstract_text: None,
        oa_url: None,
        epmc_id: None,
        url: None,
    };
    let b = SearchResult {
        title: "Different title".into(),
        pmid: Some("999".into()),
        ..a.clone()
    };

    let d1 = repo.upsert_from_search(&a).expect("insert a");
    let d2 = repo.upsert_from_search(&b).expect("insert b");
    assert_eq!(d1.doc_id, d2.doc_id);
    assert_eq!(repo.list_docs().expect("list").len(), 1);
}

#[test]
fn dedupe_by_pmid_when_doi_missing() {
    let repo = mk_repo();
    let a = SearchResult {
        source: "pubmed".into(),
        doi: None,
        pmid: Some("44444".into()),
        title: "Trial data A".into(),
        journal: None,
        year: Some(2022),
        authors: vec![],
        abstract_text: None,
        oa_url: None,
        epmc_id: None,
        url: None,
    };
    let b = SearchResult {
        title: "Trial data B".into(),
        ..a.clone()
    };

    let d1 = repo.upsert_from_search(&a).expect("insert a");
    let d2 = repo.upsert_from_search(&b).expect("insert b");
    assert_eq!(d1.doc_id, d2.doc_id);
    assert_eq!(repo.list_docs().expect("list").len(), 1);
}

#[test]
fn dedupe_by_title_hash_and_year() {
    let repo = mk_repo();
    let a = SearchResult {
        source: "crossref".into(),
        doi: None,
        pmid: None,
        title: "Endoscopic skull base approach".into(),
        journal: None,
        year: Some(2021),
        authors: vec![],
        abstract_text: None,
        oa_url: None,
        epmc_id: None,
        url: None,
    };
    let b = SearchResult {
        source: "openalex".into(),
        ..a.clone()
    };

    let d1 = repo.upsert_from_search(&a).expect("insert a");
    let d2 = repo.upsert_from_search(&b).expect("insert b");
    assert_eq!(d1.doc_id, d2.doc_id);
}

#[test]
fn dedupe_by_sha256_on_local_ingest() {
    let repo = mk_repo();
    let d1 = repo
        .upsert_from_local(
            "Local PDF One",
            None,
            None,
            "abc123".to_string(),
            Some("a.pdf".to_string()),
            None,
        )
        .expect("insert one");
    let d2 = repo
        .upsert_from_local(
            "Local PDF Two",
            None,
            None,
            "abc123".to_string(),
            Some("b.pdf".to_string()),
            None,
        )
        .expect("insert two");

    assert_eq!(d1.doc_id, d2.doc_id);
    let list = repo.list_docs().expect("list");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].updated_at <= Utc::now(), true);
}
