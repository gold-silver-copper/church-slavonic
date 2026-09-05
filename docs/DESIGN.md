# church-slavonic 2.0 — design

The design record of the lexicon-first rewrite (2026-09-04). V2-PROMPT.md
is the execution plan; this file is what the code implements and what a
change must keep true. The 1.x design and the reasons it was replaced are
summarised at the end.

## The one sentence

A form is a lexeme's letters composed with a stress position and printed
in a recension's typography; the lexicon holds lexical facts, the class
tables hold paradigms, and nothing stores the difference between them.

## The four stages

Every printed form is produced by four independent stages. Each owns one
kind of knowledge, is testable alone, and never reaches into another.

1. **Lexeme** (`lexicon`). From a lexicon line, or from the guesser for a
   lemma the lexicon lacks: id, lemma, part of speech, gender, animacy,
   letter CLASS, STRESS paradigm, the stems the class cannot derive,
   explicit per-cell overrides, variants, provenance.
2. **Letters** (`paradigm`). The class table gives, per cell, an ending
   and a stem selector (fleeting vowel dropped, velar palatalised,
   iotated present stem, …). The output is the recension's canonical
   letters — no combining marks, no wide `ѡ`/`є` unless lexical — plus
   one bit, `number_mark`: the print tells this cell apart from a
   look-alike singular (Alypy §6).
3. **Stress** (`stress`). The stress paradigm gives, per cell, a vowel
   index, or "the ending" with a fallback to the last stem vowel when the
   ending has no vowel. `None` for Old Church Slavonic and titlo lemmas.
4. **Typography** (`form::Form::print`). One pure function, in order: the
   number mark (widen the last narrow `о`/`е` at or after the stress, or
   anywhere when the stress is final; the kamora when nothing widens);
   the stress mark (oxia inside, varia on a final vowel); the print's
   conventions (psili on an initial vowel, initial `ѻ`/`є`, the
   monosyllable's varia); the `ї` rule (an unstressed non-initial `і`
   before a vowel or `й`). OCS printing drops the stress and maps the
   alphabet.

There is no fallback ladder. A lexeme is complete by construction: class
and stress answer every cell its class declares; the rare cell they get
wrong is an explicit `override` on the line. The only string a consumer
sees is stage 4's, and the analyzer reads it back to stage 1 plus a cell.

```rust
pub struct Form { pub letters: String, pub stress: Option<u8>, pub number_mark: bool }
impl Form {
    pub fn print(&self, recension: Recension) -> String;  // the print
    pub fn key(&self) -> String;                            // the one equality
    pub fn from_print(printed: &str) -> Form;               // the importer's inverse
}
```

`Form::key` is the accent-blind comparison key (`orthography::
comparison_key`): the library has ONE equality. Transliteration
conventions (Polyakov's civil «я», one acute for oxia and varia, `і` for
the print's `ї`) are absorbed at import, once, by inverting `print`.

## The lexicon

Committed, human-readable, the single source of truth. `cargo xtask
import` writes to it under review, people edit it, the library embeds it
with `include_str!`. Nothing generates it.

```
crates/church-slavonic/lexicon/
  syn/{nouns,adjectives,verbs,pronouns,closed}.tsv
  ocs/{nouns,adjectives,verbs,pronouns}.tsv
  quarantine.tsv                      source entries judged noise, WITH the reason
  classes/{noun,adj,verb,pronoun}.tsv the class tables (stage 2)
  stress.tsv                          the named stress paradigms (stage 3)
```

The class tables and the stress file are tsv, not toml: the library's only
dependency stays `unicode-normalization`, and a class is one line a reader
can diff. The noun table was seeded from Polyakov's paradigm legend by
`scripts/polyakov-legend-to-classes.py` and is measured against the source
on every import (`--fix-marks` sets each cell's number mark where the
attested primaries carry it; the alternative-preference census says which
alternative is the majority and therefore the primary).

### A class line

```
class  exemplar  strip  stems  nom.sg  gen.sg  …  voc.pl
N1t    рабъ      1      1=base 1-ъ     1-а     …  @nom.pl
N1c*   отецъ     1      1=drop;2=base;3=pal1:drop  2-ъ  1-а  …
```

- `strip`: how many letters of the lemma are its ending.
- `stems`: numbered stems and how each derives from the base — `base` (the
  lemma minus `strip` letters), `drop` (the fleeting vowel dropped, `й`
  left behind after a vowel), `insert` (a vowel inserted before the last
  consonant; the lexeme's `stems=ins=…` overrides the rule), `pal1[:x]` /
  `pal2[:x]` (the first / second palatalisation), `ext:suffix`, `cut` (the
  last letter removed). A lexeme's `stems=base=…` replaces the strip rule
  and `stems=<n>=…` spells stem n outright (`1=льв` for ле́въ : льва̀).
- a cell spec: `|`-separated alternatives, primary first — `N-ending` with
  a trailing `^` for the number mark, `@cell` for the same as that cell,
  `@lemma`, each optionally `anim:`/`inan:` for one animacy only. `inflect`
  returns the first alternative the lexeme's animacy admits; `forms` returns
  them all (the analyzer's view).

Two lexicons, one per recension: the lemmas, the sources and the
citation conventions differ (OCS `градъ` unaccented, Synodal `гра́дъ`
accented). They share the feature enums, the class tables (with a
per-recension column where the endings differ) and the engine.

### The line

Tab-separated, `-` for empty, `#` comments, one lexeme per line, sorted
by id.

```
id  lemma  pos  gender  anim  class  stress  stems  overrides  variants  src  note
рабъ.n     ра́бъ    n  m  anim  N1t   b   -        -                              gen.pl=рабѡ́въ   P:N1t;A:§12   -
ѻтецъ.n    ѻ҆те́цъ  n  m  anim  N1c*  b   obl=ѻтц  voc.sg=ѻ҆́тче                   -              P:N1c*        -
сынъ.n     сы́нъ    n  m  anim  N1t   c   -        dat.sg=сы́нови;nom.pl=сы́нове   gen.pl=сынѡ́въ   P:N1t;A:§14   u-stem relics
```

- **id**: the lemma in canonical letters with marks stripped, `.` and the
  part-of-speech tag (`n`, `a`, `v`, `pron`, `x`), and `.<n>` ONLY for a
  true homograph (a different gender, class or gloss). Assigned once at
  import; re-import matches an existing entry by lemma + pos + class
  before it creates an id. Ids are never reassigned or renumbered; a
  consumer may persist them.
- **lemma**: the citation form as the recension prints it.
- **gender** `m|f|n`; **anim** `anim|inan`.
- **class**: a row of `classes/<pos>.tsv`, seeded from Polyakov's own
  paradigm codes (`N1t` ра́бъ, `N1c*` ѻ҆те́цъ, `N1k` ѻ҆́трокъ, `A1t*`,
  `V11a` …).
- **stress**: a paradigm of `stress.tsv` or an inline spelling: `a` the
  lemma's own stem vowel everywhere (the last stem vowel when a stem has
  lost it); `a<N>` fixed on vowel N; `b` the ending wherever it has a
  vowel, else the last stem vowel; the named paradigms of `stress.tsv`
  (`c` = `S;pl=E`, `d` = `E;pl=S`, as the Part 1 census found them);
  `<name>{cell=S|E|<N>;…}` a paradigm with exceptions, with `sg`/`du`/`pl`
  accepted as keys for a whole number; `{…}` purely inline. `-` for OCS
  and titlo lemmas. The mark kind (oxia/varia/kamora) is never stored —
  stage 4 decides it.
- **stems**: `name=letters;…` — stems the class cannot derive from the
  lemma (`obl=`, `pres=`, `aor=`, `pap=` …).
- **overrides**: `cell=printform;…` — full print forms, in the print's
  typography, for cells where class + stress are wrong (a true exception)
  or where the lexeme prefers a non-primary alternative of its class (an
  alternative preference: the form is reachable through `forms` either
  way; the override makes `inflect` return it). Each is a claim the
  consistency test checks.
- **variants**: `cell=printform|printform;…` — additional attested forms
  for a cell, indexed by the analyzer, never returned by `inflect`.
  Spelling variants, source disagreements and minority stresses live
  here, on the lexeme, never as another lexeme.
- **src**: provenance tokens — `P:<class>` Polyakov, `A:§<n>` Alypy,
  `R:` ru.wiktionary, `K:` Kaikki, `U:` UD train, `W:<ref>` a witnessed
  Bible line, `H:` a hand edit (with a `W:` or a note). Import never
  touches a column of an entry carrying `H:`.

### Cell names

One canonical name per cell (`church_slavonic::cell`), used by the
lexicon columns, the treebank leaves and the eval reports:

- noun: `nom|gen|dat|acc|ins|loc|voc . sg|du|pl` — `gen.pl`;
- adjective: `[short|long .] pos|comp|sup . m|f|n . sg|du|pl . case` —
  `pos.m.sg.nom`, `short.pos.f.pl.acc` (the series prefix only where the
  class has both series);
- verb: `pres|impf|aor|fut . 1|2|3 . sg|du|pl` — `aor.3.pl`; `impv.2.sg`;
  `inf`; `lpart.m.sg`; `part . pres|past . act|pass . short|long . gender
  . number . case` — `part.pres.act.short.m.sg.nom`;
- pronoun: `[clit .] [1|2|3 .] [m|f|n .] [sg|du|pl .] case` — the personal
  pronoun's `1.sg.nom`, `3.m.sg.gen`, `clit.1.sg.dat`; a non-personal
  pronoun's `m.sg.gen`; the reflexive's `dat`, `clit.acc`.

A verb class declares WHICH cells it has (бы́ти declares a future block).
There is no global arity.

## The analyzer

`Lexicon::analyze(surface) -> Vec<Analysis { lexeme, cell, exact, variant }>`.
The index is every lexeme × every cell (variants included) keyed by
`Form::key`, built lazily on first use; a query folds the input by the
same key and ranks exact-print matches first, primaries before variants.
Ambiguity is returned, never resolved. An unknown surface returns nothing;
guessing works from lemmas, never from surfaces.

## Sources and import

Import is an occasional, reviewed operation: `cargo xtask import <source>
--pos <pos>` produces a diff against the lexicon, which is committed like
any code change.

1. Parse the source into full paradigms per lexeme with its native class
   and tags.
2. Map the native class to the inventory (identity for Polyakov; the
   best-fitting table for Alypy and Kaikki, the choice recorded).
3. Invert typography on every form (`Form::from_print`), fit the stress
   paradigm, drop what class + stress reproduce, keep the rest as
   overrides (the primary by count) or variants, stored in the print's
   typography. A source is compared under what it can encode
   (`translit_equal`: Polyakov's і for the print's positional ї, я for
   ѧ/ꙗ, a spelled-out ѡт for ѿ); a print-exact source beats a
   transliterated primary in the same cell. A form tagged for several
   cells at once (`gen/acc`) attests each only weakly: it never outranks
   a form tagged for the cell alone, and any alternative satisfies it.
   The coded class competes with its fleeting-vowel and velar twins
   (Polyakov codes ѻ҆се́лъ N1t; its forms say N1t*) and the fit keeps the
   best; numbered stems are read off the attested forms when that fits
   better still.
4. Match an existing entry by lemma + pos + class; update provenance, add
   variants; never touch an `H:` entry's columns; create an id only when
   nothing matches.
5. Flag suspects (nominative ≠ lemma, no stress in an accented source, a
   paradigm that fits no class) into `quarantine.tsv` with a reason.

| Source | Role |
|---|---|
| Polyakov corpus dictionary | primary Synodal lexicon: classes, tags, forms, counts for ranking |
| Alypy grammar | class tables, exemplars, print-exact arbitration |
| Russian Wiktionary | supplementary Synodal paradigms |
| Kaikki OCS dump | primary OCS lexicon, with a quarantine for its typos |
| UD PROIEL train | OCS attestation and variants |
| Elizabethan Bible | evaluation corpus; witness lines; per-cell frequency for ranking |
| UD PROIEL dev+test, Syntacticus | held-out evaluation only |

## Evaluation

`cargo xtask eval` prints three numbers, each of which can go down:

| Number | Measures |
|---|---|
| held-out recall | share of UD dev+test (and Syntacticus) tokens whose form lexicon + engine produce for the annotated lemma and cell |
| Bible coverage | share of Bible tokens the analyzer resolves: unambiguous / ambiguous / none |
| guesser accuracy | hide each lexeme in turn, guess it from the lemma, compare paradigms |

Lexicon self-consistency is a unit test, not a metric: every override and
variant is reproduced when asked for; every source form is reproduced, a
variant, or quarantined with a reason; the count of unaccounted forms may
not grow.

## Crates and API

```
crates/church-slavonic/          the library (dependency: unicode-normalization)
  src/grammar.rs cell.rs form.rs orthography.rs lexicon.rs
  src/paradigm/{noun,adj,verb,pronoun}.rs stress.rs inflect.rs analyze.rs guess.rs
  lexicon/                       the tsv files, include_str!
crates/church-slavonic-tools/    cargo xtask: import, eval, treebank
```

```rust
let lex = Lexicon::synodal();
let rab = lex.get("рабъ.n")?;                 // by stable id
let same = lex.find("рабъ", Pos::Noun);        // by lemma, accent-tolerant
let form = rab.inflect(NounCell::new(Case::Genitive, Number::Plural));
form.print(Recension::Synodal);                // "ра̑бъ"
for (cell, form) in rab.paradigm() { … }
lex.analyze("рабѡ́мъ");                          // [(рабъ.n, dat.pl, exact)]
lex.guess("а҆дама́нтъ", Pos::Noun);              // provenance: Guessed
```

## Standing rules

- Rules and data are independent. A class-table, stress or `print`
  change never edits a lexicon line; a lexicon edit never requires a
  regeneration. Code that subtracts a source against a rule to decide
  what to store is the 1.x design and does not come back.
- No sense numbering by sort, no `_n` keys, no fact cells in form rows,
  no per-source equality relations in the library.
- Curation is allowed and recorded (`H:` with `W:` or a note).
- Gates are the eval numbers, never self-consistency.
- The Bible round-trip invariant: `render(tree)` equals the pinned print
  byte-for-byte for every verse; ambiguity is recorded, never guessed.

## What 1.x was, and why it was replaced

1.x defined its tables as the sources minus the rule engine: the
extractor ran the rule over every attested lexeme, stored the cells the
rule got wrong (plus derived "fact" cells: an accent token, a class
override, present and participle stems, an accusative shape), and the
runtime replayed the ladder own cell → bare row → facts → rule. The
extractor's `finalize` mirrored that ladder with forward and reverse
passes; `check-registry` audited it a third time. Consequences, measured
on 2026-09-04 at e2f24f9: every rule change regenerated thousands of
rows and renumbered `_n` keys; every 1.1 and 1.2 defect was a defect of
the diff machinery; the rule picked a class from the lemma's ending, so
the commonest words were the exceptions (`ра́бъ`: 19 stored cells plus
two variant rows); Polyakov's 181 paradigm classes were parsed and
discarded; and the headline accuracy was 100.00% by construction. The
lexical facts the rows were storing are what a lexicon is for.
