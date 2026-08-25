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

### One-command wave close

`cargo xtask synodal-wave-close` runs the entire closing suite in the
canonical order with one pass/fail table and per-step timings. `--check` is
read-only and CI-safe (the CI structural job invokes exactly this, so local
and CI ordering cannot diverge); steps that recompute from the gitignored
intermediate corpus (`synodal-accent-fit --check`,
`synodal-family-review-queue --check`) self-skip where it is absent. The
default local mode adds `cargo fmt --check`, clippy, and the workspace test
suites. `--fix` first regenerates every derived artifact in the canonical
order — the family queue, the accent-fit report, the prediction feed,
marginal recovery, and the lexical source union **last** (it reads lexemes,
lexical reviews, and family reviews) — and prints undecided top-200 family
proposals as ready-to-review stubs when that gate is the failure.

### Accent suggestions

`cargo xtask synodal-accent-fit --suggest <lexeme-id> <cell>` prints the
exact `accent_paradigms.tsv` row that would realize one cell's print — with
the existing fitted block's paradigm ID and block-uniform evidence and the
line to insert after, when the lexeme already carries a block — or the
precise refusal: the cell does not expand; it is not in the accent gap; it
has no source-partition witness at all; its witnesses conflict (each variant
listed); or **every witness is itself a held-out type**, which is reported as
unfittable without memorisation and never as a row. The scope grammar itself
is documented in `SYNODAL_MORPHOLOGY.md` (“Accent paradigm scope grammar”).

### Delta coverage projection

`cargo xtask synodal-coverage --offline --delta` projects the ledger-relevant
totals (corpus top-k and top-1, cross-lexeme ambiguity, and the held-out
generalised/memorised/top-k columns) from a cached distinct-surface inventory
(`data/intermediate/synodal/coverage-surface-inventory.tsv`, written by every
canonical run) instead of re-reading the 1.3M-token corpus. It classifies
each surface with the same code path as the canonical run, so on an unchanged
corpus the numbers match the full run exactly (~18s instead of ~4min). The
output is stamped `PROJECTION — not sealable`, compares against the last
sealed ledger row, and combining `--delta` with `--seal-wave`,
`--reseal-floors`, `--check`, `--require-complete`, or `--fixture` is an
error, witness-tested in `synodal-guard-witnesses`. Sealing always uses the
canonical full run.

### Registry staleness tripwire

The generated registries compile into the binaries, so a measurement run
after `synodal-regenerate` but before a rebuild silently reflects the old
data. Each runtime crate's build script embeds a fingerprint of its
`generated/registry.rs` (`REGISTRY_FINGERPRINT`), and `synodal-evaluate`,
`synodal-coverage`, and `synodal-accent-fit` refuse to run when the on-disk
file no longer matches the compiled fingerprint, naming the exact rebuild
command. `synodal-regenerate` itself skips the in-process evaluation and
prints a loud REBUILD REQUIRED banner whenever its write changed the
registries. The refusal is witness-tested in `synodal-guard-witnesses`.

### Admission preflight

`cargo xtask synodal-admit-check` validates the committed TSVs in seconds and
reports **every** violation at once, in the four categories that caused
seal-time rework during v0.12: duplicated lexeme identities (a committed
surface owned by one identity that the registry also analyzes to another, or
two lexemes sharing a normalized lemma and part of speech), new held-out
memorisation (an exact or accent row whose normalized type is held out and not
frozen in `data/synodal/holdout_memorisation_baseline.tsv`), evaluation rows
sharing a passage with runtime-referenced evidence (matching the authoritative
extractor predicate), and generation-dead lexemes (a productive lexeme none of
whose own surfaces analyzes back to it). Reviewed genuine homonymy lives in
`data/synodal/homonymy_allowlist.tsv` with per-pair justifications; the
memorisation baseline is ratcheted down with
`cargo xtask synodal-admit-check --write-baseline` after reviewing the diff.
The preflight runs inside `synodal-check`, so CI enforces it; it duplicates
guards earlier and never replaces the sealed floors, ceilings, or late checks.

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

When a current packet decision changes to `admitted`, run
`cargo xtask synodal-v07-apply --refresh-ownership` before ordinary
materialization. The committed ownership ledger retains inactive historical
packet facts so later rejection, deferral, or re-admission is reproducible.

Full coverage reads only the Ponomar and exact-revision Wikisource records whose
target recension is `synodal-russian`, retains source, passage, and partition
identities, and writes deterministic JSON/Markdown plus two TSVs under
`reports/`. The review queue is intentionally bounded for human triage. The
separate `*-frontier.tsv` is the complete, untruncated inventory of every
strict-top-k-uncovered surface/status combination with true document frequency,
source/partition provenance, and bounded contexts. `--check` validates all four
artifacts. The fixture is committed in `data/synodal/coverage_passages.tsv` and
has a stable hash test.

The canonical report leads with the type-disjoint holdout, split by resolver
status and by morphological system, because held-out tokens reached by rule
(`generalised`) are the only measure that distinguishes a better engine from a
longer lookup table. `reports/synodal-waves.tsv` is the append-only ledger of
that measure per sealed wave, next to the corpus figure and the lexicon size
that produced it. `cargo xtask synodal-coverage --offline --seal-wave <label>
--note <text>` appends a row; `--check` and `cargo xtask
synodal-coverage-floors` fail when the last row no longer describes the live
report, and a row may never lower `holdout_generalised` or raise
`holdout_memorised`. `--reseal-floors` ratchets `data/synodal/coverage_floors.tsv`
toward the current report and is how `holdout:generalised_analyzed` is raised
after a wave that moved it.

The historical acceptance command
`cargo xtask synodal-coverage --offline --check --require-complete` still exists
but is no longer a program gate. Its locked input hashes and denominator
prevent a fixture, custom input, truncated source set, alternate policy, or
aggregate-only success from satisfying a 100% claim.

The lexical queue cross-matches target source-partition frequency with English
Wiktionary OCS semantics and exact-headword semantics from the locked SCI
Ponomar dictionary. An OCS candidate contributes only a proposed meaning; a
Ponomar dictionary candidate remains `unknown`/`untyped` until a reviewer
establishes its part of speech and target cell. Neither is target surface
evidence, and neither is admitted automatically. Already admitted `(lemma,
part of speech)` pairs are excluded, ambiguous paradigm or dictionary owners are
retained as blocked rows, and output is candidate-only. The evaluation queue
searches evaluation-partition passages that are disjoint from training and
lexical-review evidence. It blocks surface matches shared by multiple generated
cells and never promotes a match without context review.

The locked SCI Ponomar source adapter preserves both the frequency list and the
21,104-row dictionary workbook as distinct structured candidate records. The
dictionary contributes mixed-recension headword and semantic evidence for
review; it does not supply a target-recension grammatical cell, and no workbook
row is admitted automatically. The corpus archive remains unexpanded until its
component lineage and evidence role are reviewed.

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
