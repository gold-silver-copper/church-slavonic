# Rewrite plan: one kernel, small API, rules first

Status: executed (2026-08-25/26; see the execution log). Executable against this repository as it stands
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

## Execution log

- 2026-08-25 — Phase 0 complete: tag `v0.14-pre-rewrite`, baseline in
  `reports/rewrite-baseline.md`, API snapshot in
  `reports/rewrite-api-snapshot.txt`.
- 2026-08-25 — Phase 1 largely complete: ten frozen `synodal_v04–v08`
  xtask modules deleted (~13k LOC) with dispatch/help/`check_structure`
  unwired; eight unreferenced SYNODAL_V* audit docs moved to
  `docs/history/`; orphaned `verb-*` reports dropped. Deviations from the
  written plan: the V04–V07 docs/reports stay in place because
  `synodal-archive --check` pins them as immutable checksummed artifacts
  (revisit when the archive machinery retires); the `.bak`/`.old`/
  quarantine/staging/intermediate trees were already untracked, so no
  committed purge was needed; relocating the wave/holdout tooling to
  `tools/research/` is deferred — it is load-bearing inside
  `synodal::check`'s guard witnesses and still drives active v0.14 work.
- 2026-08-25 — Phase 2 slice 1: `church-slavonic-core` created; the twelve
  closed grammatical enums moved there with dual `code()`/`abbrev()`
  spellings; synodal-core re-exports them. Slice 2 (OCS adoption of the
  five identity enums) in progress. Trace/result unification is deliberately
  deferred until the rule kernels merge: the two `RuleId` models are
  semantically different (closed rule inventory vs evidence-tagged ids).

- 2026-08-25 — Phase 2 slices 2–3: OCS family adopted the kernel's five
  identity enums plus AdjectiveForm (`abbrev()` spellings; historical
  Animacy enumeration order preserved via `OCS_ANIMACY_ORDER`); no
  committed artifact changed, accuracy stays 134,761/134,761. The
  orthography-crate merge is deprioritised: the two modules are nearly
  disjoint domains (script transliteration vs liturgical normalization).
- 2026-08-25 — Phase 3 groundwork: `cargo xtask rewrite-derivability`
  measures the registry against the pure rule kernel. 84.7% of attested
  cells derive exactly; the residue is ~20.6k cells / ~347 KB of form text
  (vs the 24 MB compiled registry), confirming the PHF budget. Known
  measurement gaps: closed classes/pronouns/numerals not yet wired to
  their reviewed identity kernels; the verb shortfall is a
  verb_metadata.tsv coverage gap; ~3k adjective "divergences" are one
  animate-accusative convention difference, a policy call to adjudicate.

- 2026-08-26 — Phases 3–4 pilot landed: new `church-slavonic` facade crate
  serves nouns as residue-table → identity-kernels → class rules from a
  553 KB generated sorted-slice table (vs the 24 MB registry), validated
  at 100% of the lemma-merged attested oracle by
  `cargo xtask rewrite-pilot-accuracy`. Homograph lemmas are currently
  served with rank-merged variant lists; the english-style numeric-suffix
  scheme remains open.

- 2026-08-26 — Pilot facade COMPLETE across every attested POS: nouns
  (41,370 merged cells), adjectives (39,204; animacy proven vacuous and
  dropped from the API), verbs (13,100; 53% from rules), pronouns (1,341),
  numerals (126), determiners (36) — all at 100% of the attested oracle
  via `cargo xtask rewrite-pilot-accuracy`, from ~1.18 MB of generated
  sorted-slice tables versus the 24 MB registry. Within the ≤2 MB facade
  data budget. Open items for the full phase 3/4: shrink the verb residue
  by widening principal-part metadata coverage; replace the pilot's
  temporary dependency on the old-church-slavonic resolver crate (closed-
  class identity kernels should move into a core crate); decide the
  homograph story (currently rank-merged variants, not numeric suffixes);
  then port the dictionary/analyze layer and cut the deprecation release.

- 2026-08-26 — Pilot hardening: the facade's dependency on the fat OCS
  facade is gone (closed-class kernels were core-backed; the §316
  pronominal composer moved into core). check-structure now gates on the
  six pilot oracles and the 2 MB data budget. Principal-part synthesis
  infers verb metadata from the attested oracle: verb residue 6,178 →
  2,083 cells, 473/707 verbs fully rules-backed. Total pilot generated
  data: 964 KB — inside the english-crate silhouette.

- 2026-08-26 — Homograph policy resolved per the english scheme: per-lexeme
  suffixed keys (bare/_2) assigned by a pure inventory sort, no lockfile;
  all oracles now gate per lexeme at 100% (nouns 41,566; verbs 13,260).
  Full check-structure passes with the pilot gates included. Remaining
  major items: the new crate's README + deprecation map (in flight),
  paradigm enumeration / phrases / dictionary-analyze layers on the new
  facade, the synodal-family counterpart, and the phase-5 deprecation
  release.

- 2026-08-26 — Phase 4 progress: README + deprecation map written;
  paradigm enumeration landed with a self-consistency gate (each paradigm
  equals exactly what the single-cell API serves); value-driven
  numeral()/distributive_numeral() replaced the twelve cardinal variants
  with a 100% differential gate against the old machinery, and the
  compound composer moved into old-church-slavonic-core. Remaining:
  phrases, dictionary/analyze, orthography/transliteration, the synodal
  counterpart, and the phase-5 deprecation release.

- 2026-08-26 — Phrases layer landed on the pilot: `church_slavonic::phrases`
  serves the consumer analytic constructions (§316 pronominal families,
  absolute superlatives, да-imperative, six copular series, perfect /
  three pluperfects / future perfect, conditional-optatives with and
  without да, three infinitival futures, impersonal predicates) as
  typed-parameter String-out functions; paradigm-selecting enums became
  functions. §316 validation/composition and the past-reference future
  license moved into old-church-slavonic-core with the fat facade
  delegating. `rewrite-pilot-accuracy` gains a phrase differential gate
  (4,209 sweep cells vs the old phrase layer, 100%). The
  participle-predicated constructions (analytic/conditional passive,
  participial future) wait on declined participles;
  `elliptical_conditional_optative` collapses to `l_participle`.

- 2026-08-26 — church-slavonic-dictionary landed: 5,174 senses as 2.2 MB
  sorted-slice tables, homograph-aware lemma keys, lemmatize() by lazily
  inverting paradigm enumeration (measured lazy beats a generated index),
  check-structure-gated (all pinned keys resolve; 101,206/101,206
  round-trips). church-slavonic-orthography extraction in flight.

- 2026-08-26 — OCS scope of the rewrite EXECUTED (31 commits since
  `v0.14-pre-rewrite`, every one landing green). The target layout exists:
  church-slavonic-core (shared vocabulary), church-slavonic-orthography
  (shared text primitives + glagolitic + synodal modules, family cores
  re-exporting), church-slavonic (full inflection facade: 6 POS at 100%
  attested-oracle fidelity from 964 KB of tables, paradigm enumeration,
  value-driven numerals, analytic phrases — all differentially or
  self-consistency gated in check-structure), church-slavonic-dictionary
  (senses + lemmatize, gated). Remaining scope, deliberately not taken
  autonomously:
  (a) declined participles as adjective-lemma derivations (no attested
  oracle exists — extraction excluded them as not safely attributed);
  (b) the synodal-family counterpart (its own multi-week program: the
  8.5k-line morphology merge, exact-forms diet, accent layer);
  (c) phase 5 release mechanics — publishing, `#[deprecated]` re-export
  releases, and deleting the old crate families are user decisions.

- 2026-08-26 — Remaining-scope item (a) landed: declined participles on the
  pilot facade as `participle(lemma, kind, case, number, gender, form)` /
  `participle_variants` / `participle_paradigm` — the derivation's per-kind
  stem declined through the core adjective machinery (inanimate convention,
  like the adjective surface). Resolution: attested/metadata citation
  precedence on the citation cell, then the reviewed verb-family kernel
  (`reviewed_verb_lexemes`, the resolver's `ReviewedVerbProfile` composition
  moved down into old-church-slavonic-core), then principal-part metadata.
  With no attested oracle, `rewrite-pilot-accuracy` gates a 100% differential
  sweep against the old facade's declined-participle handles (14 verbs x 4
  kinds x case x number x gender x form; citation cells gated as
  self-consistency with the attested-first citation functions), plus a
  259k-cell participle-paradigm self-consistency gate. The three deferred
  phrase constructions ported the same way: `analytic_passive` (+
  `_imperfect`/`_aorist`/`_future`), `conditional_passive`(+`_aorist`), and
  `participial_future`, each differential-gated at 100%.

- 2026-08-26 — Final executable scope closed (36 commits since
  `v0.14-pre-rewrite`): declined participles landed with a differential
  gate plus the three passive/participial phrase constructions; the
  synodal monoliths (morphology 8,559, extractor lib 5,167, facade lib/
  phrases/numeral_phrases/spec/registry, dictionary lib/coverage) all
  split under the 1,500-line rule behind API-identical seams; succession
  notices added to the old OCS crates' READMEs; `v1.0.0-alpha` tagged
  locally. Blocked on the user (outward-facing or sequenced behind
  publishing): the crates.io deprecation release of the old names, the
  subsequent deletion of the old OCS family (the plan sequences deletion
  after that release), pushing, and the synodal one-override-precedence
  semantic change (an API-behavior decision for the surviving Inflector).

- 2026-08-26 — The one-override-precedence hard rule landed for the
  synodal family: resolve_cell is merged-irregular-table -> typed defects
  -> rule kernel; provider-exact and caller-irregular channels removed
  from the public API; registry overrides folded into the generated exact
  table as provenance stamps; homonymy allowlist reclassified as an
  identity license. All behavioral gates identical. Pushed to origin/main with both tags
  2026-08-26. Remaining, per the plan's own sequencing: the crates.io
  publication of the new crates and the deprecation release of the old
  names (requires the maintainer's registry credentials and is the
  irreversible public-claim step), then the old-family deletion.

- 2026-08-26 — PLAN COMPLETE. The release train shipped to crates.io in
  dependency order (ten crates; one broken dictionary tarball caught by
  standalone verification, republished as 0.5.1 and 0.5.0 yanked), CI's
  publish dry-run gate restored to the full list, and the superseded
  old-church-slavonic facade and dictionary crates deleted from the
  workspace with xtask's transition scaffolding — the attested-oracle,
  self-consistency, and dictionary gates carry the guarantees forward.
  Tagged v1.0.0-alpha.

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
