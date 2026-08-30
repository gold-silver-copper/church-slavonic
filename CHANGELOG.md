# Changelog

## 0.7.0 — one resolution engine and the accent-pattern cells

### Changed
- The fact-resolution order (own exact cell -> bare exact cell -> facts
  read own-else-bare -> rule) is consolidated into
  `church_slavonic_core::resolution` and `church_slavonic_core::schema`;
  the runtime facade, the extractor's subtraction and reachability passes,
  and both dead-weight audits call the one engine. The refactor is
  byte-identical: same tables, same accuracy report.

### Added
- Synodal accent-pattern cells (noun 21, adjective 126, verb 548; arities
  22/127/549): a derived `s<N>`/`e` token adopted only when every attested
  accented form of the row shares the stress shape and the re-accented
  rule reproduces it exactly. ~370 rows adopt; mobile and mixed-convention
  paradigms (~6,500 accented Synodal rows) stay stored — the finding of
  this build is that Synodal storage couples stress with the print's
  plural-letter conventions, so pure stress patterns have limited reach.
  The refresh summary reports the token/Polyakov-class agreement rate
  (85/128 across 29 classes on this data).

## 0.6.0 — class cells and the slice tables

### Added
- Per-verb conjugation-class and present-stem override cells (546/547):
  derived facts, inferred by the extractor from the attested present cells
  and validated form by form, that re-run the finite (and present
  participle) rule with the verb's true class — a misclassed verb's finite
  block collapses to two cells. The runtime resolves exact cell -> bare
  row's cell -> class/present-stem override (own, else the bare row's) ->
  rule, and every audit mirrors that order.

### Changed
- The generated tables are sorted static slices looked up by binary search;
  the `phf` dependency is gone. Same information, simpler artifact:
  byte-identical accuracy output, table-hit throughput unchanged (~6.8M
  pronoun calls/s), and the runtime crate rebuilds faster (0.65s against
  1.46s on a table touch).

## 0.5.0 — declined participles

### Added
- The full declined participle system: present and past, active and
  passive, short and long series, over the adjective-style agreement
  features. New facade call `ChurchSlavonic::participle(key, &tense,
  &voice, &series, &case, &number, &gender, &recension)` with the new
  `Voice` and `Series` enums; `verb(..., Form::Participle)` still returns
  the two citation cells. The verb schema grows append-only from 38 to 546
  cells: 504 declined-participle cells and four participle-STEM cells the
  extractor derives from the attested declensions — a regular declension of
  an irregular stem costs four cells, not five hundred, and the runtime
  expands the stem through the same declension rule.
- Table sources for the new cells: the Kaikki participle sub-tables read in
  full, Polyakov's participle declensions (corpus-frequency gated at ≥5 —
  its hapax analyses are where OCR and analysis noise lives; the citation
  cells stay ungated), and the UD PROIEL train split under the existing
  gates. The two treebank evaluations score the declined cells too.
- Participle rules per recension: the OCS long-series shapes (`-щиимь`,
  `-щиꙗ`, `-щеи`), the Synodal print's own mixed declension for the active
  stems (Alypy pp. 95–96: `-щагѡ` but `-щихъ`), citation contractions
  derived from the stem (`-ѫщ` -> `-ꙑ`, `-ꙋщ` -> `-ый`), and reflexive
  participles.

### Changed
- The generated sparse rows index cells as `u16` (the schema outgrew
  `u8`); `check-registry`'s dead-weight audit is stem-aware.

## 0.4.0 — the english-parity release (breaking)

The workspace is rebuilt in the shape of `gold-silver-copper/english`: four
crates, generated PHF tables as the whole artifact, three `xtask` commands,
one README.

### Changed (breaking)
- `church-slavonic` is one `lib.rs`: table-first, rule-fallback, case
  restoration, deterministic `_n` sense keys assigned by a pure sort (keys
  may renumber on refresh). Every call takes `&Recension`
  (`OldChurchSlavonic` | `Synodal`); the scoped handles, profiles and
  identity layer are gone.
- `church-slavonic-core` is rules only (no data): `grammar`, `noun`, `adj`,
  `verb`, `pronoun`, `orthography`, `sense_key`, `utils`; depends on
  `unicode-normalization` alone.
- The tables regenerate from two pinned sources (Kaikki OCS Wiktionary,
  the Alypy grammar pages) with `cargo xtask refresh-data`; no curated
  data files, ledgers, overrides or lockfiles remain.

### Removed
- Crates `old-church-slavonic-core`, `church-slavonic-orthography`,
  `church-slavonic-dictionary`, `synodal-church-slavonic-core`,
  `synodal-church-slavonic`, `synodal-church-slavonic-dictionary` and the
  old extractor. Each published name gets a final empty patch release
  pointing here (`deprecated/`); their sources are at tag
  `pre-english-parity`.
- `data/` (except the gitignored `data/intermediate/`), `reports/`, `docs/`,
  the root prompt files and the non-CI workflows.
- Every `xtask` command except `refresh-data`, `check-registry`, `accuracy`.

### Moved
- The Synodal text analyzer is an unmaintained experiment under
  `experiments/analyzer/`, built against the published 0.6 crates.

Earlier history (0.1–0.3 and the synodal 0.4–0.6 program) is in the git
history of `CHANGELOG.md` before this release.
