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
the varia at its end, the psili on an initial vowel, the ї before a vowel.
The analyzer reads a printed word back to stage one plus a cell through an
index of every lexeme × every cell × every alternative and variant.

## Installation

```bash
cargo add church-slavonic
```

```rust
use church_slavonic::*;

fn main() {
    let syn = Lexicon::synodal();
    // a lexeme by its stable id, a cell by its name
    let rab = syn.get("рабъ.n").unwrap();
    assert_eq!(rab.inflect(NounCell::new(Case::Dative, Number::Plural)).unwrap().print(Recension::Synodal), "рабѡ́мъ");
    // every form of a cell: the primary first, then the class's other
    // alternatives, then the lexeme's attested variants
    let gen_pl: Vec<String> = rab.forms(Cell::parse(Pos::Noun, "gen.pl").unwrap()).iter().map(|f| f.print(Recension::Synodal)).collect();
    assert_eq!(gen_pl, ["рабѡ́въ", "ра̑бъ"]);
    // a lemma to its lexemes (accent-tolerant; homographs come together)
    let verbs = syn.find("рещѝ", Pos::Verb);
    assert_eq!(verbs[0].inflect(Cell::parse(Pos::Verb, "aor.3.sg").unwrap()).unwrap().print(Recension::Synodal), "речѐ");
    // a printed word back to its readings; ambiguity is returned, never resolved
    let readings = syn.analyze("рабѡ́мъ");
    assert_eq!(readings[0].lexeme.id, "рабъ.n");
    assert_eq!(readings[0].cell.name(), "dat.pl");
    // syncretism is one reading with several cells (the underspecified cell);
    // homonymy is several readings
    let exact: Vec<_> = syn.readings("свѣ́тъ").into_iter().filter(|r| r.exact).collect();
    assert_eq!(exact.len(), 1);
    assert_eq!(exact[0].cell_set().unwrap().name(), "nom|acc.sg");
    // the other recension
    let ocs = Lexicon::ocs();
    let rab = ocs.find("рабъ", Pos::Noun)[0];
    assert_eq!(rab.inflect(NounCell::new(Case::Locative, Number::Plural)).unwrap().print(Recension::OldChurchSlavonic), "рабѣхъ");
    // a lemma the lexicon lacks: a provisional lexeme from its letters
    let guessed = syn.guess("кора́бль", Pos::Noun);
    assert_eq!(guessed.provenance, Provenance::Guessed);
}
```

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
| Synodal nouns / adjectives / verbs / pronouns / closed | 13,147 / 8,323 / 8,210 / 68 / 1,342 | 49 / 16 / 50 / 21 / 8 |
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

| Part of speech | 3.0 | 2.2 | 2.1 | 2.0 | 1.2 |
|---|---|---|---|---|---|
| nouns | 95.48% (8,419/8,818) | 95.48% | 95.48% | 94.87% | 92.04% |
| adjectives | 89.31% (2,290/2,564) | 89.31% | 89.31% | 89.35% | 83.82% |
| verbs | 90.89% (7,961/8,759) | 90.89% | 90.59% | 85.79% | 85.58% |
| personal pronouns | 99.25% (3,983/4,013) | 99.25% | 99.25% | 99.25% | 99.25% |
| other pronouns | 98.07% (1,271/1,296) | 98.07% | 98.07% | 97.84% | 93.21% |

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

| | 3.0 | 2.3 | 2.2 | 2.1 | 2.0 | 1.2 |
|---|---|---|---|---|---|---|
| analysed, one cell | 32.4% (204,769) | 32.4% | 23.8% | 23.6% | 23.4% | 21.5% |
| analysed, one lexeme in several cells | 0.3% (26.1% without the tagger) | 0.3% (26.0% without the tagger) | 34.3% | 34.0% | — | — |
| chosen by the tagger (`:by tagger :prob`) | 29.3% | 29.3% | — | — | — | — |
| closed-class | 28.0% | 28.0% | 28.0% | 28.1% | 28.1% | 27.1% |
| several lexemes (recorded `:amb n`) | 2.1% (5.6% without the tagger) | 2.1% (5.6% without the tagger) | 6.0% | 6.0% | 40.2% (with the row above) | 31.0% |
| verbatim (no reading) | 7.8% | 7.8% | 7.8% | 8.2% | 8.1% | 20.2% |

On the 2,097-leaf hand overlay (211 verses) the constraint layer alone
never excludes a hand cell (precision 100%, resolution 45.0%); the
tagger's choices are right 74.8% of the time (810 of 1,083; on Old
Church Slavonic, UD PROIEL dev+test, 86.9% of the tokens with several
readings against 38.9% for the analyzer's first reading).

A host and the enclitic that leans on it are one accentual unit in the
print (Землѧ́ же, и҆̀хже): the treebank writes the unit (`(pwa …)`, `(pw …)`)
and the crate accents it (`Form::print_unit`); 2,295 units in the Bible.

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

`cargo xtask build-treebank` lifts the whole pinned Bible into
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
