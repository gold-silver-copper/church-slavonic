# Handoff: church-slavonic 4.1.0 (tag `v4.1.0`)

Read this first in a fresh session. It is the map of what exists, what
the numbers are, and which designs are open, so work starts from
verdicts and recorded measurements. The design record is
`docs/DESIGN.md`; the diary is `NOTES.md` (dated entries, decisions and
findings per part); `CHANGELOG.md` holds every measured number per part;
`V2-PROMPT.md`, `V2.1-PROMPT.md` and `V2.2-PROMPT.md` are the executed
plans and stay as their records (tags `v2.0.0` … `v2.3.0`);
`V3.0-PROMPT.md` is executed in full (releases 3.0.0, 3.1.0, 3.2.0, tags
`v3.0.0` … `v3.2.0`, with `V3.0-CONTINUE-PROMPT.md`); `V3.3-PROMPT.md`
is executed in full (3.3.0 for Parts 0–2, 3.4.0 for Parts 3–4, tags
`v3.3.0`, `v3.4.0`); `V4-PROMPT.md` Part 0 is executed as 4.0.0 (tag
`v4.0.0`: the public API's shape — typed cells, named errors, rustdoc
examples, the `ocs` feature) and Parts 1–2 as 4.1.0 (tag `v4.1.0`: the
sentence in the library — `church_slavonic::sentence`; the Ponomar
library as a second print, 28 books, 3.19 million tokens, fetched and
pinned by `scripts/fetch-ponomar.sh`, run by every treebank command
behind `--corpus ponomar[/<book>]`, exported with the provenance of
every token by `cargo xtask export`; the intake wave its census
dictated — the importer expands titlo-written forms, rows 135 → 559,
481 lexemes added; the overlay by register begun by hand: the
Octoechos's Gospel stichera and the April Menaion's St Artemon); Part 3
is a measurement recorded in `docs/OPEN-DESIGNS.md` 1b: the rules'
376,943 resolved leaves as training examples buy 0.7 points where 1,300
of gold buy 3.3, and dilute gold — distillation is not a road. The open list
below is what the releases left, ranked by both censuses. The
game's 35 tests and headless run pass against this lexicon.
Two things before any command: export `RUSTC_WRAPPER=` (empty) in this
checkout while the shared sccache daemon lacks Desktop access; after a
lexicon change run `fix-hand-alts`, then `build-treebank` — the stored
treebank goes stale before it goes wrong. The crate on crates.io is
1.0.0; 2.x, 3.x and 4.0.0 are tags only — 4.0.0 is the first version
whose API is the one the README describes.

## What exists

- `crates/church-slavonic` — the library, dependency `unicode-normalization`
  only. A committed lexicon (`lexicon/syn/*.tsv` Synodal, `lexicon/ocs/*.tsv`
  Old Church Slavonic; one lexeme per line: id, lemma, pos, gender, anim,
  class, stress, stems, overrides, variants, src, note), class tables
  (`lexicon/classes/*.tsv`, `lexicon/classes/ocs/*.tsv`), the stress
  paradigms (`lexicon/stress.tsv`), the titlo rows (`lexicon/titlo.tsv`).
  Four stages: lexeme → letters (class) → stress (paradigm) → typography
  (`Form::print`). `Lexicon::synodal()/ocs()`, `get(id)`, `find(lemma,
  pos)`, `Lexeme::inflect(cell)`, `forms(cell)`, `Lexicon::analyze(surface)`,
  `Lexicon::readings(surface)` (grouped by lexeme: syncretism is one
  reading with several cells, homonymy several readings),
  `cell::CellSet` (`nom|acc|voc.sg`), `Lexicon::guess(lemma, pos)`;
  since 4.0 typed cell constructors (`Cell::noun(case, number)`,
  `Cell::finite(tense, person, number)`, …) beside the name parser,
  `inflect` a `Result<Form, InflectError>`, `Cell::parse` a
  `Result<Cell, CellError>`, `lexicon::parse` a `LexiconError` with its
  line, rustdoc examples run by `cargo test`, and the `ocs` feature
  (off by default) carrying the Old Church Slavonic lexicon. Ids
  are stable (`рабъ.n`, `рещи.v`, `той.pron`, `и.x`); a consumer may
  persist them. A verb's present stem is its class's derivation (Leskien
  types in OCS: `V:IV:i`, `V:III:j`, `V:I:к`, …), and so are its aorist,
  imperfect and l-participle; `stems=2=` on a line is suppletion. The
  closed lexicon (`lexicon/syn/closed.tsv`) is structured: the class is
  the subcategory, a preposition carries `gov=`, an enclitic or proclitic
  `pros=`; an adverb an adjective prints is the adjective's `adv` cell.
  The fifth stage is the phonological word (`Form::print_unit`,
  `print_hosting`, `prosody::words`); the treebank writes it `(pw …)` /
  `(pwa …)`.
- `crates/church-slavonic-tagger` — the statistical layer of homonymy
  (2.3): an averaged perceptron over the analyzer's (pos, cell) readings,
  `Tagger::bundled()` the committed model (`data/models/tagger.bin`, with
  `tagger.md` and `tagger.sha256`), `fold`/`fold_word` the manuscript
  fold, `Trainer`. Trained on UD PROIEL train and Syntacticus (UD's
  held-out sentences removed), never on the Bible.
- `crates/church-slavonic-tools` — `cargo xtask`: `import <source> --pos
  <pos> [--write]` (polyakov, alypy, ruwiktionary, kaikki, ud; Polyakov's
  import fits the accent inventory and takes the Bible as arbiter of
  stress twins from `data/treebank-forms.tsv`), `refit-stress --pos <pos>
  [--write]` (a file's stress columns re-fitted from its own forms, no
  form changed), `filter-ud`, `eval [--guess verbs [--ocs]]`, `census
  <stems --pos verb [--ocs] | verb-cells --ocs | closed [--write] |
  clitics | homonymy | stress | forms [--write]>`,
  `build-treebank` (the constraint layer `treebank/disambiguate.rs` and
  the tagger `treebank/tag.rs` run after the lift; `CS_NO_DISAMBIGUATE=1`,
  `CS_NO_TAGGER=1` turn them off), `check-treebank` (asserts every
  auto-lifted leaf names every cell that prints its token, a narrowed
  leaf's `:from` set being the lexicon's), `narrow-hand`, `fix-hand-alts`,
  `redraft-hand` (3.3: the overlay's verbatim leaves become leaves when
  the lexicon catches up, narrowed by their notes), `hand-draft <book>
  <chapter>`, `score-disambiguation` (the rules and the tagger against
  the hand overlay; a hand set counted apart), `tagger-curve`,
  `train-tagger [--epochs n]`, `tagger-transfer` (3.4: the five-fold
  measurement over the overlay, no model shipped; `CS_DEBUG_VERSE=3:14`
  makes `score-disambiguation` print an overlay verse's auto tree),
  `analyze [--ocs] <word>…`, `titlo
  <surface>…` (the titlo index's entries), `census verbatim [--write]`
  (the verbatim leaves by cause; `--write` writes
  `data/loanword-iota.tsv`, the importer's evidence for the loanword's ї). Class tables are generated by the scripts under
  `scripts/` (`polyakov-legend-to-classes.py`, `legend-adj-verb-pron.py`,
  `kaikki-to-classes.py`): edit the script, never the tsv.
- The Bible treebank (`treebank/`, gitignored, rebuilt from the print in
  ~70 s with both layers) and the hand overlay (`data/treebank-hand/`:
  Genesis 1–3, Exodus 1, Leviticus 1, Proverbs 1, Isaiah 53, Matthew 1,
  Luke 2, John 1, Romans 1, 1 Corinthians 13 — 337 verses, 3,757
  leaves, flat; a leaf may be a set where the annotator's note named no
  cell). `lexicon/titlo.tsv` (135 rows, by hand with the Bible count),
  `data/loanword-iota.tsv`, `data/twins.tsv`.
- `~/Desktop/code/vertograd` — the monastery game, the crate's consumer;
  its `slavonic.rs` adapts the game's lemma-keyed calls to the lexicon.
  `./scripts/headless-test.sh` must stay green.

## Standing rules

- The lexicon holds lexical facts; the class tables hold paradigms; a
  form the class produces is never stored, a form it does not produce is
  an `override` or a `variant` on the lexeme line with provenance.
- The lexicon files are `include_str!`-embedded: `cargo build` before any
  `xtask` sees an edit.
- Every attested print round-trips through `Form::from_print`/`print`
  (the flags `varia`, `kamora`, `mark_skip` exist for that); the
  consistency test (`cargo test -p church-slavonic`) enforces it.
- The treebank's round-trip invariant: zero mismatches over 34,470
  verses; syncretism is recorded as the leaf's set (`:case nom|acc`),
  homonymy as `:amb n`; nothing is guessed.
- A stem the class can derive is never stored; `census stems` is the
  arbiter.
- Gate on the three eval numbers, never on self-consistency.
- Game: exact-output tests before a lemma enters content; when the game
  and the crate disagree, check the crate against its pinned sources
  before deciding which side is wrong.

## The numbers (4.1.0)

Start-up: the tsv parse 0.1 s; the analyzer's index 11.7 s on twelve
cores; embedded lexicon 4.0 MB without the `ocs` feature, 6.4 with.
Synodal lexemes: nouns 13,303, adjectives 8,588, verbs 8,292, pronouns
72, closed 1,366 (4.1 added 131 / 267 / 68 / 15 — the lexemes Polyakov
writes only under a titlo, which the importer now expands through the
titlo rows' skeletons — and absorbed 2 adjective ids and 27 adverb ids
into `data/twins.tsv`; ids never move — `restore_ids`, which also looks
an id up by the source's headword and by the collapsed spelling of a
prefix's о before ꙋ). Titlo rows 559. Stress: 47 named paradigms. The
print writes the initial uk as ѹ, the prefix от- as ѿ, the loanword's ї
(a vote over every spelling the Bible and the library print, per
position) and the paerok as letters of the lexeme, the izhitsa's
kendema by rule; Polyakov's у is read as the print's ꙋ (поꙋче́нїе).
Held-out recall (UD PROIEL dev+test): nouns 95.48%, adjectives 89.31%,
verbs 90.89%, personal pronouns 99.25%, other pronouns 98.07%.
Bible treebank (631,946 tokens): one cell 243,358 (38.5%), sets 2,047
(0.3%), tagger 187,993 (29.7%), closed 179,015 (28.3%), several lexemes
12,645 (2.0%), verbatim 4,789 (0.8%), apparatus 2,099 (the titlo
numerals count as apparatus since 4.1); zero mismatches; it rebuilds in
95 s. `census verbatim`: 4,763 leaves — 2,007 found by key but printed
otherwise (the clitic after a host that is several lexemes: ты́ 180 in
the Bible; head ѧ҆зы́къ), 646 under a titlo with no row, 2,110 with no
reading. The Ponomar library (28 books, 142,620 units, 3,190,662
tokens; builds in about ten minutes): one cell 1,206,378 (37.8%), sets
26,165, tagger 1,035,721 (32.5%), closed 638,779 (20.0%), several
lexemes 102,102 (3.2%), verbatim 70,210 (2.2%; 5.6% before the intake),
apparatus 111,307; every unit round-trips; `census verbatim --corpus
ponomar`: 70,206 leaves — (a) 15,397 by letters (marks only 11,124: the
clitic hosts ты́ 1,676, мѧ 1,030, ны 526, мы́ 405; і/ї/и 1,112; head ѧ
1,079; wide/narrow о 659), (b) 13,676 titlo tokens with no row (ѻ҆ц҃а̀
with the varia, the rubric's abbreviations), (c) 41,133 with no reading
(꙳ as a token of its own 25,307, `]]` 3,262, ст 1,832, гл 1,246, the
-десѧть compounds, the capitals of titles). Hand overlay (Bible): 337
verses, 3,757 leaves; the rules exclude no hand cell and resolve 446;
the tagger 74.58% (1,212 of 1,625). The overlay by register: the
Octoechos's Gospel stichera (22 units, 483 leaves) — rules and tagger
contain the hand cell 77.99%, resolve 77.57%, the tagger 63.29%; the
April Menaion's St Artemon (73 units, 736 leaves, 361 left as sets) —
92.00%, the tagger 71.88%; neither is called gold before a second pass.
The five-fold transfer (`tagger-transfer`): OCS + four folds of the
overlay 78.93%, OCS only 75.12%, the bundled model 74.58%; the shipped
model stays OCS-only. The tables are in `CHANGELOG.md` under "4.1.0",
"3.4.0", "3.3.0", "3.1.0" and "3.0.0" and in the README.

## Open designs, in order

The analysis of each — what it is, what the measurements say, and the
linguistically proper answer — is in `docs/OPEN-DESIGNS.md`, which also
records what 2.1 executed (present stems by derivation, syncretism by
underspecification).

1. **What 4.1 left of the verbatim residue**, ranked by both censuses
   (`census verbatim`, `census verbatim --corpus ponomar`): the clitic
   after a host that is itself several lexemes (ты́ 1,676 in the
   library and 180 in the Bible — ты.pron beside the aorist of тыти.v;
   мы́ 405, ѻ҆ни́; never "the commoner lexeme wins"); the clitic after a
   titlo host (сп҃си́ мѧ 824 — `lift_apart` looks the host up without the
   titlo index); head ѧ҆зы́къ (1,079 + 161: a stem variant the line
   format lacks — decide the format in DESIGN first); the wide/narrow о
   of самаго̀, отъ (659); ꙳ as a token of its own (25,307 — punctuation
   only when glued today) and the `]]` of a note; the rubric's
   abbreviations (ст, гл: a rubric bucket the census never built, Part
   2.4); the -десѧть compounds and the capitals of titles; ѻ҆ц҃а̀ with
   the varia under a row that prints ѻ҆тца̀; the Psalter, Apostol and
   Gospel volumes against the Bible's own text (Part 2.3, not done);
   the collective numerals дво́е/ѻ҆боѧ̀; the paerok inside a prefixed
   verb (ѡ҆б̾ѧ́тъ); господь.n / .n.2; the plural-mark cells
   кѡры́сти/четверонѡ́гъ/ложесна̀, всѣ́мъ, и҆̀же/ꙗ҆̀же with the varia; the
   names measured through the guesser (Part 2.6.8, never done); нога's
   dual но́зѣ (the lexicon prints нѡ́зѣ and но́ѕѣ). And the overlay by
   register: a second pass over a sample of each service before it is
   called gold, then a Sunday vespers and a full Menaion service decided
   leaf by leaf.
2. **What the clause rule cannot reach** (3.2 Part 5, 3.4 Part 3): a
   clause whose verb is itself several lexemes gives the rules nothing
   without a selection — bare-loc now reaches the locative twin (ви́дѣ),
   not the genitive, accusative, closed-word or participle twins (135
   tokens; the genitive has too many uses to name); the tagger's
   transfer gap (89% on OCS dev, 74.6% on the overlay) is measured: four
   folds of the overlay buy 3.8 points, and 90% wants a Synodal source
   the size of the OCS material in the print's conventions (OPEN-DESIGNS
   1b) — the nominative/accusative errors (114) are syntax a one-token
   window does not see.
3. **What the homonymy layers left** (`docs/OPEN-DESIGNS.md` 1b, executed
   in 2.3). The tagger's errors on the overlay are the syntax a one-token
   window does not see: nominative against accusative of an inanimate
   (52 of 274), a pronoun's gender from its antecedent, the treebanks'
   conventions (по with the dative); its softmax share is not
   calibrated, so no confidence threshold is applied. The rules never
   exclude a hand cell but resolve 45%: a subject/object rule needs
   structure the flat tree lacks. The lexicon duplicates the homonymy
   census named (гдⷭ҇ь 3,579, а҆́зъ 1,939, ва́мъ, мнѣ̀) are still several
   lexemes to the lifter and a cleaning would move them to one. The
   overlay is 211 verses and should grow by register (the Psalter's
   poetry, the Epistles).
4. **What 2.2 left.** Kaikki's aorist and l-participle junk is counted,
   not kept (1,105 cells); the OCS UD variants that remain in those
   blocks are manuscript spellings. The print writes the phonological
   word apart more than solid, and the pronoun hosts whose accented form
   is several lexemes (ѻ҆на́, ты́) wait for the disambiguator. A
   generator's second-position placement has a function and no caller.
   The Synodal нн/н long-participle spelling stays a lexeme fact.
