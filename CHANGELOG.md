# Changelog

## Unreleased (the workspace rewrite)

### Added
- New crate family realizing `docs/REWRITE_PLAN.md`'s target layout:
  - `church-slavonic-core` — the shared closed grammatical vocabulary
    (dual `code()`/`abbrev()` spellings), adopted by both existing families
    with unchanged public APIs.
  - `church-slavonic-orthography` — shared text primitives plus the
    Glagolitic transliteration engine and the Synodal liturgical
    normalization as recension-named modules; family cores re-export.
  - `church-slavonic` — the rule-first OCS inflection facade: six POS at
    100% attested-oracle fidelity from 964 KB of generated sorted-slice
    residue tables (versus the 24 MB compiled registry), paradigm
    enumeration, value-driven `numeral()`/`distributive_numeral()`, the
    analytic phrase constructions, deterministic homograph suffix keys.
  - `church-slavonic-dictionary` — senses (5,174, homograph-aware lemma
    keys) and `lemmatize()` by inverting paradigm enumeration.
- Rewrite gates in `cargo xtask check-structure`: the per-POS attested
  oracles, differential gates against the old facade (numerals, phrases),
  the paradigm self-consistency gate, the dictionary round-trip gate, and
  the 2 MB facade data budget. New commands `rewrite-derivability`
  (with `--emit-residue`), `rewrite-pilot-accuracy`, `rewrite-dictionary`.

### Removed
- The ten frozen `synodal-v04`–`v08` one-shot xtask migration commands
  (~13k LOC); their audit documents moved to `docs/history/`.

### Deprecated
- The `old-church-slavonic*` surface is mapped item-by-item onto the new
  facade in `docs/DEPRECATION_MAP.md` (replaced / planned / dropped).

## Unreleased (the v0.12 program)

### Added
- `synodal_church_slavonic_dictionary::analyze_text` — the passage-level
  consumer entry point with per-reading provenance and stable serialisation;
  `synodal-dict analyze-text` on the command line.
- `synodal_church_slavonic_dictionary::prediction` — the exploratory
  segmentation tier (`predict_under`, walled behind
  `GenerationPolicy::Exploratory`), its masked-precision gate
  (`cargo xtask synodal-predict`) and ranked review-candidate feed.
- Alypy §73 reflexive voice: reflexive verb lexemes (lemma in `-сѧ`, bare
  stems) and rule-derived reflexive/passive readings of registered active
  verbs (`Analysis.reflexive`).
- Alypy §93 `j-series` imperative and §86 `vowel-t` aorist formations.
- The Synodal mixed sibilant series for long participles on stems in
  `ш/щ/ж/ч`.
- The `ᲂу` digraph fold in the lookup projections, digraph-aware accent
  placement, and `present_initial_uk_digraph` on generated liturgical prints.
- The per-wave generalisation ledger (`reports/synodal-waves.tsv`) and the
  holdout-led coverage report with per-system and predicted slices.

### Changed
- Coverage attribution prefers a token's first *typed* reading over a
  lexical-form row (reporting only; strict top-k is unchanged).
- Fifteen verbs admitted productively; three duplicate identities merged onto
  their reviewed ids; three mislabelled v0.6/v0.7 exact cells retracted
  through the correction ledger.
