# Rewrite plan: one kernel, small API, rules first

Status: proposed (2026-08-25). Executable against this repository as it stands
after v0.14 phase 1. Each phase is independently landable and leaves the
workspace green (`cargo test --workspace` passes at every merge point).

## Goal

Reshape this workspace into the silhouette of `gold-silver-copper/english`
(tiny rule core + PHF irregular tables, ~1 MB bundled data) and
`gold-silver-copper/ruthenian` (pure rules, five enums, "an enum that selects
a paradigm becomes a function; an enum that indexes within one stays a
parameter"), while keeping the three ideas this project has that they don't:

1. Typed defects: `HistoricallyAbsent` vs `EvidenceIncomplete` ("cell cannot
   exist" vs "not yet reviewed") as distinct error values, never empty strings.
2. Provenance: `RuleTrace` / evidence ids on every produced form.
3. `analyze_text` with attested-before-predicted reading order as the
   headline consumer entry point.

## Target crate layout

```
crates/
  church-slavonic-core/         # pure rules + shared grammar types, zero data
                                # files, only dep: unicode-normalization
  church-slavonic-orthography/  # recension-aware reading/writing/accents;
                                # Recension is a parameter here, not a crate family
  church-slavonic/              # facade: irregular PHF tables + core;
                                # bundled data budget: <= 2 MB
  church-slavonic-dictionary/   # optional; attested corpus + analyze_text;
                                # loads a compact binary artifact, not .rs source
  extractor/                    # NOT published; may move to a sibling repo
  xtask/                        # accuracy gate + codegen only (see Phase 5)
```

Hard rules for the target:

- No generated `.rs` file over ~5k lines; the 24 MB
  `old-church-slavonic/generated/registry.rs` (147,765 lines) becomes PHF
  tables for true irregulars plus an optional loaded binary artifact.
- No hand-written module over ~1,500 lines (today:
  `synodal-church-slavonic-core/src/morphology.rs` 8,559;
  `old-church-slavonic/src/resolver.rs` 5,727;
  `synodal-church-slavonic-extractor/src/lib.rs` 5,167).
- One override precedence only: irregular table → rule kernel. The five
  current channels (provider exact forms, caller overrides, registry
  irregular overrides, `exact_forms.tsv`, homonymy allowlists) collapse into
  the irregular table plus lemma markers.
- Lexeme records are named-field structs or a schema'd format — never
  positional string arrays with empty-string holes.
- Unpredictable lexical facts ride on the citation form (ruthenian's lemma
  markers) wherever possible, so unseen words inflect as well as seen ones.

## Phase 0 — Freeze and measure (half a day)

- Tag the current state (`v0.14-pre-rewrite`).
- Record the accuracy baseline: run the existing coverage fixture and
  evaluation queue; commit the numbers to `reports/rewrite-baseline.md`.
  Every later phase must meet or beat these numbers.
- Snapshot the public API of all published crates
  (`cargo public-api` or `cargo doc` listing) into `reports/` for the Phase 4
  deprecation map.

## Phase 1 — Evict the research harness from the build (1–2 days, pure deletion)

- Delete the frozen milestone scripts from `crates/xtask/src`:
  `synodal_v04_audit.rs`, `synodal_v05_audit.rs`, `synodal_v05_baseline.rs`,
  `synodal_v06_audit.rs`, `synodal_v06_baseline.rs`,
  `synodal_v06_review_packets.rs`, `synodal_v07_apply.rs`,
  `synodal_v07_audit.rs`, `synodal_v07_review_packets.rs`,
  `synodal_v08_engine_audit.rs` (~12k LOC). Their outputs are already in git
  history; anything still needed goes to `tools/archive/` outside the
  workspace members list.
- Keep exactly one accuracy command, modeled on english's
  `cargo xtask accuracy`: it replays `data/extracted/forms.tsv` and the
  evaluation queue against the library and prints per-POS accuracy. Wave/
  holdout tooling (`synodal_waves.rs`, `synodal_wave_close.rs`,
  `synodal_type_holdout.rs`) moves to `tools/research/` — runnable, but not a
  workspace member and not a release gate.
- Purge committed generated artifacts: `reports/*` (keep only the two
  baseline files from Phase 0), 47 `.bak` / 7 `.old` files under `data/`,
  `data/quarantine/`, `data/staging-*/`. Confirm `data/intermediate/`
  (3.1 GB) is derived-and-checksummed, then gitignore it fully.
- Trim `docs/`: SYNODAL_V02–V14 audit docs move to `docs/history/`.

## Phase 2 — Extract the shared kernel (the load-bearing phase)

Create `church-slavonic-core` and move into it, unifying the duplicate pairs:

- Grammar vocabulary: one `Case`/`Number`/`Gender`/`Person`/`Animacy`/
  `Tense` set, one `RuleId`/`RuleTrace`, one `FormSet`/`FormVariant`, one
  defect/error enum. Sources: both families' `grammar.rs`, synodal core's
  `Error`/`ErrorCode`, OCS core's `PredictedForm`/`RuleStep`.
- Rule kernels: port the productive engines (`decline_noun`,
  `decline_adjective`, `decline_participle`, `present`/`aorist`/`imperfect`/
  `imperative`) from `synodal-church-slavonic-core/src/morphology.rs`, split
  into per-POS modules (`noun.rs`, `adjective.rs`, `verb.rs`, `pronoun.rs`,
  `numeral.rs`) the way ruthenian-core is laid out. OCS-specific paradigms
  (dual, aorist classes, twofold nouns) come in as variants of the same
  enums, not a parallel module tree.
- Lemma encoding: define the marker scheme (module `lemma.rs`) that carries
  animacy/class/stem facts on the citation form; migrate what currently
  lives in `noun_restrictions.tsv` / metadata slots into it where the fact
  is per-lexeme and unpredictable.
- Recension: `Recension { OldChurchSlavonic, Synodal }` becomes a parameter
  of the orthography/accent layer only. `church-slavonic-orthography`
  absorbs both `orthography.rs` files, `accent.rs`, and the positional/
  accent paradigm passes; the current `RecensionMapping` bridge machinery is
  deleted rather than ported.
- Exit criterion: both existing facades compile against the new core with
  their old test suites passing, so the Phase 0 baseline is provable before
  any API change.

## Phase 3 — Data diet (registry → PHF + binary artifact)

- Classify every row of the OCS registry and `data/synodal/exact_forms.tsv`
  as rule-derivable or genuinely irregular by replaying it through the
  Phase 2 kernel (this is exactly english's extractor filter step).
  Rule-derivable rows are dropped from shipped data; they remain in
  `data/extracted` as test oracle only.
- Genuine irregulars → `phf` tables generated into
  `crates/church-slavonic/generated/` with named-field record types.
  Budget: ≤ 2 MB; if the OCS residue exceeds that, the overflow goes into
  the dictionary crate's binary artifact (postcard/flatbuffer + include_bytes
  or lazy load), never into `.rs` source.
- Homograph policy: adopt english's deterministic numeric-suffix scheme
  (pure sort of emitted forms) and delete `homonymy_allowlist.tsv` /
  `target_identity_ambiguities.tsv` in its favor.
- Keep the registry-fingerprint idea from `build.rs` (stale-binary guard for
  the accuracy gate); drop the other build.rs contents.

## Phase 4 — The small API

- Facade surface follows the ruthenian rule. Per POS: one inflection
  function with typed parameters; paradigm-selecting distinctions become
  separate functions (`adjective` vs `short_adjective`); word-building
  operations become derivations returning a lemma that feeds back into the
  inflection functions (comparative, participle stems, gerunds).
- Collapse the OCS facade's ~70 free functions; the six `compound_cardinal*`
  variants become `numeral(value, case, gender, animacy)` plus at most one
  options struct.
- Keep and promote the Synodal `Inflector`/`InflectorBuilder` + `*Spec`
  shape as the advanced/configurable layer; the free functions above are
  thin wrappers over a default `Inflector`.
- Dictionary crate keeps `analyze_text`/`lookup`/`search`/`lemmatize` and
  the `synodal-dict` CLI; split today's 4,137-line `coverage.rs` and
  3,487-line `lib.rs` into focused modules.
- Publish the deprecation map: old crate names get one final release
  depending on the new crates with `#[deprecated]` re-exports where
  signatures allow, and a README pointer where they don't.

## Phase 5 — Gate and release

- CI gates: `cargo xtask accuracy` ≥ Phase 0 baseline per POS; bundled-data
  size check (≤ 2 MB in the facade); max-file-size lint; `cargo public-api`
  diff review.
- Delete the old crate families from the workspace once the new ones pass
  the full oracle replay; tag `v1.0.0-alpha`.
- Move `extractor` and `tools/research/` out of the published workspace
  (separate workspace in-repo, or sibling repo), keeping the checksummed
  `references/` discipline exactly as is.

## What is explicitly preserved

- Typed defects, `RuleTrace` provenance, attested-before-predicted analysis
  ordering, the 98 Alypy §104 irregular verbs inventory, the checksum-locked
  `references/` sources, the reviewed-lexeme review process (as research
  tooling, not build machinery).

## Order-of-work summary

| Phase | Deliverable | Risk |
|---|---|---|
| 0 | Baseline numbers + API snapshot | none |
| 1 | ~12k LOC deleted, repo hygiene | low (pure removal) |
| 2 | `church-slavonic-core` + `-orthography`, old facades ported | **high — do smallest POS (pronoun) first end-to-end** |
| 3 | PHF/binary data, registry.rs gone | medium (oracle replay catches regressions) |
| 4 | New facade + dictionary API | medium (API design review) |
| 5 | Gates, deprecations, v1.0.0-alpha | low |
