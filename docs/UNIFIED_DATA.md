# Unified extractor and data tree (merge phase 6)

Survey, decisions, and contract for the last phase of
`docs/UNIFIED_LANGUAGE_PROMPT.md` (execution plan step 6: "one extractor, one
`data/` layout with per-source, recension-tagged inputs; the cross-recension
evidence tables collapse into the identity layer"). Entered after phase 5
(`docs/UNIFIED_FACADE.md`, commit cd2047e) on top of the identity layer
(`docs/UNIFIED_IDENTITY.md`).

Scope rule for this phase: every move is behaviour-preserving by
construction. Every generated artifact (the OCS extracted registry and
dictionary, the Synodal generated registries, the gold oracles and gap, the
residue tables, the identity tables) regenerates byte-identically from the
merged crate; a move that would change even a banner line of a generated
artifact is deferred, not forced (§5 lists them with reasons).

## 1. The two pipelines as they were

### 1.1 OCS: Kaikki/Wiktextract (`old-church-slavonic-extractor`)

| Stage | Reads | Owns |
|---|---|---|
| `refresh` (`cargo xtask refresh-data --dump PATH`) | one local Kaikki JSONL (gitignored; `data/extracted/source.json` pins its size + SHA-256) | `data/extracted/{lexemes,aliases,forms,verb_metadata}.tsv`, `source.json`, `reports/extraction-coverage.{json,md}` |
| `refresh-derived-registry` | the committed normalized TSVs | `verb_metadata.tsv` only |
| `check` (`check-registry`) | the committed TSVs + `data/ocs/overrides.tsv` + `data/ocs/citation-exemptions.tsv` | nothing: re-derives the metadata and regenerates the runtime tables byte-for-byte |
| `dictionary-refresh` / `dictionary-check` | the same dump | `data/dictionary/{senses,source}.json`, `reports/dictionary-coverage.{json,md}` |

Stage shape: JSON table parsing (Kaikki `forms`/`tags`/`head_templates`
shapes), schema-drift accounting by exact drop reason, closed-vocabulary
validation (`validate.rs`), verb-metadata derivation
(`verb_metadata.rs`), emission of sorted TSVs and generated Rust
(`emit.rs`). Provenance is per row: content-derived lexeme ids that embed
the source signature, the raw source spelling and tags on every form, the
`provenance`/`source_feature`/`authority` triple on every metadata field.

### 1.2 Synodal: reviews/evidence/registry (`synodal-church-slavonic-extractor`)

| Stage | Reads | Owns |
|---|---|---|
| `candidates` (the adapter pipeline, `pipeline.rs` + `adapters.rs`) | `references/SOURCE_LOCK.tsv`-pinned downloads (Ponomar, Wikisource, CrossWire, Alypy HTML, D'yachenko OCR, Polivanova TEI, Kaikki, treebanks) | `data/intermediate/synodal/*.jsonl` candidate records and `data/quarantine/` (both gitignored), `reports/synodal-extraction.*` |
| evidence review (`evidence.rs`, `reviews.rs`) | the candidate records + the curated `data/synodal/*.tsv` review tables | validation only: every reviewed row must link to a candidate, every source id/recension must be in `APPROVED_SOURCE_RECENSIONS` |
| `generate_registry` / `generate_dictionary_registry` (`generate.rs`, `validate_registry.rs`, `validate_grammar.rs`, `emit.rs`) | `data/synodal/*.tsv` | `crates/synodal-church-slavonic/generated/registry.rs`, `crates/synodal-church-slavonic-dictionary/generated/registry.rs` |
| `wikisource-split` | one Wikisource export + `references/WIKISOURCE_REVISIONS.tsv` | exact revision-pinned wikitext artifacts |

Stage shape: many source formats (HTML, XLS, XML, SWORD, OCR text, TEI),
one candidate-record schema with `source_recension`/`target_recension`
tags per record, and a review layer in which a human decision row is the
unit of admission. Provenance is per evidence row: candidate ids
(`synodal:candidate:<sha256[..24]>`), passage citations, and the reviewed
decision.

## 2. What is genuinely shared — merged now

| Concern | Before | Now (`crates/church-slavonic-extractor`) |
|---|---|---|
| Source checksumming | three private streaming SHA-256 loops (`extract.rs`, `semantics.rs`, `pipeline.rs`) and one in-memory helper (`emit.rs`) | `shared::sha256_file`, `shared::hex_sha256` — the `references/SHA256SUMS` / `SOURCE_LOCK.tsv` / `source.json` / registry-fingerprint convention in one place |
| Atomic artifact installation | `output.rs` (OCS batch with rollback), `emit.rs::atomic_write` (Synodal single file) | `shared::atomic_write_batch` (the batch primitive is public; the Synodal single-file writer stays local because its temp-name convention is part of the observed pipeline behaviour) |
| TSV conventions | identical by convention: tab-separated, header row, no tab/newline in fields, sorted rows, `-` for absent | unchanged in both modules; the field-hygiene check (`ocs::normalize::checked_tsv`, `synodal::schema::read_table`) is deliberately not unified because the two validators reject different things (wiki markup vs forbidden authorities) |
| Provenance shapes | content-derived ids in both (`<lemma>\|<pos>\|<sha[..16]>` OCS, `synodal:<ns>:<sha[..24]>` Synodal); source id + passage on every Synodal evidence row; source spelling + tags on every OCS form | unchanged: the shapes are already the same idea (id embeds a content digest; every row names its source), and the identity layer is where they meet (§4) |
| Crate plumbing | two unpublished crates, two `main.rs`, two dependency lists | one unpublished crate (`publish = false`), modules `ocs` and `synodal`, one binary `church-slavonic-extractor <ocs\|synodal> ...` |

The old crate names are deleted outright rather than shimmed: both were
unpublished, and their only dependant was `xtask` (verified by grepping
every manifest, source file, workflow, and script). A re-export shim
would have been pure ceremony.

The `// @generated by old-church-slavonic-extractor` and
`// @generated by synodal-church-slavonic-extractor` banner lines inside
the generated artifacts are **kept verbatim**: they are part of the
committed bytes that the byte-identity gates compare, and the checked-in
`data/extracted` / Synodal registries name the generator that produced
them. They flip to the new crate name in the next regeneration that
changes content anyway (§5).

## 3. What is recension-specific by nature — kept separate

- **Input parsing.** Kaikki is one JSON schema with table sentinels and
  tag vocabularies; the Synodal adapters are a per-source zoo (HTML
  tables, XLS wordlists, SWORD modules, OCR output, TEI). Nothing in the
  parsing layer transfers.
- **Admission.** The OCS registry admits a cell when the source table is
  safe (closed feature vocabulary, complete tags); the Synodal registry
  admits a cell when a reviewed decision row links a candidate to a
  lexeme. The first is a parser contract, the second a review contract.
- **Validation vocabularies.** `ocs::validate` (feature grammar, script,
  NFC, override precedence) and `synodal::validate_registry` /
  `validate_grammar` (accent scopes, positional paradigms, abbreviation
  families, defective inventories) share no rules.
- **Accent.** OCS sources are unaccented; the Synodal pipeline carries
  accents as evidence on every form. This is the accent asymmetry of the
  prompt and it is why the pipelines cannot share a surface-comparison
  step below the projection layer.

## 4. The cross-recension evidence tables and the identity layer

Inventory of what asserted cross-recension identity outside
`data/unified/`:

| Table | Producer | Consumers | Disposition |
|---|---|---|---|
| `data/synodal/lexical_source_claims.tsv` (2,240 claims) | `cargo xtask synodal-lexical-union` from `lexemes.tsv`, `lexical_reviews.tsv`, `lexical_source_decisions.tsv`, and the preserved proposal queue `reports/synodal-lexical-review-queue.tsv` | its own `--check` (a CI step), `morphology_completeness::check_progress_artifacts` (inside `check-structure`) and `check_complete` (the v14 completion gate) | **kept**; its 1,000 cross-recension rows are now *also* expressed as identity candidates (below), so the identity layer is the single review queue and the ledger is the audit of the Synodal union |
| `reports/synodal-lexical-review-queue.tsv` (1,000 rows) | frozen (the wave-era queue generator is retired) | the ledger; now the identity generator | **kept** as the committed input of both |
| `data/synodal/target_identity_ambiguities.tsv` (1 row) | curated | the Synodal extractor's evidence validation (`validate_target_identity_ambiguities`) | **kept**: it is a *within-Synodal* homograph exception (adjective vs participle), not a cross-recension claim |
| `data/synodal/alignments.tsv`, `semantic_alignments.tsv`, `transformation_rules.tsv`, `conflicts.tsv` | curated | the Synodal registry (`RecensionMapping`, dropped per the deprecation map) | **unchanged** in this phase: their runtime consumer is the `synodal-church-slavonic` crate that phase 5 still serves; they retire with it |
| `data/ocs/lexical_source_claims.tsv` (14,114 claims) | `cargo xtask ocs-lexical-union` | its own `--check`, `check-structure` | **kept**: an OCS-internal source-union ledger (Kaikki vs Polivanova), nothing cross-recension in it |

### 4.1 Claims as identity candidates

`cargo xtask unified-identity` now ingests the preserved proposal queue —
the exact rows the ledger turns into its `disputed`
(`cross-recension-identity-unconfirmed`, 957) and `ambiguous`
(`cross-source-homograph-ambiguity`, 43) claims — into
`data/unified/identity-candidates.tsv`, one row per queue claim:

| column | value for a lexical-union row |
|---|---|
| `ocs_lexeme_id` | every OCS extracted lexeme whose projected candidate keys (the same projection as the identity pairing) contain the claim lemma's projection-normal key, `;`-joined and id-sorted; `-` when none (835 of 1,000: mostly Ponomar mixed-recension headwords absent from Kaikki) |
| `pos` | the queue's part of speech (`unknown` for most) |
| `ocs_citation` | the claim lemma as the queue records it |
| `kind` | `lexical-union-proposal` (candidate-unreviewed) or `lexical-union-homograph` (blocked-ambiguous-homograph) |
| `candidates` | `<printed surface>@<passage>` — the Synodal attestation |
| `provenance` | `synodal-lexical-union:<claim_id>`, i.e. the ledger's `queue:<semantic candidate>:<attestation candidate>` key, so a candidate row and its ledger claim are joinable without a parallel table |

The candidates table gained the `provenance` column for every row
(projection-study rows carry `projection-study`). The table stays a
review queue, never an identity claim; `identity.tsv` and
`coherence-baseline.tsv` are byte-identical to the phase-5 state
(599 entries; OCS 10152/24135, Synodal 1255/1892). Delta: 631 → 1,631
candidate rows.

Why the ledger is not retired in this slice: its `check_complete`
completion gate counts every claim and requires a final disposition, and
`synodal-lexical-union --check` is a CI step and a `check-structure`
progress check. Re-pointing those to the candidates table would change
what the v14 completion gate measures (it would stop counting the seed and
review claims, which are not cross-recension). The dependency is
therefore: the ledger derives its cross-recension rows from the same
committed queue the identity generator reads; the two cannot drift
because neither is hand-edited. A later slice may make the ledger read
the cross-recension rows *from* the candidates table once the completion
gate has crossed.

Known inaccuracy carried forward unchanged: the ledger stamps every queue
claim with the source pair
`english-wiktionary-ocs-kaikki-2026-08-07+ponomar-elizabeth-bible-2026-08-09`
although some semantic candidates come from the Ponomar 2016 dictionary
(`source_recension: mixed`). Fixing it changes the ledger bytes and is a
reviewed ledger diff, not a plumbing move.

## 5. Data tree: target layout, moves made, moves deferred

Target layout (recension tag per input, one identity layer):

```
data/
  SOURCES.toml, evaluation-sources.json      shared source identity/pins (with references/)
  ocs/                                       recension: Old Church Slavonic
    extracted/                               (deferred; today data/extracted)
    dictionary/                              (deferred; today data/dictionary)
    overrides.tsv, citation-exemptions.tsv   curated OCS inputs (moved in this phase)
    polivanova_regular_{nouns,verbs}.tsv     curated OCS source tables
    lexical_source_claims.tsv                OCS source-union ledger
  synodal/                                   recension: Synodal Russian
    *.tsv                                    curated reviews, evidence, oracles, ledger
  unified/                                   identity layer (identity, candidates, coherence)
  morphology/, normalization/                cross-recension completion + normalization manifests
  intermediate/, quarantine/                 gitignored pipeline work (Synodal candidates)
references/                                  SOURCE_LOCK.tsv + SHA256SUMS pins for every raw download
```

Moved now (pure renames, every reference updated, no generated artifact
embeds the path): `data/overrides.tsv` → `data/ocs/overrides.tsv`,
`data/citation-exemptions.tsv` → `data/ocs/citation-exemptions.tsv`.

Deferred, with reasons:

- `data/extracted` → `data/ocs/extracted` and `data/dictionary` →
  `data/ocs/dictionary`: the path is embedded in the banner of every
  generated residue table (`crates/church-slavonic/generated/*.rs`) and the
  dictionary entries table, in the published crates' `README.md`/
  `ATTRIBUTION.md`, and in `data/extracted/source.json`'s sibling reports.
  Moving them changes generated bytes and published packaging text;
  schedule with the next content-changing OCS regeneration (a Kaikki
  refresh), which flips the generator banner at the same time.
- Per-source splitting of `data/synodal` (`data/<source-id>/...`): the
  curated Synodal TSVs are keyed by evidence row, each row already naming
  its source id and passage; a per-source directory split would churn
  every curated table's location for no change in content. Recension
  tagging is already carried per record (`source_recension`,
  `target_recension`, `APPROVED_SOURCE_RECENSIONS`). Deferred until the
  `synodal-church-slavonic*` crates are deleted (facade merge plan,
  `docs/UNIFIED_FACADE.md` §5), when the tables' consumer set changes
  anyway.
- The generator banners in generated artifacts (§2): flip with the next
  content change.

## 6. Crate layout after this phase

```
crates/church-slavonic-extractor        publish = false; bin church-slavonic-extractor
  src/lib.rs                            modules ocs, synodal, shared
  src/shared.rs                         sha256_file, hex_sha256, atomic_write_batch
  src/ocs/{mod,extract,emit,normalize,report,schema,semantics,validate,verb_metadata}.rs
  src/synodal/{mod,adapters,pipeline,evidence,reviews,generate,validate_registry,validate_grammar,emit,schema,tests}.rs
  tests/ocs_fixture_refresh.rs + tests/fixtures/schema.jsonl
```

`xtask` is the only dependant (`church_slavonic_extractor::ocs::extract`,
`::ocs::semantics`, `::ocs::schema`, `::synodal::pipeline`,
`::synodal::{generate_registry, generate_dictionary_registry,
validate_candidate_links, APPROVED_SOURCE_RECENSIONS,
source_recension_is_approved}`). `data/morphology/completion.toml`'s
implementation/test references were re-pointed to the new paths (the
completeness checker validates that each referenced file and test exists).

## 7. Gates

Unchanged and fresh at the end of the phase: `cargo test --workspace`,
`cargo xtask check-structure` (which includes `check-registry`,
`check-dictionary`, `synodal-check`, the completeness progress checks,
`rewrite-pilot-accuracy`, `synodal-gold --check` at the committed 53,879
gap rows, `rewrite-dictionary --check`, `unified-identity --check`),
`synodal-lexical-union --check`, `synodal-guard-witnesses`,
`synodal-fixture-bootstrap`, `synodal-archive --check`, clippy with
`-D warnings`, `cargo fmt --check`, and `git diff --exit-code` after
regenerating every generated artifact from the merged crate.
