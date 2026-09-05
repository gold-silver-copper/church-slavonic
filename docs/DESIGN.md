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
   anywhere when the stress is final, skipping a solid enclitic's vowels
   — є҆гѡ́же, боѧ́щихсѧ; the kamora when nothing widens); the stress mark
   (oxia inside, varia on a final vowel); the print's conventions (psili
   on an initial vowel, initial `ѻ`/`є`, the monosyllable's varia); the
   `ї` rule (an unstressed non-initial `і` before a vowel or `й`). OCS
   printing drops the stress and maps the alphabet. Three flags let an
   ATTESTED print round-trip where its choice is not the rule's:
   `varia` (a varia where the rule writes an oxia: и҆̀хъ the accusative
   against и҆́хъ the genitive, ꙗ҆̀же), `kamora` (the kamora where a wide
   letter was available: своѧ̑ beside свѡѧ̀) and `mark_skip` (the
   enclitic's vowels). A class-built form leaves them unset and takes
   the rule; `from_print` sets them from the print, so every override
   and variant is stored exactly as printed and printed exactly as
   stored.

There is no fallback ladder. A lexeme is complete by construction: class
and stress answer every cell its class declares; the rare cell they get
wrong is an explicit `override` on the line. The only string a consumer
sees is stage 4's, and the analyzer reads it back to stage 1 plus a cell.

```rust
pub struct Form {
    pub letters: String, pub stress: Option<u8>, pub number_mark: bool,
    pub mark_skip: u8, pub varia: bool, pub kamora: bool,   // the print's own choices
}
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
  `pal2[:x]` (the first / second palatalisation), `iot` (iotation of the
  final consonant: люб → любл, род → рожд, пис → пиш; vacuous on a
  palatal), `ov` (-ова- → -ꙋ-, -ева- → -ю-), `jer` (the tense jer before
  j: пи → пь, ры → ръ), `nasal`, `iota`, `ext:suffix`, `cut` (the last
  letter removed); a chain applies right to left (`ext:ен:iot` iotates,
  then appends). A lexeme's `stems=base=…` replaces the strip rule and
  `stems=<n>=…` spells stem n outright (`1=льв` for ле́въ : льва̀) — for a
  verb only where the stem is suppletive (възьмати 2=въземл): the present
  stem of a verb is the class's derivation, never stored (2.1).
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
- **stress**: a paradigm of the accent inventory `stress.tsv` (3.0) or
  an inline spelling: `a` the lemma's own stem vowel everywhere (the last
  stem vowel when a stem has lost it); `a<N>` fixed on vowel N; `b` the
  ending wherever it has a vowel, else the last stem vowel (a participle
  on the stem); `c` = `S;pl=E`, `d` = `E;pl=S`; and the named finer
  types the 2.2 census showed, each a row `name spec exemplar count` —
  the plural cells that go to the ending (`a.gpl`, `a.dpl`, `a.ipl`,
  `a.gdpl`, `a.gipl`, `a.gdipl`, `c.na`), the retractions (`b.acc` рꙋка̀ :
  рꙋ́кꙋ, `b.voc` вра́же, `b.npl`, `b.gen`, `a.dat`, `a.nom`, `a.obl`), the
  adjective's short-form and comparative types (`a.short`, `a.shortn`,
  `b.shortn`, `a.comp`, `a.compn`, `a.plL`), the verb's present
  retraction (`b.pres` вожꙋ̀ : во́диши, with the first-plural imperative),
  the second plural's final syllable (`b.2pl` веселитѐ), the participle
  types (`b.part`, `b.part2`, `b.part3`), the aorist's third plural
  (`a.aor3`), the pronoun's oblique singular (`pr.obl`, `pr.kto`,
  `pr.moj`). `<name>{cell=S|E|L|F|<N>;…}` is a paradigm with exceptions —
  places: `S` stem, `E` ending, `L` last stem vowel, `F` last vowel of the
  word, `<N>` an index; keys: a cell, `sg`/`du`/`pl`, a block (`part`,
  `short.comp`), a finite tense (`pres`, `aor`, `impf`), `impv`. A solid
  enclitic's vowels never carry the stress (возда́стсѧ, блюсти́сѧ: the
  ending's count stops before it). `{…}` purely inline. `-` for OCS and
  titlo lemmas. The importer fits the inventory before it writes a list
  (`fit::stress_column`: every paradigm tried, the fewest exceptions win,
  ties to the simpler column and the inventory's order) and
  `cargo xtask refit-stress` re-fits a file whose lines another source
  made, keeping a new column only when every form prints the same; an
  exception list survives only where no paradigm fits. The mark kind
  (oxia/varia/kamora) is stage 4's; a stored print form carries its own
  choice only where it deviates.
- **stems**: `name=letters;…` — `base=` the base stem where the class's
  strip rule does not give it (a plurale tantum); `<n>=` a numbered stem
  of the class read off the attested forms (the present stem, a
  participle stem); `encl=сѧ|же|жде|ждо|либо` an enclitic the print writes
  solid after every ending, the jer before it dropped (бои́тсѧ, тогѡ́же,
  кі́йждо, кто́либо) — the class works on the lemma without it. On a
  closed-class line (2.2): `gov=<case>|…` the cases a preposition governs,
  commonest first (`Lexeme::government()`); `pros=encl|procl` the word's
  place in the accentual unit (`Lexeme::prosody()`; a word without it is
  tonic); `adv-of=<adjective id>` on an adverb line the adjective prints
  with another accent or letter. A closed lexeme's class is its
  subcategory (`prep`, `conj`, `part`, `adv`, `advpro`, `intj`, `pred`,
  `contr`), a row of `classes/closed.tsv`. An adverb an adjective prints
  exactly is not a line: it is the adjective's `adv` cell (`adv=1-о^|1-ѣ`,
  the mark printing the wide ѡ — мꙋ́дрѡ against the neuter мꙋ́дро) and
  the adjective's note carries the source's count (`adv P:12`).
- **overrides**: `cell=printform;…` — full print forms, in the print's
  typography, for cells where class + stress are wrong (a true exception)
  or where the lexeme prefers a non-primary alternative of its class (an
  alternative preference: the form is reachable through `forms` either
  way; the override makes `inflect` return it). Each is a claim the
  consistency test checks.
- **variants**: `cell=printform[×n]|printform;…` — additional attested
  forms for a cell, indexed by the analyzer, never returned by `inflect`.
  Spelling variants, source disagreements and minority stresses live
  here, on the lexeme, never as another lexeme. `×n` (3.0) is the
  source's count on the form, the analyzer's weight (`Lexeme::
  variant_weight`, `Analysis::weight`, `Reading::weight`): a reading is
  ranked by exactness, then the primary before everything, then weight,
  then the form's place in the cell — a variant attested fourteen times
  before one attested once and before the class's unattested
  alternatives.
- **src**: provenance tokens — `P:<class>` Polyakov, `A:<page>` Alypy
  (`A:p034`; the 1.x pronoun tables' `A:§47`/`A:§48`), `R:`
  ru.wiktionary, `K:` Kaikki, `U:` UD train, `W:<file>` a witnessed Bible
  line, `H:` a hand edit (with a `W:` or a note). Import never touches a
  column of an entry carrying `H:`; a Polyakov re-import keeps the
  variants and tokens the cross-checking sources added.
- **note**: free text; `headword <form>` records a source headword the
  attested citation form replaced (Polyakov's тьма̀ for the print's тма̀).

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

### Old Church Slavonic

The OCS lexicon (`lexicon/ocs/*.tsv`) lives in its own class tables
(`classes/ocs/*.tsv`), seeded from Kaikki's paradigm tables by
`scripts/kaikki-to-classes.py`: a noun or adjective class is a group of
entries with one paradigm shape (Kaikki's stem class, the nominative's
ending, the gender), its row the majority ending per cell with a second
ending as an alternative where a quarter of the group uses it; the
pronoun table is hand-written from the tables of тъ, сь, иже, къто, чьто
and the treebank's personal forms.

A verb class is a Leskien type — (infinitive type, present type) — and
its stems column is derivations: `V:IV:i` (любити: `2=iot`, the first
person on stem 2 with -ѭ, the other persons on stem 1 with -и-, the
imperfect on 2, `8=ext:ен:iot`), `V:IV:ě`, `V:IV:a` (лежати), `V:III:j`
(писати: the whole present on `2=iot`), `V:III:aje` (дѣлати), `V:III:ja`
(таꙗти), `V:III:ov` (`2=ov`), `V:III:jer` (пити: `2=jer`), `V:I:C`
(нести), `V:I:к`/`V:I:г` (рещи: `1=base` the infinitive, `2=ext:к`,
`3=pal1:ext:к` for the other persons, `4=pal2:ext:к` for the imperative),
`V:I:т`/`V:I:д`/`V:I:з` (грѧсти: the dental hidden by -сти restored on
stem 2), `V:I:ьн`/`V:I:ьм` (клѧти: `2=ext:ьн:cut`), `V:I:a` (ковати),
`V:II` (двигнѫти). Each Kaikki entry is placed by predicting its attested
first and third person singular from the derived stems; each present
cell reads its ending against the stem the type declares, so the members
that iotate and the members that do not vote the same ending. The
aorist, the imperfect and the l-participle are the type's too (2.2): the
sigmatic aorist on a vowel stem (дѣлахъ, дѣла; любихъ; клѧхъ), the -ох-
aorist on a consonant stem with the palatalised velar before the bare е
of the second and third person singular (несохъ, несе; рекохъ, рече;
грѧдохъ), class II with -нѫ- first and the root aorist as the alternative
on `13=pal1` (двигнѫхъ | двигохъ, двигнѫ | движе); the imperfect -ѣа-
after a consonant stem, -аа- after the palatalised velar and the a-types,
-ꙗа- on the iotated stem of -ити and the jer type, -ѣа- on -ѣти, -а- after
a vowel stem; the l-participle on the infinitive stem. Where Kaikki's
majority disagrees with the type (its tables are template output:
косехъ, кослъ, кльнхъ) the type's cell is the primary and Kaikki's is
counted, not kept; `cargo xtask census verb-cells --ocs` reads the
tables against the same statement in Rust. The past participles were
type-declared in 2.1. Entries no type reproduces sit in
residue classes (`V:res:<ending>`) with the stem on their line, and a
residue class is never offered to a lexeme the seeding did not place. A
guessed OCS verb therefore inflects from its infinitive alone (guessed
present cells 22.7% → 79.0%).

The spelling rule after a husher is the crate's: in OCS the iotated
vowels are written plain after ж ч ш щ ц and жд — ѭ/ѥ/ѩ/ꙗ as ѫ/е/ѧ/а,
at the ending and inside a derivation (пишѫ, пишетъ, рождѫ, хождаахъ
beside люблѭ, глаголѥтъ, гонꙗахъ) — which is what lets one class name
`2-ѭ` once; the Synodal rule (ѧ/ѣ → а after a husher) is a derivation's
only. The derivations are recension-aware (the second palatalisation of
г is ѕ in OCS, з in the print). No stress: the stress
column is `-` and the print drops it, mapping ы→ꙑ and ꙋ→оу; ѫ, ѧ, ѥ, ꙗ, ѣ
are letters of the layer. Provenance `K:<class>` Kaikki, `U:` the UD
PROIEL train split (variants on Kaikki's lexemes, new lexemes fitted to
the inventory for the rest). Kaikki's typo class — an entry whose
citation cell does not print its lemma — is quarantined as
`kaikki-nom-mismatch`.

## The analyzer

`Lexicon::analyze(surface) -> Vec<Analysis { lexeme, cell, alt, exact, print }>`.
The index is every lexeme × every cell (alternatives and variants
included) keyed by `Form::key`, built lazily on first use; a query folds
the input by the same key and ranks exact-print matches first, primaries
before variants. Ambiguity is returned, never resolved. An unknown
surface returns nothing; guessing works from lemmas, never from surfaces.

`Lexicon::readings(surface) -> Vec<Reading { lexeme, cells, exact, print }>`
groups the analyses by (lexeme, print): one lexeme and every cell whose
form prints the surface. The two shapes of "several analyses" are
different things. **Syncretism** is one reading whose `cells` has several
members — a property of the paradigm, not doubt about the word: every
masculine inanimate prints nom = acc = voc in the singular, a long
adjective's masculine and neuter share the oblique cases, an aorist's
second and third person singular coincide. **Homonymy** is several
readings — several lexemes, or several prints of one lexeme — and needs
context (`docs/OPEN-DESIGNS.md` 1b).

A syncretic reading is an underspecified cell, `cell::CellSet`: a sorted,
deduplicated set of cells of one part of speech whose `name()` factors
the shared components and writes the disjunction where they differ —
`nom|acc|voc.sg`, `long.pos.m|n.sg.gen`, `aor.2|3.sg`, `aor|impv.2|3.sg` —
and lists the cells in cell order where the set is not such a product
(`nom.pl|gen.sg|acc.pl`); `CellSet::parse(pos, text)` is the inverse. The
first cell is what a consumer renders through; every member prints the
same form.

The treebank's analyzed leaves carry the lexeme id and the cell — or the
set — spelled by the head and its features, and `:alt n` for a
non-primary form of the first cell (the index into `forms(cell)`): `(n
гадъ.n :case acc :num pl :alt 3)` renders гадѡ́въ; `(n свѣтъ.n :case
nom|acc :num sg)` is свѣ́тъ in the two cells its paradigm does not tell
apart; a product set is written as disjunctive features, any other set as
`:cell` with its name (`(n жена.n :cell nom.pl|gen.sg|acc.pl)`, `(v
сотворити.v :cell aor|impv.2|3.sg)`); `(adj мꙋдрый.a :case gen :num sg :g
m|n :series long)`, `(v рещи.v :t aor :p 2|3 :num sg)`, `(v … :form
imp|inf)`, `(lp быти.v :g m :num sg)`, `(part творити.v :t pres :voice act
:series long :case nom :num sg :g m)`, `(pn азъ.pron :p 1 :num sg :case dat
:clit yes)`, `(pn себе.pron :case dat|loc)`, `(f и.x)` a closed-class
lexeme (or `(f и҆)` by its surface where several closed lexemes print one
word). The lifter enters a leaf when the token's exact readings are one
lexeme: one cell, or the set (a titlo-written token groups the expansions
of one lexeme under one row — дх҃ъ is nom.sg|gen.pl|acc.pl of дꙋхъ, the
abbreviation having erased the accent that tells дꙋ́хъ from дꙋ̑хъ); a leaf
enters a tree only when it renders its token back byte-for-byte on its
own; a token whose readings are several lexemes stays verbatim with
`:amb n`; the treebank is rebuilt from the print every time, nothing is
carried over, and `check-treebank` asserts over every auto-lifted leaf
that it names every cell of its lexeme that prints the token. The linter
treats a disjunctive feature as satisfied when any member agrees and
never narrows a set: narrowing by agreement is disambiguation. The hand
overlay (Genesis 1) keeps fully specified leaves; `cargo xtask
narrow-hand` reports each hand cell against the lexicon's set.

Two more leaves (2.2): `(adv мꙋдрый.a [:deg comp])` an adjective's adverb;
`(pw host (f же.x.2))` a phonological word written solid and `(pwa host
(f же.x.2))` one written apart — a host (an analyzed leaf or a closed
lexeme) with the enclitics that lean on it, rendered through the fifth
stage below. The lifter reads a token with no whole reading as host +
enclitic (the enclitic stripped, the host's final oxia read as the
standalone varia or its jer restored, one lexeme), and a token with no
whole reading followed by an enclitic token as a unit written apart. The
linter and the coverage count a unit as its host; the linter reads a
preposition's case frame from the lexicon (`gov=`) and leaves a word
without one unchecked.

Two layers of homonymy sit between the lifter and the stored tree (2.3),
both outside the crate. The constraint layer (`treebank/disambiguate.rs`)
is rules over a verse's flat tree that eliminate and never select —
government (`prep-gov`, the lexicon's `gov=` frame), agreement
(`np-agree`, `subj-verb`), the vocative (`voc-drop`) — each named on the
leaf it narrowed with the set it narrowed from (`:by prep-gov :from
nom|acc|voc.sg`; a several-lexeme token reduced to one lexeme carries
`:from-lexemes n`), each leaving everything when it would leave nothing;
a rule that ever excludes a hand cell is wrong and goes. The statistical
layer (`crates/church-slavonic-tagger`) is an averaged perceptron over
the (part of speech, cell) readings, trained on the OCS treebanks' gold
morphology and transferred through a manuscript fold (never a lexeme
id); it runs only where the constraints left several readings, writes
`tagger` into `:by` and its share into `:prob`, and the coverage table
counts its leaves in their own column: a choice is not an analysis.
Both are scored against the hand overlay (`score-disambiguation`),
`CS_NO_DISAMBIGUATE=1` and `CS_NO_TAGGER=1` rebuild without them, and
neither touches the round-trip invariant or the leaf census.

## The fifth stage: the phonological word

Church Slavonic accent is lexical at the word level, but the print's
oxia against varia is decided over the accentual unit: a host with its
enclitics (же, бо, ли, ꙋ҆́бѡ) and proclitics (the prepositions, не, ни) is
one phonological word, so a stressed vowel that is final in the lexical
word but not in the unit takes the oxia — землѧ̀, but Землѧ́ же and (written
solid) землѧ́же, и҆̀хже, ѻ҆́ньже. The stage is optional and deterministic:
`Form::with_enclitic` builds the unit's form (the enclitic's letters
appended, the host's jer dropped before it in the Synodal print, the
number mark skipping the enclitic's vowels) and `Form::print_unit` prints
it; `Form::print_hosting` prints a host whose enclitic is written apart;
`church_slavonic::prosody::words` groups a token sequence into units by
the lexicon's prosody for a renderer or a generator, and second-position
placement of an enclitic is the generator's call. Everything word-level —
the number mark, the kamora, the wide letters, the monosyllabic varia —
stays where the word has it. The 2.0 `encl=` lexemes (иже, кождо, the
reflexive verbs) are this rule applied by the class at the letters
stage, and print unchanged.

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
   typography. Where a source's forms for a cell disagree in stress
   (ѻ҆́вцꙋ against ѻ҆вцꙋ̀), the print decides (3.0): the form the pinned
   Bible prints most in that cell — the treebank's one-cell leaves,
   `data/treebank-forms.tsv` — is the primary, the source's count next;
   a letter variant keeps its place, and the citation cell keeps the
   headword's choice so that lemmas and ids stay stable. The counts
   travel into the lexicon as the variants' weights. A source is compared under what it can encode
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
   nothing matches. The cross-checking sources (Alypy, ru.wiktionary,
   the witnesses) never create a lexeme: `import alypy|ruwiktionary|
   witnesses --pos <pos>` counts what the lexicon reproduces (the
   primary), reaches (an alternative or variant) and adds the rest as
   variants with the source's token; a lemma the lexicon lacks is
   quarantined.
5. Flag suspects (no stress in an accented source, a paradigm that fits
   no class, a class that does not produce the lemma) into
   `quarantine.tsv` with a reason. A headword whose attested citation
   form differs is not a suspect: the attested form is the lemma and the
   headword a note.

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
| Bible coverage | share of Bible tokens the analyzer resolves, exact readings: one reading / one lexeme in several cells / several lexemes / none |
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
crates/church-slavonic-tagger/   the statistical layer (dependency: church-slavonic); model data/models/tagger.bin
crates/church-slavonic-tools/    cargo xtask: import, eval, census, treebank, train-tagger
```

```rust
let lex = Lexicon::synodal();
let rab = lex.get("рабъ.n")?;                 // by stable id
let same = lex.find("рабъ", Pos::Noun);        // by lemma, accent-tolerant
let form = rab.inflect(NounCell::new(Case::Genitive, Number::Plural));
form.print(Recension::Synodal);                // "ра̑бъ"
for (cell, form) in rab.paradigm() { … }
lex.analyze("рабѡ́мъ");                          // [(рабъ.n, dat.pl, exact)]
lex.readings("свѣ́тъ");                          // [(свѣтъ.n, cells nom.sg, acc.sg)]
CellSet::parse(Pos::Noun, "nom|acc|voc.sg");     // the underspecified cell
lex.get("къ.x.2")?.government();                 // [Dative]
lex.get("же.x.2")?.prosody();                    // Prosody::Enclitic
Form::from_print("землѧ̀").print_unit(Recension::Synodal, &["же"]); // "землѧ́же"
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
  byte-for-byte for every verse; syncretism is recorded as the set,
  homonymy as `:amb n`; nothing is guessed.
- A stem the class can derive is never stored on a lexeme line; the
  census (`cargo xtask census stems`) is the arbiter, and removing lines
  is the measure of a derivation's success.
- A constraint eliminates and names itself; a tagger's choice carries
  `:by tagger` and is never folded into the analysed share; the Bible is
  never training material.

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
