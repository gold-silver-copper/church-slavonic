# Changelog

## 1.1.0 — the consumer-defect release

The first real consumer (the `vertograd` monastery game) audited hundreds
of generated forms; its rejections diagnosed into three defect classes,
each fixed in its proper layer — and twice the audit itself was wrong and
the crate's attested answers stand guarded.

### Fixed
- **Lookup folding**: `ѷ`-spelled Synodal input now reaches the
  `ѵ`-spelled table key (the kendema is positional typography, as
  `comparison_key` always held); new lookup invariant 4 in the crate docs.
  `кѷпарі́съ` finds its attested inanimate accusative.
- **End-stressed verbs can hypothesize**: the class/present-stem override
  inference also strips accented endings («стриже́ши», «дои́ши»), and it
  re-runs over each candidate's UNIONED cells — sources that attest one
  cell per entry (Polyakov form-of) previously starved it.
- **The stems the infinitive hid**: a Second-class fact on an `-ити`
  lemma re-derives the stems as a true i-verb (`дои́ти` "to milk" is not
  `до` + `и҆тѝ`: aorist «доѝ», imperative «доѝ», the «напоѝ» print
  type); a present-stem fact on a `-щи` lemma restores the neutralized
  velar to the aorist and l-participle (`стрищѝ` : «стри́глъ»,
  «стрижѐ»). The l-participle now enters the fact engine on class/stem
  facts, and unaccented derived stems thread the lemma's accent like the
  plain rule path.
- **The accusative-shape fact teaches both ways**: any attested
  nominative-shaped accusative (singular, dual or plural) marks the row
  inanimate for the others (`ѻ҆гꙋре́цъ`), and the extractor's re-store
  pass reads its sources live so mutually-derivable cells are stored
  once.

### Added
- **The witness source** (`data/witnesses.tsv`): curated single cells
  from running Synodal print, each citing a verbatim line of a pinned
  text, ingested like any source (own 100.00%/0 accuracy row) and
  verified by `cargo xtask check-witnesses`. First admissions: the
  inanimate accusative of `ѡ҆́блакъ` (Lk 9:34), `ꙗ҆́блонь`'s nominative
  (Joel 1:12) and feminine instrumental (Song 8:5).
- The consumer-defect ledger (`tests/consumer_defects.rs`): every
  diagnosed form as a test, including the guards for the two forms where
  the AUDIT was wrong — «пожа́тъ» (the attested `-ѧти` aorist) and
  «вожжѝ» (an attested imperative spelling; the guard asserts the
  attested set across sense keys, since sense numbers may renumber).

### Measured, unchanged
- The Synodal unattested-masculine animate accusative default stands:
  Polyakov's masculines are 72.9% animate by lemma / 53.3% by token, and
  no Synodal held-out corpus exists to arbitrate further. Unattested and
  unwitnessed accusatives (`прꙋ́дъ`, `ко́локолъ`) keep the default.
- Held-out corpus recall: unchanged except OCS dev+test verbs
  7414 → 7413 (−1 slot, recorded in NOTES).

## 1.0.0 — the schema close

### Added
- The l-participle (resultative): `ChurchSlavonic::l_participle(key,
  &gender, &number, &recension)`. The verb row grows append-only from 549
  to 558 cells (a nominative-only gender/number block); the rule builds it
  on the infinitive stem (`бꙑти` : `бꙑлъ`, `вести` : `велъ`, `рещи` :
  `реклъ`), Synodal accents ride the lemma's stress, and the reflexive
  `-сѧ` stays outside. Newly attested cells: Polyakov's `partcp,perf`
  forms (4,082 slots), the UD PROIEL train split's `PartRes` tokens, and
  the Wiktionary l-participle pages. The held-out treebank evaluations
  score the new cells.
- The non-personal pronouns: `ChurchSlavonic::npron(key, &gender,
  &number, &case, &recension)` over a new 54-cell lemma-keyed table
  (`npron_phf.rs`) and a pronominal-declension rule — the hard `тъ` type,
  the soft `сь`/`мои`/`нашь` type, mixed `вьсь`, the relative `иже` as
  the anaphoric series plus `же`, the singular-only `къто`/`чьто`, and
  the `ни-`/`нѣ-` compounds. Sources: the Wiktionary pronoun tables and
  form-of pages and the UD PROIEL train split; both treebank evaluations
  score the class (93.2% and 94.3% held-out recall on first contact).

### Fixed
- Two same-signature candidates now union their attested raw cells, so a
  rule-equal form shadowed by a stored bare cell is re-materialised on a
  variant key instead of silently dropped (`еиже` under `иже`).

### Notes
- 1.0.0 closes the schema scope. The residual held-out corpus-recall gap
  is documented data ceiling, not backlog: forms enter the tables only
  when a pinned source attests them past the gates, and no further
  qualifying machine-readable source exists today (see `NOTES.md` for the
  candidates examined). The research diary lives in `NOTES.md` from this
  release on.

## 0.9.0 — the accusative-shape fact

### Added
- The noun resolution reads a row's stored LOWER accusative as a fact
  (`schema::NOUN_SHAPE_SOURCE_CELLS`, cells 3 and 10): a stored
  nominative-shaped accusative — an inanimate, where the Synodal masculine
  rule answers the genitive shape — teaches the row's higher accusative
  cells the same shape (`вѣне́цъ` : `вѣнцы̀`, not `вѣнце́въ`). Sources
  derive upward only, so the lowest stored accusative is the anchor and
  never subtracts itself. One engine as always: the facade, the
  extractor's subtraction, the reachability passes and the audits all read
  it; no new cell, no arity change. 39 stored accusative-plural cells are
  now derived, and ~1,500 rows whose only attested accusative is the
  singular now answer their unattested plural in the attested shape.


## 0.8.0 — convention-aware accent tokens

### Changed
- The accent-pattern token now rides inside the accent pass
  (`core::accent::with_accent_pattern`) on every rule path, instead of
  bare-re-stressing the finished form: the print's stress-coupled
  conventions — the wide `ѡ`/`є`, the kamora, the word-final varia, the
  carried stem marks — follow the token's position exactly as they follow
  the lemma's. The skeleton-level stem/override paths, whose endings carry
  no convention marker, keep the bare re-stress.
- The convention itself was corrected against the corpus: the widening
  targets the last narrow `о`/`е` at or after the stress; a form stressed
  on its final vowel widens the last narrow `о`/`е` anywhere instead
  (`вѡнѝ`, `верєѝ`); and a lexical wide letter no longer excuses the
  kamora (`а҆арѡ̑нимъ`, `а҆вессалѡ̑мли` — the print writes the kamora
  anyway). 643 attested kamora-bearing cells that were stored are now
  reproduced by rule; noun bare accuracy +0.09 (Polyakov) and +0.20
  (Alypy), adjectives +0.02, nothing regresses, OCS byte-identical.


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
