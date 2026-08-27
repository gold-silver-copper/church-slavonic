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
  crates/synodal-church-slavonic/generated/registry.dat \
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
5. `crates/*/generated/` contains deterministic runtime-safe facts — the
   morphology registry as the data artifact
   `synodal-church-slavonic/generated/registry.dat` (`@TABLE <columns>`
   headers, tab-separated rows, embedded with `include_str!` and parsed once
   on first use), the dictionary registry as `generated/registry.rs`; runtime
   crates have no filesystem, network, archive, XML, JSON, TSV, or database
   reader.
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

## Coverage measurement: the gold gate

The wave-era measurement machinery (coverage fixtures and floors, review
queues, wave ledger, admission preflight, accent-fit ratchet, marginal
recovery) was retired on 2026-08-26 when the synodal-gold full-enumeration
gate replaced it; the wave-era workflow documentation is preserved in
`docs/history/` and the immutable archive. Coverage is now measured by one
command with no sampling:

```sh
cargo xtask synodal-gold --check   # full-enumeration replay, subset gate
cargo xtask synodal-gold --fix     # commit a strictly smaller gap
```

The gate replays every row of both committed gold oracles
(`data/synodal/gold_token_oracle.tsv`, `data/synodal/gold_paradigm_oracle.tsv`)
and regenerates `reports/synodal-gold-gap.tsv`, the finite worklist that may
only shrink. The contract is normative in `docs/SYNODAL_GOLD_ORACLE.md`.

### Registry staleness tripwire

The generated registries compile into the binaries, so a measurement run
after `synodal-regenerate` but before a rebuild silently reflects the old
data. Each runtime crate's build script embeds a fingerprint of its
generated registry (`registry.dat` for the morphology crate, `registry.rs`
for the dictionary; `REGISTRY_FINGERPRINT`), and the in-process evaluation
refuses to run when the on-disk file no longer matches the fingerprint of the
registry actually in use (`active_registry_fingerprint()`), naming the exact
rebuild command. `synodal-regenerate` itself skips the in-process evaluation
and prints a loud REBUILD REQUIRED banner whenever its write changed the
registries. The refusal is witness-tested in `synodal-guard-witnesses`.

The morphology crate additionally carries the `registry-override` cargo
feature, enabled only by `xtask`: `install_registry_override` swaps the parsed
artifact in-process, so the gold inner loop below reads freshly regenerated
data without recompiling anything. Every `synodal-gold` command first installs
the on-disk artifact when the binary embeds an older one (printing a notice),
which keeps the gate honest between regenerations and rebuilds. The published
crate exposes no such channel.

### The gold-gap inner loop

`docs/GOLD_GAP_BURNDOWN_PROMPT.md` organises the burn-down of the committed
gap around a fast inner loop. `cargo gold` is the release-profile alias for
`cargo xtask synodal-gold` (the full replay runs in about 2 s instead of
20–30 s in debug):

- `cargo gold --check --only <class> [--only …] [--lemma <lemma-or-id>]
  [--types-from <file>]` replays only the selected oracle rows and prints the
  per-class delta against the committed gap; `--fix` always replays the full
  oracles.
- `cargo gold propose [--only <class>] [--min-cells N] [--top N]` writes
  `reports/synodal-gold-hypotheses.tsv`: (lemma, class) hypotheses derived by
  segmenting gap surfaces against every productive class's probed ending
  inventory, verified through the registry path (placeholder lexemes installed
  via the override), with an accent paradigm fitted from the printed cells,
  clustered by the attested cells each clears and ranked by cells per
  admission. Statuses other than `fit` explain why a hypothesis is not
  admissible (partial table, accent unfit with the closest rule named,
  collateral surfaces it would misanalyse, a competing lemma or class).
- `cargo gold admit [<hypotheses.tsv>] [--take N] [--oracle token|paradigm]`
  appends lexeme, accent-paradigm, and reviewed-evidence rows for the fit
  clusters (provenance: the attesting passage or Alypy section, linked to its
  candidate record), regenerates the artifact, installs it in-process, replays
  everything each admission reaches, keeps the admissions that clear their
  cluster without moving any other row's class, and reverts the rest into
  `reports/synodal-gold-rejected-hypotheses.tsv`.
- `cargo gold loop [--only <class>] [--take N] [--min-cells N]` chains
  propose → admit → scoped replay and prints one line: rows cleared per class,
  the rules-vs-residue split of what landed, elapsed seconds.

Admissions are always a lexeme with a class plus a fitted accent paradigm;
the tooling never writes exact forms. The full gate, `synodal-check`, and
`check-structure` remain the outer loop.


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
