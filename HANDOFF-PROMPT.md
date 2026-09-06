# Handoff: church-slavonic 4.0.0 (tag `v4.0.0`)

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
examples, the `ocs` feature), its Parts 1–3 are the plan for 4.1.0;
the open list below is what the releases left. The
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

## The numbers (4.0.0 — unchanged from 3.4.0 but the start-up)

Start-up: the tsv parse 0.1 s; the analyzer's index (8,190,171 forms)
11.7 s on twelve cores (17.7 before 4.0's round-robin split); embedded
lexicon 4.0 MB without the `ocs` feature, 6.4 with.
Synodal lexemes: nouns 13,172, adjectives 8,323, verbs 8,224, pronouns
72, closed 1,351 (158 twins absorbed in 3.1; 50 lexemes added in 3.3 —
the numerals, любы̀, сто̀, the хотѣ́ти/вѣ́дѣти/спа́ти families, seven
closed words; ids never move — `restore_ids`). Stress: 47 named
paradigms. Titlo rows 135. The print writes the initial uk as ѹ, the
prefix от- as ѿ, the loanword's ї (кївѡ́тъ) and the paerok (в̾слѣ́дъ) as
letters of the lexeme (the importer, with the Bible as arbiter), the
izhitsa's kendema by rule. Held-out recall (UD PROIEL dev+test): nouns
95.48%, adjectives 89.31%, verbs 90.89%, personal pronouns 99.25%, other
pronouns 98.07%. Bible treebank (631,946 tokens): one cell 244,589
(38.7%; 27.8% without the rules), sets 2,306 (0.4%; 36.6% without the
rules and the tagger), tagger 186,594 (29.5%), closed 179,009 (28.3%),
several lexemes 12,868 (2.0%; 6.2% without), verbatim 5,430 (0.9%),
apparatus 1,150; zero mismatches. `census verbatim`: 5,408 leaves —
2,147 found by key but printed otherwise (the clitic after a host that
is several lexemes: ты́ 222, ѻ҆ни́ 121; head ѧ҆зы́къ 161), 1,117 under a
titlo with no row (бл҃ 120 for the -благо- compounds the lexicon lacks,
і҆и҃лтѧнинъ 95, the Psalter's verse numerals 380), 2,144 with no reading
(1,265 surfaces: дво́е/ѻ҆боѧ̀, ѡ҆б̾ѧ́тъ's paerok inside a prefixed verb,
names, compounds); 135 several-lexeme verbs with one finite reading
whose other reading is a genitive, an accusative, a closed word or a
participle (no rule may eliminate those). Hand overlay: 337 verses,
3,757 leaves; the rules exclude no hand cell and resolve 466 (np-agree
179/179, prep-gov 142/145, subj-verb 31/31, voc-drop 50/69, one-subject
11/11, bare-loc 24/26 alone and 120 with the others, bare-voc 3/3); the
tagger 74.6% (1,200 of 1,609; errors: nominative/accusative 114,
another feature 73, several features 56, gender 52,
genitive/accusative 46, number 23, another pos 19, по dative/locative
12). The five-fold transfer (`tagger-transfer`): OCS + four folds of
the overlay 78.93%, OCS only 75.12%, the bundled model 74.58% — 3.8
points for 1,300 Synodal examples; the shipped model stays OCS-only.
The tables are in `CHANGELOG.md` under "3.4.0", "3.3.0", "3.1.0" and
"3.0.0" and in the README.

## Open designs, in order

The analysis of each — what it is, what the measurements say, and the
linguistically proper answer — is in `docs/OPEN-DESIGNS.md`, which also
records what 2.1 executed (present stems by derivation, syncretism by
underspecification).

1. **What 3.3 left of the verbatim residue** (`census verbatim`, the
   numbers above): the clitic after a host that is itself several
   lexemes (ты́ 222, ѻ҆ни́ 121, мы́ 58 — the unit wants one host; a
   selection or the disambiguator); head ѧ҆зы́къ (161: a stem variant the
   line format lacks, the Bible spells both); the -благо- compounds and
   і҆и҃лтѧнинъ the lexicon lacks; the collective numerals дво́е/ѻ҆боѧ̀; the
   paerok inside a prefixed verb (ѡ҆б̾ѧ́тъ 20 — a letter of the lexeme the
   importer could write from the Bible like the loanword's ї); the
   names (1,265 surfaces of a long tail — the guesser question of Part
   2.4, not measured); the plural-mark cells кѡры́сти/четверонѡ́гъ/
   ложесна̀, всѣ́мъ, and и҆̀же/ꙗ҆̀же with the varia (5 overlay leaves
   where the hand's nominative meets the tagger's accusative). Then the
   hand decision 3.1 left: господь.n / .n.2 one line or two.
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
