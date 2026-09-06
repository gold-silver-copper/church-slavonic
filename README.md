# church-slavonic

[![Crates.io](https://img.shields.io/crates/v/church-slavonic)](https://crates.io/crates/church-slavonic)
[![Docs.rs](https://docs.rs/church-slavonic/badge.svg)](https://docs.rs/church-slavonic)
![License](https://img.shields.io/crates/l/church-slavonic)

Church Slavonic morphology in Rust: a curated lexicon, a paradigm
generator and an analyzer for the two recensions of the one language —
Old Church Slavonic and the Synodal print. Given a lexeme and a cell it
prints the form as the print writes it, accents and typography included;
given a printed word it returns every lexeme and cell that produce it.

## The four stages

A form is built in four stages and the only string a consumer sees is the
last one. A **lexeme** is one line of the committed lexicon (id, lemma,
part of speech, gender, animacy, letter class, stress paradigm, the stems
the class cannot derive, explicit overrides, variants, provenance). The
**letters** come from the class table: per cell an ending and a stem
selector (fleeting vowel dropped, velar palatalised, present stem
iotated), plus one bit that says the print marks this cell apart from a
look-alike singular. The **stress** paradigm places the accent: on the
lemma's stem vowel, on the ending, or per cell. **Typography**
(`Form::print`) then writes the print's conventions in one pure function:
the wide ѡ/є or the kamora on a marked plural, the oxia inside a word and
the varia at its end, the psili on an initial vowel, the ї before a vowel,
the initial uk as the one letter ѹ (3.1), the izhitsa's kendema on an
unstressed ѵ read as a vowel (мѡѷсе́й, 3.3). The prefix от- is the
ligature ѿ in the lexeme's letters, written by the importer with the
Bible as arbiter (ѿпа́даетъ, but ѡ҆трѐ: о-тре), and so are the loanword's
ї before a consonant (кївѡ́тъ, вїно̀) and the paerok of an elided jer
(в̾слѣ́дъ, ѡ҆б̾). The analyzer reads a printed word
back to stage one plus a cell through an
index of every lexeme × every cell × every alternative and variant.

## Installation

```bash
cargo add church-slavonic            # the Synodal recension
cargo add church-slavonic -F ocs     # with the Old Church Slavonic lexicon too
```

```rust
use church_slavonic::*;

fn main() {
    let syn = Lexicon::synodal();
    // a lexeme by its stable id, a cell by its features or by its name
    let rab = syn.get("рабъ.n").unwrap();
    assert_eq!(rab.inflect(Cell::noun(Case::Dative, Number::Plural)).unwrap().print(Recension::Synodal), "рабѡ́мъ");
    // every form of a cell: the primary first, then the class's other
    // alternatives, then the lexeme's attested variants
    let gen_pl: Vec<String> = rab.forms(Cell::parse(Pos::Noun, "gen.pl").unwrap()).iter().map(|f| f.print(Recension::Synodal)).collect();
    assert_eq!(gen_pl, ["рабѡ́въ", "ра̑бъ"]);
    // a cell a lexeme cannot inflect is an error that says why
    assert!(matches!(rab.inflect(Cell::infinitive()), Err(InflectError::NotThisPartOfSpeech { .. })));
    // a lemma to its lexemes (accent-tolerant; homographs come together)
    let verbs = syn.find("рещѝ", Pos::Verb);
    assert_eq!(verbs[0].inflect(Cell::finite(FiniteTense::Aorist, Person::Third, Number::Singular)).unwrap().print(Recension::Synodal), "речѐ");
    // a printed word back to its readings; ambiguity is returned, never resolved
    let readings = syn.analyze("рабѡ́мъ");
    assert_eq!(readings[0].lexeme.id, "рабъ.n");
    assert_eq!(readings[0].cell.name(), "dat.pl");
    // syncretism is one reading with several cells (the underspecified cell);
    // homonymy is several readings
    let exact: Vec<_> = syn.readings("свѣ́тъ").into_iter().filter(|r| r.exact).collect();
    assert_eq!(exact.len(), 1);
    assert_eq!(exact[0].cell_set().unwrap().name(), "nom|acc.sg");
    // the other recension (the `ocs` feature)
    #[cfg(feature = "ocs")]
    {
        let rab = Lexicon::ocs().find("рабъ", Pos::Noun)[0];
        assert_eq!(rab.inflect(Cell::noun(Case::Locative, Number::Plural)).unwrap().print(Recension::OldChurchSlavonic), "рабѣхъ");
    }
    // a lemma the lexicon lacks: a provisional lexeme from its letters
    let guessed = syn.guess("кора́бль", Pos::Noun);
    assert_eq!(guessed.provenance, Provenance::Guessed);
}
```

What a word is *here* (4.1): a sentence lifted and constrained. The
seven rules eliminate readings and name themselves on the leaf; nothing
is chosen — a statistical choice is the `church-slavonic-tagger` crate's.

```rust
use church_slavonic::{Lexicon, Recension, sentence::Sentence};

let mut s = Sentence::parse(Lexicon::synodal(), "И҆ ви́дѣ бг҃ъ свѣ́тъ, ꙗ҆́кѡ добро̀.");
s.disambiguate();
let words = s.tokens();
assert_eq!(words[1].reading.as_ref().unwrap().0, "видѣти.v");          // ви́дѣ the aorist, not ви́дъ's locative
assert_eq!(words[1].narrowed_by.as_deref(), Some("bare-loc"));
assert_eq!(words[3].narrowed_by.as_deref(), Some("one-subject"));      // свѣ́тъ: nom|acc.sg → acc.sg
assert_eq!(s.print(Recension::Synodal).unwrap(), "И҆ ви́дѣ бг҃ъ свѣ́тъ, ꙗ҆́кѡ добро̀.");
```

The library has one dependency (`unicode-normalization`) and no I/O:
the lexicon is embedded and parsed on first use (about 0.1 s), and the
analyzer's index of every form (8.2 million entries) is built on the
first `analyze` (about 12 s on twelve cores; generation only, the
inflector never waits for it). Errors are named: a cell name the
grammar does not read is a `CellError`, a malformed lexicon line a
`LexiconError` with its line, a cell a lexeme cannot inflect an
`InflectError` (another part of speech, a class the table lacks, a cell
the class does not declare); absence (`get`, `find`) is an `Option`.

Cell names: nouns `case.number` (`gen.pl`); adjectives
`[short|long.]pos|comp.gender.number.case` (`long.pos.m.sg.nom`); verbs
`pres|impf|aor|fut.person.number`, `impv.person.number`, `inf`,
`lpart.gender.number`, `part.pres|past.act|pass.short|long.gender.number.case`;
pronouns `[clit.][person.][gender.][number.]case` (`3.m.sg.gen`,
`clit.1.sg.dat`, `m.sg.gen`, `dat`); closed classes `word`. Cases `nom gen
dat acc ins loc voc`, numbers `sg du pl`, genders `m f n`. An
underspecified cell (`CellSet`) writes the disjunction where its members
differ: `nom|acc|voc.sg`, `long.pos.m|n.sg.gen`, `aor.2|3.sg`.

## The lexicon

`crates/church-slavonic/lexicon/syn/*.tsv` (Synodal) and `lexicon/ocs/*.tsv`
(Old Church Slavonic), one lexeme per line, tab-separated:

```
id       lemma   pos gender anim class stress stems overrides    variants        src         note
рабъ.n   ра́бъ    n   m      anim N1t   b      -     -            gen.pl=ра̑бъ    P:N1t;A:p034 -
дати.v   да́ти    v   -      -    Vdat  a      -     aor.2.sg=дадѐ -              P:Vdat      pf; tran
иже.pron и҆́же    pron m     -    PPize a{…}   encl=же …            …               A:§48;P:PNkto -
```

- **id** — the lemma's bare letters, the part-of-speech tag (`n a v pron
  x`), `.n` for a homograph. Stable: never renumbered.
- **class** — a row of `lexicon/classes/<pos>.tsv` (Synodal, seeded from
  Polyakov's paradigm legend) or `classes/ocs/<pos>.tsv` (seeded from
  Kaikki's tables): per cell `<stem>-<ending>[^]` alternatives separated
  by `|`, `@cell` references, `<stem>~<class>` delegation of a block to an
  adjective class, `anim:`/`inan:` readings.
- **stress** — a paradigm of the accent inventory `lexicon/stress.tsv`
  (3.0): `a` the lemma's stem vowel everywhere, `b` the ending, `c`/`d`
  the number-mobile types, and the named finer types the census showed —
  the plural cells that go to the ending (`a.gpl` дꙋхѡ́въ, `a.ipl`), the
  retractions (`b.acc` вода̀ : во́дꙋ, `b.voc` вра́гъ : вра́же), the
  present's retraction (`b.pres` вожꙋ̀ : во́диши), the second plural's
  final syllable (`b.2pl` веселитѐ), the participle types (`b.part`
  влекі́й) — each with an exemplar and its count; `<name>{cell=S|E|L|F|<n>;…}`
  keeps an exception list only where no paradigm fits (places: stem,
  ending, last stem vowel, last vowel of the word, `P` the last vowel of
  the stem before the class's extension — и҆зго́нимъ, `b.pres.ppm` —, an
  index; keys: a cell, `sg/du/pl`, a block — the most specific wins —,
  a tense, `impv`). The stem place is the lemma's stressed vowel while
  the stem has it; where a derivation removed it (-ова- → -ꙋ-, the
  iotated -ати presents) the stress stays on the derived stem's last
  vowel and never enters the extension (цѣлꙋ́ющїй, пи́шꙋщїй), while a
  lemma stressed on its ending keeps the thematic index (твори́мый). 47
  named paradigms; 1,010 lines keep a list. `-` for OCS.
- **stems** — `base=`, numbered stems the class cannot derive (a verb's
  present stem only where it is suppletive: `2=въземл` for възьмати;
  regular presents — люблѭ, пишѫ, рекѫ/речеши — are the class's
  derivations, and so are the aorist, the imperfect and the
  l-participle), `encl=сѧ|же|либо` an enclitic written solid after every
  ending, `tail=на́десѧть` a stressed solid tail after every ending of a
  compound numeral's first element (первыйна́десѧть, первагѡна́десѧть:
  the one stress sits in the tail); on a closed line `gov=`, `pros=`,
  `adv-of=`.
- **overrides** / **variants** — print forms the class and stress do not
  produce: the override is what `inflect` returns, a variant is reachable
  through `forms` and the analyzer. A variant carries the source's count
  as its weight (`acc.sg=ѻ҆́вцꙋ×14`), and the analyzer ranks a reading by
  it after exactness and the primary; where a source's forms disagree in
  stress, the form the pinned Bible prints most is the primary
  (`data/treebank-forms.tsv`, `cargo xtask census forms --write`).
- **src** — provenance: `P:` Polyakov, `A:` Alypy, `R:` ru.wiktionary,
  `K:` Kaikki, `U:` UD PROIEL train, `H:` a hand edit (import never
  touches such a line).

Every attested print round-trips through `Form::from_print` and
`Form::print`; the consistency test enforces it, and every source form is
reproduced, a variant, or quarantined with a reason in
`lexicon/quarantine.tsv`.

## Sizes

| Lexicon | Lexemes | Classes |
|---|---|---|
| Synodal nouns / adjectives / verbs / pronouns / closed | 13,303 / 8,588 / 8,292 / 72 / 1,366 (4.1: the titlo-written lexemes) | 50 / 16 / 52 / 26 / 8 |
| OCS nouns / adjectives / verbs / pronouns | 3,493 / 1,527 / 2,455 / 82 | 44 / 6 / 27 / 17 |

The closed lexicon is structured (2.2): a line's class is its
subcategory, a preposition carries the cases it governs (`gov=acc|loc`),
an enclitic or proclitic its prosody (`pros=encl`); an adverb an
adjective prints (мꙋ́дрѡ, with the wide ѡ that tells it from the neuter
мꙋ́дро) is the adjective's `adv` cell, not a line, and Polyakov's adverb
is that cell's attested evidence (3.0: 887 adjectives; `b.adv` бла́гѡ ~
благі́й). The Synodal analyzer
index holds 8.2 million entries and builds in about 16 seconds (release,
on first use).

## Evaluation

`cargo xtask eval` prints three numbers, each of which can go down.

**Held-out recall** — the share of annotated tokens of the UD PROIEL
dev+test splits (never an import source) whose form the lexicon produces
for the annotated lemma and cell, under the manuscript-spelling fold the
1.x harness used (so the 1.2 numbers compare):

| Part of speech | 3.3 | 3.1 | 3.0 | 2.2 | 2.1 | 2.0 | 1.2 |
|---|---|---|---|---|---|---|---|
| nouns | 95.48% (8,419/8,818) | 95.48% | 95.48% | 95.48% | 95.48% | 94.87% | 92.04% |
| adjectives | 89.31% (2,290/2,564) | 89.31% | 89.31% | 89.31% | 89.31% | 89.35% | 83.82% |
| verbs | 90.89% (7,961/8,759) | 90.89% | 90.89% | 90.89% | 90.59% | 85.79% | 85.58% |
| personal pronouns | 99.25% (3,983/4,013) | 99.25% | 99.25% | 99.25% | 99.25% | 99.25% | 99.25% |
| other pronouns | 98.07% (1,271/1,296) | 98.07% | 98.07% | 98.07% | 98.07% | 97.84% | 93.21% |

3.3 changed no letter of Old Church Slavonic and no recall. The Synodal
print gained two rules of typography (the izhitsa's kendema, the paerok
as a letter) and the loanword's ї written where the Bible prints it,
113 titlo rows, the numerals (пѧ́ть … де́сѧть, сто̀, the -десѧть
compounds; два̀, ѻ҆́ба, трѝ, четы́ре as pronoun-class lexemes), the
pronominal adjectives' adjective endings (є҆ди́нагѡ), хотѣ́ти's composite
class, любы̀, and the pronoun clitics as the phonological word's
enclitics (прельсти́ мѧ).

3.1 changed no letter of Old Church Slavonic. In the Synodal lexicon a
source's sense entries became one lexeme per line (158 lines absorbed,
ids unchanged), the print's initial uk (ѹ) and prefix ligature (ѿ) are
written as the pinned Bible writes them, and the hand overlay grew to
337 verses.

3.0 changed no letter and no recall: it changed how every Synodal stress
column reads (an inventory of paradigms, 47 named), the primary form
where a source's stress twins disagree (the pinned Bible decides), and
the analyzer's ranking (weights). Polyakov's cells reproduced by the
primary: nouns 94.67%, adjectives 93.17% (the wide-letter forms the
arbiter made primaries are overrides the class does not spell),
verbs 91.63%.

Syntacticus (which overlaps the train split): nouns 95.3%, adjectives
95.2%, verbs 95.0%, pronouns 99.2%, other pronouns 96.2%.

**Bible coverage** — every token of the Elizabethan Bible through the
Synodal analyzer (631,946 tokens; `cargo xtask check-treebank`). A token
whose exact readings are one lexeme is analysed — in one cell, or in the
cells its paradigm does not tell apart (syncretism, recorded as the set);
a token whose readings are several lexemes is homonymy, recorded and
never guessed. Since 2.3 a constraint layer (agreement, government, the
vocative) eliminates readings and names itself on the leaf (`:by
prep-gov :from nom|acc|voc.sg`), and a statistical tagger chooses among
what the constraints leave, in its own row — a choice, never counted as
analysed (`CS_NO_TAGGER=1` rebuilds without it):

| | 4.1 | 3.4 | 3.3 | 3.2 | 3.1 | 3.0 | 2.3 | 2.2 | 2.1 | 2.0 | 1.2 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| analysed, one cell | 38.5% (243,358) | 38.7% (244,589; 27.8% without the rules) | 38.1% (240,672) | 34.0% (214,958) | 33.7% | 32.4% | 32.4% | 23.8% | 23.6% | 23.4% | 21.5% |
| analysed, one lexeme in several cells | 0.3% (2,047) | 0.4% (36.6% without the rules and the tagger) | 0.4% | 0.3% | 0.3% (26.9% without the tagger) | 0.3% (26.1%) | 0.3% (26.0%) | 34.3% | 34.0% | — | — |
| chosen by the tagger (`:by tagger :prob`) | 29.7% | 29.5% | 30.1% | 29.7% | 30.1% | 29.3% | 29.3% | — | — | — | — |
| closed-class | 28.3% | 28.3% | 28.3% | 28.2% | 28.2% | 28.0% | 28.0% | 28.0% | 28.1% | 28.1% | 27.1% |
| several lexemes (recorded `:amb n`) | 2.0% (12,645) | 2.0% (6.2% without) | 2.0% | 1.9% | 1.9% (5.5% without the tagger) | 2.1% (5.6%) | 2.1% (5.6%) | 6.0% | 6.0% | 40.2% (with the row above) | 31.0% |
| verbatim (no reading) | 0.8% (4,789) | 0.9% (5,430) | **0.9%** (5,430) | 5.7% | 5.7% (35,731) | 7.8% | 7.8% | 7.8% | 8.2% | 8.1% | 20.2% |

The verbatim share fell in 3.1 by typography alone: 11,777 tokens
began with the uk the crate wrote as two letters, 4,300 with the prefix
ligature it wrote as ѡ҆т. In 3.3 it fell from 35,731 to 5,430 by the
print's last letters (the kendema, the paerok, the loanword's ї, the
prepositions' во/ко/со as the leaf's alternative, 113 titlo rows) and
the lexicon's gaps (the numerals, the pronominal adjectives' adjective
endings, хотѣ́ти, любы̀, the closed words, the pronoun clitics as
enclitics, the apparatus counted apart); `cargo xtask census verbatim`
names what remains (5,408 leaves: 1,117 under a titlo the table lacks a
row for, 2,144 with no reading — names and a long tail of 1,265
surfaces, 2,147 found by key but printed otherwise — the clitic after
a host that is itself several lexemes, ты́ 222).

On the 3,757-leaf hand overlay (337 verses: Genesis 1–3, Exodus 1,
Leviticus 1, Proverbs 1, Isaiah 53, Matthew 1, Luke 2, John 1, Romans
1, 1 Corinthians 13; 389 of its leaves were verbatim until 3.3's
lexicon caught up and `redraft-hand` re-drafted them) the constraint
layer alone never excludes a hand cell (precision 100%; np-agree
resolves 179 of the 179 leaves it touches, prep-gov 142 of 145,
subj-verb 31 of 31, voc-drop 50 of 69, since 3.2 the clause rule
one-subject 11 of 11 — a transitive verb, one noun that can only be
nominative, every other nominative-or-accusative noun drops the
nominative — and since 3.4 bare-loc, 24 of 26 alone and 120 with the
others — a locative with no preposition to govern it is impossible, so
ви́дѣ is the aorist and not ви́дъ's locative — and bare-voc, 3 of 3; 466
leaves in all; a hand leaf that is itself a set is counted apart); the
tagger's choices are right 74.6% of the time (1,200 of 1,609; on Old
Church Slavonic, UD PROIEL dev+test, 86.9% of the tokens with several
readings against 38.9% for the analyzer's first reading).

A host and the enclitic that leans on it are one accentual unit in the
print (Землѧ́ же, и҆̀хже, and since 3.3 a pronoun's clitic: прельсти́ мѧ):
the treebank writes the unit (`(pwa …)`, `(pw …)`) and the crate accents
it (`Form::print_unit`).

**Guesser accuracy** — hide each lexeme in turn, guess it from the lemma
alone, compare paradigms: Synodal nouns 93.9% of classes, 93.3% of cells;
OCS verbs, present cells only, 79.0% (22.7% in 2.0: the present stem is
the class's derivation now).

Polyakov's own cells reproduced by the primary form, for the record:
nouns 94.7%, adjectives 93.3%, verbs 91.6% (3.0 in progress: the
primary is the Bible's where the print arbitrates); the rest are reachable as
alternatives or stored as overrides and variants, and the import report
(`cargo xtask import polyakov --pos <pos>`) lists the residue by class and
cell.

## The treebank

`cargo xtask build-treebank` lifts the whole pinned Bible (through the
library's `sentence` module since 4.1) into
`treebank/` (gitignored) in about 70 s with both layers of homonymy: every token whose exact readings
are one lexeme becomes a leaf carrying the lexeme id and its cell or set
— `(n землѧ.n :case acc :num sg)`, `(n свѣтъ.n :case nom|acc :num sg)`,
`(v рещи.v :t aor :p 2|3 :num sg)`, `(pn азъ.pron :p 1 :num sg :case dat
:clit yes)`, `(f и.x)` — and every tree renders the verse back
byte-for-byte (`check-treebank` enforces it over all 34,470 verses, and
that every leaf names every cell of its lexeme that prints the token).
The hand overlay (`data/treebank-hand/`: Genesis 1–3, Exodus 1, Proverbs
1, Matthew 1, John 1) is committed, linted, and checked against the
lexicon's sets by `narrow-hand`; `score-disambiguation` scores the
constraint layer and the tagger against it. The constraint layer
(`treebank/disambiguate.rs`) eliminates and never selects; the tagger
(`crates/church-slavonic-tagger`, model `data/models/tagger.bin`,
rebuilt by `cargo xtask train-tagger` from UD PROIEL train and
Syntacticus, never from the Bible) chooses only where the constraints
left several readings and says so on the leaf.

## A second print: the Ponomar library (4.1)

The service books of the Ponomar library — 28 books, 930 pages, the
twelve Menaia, the Octoechos, the Triodia, the Psalter, the Horologion,
the Apostol and Gospel, the Sluzhebnik and Trebnik, the Typikon, the
Dobrotolyubie — are a second print of the same orthography, licensed to
this project by the library's maintainer (`data/corpus/ponomar/LICENSE.md`).
`scripts/fetch-ponomar.sh` pins the pages (a manifest with the sha256 of
each; the pages themselves are not committed) and every treebank command
runs over a book or the whole library unchanged behind `--corpus
ponomar[/<book>]`: a book's pages are its chapters, its paragraphs the
units, and `cargo xtask --corpus ponomar export` writes one
tab-separated file per book under `export/` — chapter, unit, token, the
form as printed, the lexeme id, the cell, and the provenance of the
reading (`lexicon`, `rule:<name>`, `tagger:<p>`, `set`, `function`,
`amb`, `verbatim`, `apparatus`) — with a manifest of each book's counts.
The library's coverage (4.1):

| Book | Units | Tokens | One cell | Sets | Tagger | Closed | Several lexemes | Verbatim | Apparatus |
|---|---|---|---|---|---|---|---|---|---|
| Apostol1989 | 2,617 | 105,355 | 36,110 (34.3%) | 586 (0.6%) | 29,877 (28.4%) | 24,349 | 2,385 | 1,863 | 10,185 |
| AugmentedPsalter1978 | 4,158 | 70,312 | 27,251 (38.8%) | 474 (0.7%) | 21,459 (30.5%) | 14,215 | 2,343 | 969 | 3,601 |
| AugmentedPsalter21993 | 77 | 2,705 | 1,097 (40.6%) | 23 (0.9%) | 779 (28.8%) | 613 | 114 | 46 | 33 |
| Chasoslov1991 | 1,462 | 29,395 | 11,643 (39.6%) | 257 (0.9%) | 8,940 (30.4%) | 6,254 | 1,201 | 424 | 676 |
| Dobrotolyubie2000 | 3,220 | 314,367 | 107,122 (34.1%) | 1,319 (0.4%) | 93,306 (29.7%) | 92,017 | 4,611 | 7,621 | 8,371 |
| Evangelie1984 | 2,544 | 88,651 | 27,589 (31.1%) | 510 (0.6%) | 27,309 (30.8%) | 20,372 | 1,482 | 1,605 | 9,784 |
| Irmologii1995 | 2,615 | 44,574 | 17,608 (39.5%) | 378 (0.8%) | 13,981 (31.4%) | 8,272 | 1,975 | 661 | 1,699 |
| MineyaAprel1996 | 3,450 | 58,136 | 24,040 (41.4%) | 525 (0.9%) | 19,319 (33.2%) | 9,079 | 2,232 | 1,770 | 1,171 |
| MineyaAugust1996 | 7,655 | 146,032 | 57,645 (39.5%) | 1,368 (0.9%) | 49,353 (33.8%) | 26,058 | 4,724 | 3,776 | 3,108 |
| MineyaDecember1997 | 6,842 | 131,763 | 52,140 (39.6%) | 1,193 (0.9%) | 43,641 (33.1%) | 23,482 | 4,842 | 3,451 | 3,014 |
| MineyaFebvral1996 | 4,961 | 97,237 | 38,620 (39.7%) | 814 (0.8%) | 31,662 (32.6%) | 17,692 | 3,550 | 2,498 | 2,401 |
| MineyaIun1996 | 6,467 | 112,808 | 45,296 (40.2%) | 911 (0.8%) | 38,004 (33.7%) | 19,190 | 3,807 | 3,121 | 2,479 |
| MineyaIyul1996 | 6,362 | 123,169 | 49,947 (40.6%) | 1,067 (0.9%) | 41,389 (33.6%) | 21,048 | 4,191 | 3,183 | 2,344 |
| MineyaMart1996 | 3,963 | 81,137 | 31,581 (38.9%) | 786 (1.0%) | 26,411 (32.6%) | 14,956 | 2,975 | 2,154 | 2,274 |
| MineyaMay1996 | 5,832 | 110,061 | 44,912 (40.8%) | 901 (0.8%) | 36,779 (33.4%) | 18,143 | 3,857 | 3,249 | 2,220 |
| MineyaNovember1997 | 7,805 | 142,904 | 57,985 (40.6%) | 1,447 (1.0%) | 48,100 (33.7%) | 23,849 | 4,900 | 3,652 | 2,971 |
| MineyaObshchaya2002 | 6,495 | 103,414 | 38,213 (37.0%) | 1,060 (1.0%) | 34,406 (33.3%) | 18,769 | 3,462 | 3,182 | 4,322 |
| MineyaOctober1997 | 6,952 | 129,130 | 52,429 (40.6%) | 1,172 (0.9%) | 43,475 (33.7%) | 21,515 | 4,510 | 3,387 | 2,642 |
| MineyaSeptember1997 | 7,696 | 147,483 | 59,134 (40.1%) | 1,414 (1.0%) | 49,711 (33.7%) | 25,832 | 4,979 | 3,353 | 3,060 |
| MineyaYanvar1996 | 8,057 | 155,176 | 62,474 (40.3%) | 1,156 (0.7%) | 52,134 (33.6%) | 27,632 | 5,364 | 3,368 | 3,048 |
| Oktoih1981 | 14,030 | 253,816 | 100,186 (39.5%) | 3,076 (1.2%) | 88,267 (34.8%) | 44,225 | 9,404 | 3,466 | 5,192 |
| PostnayaTriod1992 | 9,014 | 216,539 | 81,023 (37.4%) | 1,547 (0.7%) | 71,426 (33.0%) | 44,852 | 7,313 | 3,150 | 7,228 |
| Sluzhebnik1906 | 2,605 | 45,429 | 17,533 (38.6%) | 509 (1.1%) | 13,596 (29.9%) | 11,036 | 1,553 | 661 | 541 |
| StJamesLiturgyBulg1948 | 301 | 8,638 | 2,299 (26.6%) | 10 (0.1%) | 1,513 (17.5%) | 689 | 84 | 3,978 | 65 |
| StJamesLiturgyROCOR1970 | 479 | 10,107 | 4,267 (42.2%) | 114 (1.1%) | 2,854 (28.2%) | 2,365 | 340 | 139 | 28 |
| Tipikon | 5,428 | 240,109 | 73,209 (30.5%) | 1,643 (0.7%) | 77,580 (32.3%) | 57,285 | 9,261 | 2,587 | 18,544 |
| Trebnik1906 | 4,837 | 91,715 | 35,370 (38.6%) | 1,093 (1.2%) | 27,402 (29.9%) | 19,133 | 2,290 | 1,354 | 5,073 |
| TsvetnayaTriod1992 | 6,696 | 130,500 | 49,655 (38.0%) | 812 (0.6%) | 43,048 (33.0%) | 25,857 | 4,353 | 1,542 | 5,233 |
| All | 142,620 | 3,190,662 | 1,206,378 (37.8%) | 26,165 (0.8%) | 1,035,721 (32.5%) | 638,779 (20.0%) | 102,102 (3.2%) | 70,210 (2.2%) | 111,307 |

The library builds in about ten minutes; the Bible in 95 s.

## Sources and import

| Source | Role | Terms |
|---|---|---|
| A. E. Polyakov's corpus-based grammatical dictionary | the Synodal lexicon: classes, tags, forms, counts | institutional grant (references/TERMS.md) |
| Alypy (Gamanovich), *Grammar of the Church Slavonic Language* | class exemplars, cross-check | public domain text |
| Russian Wiktionary (Church Slavonic section) | cross-check | CC BY-SA 4.0 |
| English Wiktionary via Kaikki (Old Church Slavonic) | the OCS lexicon and classes | CC BY-SA 4.0 |
| UD_Old_Church_Slavonic-PROIEL r2.18, train split | OCS attestation and variants | institutional grant; dev+test held out |
| UD PROIEL dev+test, Syntacticus | evaluation only | — |
| The Elizabethan Bible (Church Slavonic) | evaluation corpus, the treebank | public domain text |
| The Ponomar library (28 service books, ponomar.net/maktabah) | a second print: the register's treebank and export (4.1) | licence from the maintainer to this project (data/corpus/ponomar/LICENSE.md) |

The raw artifacts are pinned in `references/` (`scripts/fetch-sources.sh`,
sha256-verified) and never committed. Import is an occasional, reviewed
operation: `cargo xtask import <source> --pos <pos>` prints the report and
the diff, `--write` updates the lexicon, and the change is committed like
code. The class tables are generated by the scripts under `scripts/`;
`cargo xtask filter-ud` regenerates the UD train attestations.

## License

Code: MIT OR Apache-2.0 © gold-silver-copper. The lexicon's OCS lines
derive from Wiktionary content (CC BY-SA 4.0); the Synodal lines
reproduce the forms of Polyakov's dictionary and the Russian Wiktionary
under the terms above; see `references/TERMS.md`.
