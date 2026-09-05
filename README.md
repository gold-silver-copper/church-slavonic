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
- **stress** — `a` the lemma's stem vowel everywhere, `b` the ending, the
  named paradigms of `lexicon/stress.tsv` (`c` = `S;pl=E`, `d` = `E;pl=S`),
  `<name>{cell=S|E|L|<n>;…}` with `sg/du/pl` and block names as keys. `-`
  for OCS.
- **stems** — `base=`, numbered stems the class cannot derive (a verb's
  present stem only where it is suppletive: `2=въземл` for възьмати;
  regular presents — люблѭ, пишѫ, рекѫ/речеши — are the class's
  derivations), `encl=сѧ|же|либо` an enclitic written solid after every
  ending.
- **overrides** / **variants** — print forms the class and stress do not
  produce: the override is what `inflect` returns, a variant is reachable
  through `forms` and the analyzer.
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
| Synodal nouns / adjectives / verbs / pronouns / closed | 13,205 / 8,344 / 8,284 / 68 / 2,503 | 49 / 16 / 50 / 21 |
| OCS nouns / adjectives / verbs / pronouns | 3,493 / 1,527 / 2,455 / 82 | 44 / 6 / 27 / 17 |

The Synodal analyzer index holds 8.2 million entries and builds in about
16 seconds (release, on first use).

## Evaluation

`cargo xtask eval` prints three numbers, each of which can go down.

**Held-out recall** — the share of annotated tokens of the UD PROIEL
dev+test splits (never an import source) whose form the lexicon produces
for the annotated lemma and cell, under the manuscript-spelling fold the
1.x harness used (so the 1.2 numbers compare):

| Part of speech | 2.1 | 2.0 | 1.2 |
|---|---|---|---|
| nouns | 95.48% (8,419/8,818) | 94.87% | 92.04% |
| adjectives | 89.31% (2,290/2,564) | 89.35% | 83.82% |
| verbs | 90.59% (7,935/8,759) | 85.79% | 85.58% |
| personal pronouns | 99.25% (3,983/4,013) | 99.25% | 99.25% |
| other pronouns | 98.07% (1,271/1,296) | 97.84% | 93.21% |

Syntacticus (which overlaps the train split): nouns 95.3%, adjectives
95.2%, verbs 94.9%, pronouns 99.2%, other pronouns 96.2%.

**Bible coverage** — every token of the Elizabethan Bible through the
Synodal analyzer (631,946 tokens; `cargo xtask check-treebank`). A token
whose exact readings are one lexeme is analysed — in one cell, or in the
cells its paradigm does not tell apart (syncretism, recorded as the set);
a token whose readings are several lexemes is homonymy, recorded and
never guessed:

| | 2.1 | 2.0 | 1.2 |
|---|---|---|---|
| analysed, one cell | 23.6% | 23.4% | 21.5% |
| analysed, one lexeme in several cells | 34.0% | — | — |
| closed-class | 28.1% | 28.1% | 27.1% |
| several lexemes (recorded `:amb n`) | 6.0% | 40.2% (with the row above) | 31.0% |
| verbatim (no reading) | 8.2% | 8.1% | 20.2% |

**Guesser accuracy** — hide each lexeme in turn, guess it from the lemma
alone, compare paradigms: Synodal nouns 93.9% of classes, 93.3% of cells;
OCS verbs, present cells only, 79.0% (22.7% in 2.0: the present stem is
the class's derivation now).

Polyakov's own cells reproduced by the primary form, for the record:
nouns 94.7%, adjectives 94.1%, verbs 91.5%; the rest are reachable as
alternatives or stored as overrides and variants, and the import report
(`cargo xtask import polyakov --pos <pos>`) lists the residue by class and
cell.

## The treebank

`cargo xtask build-treebank` lifts the whole pinned Bible into
`treebank/` (gitignored) in about 30 s: every token whose exact readings
are one lexeme becomes a leaf carrying the lexeme id and its cell or set
— `(n землѧ.n :case acc :num sg)`, `(n свѣтъ.n :case nom|acc :num sg)`,
`(v рещи.v :t aor :p 2|3 :num sg)`, `(pn азъ.pron :p 1 :num sg :case dat
:clit yes)`, `(f и.x)` — and every tree renders the verse back
byte-for-byte (`check-treebank` enforces it over all 34,470 verses, and
that every leaf names every cell of its lexeme that prints the token).
The Genesis 1 hand overlay (`data/treebank-hand/b00.sexp`) is committed,
linted, and checked against the lexicon's sets by `narrow-hand`.

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
