# lit-harvester

Production-focused CLI for biomedical literature harvesting, lawful OA retrieval, manual paywalled ingest, citation-grounded extraction, and neurosurgery-focused synthesis.

## Core guarantees

- No paywall/CAPTCHA/bot-control bypass.
- Public API + lawful OA only (`PubMed`, `Europe PMC`, `Crossref`, `OpenAlex`, `Unpaywall`, `ClinicalTrials.gov`).
- Non-OA handled via manual local ingest (`data/inbox`).
- Numeric claims/conclusions must have anchors (`pdf page + <=25-word quote` or `xml section + <=25-word quote`) or become `"unknown"` with errors.

## Quickstart

```powershell
cargo build
copy .env.example .env
```

Set required env vars:

- `UNPAYWALL_EMAIL` (recommended for OA resolution)
- `PUBMED_API_KEY` (optional, higher limits)

## Commands

```powershell
lit search --query "aneurysm clipping" --since 30d --limit 200
lit fetch --input data/artifacts/search_results.json --enrich
lit download-oa
lit ingest-local --inbox data/inbox --recursive
lit extract
lit build-digest --query "aneurysm clipping"
lit brief --brief-slug aneurysm-clipping --with-pdf
lit qa
lit run --query "aneurysm clipping" --since 30d --limit 500
```

## Outputs

- `data/artifacts/evidence_ledger.jsonl`
- `data/artifacts/figures_index.jsonl`
- `data/artifacts/digest.md`
- `data/artifacts/access_needed_stubs.json`
- `data/briefs/{slug}/brief.json`
- `data/briefs/{slug}/brief.md`
- `data/briefs/{slug}/brief.pdf` (optional)

## QA gate

`lit qa` reports:

- unique studies
- duplicates removed
- OA retrieval rate
- extraction success rate
- unanchored claim count

Default strict behavior exits nonzero when unanchored claims exist.

## Testing

```powershell
cargo test
```

Included tests:

- dedupe correctness (DOI/PMID/title hash/sha256)
- evidence and figure schema validation
- brief citation/anchor validation
- figure existence + index validation
