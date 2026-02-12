# lit-harvester

Production-focused CLI for biomedical literature harvesting, lawful OA retrieval, manual paywalled ingest, citation-grounded extraction, and neurosurgery-focused synthesis.

## Core Guarantees

- No paywall/CAPTCHA/bot-control bypass.
- Public API + lawful OA only (`PubMed`, `Europe PMC`, `Crossref`, `OpenAlex`, `Unpaywall`, `ClinicalTrials.gov`).
- Non-OA handled via manual local ingest (`data/inbox`).
- Numeric claims/conclusions must have anchors (`pdf page + <=25-word quote` or `xml section + <=25-word quote`) or become `"unknown"` with errors.

## Step-by-Step Usage (PowerShell)

### 1. Prerequisites

- Rust toolchain installed (`cargo`, `rustc`)
- Internet access for API calls
- An email for Unpaywall (recommended)

Check tools:

```powershell
cargo --version
rustc --version
```

### 2. Build the project

From repo root:

```powershell
cargo build
```

### 3. Configure environment variables

Create your local env file:

```powershell
copy .env.example .env
```

Populate `.env` with:

- `UNPAYWALL_EMAIL=you@example.com` (recommended)
- `PUBMED_API_KEY=...` (optional)

In your current PowerShell session, export vars:

```powershell
$env:UNPAYWALL_EMAIL="you@example.com"
# optional:
# $env:PUBMED_API_KEY="your_key_here"
```

### 4. Verify the CLI is available

```powershell
cargo run -- --help
```

You should see commands like `search`, `fetch`, `download-oa`, `extract`, `brief`, `qa`, `run`.

### 5. Run end-to-end pipeline (fast first run)

```powershell
cargo run -- run --query "aneurysm clipping microsurgery" --since 30d --limit 50
```

This executes discovery, metadata enrichment, OA download, extraction, digest/brief generation, and QA.

### 6. Inspect output artifacts

After a successful run, check:

- `data/artifacts/search_results.json`
- `data/artifacts/evidence_ledger.jsonl`
- `data/artifacts/figures_index.jsonl`
- `data/artifacts/digest.md`
- `data/artifacts/access_needed_stubs.json`
- `data/briefs/{brief_slug}/brief.json`
- `data/briefs/{brief_slug}/brief.md`
- `data/briefs/{brief_slug}/brief.pdf` (if generated)

### 7. Ingest non-OA PDFs manually (optional but recommended)

For paywalled studies you obtained legally:

1. Drop PDFs into `data/inbox/`
2. Run:

```powershell
cargo run -- ingest-local --inbox data/inbox --recursive
```

3. Re-run extraction + synthesis:

```powershell
cargo run -- extract
cargo run -- build-digest --query "aneurysm clipping microsurgery"
cargo run -- brief --brief-slug aneurysm-clipping-microsurgery --with-pdf
```

### 8. Run QA explicitly

```powershell
cargo run -- qa
```

QA reports:

- unique studies
- duplicates removed
- OA retrieval rate
- extraction success rate
- unanchored claim count

By default, QA is strict and exits nonzero if unanchored claims exist.

## Command Reference

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

## Troubleshooting

- `UNPAYWALL_EMAIL is required`: set `$env:UNPAYWALL_EMAIL`.
- QA fails due to unanchored claims: inspect `evidence_ledger.jsonl` rows with `"claim_text":"unknown"` and `errors`.
- No figures extracted from some PDFs: not all PDFs expose images as extractable XObjects.

## Run Tests

```powershell
cargo test
```

Included tests:

- dedupe correctness (DOI/PMID/title hash/sha256)
- evidence and figure schema validation
- brief citation/anchor validation
- figure existence + index validation
