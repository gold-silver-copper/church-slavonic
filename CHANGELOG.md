# Changelog

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
