# V4 — the public API first, then the sentence in the library, the Ponomar library, the training data measured

The prompt for the sessions after `v3.4.0` (`V3.3-PROMPT.md` done, CI
rewritten at 1f062a7). Read first, in this order: `HANDOFF-PROMPT.md`
(the numbers and the open list), `docs/DESIGN.md` 3c–3d and the treebank
section, `docs/OPEN-DESIGNS.md` 1b and 7, the "3.4.0" and "3.3.0" tables
of `CHANGELOG.md`, the last five entries of `NOTES.md`. Two releases and
a measurement: **4.0.0** after Part 0 (the public API's shape — a
breaking change, hence the major, and first, because a consumer must be
able to depend on the crate before anything is built for consumers),
**4.1.0** after Parts 1–3 (the sentence API, then the Ponomar library as
a second print with the intake wave its census dictates — the Bible's
own residue folded into that wave, since the library will re-rank it),
Part 4 after them as a measurement with no model shipped. Every part
ends with its gate measured, a CHANGELOG entry under the release in
progress, a dated NOTES entry (decisions and findings, what was rejected
and why), a commit, a push, and a green CI run (`gh run watch`) before
the next part begins. The order after Part 0 is the order of value per
hour: the sentence API before the corpus because everything after it
calls the library instead of the tools; the library before the Bible's
residue because the Bible's last 5,408 verbatim leaves are the long tail
of one book and the Menaia will show the lexicon's real gaps by count in
a register it has never seen.

## The environment, before any command

- `export RUSTC_WRAPPER=` (empty) in every shell that runs cargo here.
- The lexicon files are `include_str!`-embedded: `cargo build --release`
  before any `xtask` sees an edit.
- After any lexicon, class or titlo change: `cargo xtask fix-hand-alts`,
  then `cargo xtask build-treebank` (~70 s), then `cargo xtask census
  verbatim`; read the last line of each. `check-treebank` is the
  assertion on a rebuilt treebank, not the rebuild.
- Class tsvs are generated (`scripts/legend-adj-verb-pron.py`,
  `scripts/polyakov-legend-to-classes.py`); after the noun script run
  `cargo xtask import polyakov --pos noun --fix-marks` or 14 marks are
  lost. Never re-import the pronouns through Polyakov; `H:` lines and
  the pronouns file are the hand-editable lexicon.
- Id diffs with `LC_ALL=C sort` and `comm`; the default locale invents
  "added ids".
- The gate every part shares: `cargo test --release --workspace`,
  `cargo +stable clippy --workspace --all-targets -- -D warnings` (what
  CI runs, on the stable toolchain — nightly's clippy is not stable's;
  test code may unwrap, shipped code may not), the game
  (`~/Desktop/code/vertograd`: `cargo test --release`,
  `./scripts/headless-test.sh`; a changed string is re-pasted as a
  finding), zero mismatches, ids unchanged (or added, never moved).
- Publishing to crates.io is the user's call; the published crate is
  1.0.0, a different library. Say so at each close; do not publish.

## Part 0 — the public API's shape (release 4.0.0)

First: the crate's shape is what a consumer depends on, and every part
after this one builds on it. Breaking changes go together in one major;
the game migrates in the same part. Measure before changing (the
baseline below) so the release can say that no number moved.

0. **The baseline.** `cargo xtask build-treebank` (the 3.4.0 table: one
   cell 244,589 / 38.7%, sets 2,306, tagger 186,594, closed 179,009,
   several lexemes 12,868, verbatim 5,430, apparatus 1,150),
   `score-disambiguation` (466 resolved, 0 excluded; the tagger 1,200 of
   1,609), `census verbatim` (5,408: (a) 2,147, (b) 1,117, (c) 2,144),
   `eval`. These numbers are the invariant of Parts 0 and 1: neither
   changes a number.

1. **Typed cells beside the string parser.** Constructors for every
   cell kind (`Cell::noun(case, number)`, `Cell::adj(series, degree,
   gender, number, case)`, `Cell::finite(tense, person, number)`,
   `Cell::participle(…)`, `Cell::pron(…)`), the parser kept
   (`Cell::parse(pos, "aor.3.sg")`) and documented as the notation of
   the class tables and the treebank. `CellSet` builders the same way.
2. **Errors with names where `Option` hides the reason.** `inflect`
   returns `Result<Form, InflectError>` (the class lacks the cell, the
   lexeme lacks the stem, the cell is not this part of speech);
   `Lexicon::parse` a `LexiconError` with the line; `Cell::parse` a
   `CellError` with what it could not read; `get`/`find` stay `Option`
   (absence is not an error). One error module, `std::error::Error`
   on each, no dependency.
3. **Rustdoc examples on the ten functions people call**: `Lexicon::
   synodal`, `ocs`, `get`, `find`, `analyze`, `readings`, `guess`,
   `Lexeme::inflect`, `forms`, `Form::print` — each example an
   assertion the crate passes (`cargo test --doc` joins CI); the
   sentence's two join them in Part 1. The module docs say the four
   stages in ten lines and point at DESIGN.
4. **Size and start-up, measured then decided.** Measure first: the
   time of `Lexicon::synodal()`'s first use, the rlib's size, the share
   of each embedded file. Then: a feature `ocs` (default off) so a
   Synodal consumer carries no OCS lexicon, and a compact form of the
   Synodal lexicon built at build time from the tsv (`build.rs`, the tsv
   the source of truth, the compact form never committed) if the
   measurement says the parse is the cost. Gate: the same numbers, the
   README example, the game; start-up before and after in the
   CHANGELOG.
5. **The game migrated** in this part (`slavonic.rs`: the new error
   types, the typed cells where it used strings, the `ocs` feature off);
   its 35 tests and headless run green before the tag.
6. **Gate.** Every test and example green; `cargo doc --no-deps` with
   no warning; no number of the treebank or the evaluation changed; the
   migration written in the CHANGELOG under "4.0.0" as a list a
   consumer can follow.

## The close of 4.0.0

README: the installation section with the features, the line-format
and API sections as they now read, the sizes and start-up numbers;
CHANGELOG 4.0.0 with the migration list a consumer can follow and the
release table (every number unchanged); DESIGN for the error and cell
types; HANDOFF rewritten for 4.0.0 (the numbers, the open list);
NOTES close; version 4.0.0 in `crates/church-slavonic/Cargo.toml` and
the workspace dependency; tag `v4.0.0`; push with tags; CI green; the
game migrated and green. Publishing is the user's call; say that the published crate
is 1.0.0 and that 4.0.0 is the first version whose API is the one the
README describes.

## Part 1 — the sentence in the library (release 4.1.0)

The crate answers "what could this word be" (`analyze`, `readings`) and
not "what is this word here". The constraint layer is pure, small and
100% precise on the gold; the lifter that feeds it needs only the
lexicon, the titlo rows and the closed lexicon — all already in the
crate. Move them, as one module, without changing a number.

1. **The representation.** A module `church_slavonic::sentence` with
   `Sentence::parse(&Lexicon, &str) -> Sentence`: the verse's tokens in
   order, each a `Token` — a word with its readings (lexeme id, the cell
   set, the alternative index; `exact` as the analyzer says it), a
   function word, a phonological word (host and enclitics, solid or
   apart), a titlo-written word (the row's prefix and skeleton), a
   capitalised word, punctuation, the apparatus, a verbatim token, and a
   token that is several lexemes (the readings kept, none chosen). Then
   `Sentence::disambiguate(&mut self)` — the seven eliminations, each
   leaving on the token the rule that narrowed it and the set it
   narrowed from (`by`, `from`, `from_lexemes`) — and
   `Sentence::print(&self, Recension) -> String`, the round trip. The
   treebank's `Node` stays the tools' type for the stored s-expressions:
   decide whether `Node` becomes a thin view over `Token` (preferred:
   one type, the sexp reader and writer on the tools' side) or a
   conversion in both directions; either way the stored format and the
   hand overlay do not change by a byte, and the tools' lifter,
   disambiguator and coverage are calls into the library.
2. **What moves and what stays.** Moves: the tokenizer, `token_core`,
   `decapitalized`, `host_standalone`, the apparatus rule, the titlo
   index (build it from `titlo::rows` in the crate; `TitloIndex` is a
   library type), the closed-word list (`treebank/closed.rs`: what of it
   is data belongs in `lexicon/syn/closed.tsv`, the rest is code), the
   enclitic list, `lift_apart`, the seven rules with `narrow` and
   `reduce`, the clause boundary. Stays in the tools: the Bible, the
   books and verses, the stored treebank and its round-trip check, the
   hand overlay and every command over it, the coverage table, the
   census, `hand-draft`, `redraft-hand`, `score-disambiguation`. The
   tagger stays in `church-slavonic-tagger` and the tools' `tag.rs`
   applies it over a `Sentence` — a consumer who wants the choice
   depends on the tagger crate; the library ships facts only.
3. **The API's surface, minimal, in Part 0's shape.** `Sentence::parse`,
   `disambiguate`, `print`, `tokens()`; on a `Token`: `surface()`,
   `readings()`, `narrowed_by()`, `is_ambiguous()`; a `Reading` with
   `lexeme()`, `cells()`, `alternative()` — the typed cells and the
   named errors of Part 0, rustdoc examples on `parse` and
   `disambiguate`. No environment variable in the library
   (`CS_NO_DISAMBIGUATE` is the tools' switch: they simply do not call
   `disambiguate`). The README's example gains a sentence: `let mut s =
   Sentence::parse(syn, "И҆ ви́дѣ бг҃ъ свѣ́тъ, ꙗ҆́кѡ добро̀."); s.disambiguate();`
   and the assertions on ви́дѣ (the aorist by bare-loc), бг҃ъ (the titlo
   row, nom.sg by one-subject), свѣ́тъ (nom|acc.sg narrowed to acc by
   one-subject) — every string the crate's real output, in
   `tests/readme.rs`.
4. **Gate.** The baseline's numbers to the digit (the coverage table, the score
   on the overlay, the census), zero mismatches, `CS_NO_DISAMBIGUATE=1`
   and `CS_NO_TAGGER=1` still give their tables; the library's tests
   cover the seven rules with one verse each (the overlay's verses,
   asserted by cell and by `by`); `cargo doc` clean; the tools crate
   smaller by what moved; the game unchanged (it does not call the new
   API yet — a later phase may).

## Part 2 — the Ponomar library: a second print, the largest intake (release 4.1.0)

`https://www.ponomar.net/maktabah/index.html`: 28 books in Unicode
Church Slavonic HTML, typeset from Synodal editions of 1906–2002 in the
print's own conventions — the Apostol (Moscow 1989), the Gospel (1984),
the Irmologion (1995), the General Menaion (2002) and the twelve monthly
Menaia (1996–1997), the Octoechos (1981), the Augmented Psalter (1978),
the Sluzhebnik (1906), the Typikon (2002), the Trebnik (1906), the Lenten
and Flowery Triodia (1992), the Horologion (1991), the Philokalia (2000),
two St James liturgies (Sofia 1948, Rome 1970). The maintainer has given
the user licence to use all of them; record that in `data/corpus/
LICENSE.md` with the date and the page, and nothing else changes about
how a text is handled: fetched and pinned by a script, never edited.
These texts carry no morphology; they are a second print and the
register the lexicon has never been measured on (hymnography, rubrics),
not training material (see the standing rules: no self-training).

1. **Fetch and pin.** `scripts/fetch-ponomar.sh`: every book directory
   listed on the index (`Apostol1989`, `Evangelie1984`, `Irmologii1995`,
   `MineyaObshchaya2002`, `MineyaYanvar1996` … `MineyaDecember1997`,
   `Oktoih1981`, `AugmentedPsalter1978`, `AugmentedPsalter21993`,
   `Sluzhebnik1906`, `Tipikon`, `Trebnik1906`, `PostnayaTriod1992`,
   `TsvetnayaTriod1992`, `Chasoslov1991`, `Dobrotolyubie2000`,
   `StJamesLiturgyBulg1948`, `StJamesLiturgyROCOR1970`), every HTML page
   of each, into `data/corpus/ponomar/<book>/`, with a manifest of
   URLs and sha256 per page (`data/corpus/ponomar/MANIFEST.tsv`) so a
   re-fetch is verified, not trusted. Politely: one request at a time,
   a pause between pages. Decide with the numbers whether the pinned
   pages are committed (the Bible's JSON is not — a fetch script and a
   check; do the same unless the total is small) — say the size in the
   NOTES entry.
2. **The corpus module** (tools crate, beside `treebank/bible.rs`): a
   `Corpus` of books, each a list of sections (the HTML's headings:
   a service, a canon, an ode, a rubric) and units (a paragraph or a
   troparion: the smallest span the HTML delimits; a verse where the
   text has verses — the Psalter, the Apostol, the Gospel, which are
   the Bible's text and should be checked against it, not re-lifted
   blind). The HTML's markup that is not text — the apparatus, the
   rubrics in red, the marginal numbers, `[[…]]` notes — is classified
   once and recorded, the way the Bible's `꙾[n]` apparatus is; nothing
   is dropped silently. The same round-trip invariant: every unit
   renders back byte-for-byte from its tree.
3. **The treebank per book.** `cargo xtask build-treebank --corpus
   ponomar/<book>` (all books with `--corpus ponomar`), the same lift,
   the same rules, the same tagger column, the same coverage table per
   book and in total; the stored trees under `treebank/ponomar/<book>/`,
   gitignored like the Bible's. Report, per book: tokens, one cell,
   sets, tagger, closed, several lexemes, verbatim, apparatus. The
   Psalter, Apostol and Gospel volumes measured against the Bible's own
   text of the same passages: where the two prints differ by a letter
   or an accent, the census names it (a second print is the arbiter's
   check, not a second truth: the Bible stays the pinned print of
   record for the lexicon's letters, and a difference is recorded, not
   imported, unless the Bible never shows the form).
4. **The census by cause, per book and in total.** `census verbatim
   --corpus ponomar`: the same three buckets ((a) by key not exactly,
   (b) a titlo token with no row, (c) no reading) with the surfaces by
   count, and a fourth the Bible never needed — the rubric and the
   heading (words of the Typikon's language, abbreviations of the
   service books: `Сла́ва:`, `И҆ ны́нѣ:`, `Глаⷭ҇`, `Пѣ́снь`, the `Ѱ.` marks).
   Expect the verbatim share to start far above the Bible's 0.9%; the
   census says where, and that is the point.
5. **The export, with provenance per token.** `cargo xtask export
   --corpus ponomar` (and `--corpus bible`): one CoNLL-U-style file per
   book — the token, the lemma, the lexeme id, the cell (or the set),
   and a provenance column that says where the annotation came from:
   `lexicon` (one lexeme, one cell, no rule needed), `rule:<name>` (an
   elimination that names itself: prep-gov, np-agree, subj-verb,
   voc-drop, one-subject, bare-loc, bare-voc, joined by `+`),
   `tagger:<p>` (a choice, never a fact), `set`, `amb`, `verbatim`,
   `apparatus` — with a manifest carrying each book's coverage table.
   This is the corpus a user or a linguist takes; its value is that
   every token says how sure it is. Nothing in the export is training
   material for the tagger (the tagger's own column least of all — see
   the standing rules); the rules' column is the one exception Part 3
   measures.
6. **The intake wave the census dictates**, in the shape of 3.3 Parts 1–2
   and in this order: rows (the titlo table), letters (the print's
   conventions the Bible never showed — measure whether any typography
   rule is missing before touching the lexicon), lexemes by count
   (Polyakov first: an entry that exists and was quarantined or never
   reached; then hand lines with `H:` only where the count earns them —
   the hymnographic compounds, the saints' names through the guesser's
   measurement of Part 2.6.8), classes last (a paradigm the Menaion shows
   at ≥ 3 lines and the tables lack). The Bible's own residue is part
   of this wave, ranked by the two censuses together (the items below);
   each is one cause, one measurement before and after, one decision
   recorded. Rows before lines, lines before rules, rules before
   machinery. Gate per step: the corpus's verbatim count down by what
   the step explains, the Bible's numbers unchanged or better, recall
   unchanged, ids unchanged or added.

   1. **The clitic after a host that is several lexemes** (ты́ 222, ѻ҆ни́
      121, мы́ 58 — census (a) "marks only"). The unit wants one host and
      the lifter refuses several. Census first: which lexemes compete for
      each host shape (ты.pron nominative against ты́ти's imperative; ѻ҆нѝ
      the pronoun against what). Then, per shape, one of three: an existing
      rule eliminates the twin once the pair is lifted as a unit with a
      several-lexeme host (`lift_apart` accepting an `:amb` host, the
      disambiguator reducing it — nothing selected); a new elimination that
      names itself and is 100% on the overlay; or it stays verbatim with
      the reason. Never "the commoner lexeme wins".
   2. **Head ѧ҆зы́къ** (161 tokens, 16 surfaces, one lexeme; the Bible
      spells ѧ҆зы́къ 56 beside ꙗ҆зы́къ 129). A letter variant of the citation
      form is a stem variant the line format lacks. Decide the format in
      DESIGN before code: a `stems=letters=…` alternative on the line that
      the class inflects as a second paradigm under the same id (the
      analyzer indexes both; the importer writes it with the Bible as
      arbiter and the count), or the sixteen cells as variants with the
      count. The id never moves (`id_lookup_key` folds the head ѧ/ꙗ).
   3. **The titlo rows the table still lacks** (census (b) 1,117): бл҃ 120
      (the -благо- compounds: rows for lexemes that exist — бл҃гослове́нъ,
      бл҃говѣсти́ти, бл҃года́ть, бл҃гочести́вый, бл҃гоꙋстро́ити …; a lexeme
      Polyakov lacks is a hand line with `H:` only where the Bible count
      earns it), і҆и҃л- 95 (ізраильтѧнинъ.n exists: rows і҆и҃л/ізраил for the
      noun and the -тескїй adjective), the rest by prefix as the census
      lists them; the Psalter's verse numerals (ѻ҃, г҃ …, 380) are not words
      and stay. Every row probed with `cargo xtask titlo <surface>`.
   4. **The collective numerals** дво́е/двои́хъ/ѻ҆боѧ̀ (40): Polyakov's entry
      and class first (двой, обой as pronominal adjectives?); a class row
      from the legend if it has one, a hand line if not.
   5. **The paerok inside a prefixed verb** (ѡ҆б̾ѧ́тъ 20, пред̾и́детъ 9,
      ѡ҆б̾и́метъ 7, воз̾ѡблада́етъ 6, и҆з̾ѧдѧ́тъ 6, под̾ѧре́мника …): a letter
      of the lexeme, written by the importer from the Bible the way the
      loanword's ї is (`census verbatim --write` to a data file of
      (lemma key, surface, count); `bible_spelling` writes ꙿ at the
      prefix boundary the Bible prints it at; vetoed where the lifted
      prints write none). Ids fold ꙿ already (`comparison_key`); verify.
   6. **господь.n / господь.n.2** (гдⷭ҇ь `:amb 2` on 3,528 tokens today;
      the lines differ only in the stress column because one was fitted to
      fewer cells, nom.pl госпо́дїе on the second). The 3.1 rule of identity
      (same accent-blind lemma, pos, gender, animacy, agreeing shared
      attested cells) should merge them; find why it did not, fix the rule
      or record the exception, absorb the id into `data/twins.tsv`. Then
      the same for а҆́зъ, ва́мъ, мнѣ̀ if the census still names them.
   7. **The small ones, recorded or fixed with `H:`**: the plural-mark cells
      кѡры́сти, четверонѡ́гъ, ложесна̀ (a class cell or a variant), всѣ́мъ as
      the plural dative, и҆̀же/ꙗ҆̀же with the varia as a nominative (5 overlay
      leaves where the hand's nominative meets the tagger's accusative —
      decide by the Bible's count which cells carry the varia and give the
      lexeme those cells).
   8. **The names, measured** (Part 2.4 of V3.3, never done): over the
      capitalised once-only surfaces of census (c), how many would the
      guesser (`Lexicon::guess`) print correctly from the surface's ending
      against the class it picks; then decide — guessed leaves as their own
      fate (`:by guess`, never counted as analysed, like the tagger's
      column) or nothing. A number in NOTES either way.

7. **The overlay by register, begun — by the session, by hand.** One
   service from the Octoechos (a Sunday vespers of one tone) and one
   from a Menaion, drafted with `hand-draft` and decided in a decisions
   file, exactly as the Bible's chapters were: the executor of this
   prompt is the hand. The check is the session's own, and it is made
   trustworthy the way the Bible overlay was — every leaf renders its
   token back, `narrow-hand` 0, the decision recorded with its reason
   wherever the cell is not the only one the grammar allows, the
   several-lexeme tokens decided by the lemma and not by frequency,
   and a second pass over a sample of the chapter in a later session
   before it is called gold. Then `score-disambiguation --corpus`: the
   rules' precision in hymnography as a number. Small — a few hundred
   leaves — and measured, not grown for the tagger.
8. **Gate.** Every book pinned and verified by hash; every unit
   round-trips; the coverage table per book in the CHANGELOG; both
   censuses recorded before and after the intake, per item; the
   export written and its manifest's numbers the coverage table's; the
   Bible's verbatim share below 0.9% and its several-lexeme count not
   up; the overlay re-drafted (`redraft-hand`), `narrow-hand` 0; the overlay's rules still excluding none; the
   game green; the time of a full corpus build recorded (the Bible
   rebuilds in 70 s; the corpus is several times it — say how many).

## The close of 4.1.0

README: a 4.1 column in the coverage table, the sentence example, the
sizes, a second coverage table for the Ponomar corpus by book, the
export's format and the licence line; the stages paragraph unchanged unless Part 2.6.2 chose a format
(then the line-format section). CHANGELOG 4.1.0 with the release table;
DESIGN: the sentence module (what moved, the token, the rules as the
library's), Part 2's decisions; OPEN-DESIGNS 7 amended; HANDOFF
rewritten for 4.1.0 (the numbers, the open list); NOTES close; version
4.1.0 in `crates/church-slavonic/Cargo.toml` and the workspace
dependency; tag `v4.1.0`; push with tags; CI green; the game green.

## Part 3 — the training data, measured (no release, no model shipped)

The five-fold measurement (3.4 Part 4) settled that overlay gold buys
about 3 points per 1,300 examples and that the tagger's errors are
syntax. One candidate for data that is neither the test nor a guess;
its number goes in `docs/OPEN-DESIGNS.md` 1b, and only a number above
the bar becomes a wave. The harness is
`cargo xtask tagger-transfer` (the folds by chapter, the OCS material
as the base).

1. **The rules, distilled (free).** The constraint layer resolves one
   token in nine of the Bible (one cell 244,589 with the rules against
   175,759 without) and, after Part 2, its share of the Ponomar library;
   each such token had several candidates and got one answer from an
   elimination verified at 100% precision on the gold — supervision
   from the rules, not from the model. Train on the OCS material plus
   the rule-resolved tokens of every Bible chapter and every library
   book outside the overlay; score on the overlay's folds against the
   OCS-only training and the bundled model. The bias is known and must
   be reported: the examples cover only the contexts where a rule
   fires, so the question the number answers is whether that
   generalises to the contexts the rules do not reach. State the bar
   before running (the 3.4 numbers are 74.6% bundled, 78.9% with four
   folds of gold).
2. **What is not a source**, written down once: the tagger's own column
   over any text (self-training — a selection dressed as data);
   sentences generated from the lexicon by templates (they encode our
   own assumptions, and anything a template knows a rule could state
   directly); the Bible's overlay chapters (the test).
3. **The note.** OPEN-DESIGNS 1b rewritten as the decision the number
   supports: the road chosen, the number it rests on, the first
   step — or the honest sentence that it does not clear the bar and
   that gold in the register (Part 2.7, drafted by `hand-draft` and
   decided by the session, a sample re-read in a later session) is the
   remaining road.

## Standing rules

- Every rule eliminates and names itself on the leaf, or it goes; a
  rule that excludes a hand cell goes the first time.
- The gold is the arbiter — the hand overlay for the rules and the
  tagger, the pinned Bible for the print, the held-out corpus for
  recall; never the lexicon's self-consistency, never a self-count.
- Ids never move; they are added. The game persists them.
- The game is re-tested at every release; a changed string is a finding
  to re-paste, not a bug to hide.
- Numbers per part in the CHANGELOG, before and after; decisions and
  findings in NOTES with what was rejected and why.
- Census first: measure the residue, fix by cause, measure again. The
  shape of every wave since 2.0; keep it.
- A class tsv is generated, never edited; a form the class produces is
  never stored; a stem the class derives is never stored.
- Do not put morphology in the typography stage; do not put a source's
  transcription in the crate; the importer with the Bible as arbiter
  writes what is a letter of the lexeme.
- Do not select. A tagger's choice is a column of its own; the library
  ships facts. No self-training: a text without morphology is a print
  to measure against, never training material through the tagger's own
  output.
- Do not grow the overlay expecting the tagger to follow; the
  measurement says it will not much. Grow it for the rules' gate, in
  the register the corpus adds; the session is the hand that decides
  and the hand that re-reads a sample later, and both are recorded.
- A corpus the pipeline annotates is a deliverable only with the
  provenance per token; the rules' column is a fact, the tagger's a
  choice, and the export says which.
- Do not reformat the tree for rustfmt; do not run `check-treebank` on
  a stale treebank and call it a bug; do not publish without the
  user's word.

## Execution postscript (2026-09-06)

Part 0 executed and closed as 4.0.0 (tag `v4.0.0`): the typed cells, the
named errors, twelve rustdoc examples, the `ocs` feature, the game
migrated; start-up measured (the parse 0.1 s, the index 17.7 → 11.7 s by
a round-robin split; no compact file, the parse is not the cost). One
departure: the sentence's rustdoc examples wait for Part 1, where the
sentence is. Parts 1–3 remain.

Part 1 (2026-09-06): executed — `church_slavonic::sentence` (the tree, the
lifter, the rules, the closed table moved whole; one type; the tools call
the library); no number moved. Parts 2–3 remain.

Part 2 (2026-09-06): executed and closed with Part 1 as 4.1.0 (tag
`v4.1.0`): the library fetched and pinned (930 pages, 28 books), the
corpus module and `--corpus`, the export with provenance, the census by
cause over the whole library, the intake wave it ranked (the importer
expands titlo-written forms — 578 Polyakov entries had no other spelling;
rows 135 → 559; 481 lexemes; Polyakov's у as the print's ꙋ; the ї vote
with the library's evidence; the closed word's citation cell exempt from
the vote; ids never moved), the overlay by register begun by the session
(the Octoechos's Gospel stichera, the April Menaion's St Artemon) and two
rule amendments it exposed (np-agree between two agreeing nouns,
one-subject and the apposed noun). The library's verbatim 5.6% → 2.2%;
the Bible's 0.9% → 0.8%. Departures, recorded in NOTES: the Psalter,
Apostol and Gospel volumes were not measured against the Bible's text;
the rubric bucket was not built; the intake items 2 and 4–8 were not
taken and stay on the HANDOFF's open list. Part 3 follows as a
measurement.
