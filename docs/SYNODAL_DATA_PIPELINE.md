# Synodal data pipeline

The authoritative source workflow is implemented in Rust. The shell script in
`references/fetch-sources.sh` is only a convenience wrapper. Normal fetch and
verification never rewrite a checksum or revision lock.

## Prerequisites and storage

- Rust 1.85 or newer;
- `curl` 7.71 or newer, `tar`, and `unzip` for acquisition/extraction;
- DjVuLibre (`djvutxt`, `djvused`, and `ddjvu`) and Tesseract 5 for the
  D’yachenko scan; the Russian recognition model itself is checksum-pinned in
  the source lock;
- SWORD utilities (`mod2imp`) for CrossWire CSlElizabeth; and
- approximately 4.6 GB for raw downloads plus 5–8 GB of temporary and
  intermediate space during a full run.

The currently tested local toolchain is DjVuLibre 3.5.30, Tesseract 5.5.1, and
SWORD 1.9.0. Their output is guarded by the committed candidate/review links and
determinism checks; changing an external tool requires reviewing output drift.

## Source commands

```sh
cargo xtask synodal-sources list
cargo xtask synodal-sources status
cargo xtask synodal-sources fetch
cargo xtask synodal-sources fetch --source alypy-gamanovich-grammar-web-2023
cargo xtask synodal-sources verify
cargo xtask synodal-sources verify --offline
```

Use `--cache PATH` with fetch, verify, status, or bootstrap to avoid the default
`references/downloads` cache. A wrong nonempty cached file is rejected; move it
aside or delete that one verified target and fetch again. A `.partial` file is
resumed only when the server proves byte-range support. It is admitted under the
final name only after size, signature, archive, and SHA-256 validation.

An offline full reconstruction is:

```sh
cargo xtask synodal-bootstrap --offline --cache references/downloads
```

To prove a clean, network-backed reconstruction without touching the normal
cache, start from a clean worktree and use a disposable directory:

```sh
SYNODAL_TEST_CACHE="$(mktemp -d)"
cargo xtask synodal-bootstrap --cache "$SYNODAL_TEST_CACHE"
cargo xtask synodal-sources verify --offline --cache "$SYNODAL_TEST_CACHE"
git diff --exit-code -- \
  crates/synodal-church-slavonic/generated/registry.rs \
  crates/synodal-church-slavonic-dictionary/generated/registry.rs \
  reports/synodal-bootstrap.json \
  reports/synodal-evaluation.json \
  reports/synodal-evaluation.md \
  reports/synodal-extraction.json \
  reports/synodal-extraction.md \
  reports/synodal-verse-disagreement.json
```

The cache path is deliberately outside `references/downloads`; remove that
specific disposable directory after review. A complete fetch is approximately
4.6 GB, and D’yachenko OCR makes this a long-running acceptance check.

The bounded acceptance reconstruction used by default CI needs no raw cache or
network:

```sh
cargo xtask synodal-fixture-bootstrap
```

It starts with two empty disposable caches, serves a locked miniature Alypy page
from local HTTP, fetches and verifies each cache independently, compares adapter
bytes, reconstructs both runtime registries twice, compares them to the committed
files, evaluates held-out rows, and proves that source locks did not change.

## Data boundaries

1. `references/downloads/` contains immutable raw bytes and is gitignored.
2. `data/intermediate/synodal/` contains deterministic automatic JSONL
   candidates and is gitignored.
3. `data/quarantine/synodal/` contains rejected JSONL records with counted
   reasons and is gitignored.
4. `data/synodal/reviewed_evidence.tsv` and the other committed TSVs are the
   human-reviewed overlays. They are never written by source adapters.
5. `crates/*/generated/registry.rs` contains deterministic runtime-safe facts;
   runtime crates have no filesystem, network, archive, XML, JSON, TSV, or
   database reader.
6. `data/synodal/evaluation.tsv` and raw corpus passages are evaluation-only and
   cannot enter runtime packages.

`data/synodal/lexical_reviews.tsv` is the v0.3 admission overlay. A reviewed row
links one independently sourced semantic candidate to one locked target passage;
rejected rows and their reasons remain in the same table. Inflectable rows
without independently reviewed class or principal-part metadata become exact
`LexicalForm` evidence only. Source adapters and queue generators are forbidden
from writing this table. A semantic candidate may be inherited OCS evidence or
a Synodal normative source, but it must name a different source and candidate
from the target-passage attestation. A normative semantic candidate therefore
cannot serve as its own corpus witness.

`data/synodal/family_reviews.tsv` is the v0.4 morphological-family decision
layer. Its admitted rows cite the target lexeme, exact table or class scope,
stems/alternants where applicable, accent and positional metadata, normative and
target evidence, semantic identity, confidence, assumptions, and an explicit
review note. Deferred and rejected top-200 proposals remain in the same durable
table. Queue generation never edits it.

Reviewed evidence points to stable candidate IDs. A full bootstrap fails if a
reviewed candidate disappears or changes. OCS records retain an OCS source
recension and can affect generation only through a reviewed mapping; they never
become exact Synodal rows.

## Corpus coverage and review queues

The report-producing commands use existing normalized intermediate candidates;
they do not add raw-source adapters to the runtime CLI:

```sh
cargo xtask synodal-coverage --fixture --offline
cargo xtask synodal-coverage --offline
cargo xtask synodal-lexical-review-queue
cargo xtask synodal-evaluation-queue
cargo xtask synodal-family-review-queue
cargo xtask synodal-marginal-recovery
cargo xtask synodal-v06-review-packets
cargo xtask synodal-v07-review-packets
cargo xtask synodal-v07-apply --check
cargo xtask synodal-v04-audit --check
cargo xtask synodal-v05-audit
cargo xtask synodal-v06-audit --check
cargo xtask synodal-v07-audit --check
```

Full coverage reads only the Ponomar and exact-revision Wikisource records whose
target recension is `synodal-russian`, retains source and passage identities,
and writes deterministic JSON/Markdown plus the ordered gap TSV under
`reports/`. The fixture is committed in `data/synodal/coverage_passages.tsv` and
has a stable hash test.

The lexical queue cross-matches target source-partition frequency with English
Wiktionary OCS semantics. The OCS candidate contributes only a proposed meaning;
it is never target surface evidence. Already admitted `(lemma, part of speech)`
pairs are excluded, ambiguous OCS paradigm owners are retained as blocked rows,
and output is candidate-only. The evaluation queue searches evaluation-partition
passages that are disjoint from training and lexical-review evidence. It blocks
surface matches shared by multiple generated cells and never promotes a match
without context review.

The family queue groups repeated unresolved surfaces conservatively for human
review and records contexts, possible cells, known and dictionary candidates,
missing metadata, assumptions, contradictions, and stable candidate IDs. Its
`--check` gate requires decisions for the top 200 current proposals. The
marginal-recovery report overlap-adjusts unresolved candidate batches and makes
the remaining token requirement explicit without granting coverage. The v0.4
audit verifies a locked historical snapshot and digest. The v0.5 audit
deterministically compares that baseline with the current registries, coverage,
evaluation, family decisions, marginal diagnostics, and remaining gaps.
The v0.6 audit remains an immutable 65% historical record. The v0.7 packet
generator distinguishes current exact-surface work from prior decisions, the
apply gate validates every admitted evidence role before regeneration, and the
v0.7 audit freezes the 70% result plus its verification and full-diff review.

See `SYNODAL_CLI_AND_COVERAGE.md` for normalized input formats, gap precedence,
and consumer-facing commands.

## Refresh review

Upstream drift is never accepted by fetch or verify. Refresh is deliberately
separate and must name one source:

```sh
cargo xtask synodal-sources refresh \
  --source SOURCE_ID \
  --accept-new-checksums
```

Refresh downloads and validates every selected artifact into staging first,
then writes a review report at `reports/synodal-source-refresh.json`. Review the
old/new URL, revision, size, and checksum for every row, inspect candidate and
evaluation changes, and commit lock changes only with the corresponding review.
Exact Wikisource revision IDs live in `references/WIKISOURCE_REVISIONS.tsv`.
The separately reviewed title-to-canonical-book mapping lives in
`references/WIKISOURCE_BOOKS.tsv`; both inventories must cover the same 78
pages.

A full bootstrap also writes `reports/synodal-verse-disagreement.json`. It
aligns canonical book/chapter/verse identities across Ponomar, exact-revision
Wikisource, and CrossWire, reports pairwise exact normalized-text agreement,
and includes hashes—not redistributed verse text—for deterministic disagreement
samples.

## Licensing and metadata-only sources

Restrictive, noncommercial, evaluation-only, mixed, and unresolved-license raw
corpora stay outside packages. GORAZD and the Russian National Corpus remain
metadata-only because no authorized bulk endpoint and applicable redistribution
terms have been established. Individual Ponomar Library editions are not fetched
until their work-level rights and transcription lineage are reviewed.

## Troubleshooting

- `wrong checksum`: upstream or local bytes differ; do not refresh reflexively.
  Inspect the artifact and use the explicit refresh procedure only after review.
- `HTML error page`: the server returned an error document under a data suffix;
  retry later without changing the lock.
- missing `mod2imp`, DjVu, or Tesseract: install the prerequisite named in the
  error and rerun the same offline cache.
- orphaned reviewed evidence: run a full, not source-filtered, bootstrap; if it
  persists, the upstream candidate changed and requires human re-review.
- parse ceiling exceeded: inspect `data/quarantine/synodal`, fix the adapter or
  review upstream drift, and do not increase the ceiling merely to hide errors.
