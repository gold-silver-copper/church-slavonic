# Build a Reproducible Synodal Church Slavonic Data and Coverage Pipeline

Implement the next major version of the Synodal Russian Church Slavonic tooling
in the `church-slavonic` Rust workspace.

The result must provide a checksum-locked, end-to-end pipeline that can
reconstruct the reviewed generated registries from an empty source cache, while
substantially expanding corpus-backed vocabulary, morphology, accentuation, and
orthographic coverage.

Backwards compatibility is not required. Preserve unrelated work. Do not commit,
push, publish, tag, or deploy unless separately requested.

## Initial audit

Read completely before modifying anything:

- `references/fetch-sources.sh`
- `references/SOURCES.toml`
- `references/SHA256SUMS`
- `data/normalization/`
- `data/synodal/`
- `crates/synodal-church-slavonic-*`
- `crates/xtask/src/synodal.rs`
- `docs/SYNODAL_*.md`
- `reports/synodal-*.{json,md}`

Document the current data flow, every mutable download URL, missing adapter,
external dependency, licensing restriction, and manual step.

Do not claim reproducibility merely because the current cache happens to exist.

## Non-negotiable boundaries

- The target recension is exclusively Synodal Russian Church Slavonic.
- Modern constructed Slavic standards and Slovowiki are forbidden as linguistic
  authorities.
- OCS may supply explicitly mapped inherited evidence, never unmarked Synodal
  attestation.
- Runtime crates must perform no filesystem, network, archive, XML, JSON, TSV, or
  database access.
- Raw corpora with restrictive or unresolved licenses must remain local and must
  not enter crates.io packages.
- Generated forms must never be labeled attested.
- Human-reviewed decisions must remain separate from automatically extracted
  candidates.
- Do not download GORAZD, RNC, or unresolved Ponomar editions unless an authorized
  bulk endpoint and applicable terms have been established.

## 1. Implement a checksum-locked source manager

Replace the fragile fetch workflow with a manifest-driven `xtask` source manager.
Retain a small shell wrapper if useful, but the authoritative behavior must be
tested Rust code.

Provide commands equivalent to:

```text
cargo xtask synodal-sources list
cargo xtask synodal-sources fetch
cargo xtask synodal-sources fetch --source SOURCE_ID
cargo xtask synodal-sources verify
cargo xtask synodal-sources verify --offline
cargo xtask synodal-sources status
cargo xtask synodal-sources refresh --source SOURCE_ID
```

Requirements:

- Drive artifact URLs, destination paths, revisions, sizes, formats, and expected
  SHA-256 values from a machine-readable lock file.
- Record every individual artifact, not only an aggregate directory checksum.
- Treat committed hashes as immutable during normal fetching.
- Never overwrite lock checksums during `fetch` or `verify`.
- Reject an existing nonempty file when its checksum is wrong.
- Download to a partial file and atomically rename only after validation.
- Support safe resumption when the server supports byte ranges.
- Validate HTTP status, content type where dependable, archive structure,
  expected file signature, and nontrivial size.
- Detect HTML error pages saved as XML, PDF, ZIP, DjVu, or archive data.
- Provide clear diagnostics for unavailable or changed upstream artifacts.
- Make source refresh an explicit operation that produces a reviewable lock-file
  diff.
- Preserve the previous revision and checksum in refresh reports.
- Never silently accept upstream drift.
- Support a configurable cache directory so tests never modify the real cache.
- Give actionable prerequisite errors.
- Avoid embedding credentials or tokens.

For mutable sources:

- Fetch the dated English Wiktionary dump as the primary reproducible input.
- Where practical, derive the OCS extraction from the dated dump rather than
  relying on the moving Kaikki “latest” URL.
- Pin Wikisource page revision IDs and fetch those exact revisions.
- Store the complete expected Alypy page inventory rather than trusting a newly
  changed table of contents.
- Lock CCMH, Ponomar, CrossWire, Unicode index, and similar mutable endpoints by
  individual hashes.
- If an upstream does not expose historical bytes, fail explicitly on drift and
  require reviewed refresh.

Keep metadata-only sources visible in status output with their exact reason for
not being downloaded.

## 2. Implement the end-to-end bootstrap

Add:

```text
cargo xtask synodal-bootstrap
```

It must orchestrate:

```text
fetch
→ artifact verification
→ extraction
→ source-specific normalization
→ quarantine reporting
→ application of committed reviewed overlays
→ generated registry creation
→ evaluation
→ freshness and package-boundary checks
```

Support:

```text
--cache PATH
--offline
--source SOURCE_ID
--skip-fetch
--keep-intermediate
```

The pipeline must distinguish:

1. raw immutable downloads;
2. automatically normalized candidates;
3. quarantine and rejection records;
4. committed human-reviewed overlays;
5. runtime-safe generated facts; and
6. evaluation-only data.

A clean rebuild must not depend on timestamps, directory enumeration order,
locale, absolute paths, network response order, or hash-map iteration.

Use temporary directories and atomic replacement. Never test clean reconstruction
by deleting the user's real cache.

## 3. Complete source adapters

Implement command-line-accessible, fixture-tested adapters for the following
sources.

### Ponomar Elizabeth Bible

- Parse every `.text` file from the pinned archive.
- Preserve book, chapter, verse, token order, raw spelling, and markup.
- Remove editorial markup without turning it into tokens.
- Preserve accents, breathings, abbreviations, and positional letters.
- Produce corpus-frequency tables and passage-partition metadata.
- Keep source and evaluation partitions disjoint.

### Alypy/Gamanovich grammar

- Parse the complete locked HTML page set.
- Extract rule sections, paradigm witnesses, and examples with stable section
  identifiers.
- Treat extraction as candidate evidence requiring review.
- Preserve the exact section and source spelling for every promoted rule or form.

### D’yachenko dictionary

- Extract the embedded DjVu OCR or use a clearly checked external tool.
- Preserve page numbers, coordinates where available, raw OCR, and correction
  status.
- Emit lexical and semantic candidates, never automatic exact Synodal forms.
- Reject promotion without Synodal corpus or reviewed grammatical evidence.

### Wikisource Church Slavonic Bible

- Fetch exact page revisions.
- Preserve template and transclusion lineage.
- Parse MediaWiki XML/wikitext deterministically.
- Produce book/chapter/verse-aligned text.
- Keep community transcription evidence distinct from normative evidence.

### CrossWire CSlElizabeth

- Extract through documented SWORD tooling or a fully tested module parser.
- Preserve module version and verse identity.
- Label its modernized spelling appropriately.
- Use it for alignment, vocabulary coverage, and orthographic contrast—not
  unquestioned printed-form authority.

### Polivanova

- Parse the OSD spreadsheet and scholarly TEI XML.
- Preserve their common lineage so they are not counted as independent witnesses.
- Emit OCS classes, roots, principal-part candidates, and rule evidence.

### UD and Syntacticus

- Parse CoNLL-U and native XML respectively.
- Preserve their shared PROIEL/TOROT lineage.
- Use them for inherited OCS evidence and evaluation without copying surface rows
  into Synodal exact-form tables.

### CCMH and DIACU

- Parse supported text/XML formats.
- Use CCMH for historical witness comparison.
- Use DIACU for recension-contamination and period-classification tests.
- Quarantine records whose recension or license cannot be established.

Every adapter must stream large inputs, preserve source order, count every
rejection reason, enforce a configurable failure ceiling, and emit normalized
plus quarantine reports.

## 4. Separate automatic candidates from reviewed data

Refactor the data layout so regeneration never overwrites human decisions.

Use clear layers such as:

```text
data/intermediate/synodal/       local generated candidates
data/quarantine/synodal/         local rejected records
data/synodal/reviewed/           committed review decisions
data/synodal/generated/          deterministic runtime inputs
```

Names may differ, but the boundaries must be enforced.

Every normalized candidate must include:

- stable source record ID;
- source ID, revision, and artifact hash;
- source recension;
- target recension, if applicable;
- work, edition, and passage;
- raw and normalized spelling;
- part of speech and typed grammatical cell;
- authority and epistemic roles;
- upstream lineage;
- license and redistribution class;
- confidence;
- parse/review status; and
- transformation history.

Reviewed overlays must refer to stable candidate IDs and fail when their source
candidate disappears or changes unexpectedly.

## 5. Expand real Synodal coverage

Use corpus frequencies to prioritize frequently occurring liturgical vocabulary
rather than adding arbitrary demonstration lemmas.

Implement and test:

- Complete supported paradigms for registered nouns and adjectives.
- Additional noun classes and documented alternants.
- Comparative and superlative adjectives.
- Broader personal, demonstrative, relative, and indefinite pronouns.
- Broader determiners, cardinal numerals, and ordinal numerals.
- Additional irregular and suppletive verbs.
- Independently specified present, imperfect, aorist, and imperative principal
  parts.
- Productive active and passive participles.
- Declined participles.
- Supines and verbal nouns where normative evidence supports them.
- Lexical and productive accent classes.
- Positional printed orthography.
- Semantically controlled nomina sacra and abbreviations.
- Traditional numeral parsing and formatting regression coverage.

Do not infer aorist formation from aspect or present class. Do not generate
participles from an undifferentiated verb stem.

Every productive rule must cite a reviewed Synodal grammatical authority. OCS
metadata may help select an inherited class only through an explicit recension
mapping.

Maintain the behavior of:

```rust
GenerationPolicy::Strict
GenerationPolicy::Productive
GenerationPolicy::Exploratory
```

Strict must remain conservative; increasing coverage must not weaken its
evidential requirements.

## 6. Expand real-world evaluation

Replace the tiny seed evaluation with passage-disjoint and lemma-disjoint test
sets drawn from pinned sources.

Add:

- Exact registry round trips.
- Masked-cell completion.
- Lemma-disjoint class inference.
- Held-out Ponomar Bible passages.
- Held-out non-biblical liturgical passages when legally available.
- Expanded spelling agreement.
- Printed orthography agreement.
- Accent agreement.
- Abbreviation expansion/contraction tests.
- Numeral tests.
- Irregular verb and participle tests.
- OCS-to-Synodal inheritance tests.
- False-friend and semantic-drift controls.
- Other-recension contamination tests.
- Hostile Unicode and malformed-mark tests.
- Verse-alignment disagreement reports across Ponomar, Wikisource, and CrossWire.

Prevent leakage: if a form, principal part, rule, or mapping is learned from a
passage or lexeme, that same evidence cannot score the prediction.

Report separately:

- returned-form coverage;
- exact expanded agreement;
- exact printed agreement;
- accent agreement;
- top-1 and top-k accuracy;
- abstention;
- attested versus predicted results;
- regular versus irregular systems;
- Strict, Productive, and Exploratory policies;
- direct Synodal versus inherited OCS paths;
- identity versus transformed mappings;
- confidence calibration; and
- results by grammatical system and source lineage.

Exact Synodal evidence must always defeat a conflicting inherited or analogical
prediction.

## 7. CI and reproducibility tests

Default CI must remain reasonably sized and must not download the entire 4.6 GB
archive.

Add:

- Unit tests using miniature HTTP fixtures.
- Corrupted-file and wrong-checksum tests.
- Interrupted-download and resume tests.
- Mutable-upstream drift tests.
- Archive-signature and HTML-error-page tests.
- Fixture-based adapter golden tests.
- Determinism tests across repeated temporary directories.
- A clean fixture bootstrap that ends with no generated diff.
- A manually triggered full-source workflow, if repository storage and source
  terms permit it.
- A source-availability audit that reports upstream drift without rewriting
  locks.

The principal acceptance test must reconstruct the fixture cache in a temporary
directory and prove byte-identical generated outputs.

## Required structural guards

Add guards proving:

- fetch does not mutate the source lock;
- refresh is the only operation allowed to update hashes;
- corrupted cached files are rejected;
- mutable-source drift cannot silently enter normalized data;
- every runtime fact links to reviewed evidence;
- evaluation-only or nonredistributable corpus text cannot enter packages;
- all OCS inheritance uses explicit mappings;
- other recensions cannot enter exact Synodal registries;
- modern constructed Slavic standards and Slovowiki remain forbidden;
- human-reviewed overlays cannot be silently orphaned;
- generated artifacts and reports are current; and
- runtime crates remain free of filesystem and network access.

## Documentation

Update documentation with:

- prerequisites;
- expected disk space and download size;
- complete source commands;
- offline workflow;
- source-refresh review procedure;
- cache recovery;
- licensing restrictions;
- raw/intermediate/reviewed/generated boundaries;
- adapter coverage;
- known metadata-only sources;
- exact clean-reconstruction procedure; and
- troubleshooting for upstream drift.

Correct any existing claim that overstates reproducibility or adapter
completeness.

## Verification

Run at minimum:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
cargo test --workspace --doc
cargo xtask check-all
cargo xtask guard-witnesses
cargo xtask synodal-guard-witnesses
cargo xtask synodal-sources verify --offline
cargo xtask synodal-bootstrap --offline --cache TEST_CACHE
git diff --check
```

Also verify:

- no-default-feature native builds;
- WebAssembly builds for runtime crates;
- package contents for all three runtime crates;
- `cargo publish --dry-run` in dependency order;
- a temporary clean-cache fixture bootstrap; and
- byte-identical output across two independent bootstrap runs.

Do not publish the offline extractor unless explicitly requested and made
package-ready.

## Final report

Report:

- every changed file and command;
- source-lock design;
- mutable sources and how each was pinned;
- prerequisites and disk requirements;
- adapter completion by source;
- accepted, quarantined, and rejected record counts;
- reviewed vocabulary and paradigm growth;
- exact and predicted coverage;
- real-world evaluation metrics;
- inheritance-specific coverage and precision;
- remaining unsupported morphology;
- remaining metadata-only sources;
- verification results;
- package contents and licensing boundaries; and
- any step that still prevents a truly clean, byte-identical reconstruction.

The task is complete only when a disposable empty cache can be bootstrapped into
byte-identical generated registries without manual data editing, while restrictive
raw corpora remain outside distributable runtime packages.
