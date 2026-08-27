# Changelog

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
