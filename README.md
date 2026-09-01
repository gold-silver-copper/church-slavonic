# church-slavonic

[![Crates.io](https://img.shields.io/crates/v/church-slavonic)](https://crates.io/crates/church-slavonic)
[![Docs.rs](https://docs.rs/church-slavonic/badge.svg)](https://docs.rs/church-slavonic)
![License](https://img.shields.io/crates/l/church-slavonic)

**church-slavonic** is a fast, lightweight Church Slavonic inflection library
written in Rust, covering the Old Church Slavonic and Synodal recensions of
the one language. The bundled data is about 5 MB of generated lookup-table
source (the attested exceptions of a 31,000-lexeme corpus dictionary, a
grammar, two Wiktionaries and an annotated corpus; the rules predict the
rest, accents included). It provides noun, adjective, verb, pronoun
(personal and non-personal) and declined-participle inflection from
processed Wiktionary, grammar-table, corpus-dictionary and treebank data,
making it useful for real-time procedural text generation. The verb schema
covers the finite paradigm, the imperative, the l-participle
(resultative), and the full declined participle system — present and past,
active and passive, short and long series — with per-verb participle STEMS
derived from the attested declensions, so a regular declension of an
irregular stem costs four table cells, not five hundred — and, in the same
spirit, a per-verb conjugation-class and present-stem override that re-runs
the finite rule with the verb's true class, collapsing a misclassed verb's
finite block to two cells. A fourth derived fact, the Synodal
ACCENT-PATTERN cell (`s<N>` stem-fixed or `e` ending-stressed), re-accents
the rule's answer where a row's whole paradigm shares one stress shape —
riding inside the accent pass, so the print's stress-coupled conventions
(the wide `ѡ`/`є`, the kamora, the final varia) follow the token's
position; mobile paradigms stay stored — their stress shapes were measured
too fragmented for any closed scheme vocabulary (the commonest shape
recurs on 15 rows). A fifth fact is read
from the row's own form cells: a stored lower accusative that is
nominative-shaped where the Synodal masculine rule answers the genitive
shape (an inanimate) teaches the row's higher accusative cells the same
shape (`вѣне́цъ` : `вѣнцы̀`), so an inanimate's unattested plural
accusative answers in its attested shape. The fact-resolution
order — the row's exact cell, the bare row's, the facts read
own-else-bare, the rule — lives in one place,
`church_slavonic_core::resolution`, which the runtime, the extractor and
both audits all call.

## ⚡ Speed and Accuracy

Evaluation (`cargo xtask accuracy`, against the pinned sources listed under
[Obtaining Data](#-obtaining-data--running-the-extractor)) and performance
benchmarking (`examples/speedmark.rs`, release):

| Part of Speech | Recension | Correct / Total | Accuracy | Variant gap |
|----------------|-----------|-----------------|----------|-------------|
| **Nouns** | OCS | 40171 / 40171 | 100.00% | 0 |
| **Nouns** | OCS (UD PROIEL train) | 2098 / 2098 | 100.00% | 0 |
| **Nouns** | Synodal (Alypy) | 502 / 502 | 100.00% | 0 |
| **Nouns** | Synodal (Polyakov) | 48135 / 48135 | 100.00% | 0 |
| **Nouns** | Synodal (ru.wiktionary) | 651 / 651 | 100.00% | 0 |
| **Nouns** | Synodal (witnessed print) | 3 / 3 | 100.00% | 0 |
| **Adjectives** | OCS | 38960 / 38960 | 100.00% | 0 |
| **Adjectives** | OCS (UD PROIEL train) | 502 / 502 | 100.00% | 0 |
| **Adjectives** | Synodal (Alypy) | 441 / 441 | 100.00% | 0 |
| **Adjectives** | Synodal (Polyakov) | 87997 / 87997 | 100.00% | 0 |
| **Verbs** | OCS | 236411 / 236411 | 100.00% | 0 |
| **Verbs** | OCS (UD PROIEL train) | 1802 / 1802 | 100.00% | 0 |
| **Verbs** | Synodal (Alypy) | 262 / 262 | 100.00% | 0 |
| **Verbs** | Synodal (Polyakov) | 60517 / 60517 | 100.00% | 0 |
| **Verbs** | Synodal (ru.wiktionary) | 78 / 78 | 100.00% | 0 |
| **Pronouns** | OCS | 60 / 60 | 100.00% | 0 |
| **Pronouns** | OCS (UD PROIEL train) | 67 / 67 | 100.00% | 0 |
| **Pronouns** | Synodal (Alypy) | 90 / 90 | 100.00% | 0 |
| **Pronouns** | Synodal (Polyakov) | 75 / 75 | 100.00% | 0 |
| **Non-personal pronouns** | OCS | 811 / 811 | 100.00% | 0 |
| **Non-personal pronouns** | OCS (UD PROIEL train) | 191 / 191 | 100.00% | 0 |
| **Nouns** | OCS (UD PROIEL r2.18 dev+test, corpus recall) | 8116 / 8818 | 92.04% | 702 |
| **Adjectives** | OCS (UD PROIEL r2.18 dev+test, corpus recall) | 2134 / 2546 | 83.82% | 412 |
| **Verbs** | OCS (UD PROIEL r2.18 dev+test, corpus recall) | 7413 / 8662 | 85.58% | 1249 |
| **Pronouns** | OCS (UD PROIEL r2.18 dev+test, corpus recall) | 4918 / 4960 | 99.15% | 42 |
| **Non-personal pronouns** | OCS (UD PROIEL r2.18 dev+test, corpus recall) | 1208 / 1296 | 93.21% | 88 |
| **Nouns** | OCS (Syntacticus 2023-04-28, corpus recall) | 44305 / 48825 | 90.74% | 4520 |
| **Adjectives** | OCS (Syntacticus 2023-04-28, corpus recall) | 11698 / 13901 | 84.15% | 2203 |
| **Verbs** | OCS (Syntacticus 2023-04-28, corpus recall) | 38709 / 45179 | 85.68% | 6470 |
| **Pronouns** | OCS (Syntacticus 2023-04-28, corpus recall) | 26764 / 27025 | 99.03% | 261 |
| **Non-personal pronouns** | OCS (Syntacticus 2023-04-28, corpus recall) | 17765 / 18835 | 94.32% | 1070 |

OCS (UD PROIEL r2.18 dev+test, corpus recall): 39133 tokens, 26282 slots mapped, 15243 skipped: adjective: no long lemma from the short one=17; adjective: superlative=1; ambiguous case=1; no number=791; part of speech outside the four tables=13233; pronoun: reciprocal=14; pronoun: reflexive=1069; verb: future=97; verb: subjunctive=20;

OCS (Syntacticus 2023-04-28, corpus recall): 213658 tokens, 153765 slots mapped, 75729 skipped: adjective: no long lemma from the short one=42; adjective: strength unspecified=334; adjective: superlative=1; ambiguous case=11; no number=4088; part of speech outside the four tables=63632; pronoun: reciprocal=113; pronoun: reflexive=5775; verb: future=570; verb: participle strength unspecified=13; verb: participle without a tense=1004; verb: subjunctive=145; verb: tense outside the schema=1;

Bare-lemma correctness (does the natural bare-lemma call return the primary, first-listed, attested form?):

| Part of Speech | Recension | Bare Primary / Total | Bare Accuracy | Demoted to `_n` |
|----------------|-----------|----------------------|---------------|-----------------|
| **Nouns** | OCS | 39613 / 40171 | 98.61% | 558 |
| **Nouns** | OCS (UD PROIEL train) | 1736 / 2098 | 82.75% | 362 |
| **Nouns** | Synodal (Alypy) | 418 / 502 | 83.27% | 84 |
| **Nouns** | Synodal (Polyakov) | 46415 / 48135 | 96.43% | 1720 |
| **Nouns** | Synodal (ru.wiktionary) | 545 / 651 | 83.72% | 106 |
| **Nouns** | Synodal (witnessed print) | 3 / 3 | 100.00% | 0 |
| **Adjectives** | OCS | 38959 / 38960 | 99.997% | 1 |
| **Adjectives** | OCS (UD PROIEL train) | 370 / 502 | 73.71% | 132 |
| **Adjectives** | Synodal (Alypy) | 431 / 441 | 97.73% | 10 |
| **Adjectives** | Synodal (Polyakov) | 84429 / 87997 | 95.95% | 3568 |
| **Verbs** | OCS | 218511 / 236411 | 92.43% | 17900 |
| **Verbs** | OCS (UD PROIEL train) | 1395 / 1802 | 77.41% | 407 |
| **Verbs** | Synodal (Alypy) | 232 / 262 | 88.55% | 30 |
| **Verbs** | Synodal (Polyakov) | 58752 / 60517 | 97.08% | 1765 |
| **Verbs** | Synodal (ru.wiktionary) | 73 / 78 | 93.59% | 5 |
| **Pronouns** | OCS | 60 / 60 | 100.00% | 0 |
| **Pronouns** | OCS (UD PROIEL train) | 48 / 67 | 71.64% | 19 |
| **Pronouns** | Synodal (Alypy) | 62 / 90 | 68.89% | 28 |
| **Pronouns** | Synodal (Polyakov) | 75 / 75 | 100.00% | 0 |
| **Non-personal pronouns** | OCS | 805 / 811 | 99.26% | 6 |
| **Non-personal pronouns** | OCS (UD PROIEL train) | 170 / 191 | 89.01% | 21 |

The accuracy percentages measure **recall through any published key**: the
share of attested source slots (a lemma's cell, with every form the source
lists for it) reproducible via the bare lemma **or** any `_n` sense key. Each
table source is scored on its own against the tables all of them fed. The
*variant gap* counts attested forms no key produces. The *corpus recall*
rows are different in kind: they score annotated treebank text the tables
never saw. Under the institutional grant (`references/TERMS.md`) the UD
PROIEL **train split** is itself a table source — its tokens, normalised
and gated (no titlo abbreviations, no one-letter or elided scraps or
spelling doublets, at least 3 attestations per form), feed cells like any
dictionary — so the held-out rows are the UD **dev+test splits**;
Syntacticus's texts overlap the train split, so its rows measure
manuscript-spelling robustness, not generalisation. Every
annotated token whose lemma and features name a schema cell — the finite
cells and, since the participle widening, the full declined participle
system — is scored,
lemma and surface compared through a manuscript-lax fold layered on
`orthography::comparison_key` because manuscript spelling varies (`ъі`
for `ꙑ`, `ꙙ` for `ѧ`, `шт` for `щ`, dropped or vocalised jers, `ѣ`~`ѧ`~`е`
interchange, contracted `-ааго`, editorial brackets); a surface written
under a titlo matches when its letters are an ordered subsequence of the
full form (`г҃мь` for `господьмь`), a third-person pronoun may carry
the post-prepositional `н`-, and the copula's imperfective aorist (`бѣ`,
`бѣшѧ`) is accepted under either past tense, since the treebanks and the
schema file it differently. The held-out UD dev+test files gave 39,133
tokens and 26,282 slots (15,243 skipped and counted by reason — other
parts of speech, reflexives and reciprocals, the periphrastic future and
the supine); 213,658 Syntacticus tokens gave 153,765 slots (75,729
skipped). The residual corpus-recall gap is not an
error budget to spend: forms enter the tables only when a registered
source attests them past the gates, so what remains is dev+test forms too
rare to clear the train split's frequency gate, annotation noise
(editorial lemmas like `братръ`, truncated surfaces, homograph lemmas),
and lemmas the train split never saw. The schema scope is closed as of
1.0.0 — the deferred edges and the sources examined and rejected for a
further widening are recorded in `NOTES.md`. `cargo xtask accuracy` also reports bare-lemma
correctness — whether the natural bare-lemma call returns the primary
(first-listed) attested form:

| Part of Speech | Recension | Bare Primary / Total | Bare Accuracy | Demoted to `_n` |
|----------------|-----------|----------------------|---------------|-----------------|
| **Nouns** | OCS | 39613 / 40171 | 98.61% | 558 |
| **Nouns** | OCS (UD PROIEL train) | 1736 / 2098 | 82.75% | 362 |
| **Nouns** | Synodal (Alypy) | 418 / 502 | 83.27% | 84 |
| **Nouns** | Synodal (Polyakov) | 46411 / 48135 | 96.42% | 1724 |
| **Nouns** | Synodal (ru.wiktionary) | 545 / 651 | 83.72% | 106 |
| **Adjectives** | OCS | 38959 / 38960 | 99.997% | 1 |
| **Adjectives** | OCS (UD PROIEL train) | 370 / 502 | 73.71% | 132 |
| **Adjectives** | Synodal (Alypy) | 431 / 441 | 97.73% | 10 |
| **Adjectives** | Synodal (Polyakov) | 84429 / 87997 | 95.95% | 3568 |
| **Verbs** | OCS | 218511 / 236411 | 92.43% | 17900 |
| **Verbs** | OCS (UD PROIEL train) | 1395 / 1802 | 77.41% | 407 |
| **Verbs** | Synodal (Alypy) | 232 / 262 | 88.55% | 30 |
| **Verbs** | Synodal (Polyakov) | 58759 / 60517 | 97.10% | 1758 |
| **Verbs** | Synodal (ru.wiktionary) | 73 / 78 | 93.59% | 5 |
| **Pronouns** | OCS | 60 / 60 | 100.00% | 0 |
| **Pronouns** | OCS (UD PROIEL train) | 48 / 67 | 71.64% | 19 |
| **Pronouns** | Synodal (Alypy) | 62 / 90 | 68.89% | 28 |
| **Pronouns** | Synodal (Polyakov) | 75 / 75 | 100.00% | 0 |
| **Non-personal pronouns** | OCS | 805 / 811 | 99.26% | 6 |
| **Non-personal pronouns** | OCS (UD PROIEL train) | 170 / 191 | 89.01% | 21 |

A *demotion* is a slot whose first-listed form lives at a `_n` key because the
deterministic sort put a lexicographically earlier variant on the bare key, or
because a regular paradigm was attested and reserved the bare key for the
rule (`сꙑнъ` -> `сꙑнови` by rule, `сꙑнъ_2` -> `сꙑноу`). Every attested form
stays reachable. The Synodal sources attest the same slot with a different
primary in 113 cases between Alypy and Polyakov (73 once accents, breathings
and the print's letter choices are folded) and in 85 between ru.wiktionary
and Polyakov (57 beyond those conventions); each becomes a variant row,
never adjudicated, and the sort decides which holds the bare key. A corpus
also lists rare unaccented spellings (`рабъ` next to `ра́бъ`) and the
abbreviations under a titlo are their own lemmas (`бг҃ъ`, `гл҃ати`), so the
Synodal demotion counts are mostly the sort's choice among such spellings.

Throughput on an Apple M-series laptop (`cargo run --release --example
speedmark`): 5–10 million calls per second for a table hit (nouns,
pronouns) and 50–150 thousand per second for a rule fallback on a long
out-of-vocabulary word (the fold, the realisation, the rule and the accent
placement).

## 📦 Installation

```bash
cargo add church-slavonic
```

```rust
use church_slavonic::*;

fn main() {
    // Every call names the recension. A Synodal lemma is its accented
    // citation form: the accent is the rule's input, and rule output and
    // table cells alike are spelled in the print's typography.
    assert_eq!(
        ChurchSlavonic::noun("градъ", &Case::Genitive, &Number::Singular, &Recension::OldChurchSlavonic),
        "града"
    );
    assert_eq!(
        ChurchSlavonic::noun("ра́бъ", &Case::Dative, &Number::Singular, &Recension::Synodal),
        "рабꙋ̀"
    );
    assert_eq!(
        ChurchSlavonic::noun("рꙋка̀", &Case::Genitive, &Number::Singular, &Recension::Synodal),
        "рꙋкѝ"
    );
    assert_eq!(
        ChurchSlavonic::verb("бꙑти", &Person::First, &Number::Singular, &Tense::Present, &Form::Finite, &Recension::OldChurchSlavonic),
        "ѥсмь"
    );
    // Declined participles: tense, voice, series, and agreement.
    assert_eq!(
        ChurchSlavonic::participle("нести", &Tense::Present, &Voice::Active, &Series::Short, &Case::Genitive, &Number::Singular, &Gender::Masculine, &Recension::OldChurchSlavonic),
        "несѫща"
    );
    // Sense-numbered keys expose homographs and attested variants.
    assert_eq!(
        ChurchSlavonic::noun("сꙑнъ_2", &Case::Dative, &Number::Singular, &Recension::OldChurchSlavonic),
        "сꙑноу"
    );
}
```

## 🔧 Crate Overview

### `church-slavonic`

> The public API for noun, adjective, verb, and pronoun inflection in either
> recension.

* Combines generated tables from `extractor` with rules from
  `church-slavonic-core`.
* Pure Rust; dependencies: `unicode-normalization`, and the
  first-party `church-slavonic-core`.
* Sorted static-slice tables, binary-searched, of the attested exceptions
  with regular-rule fallback;
  case restoration and recension realisation applied on output.

### `church-slavonic-syntax`

> Syntax trees that round-trip the Church Slavonic Bible.

**The invariant comes first**: for every verse that has a tree,
`render(tree)` equals the pinned print byte-for-byte — there is no other
definition of correct. Free generation of new sentences is what the
invariant earns, not the headline.

S-expression trees with ordered children (word order is recorded, never
derived); analyzed leaves inflect through this crate's public API;
features are explicit and checked by a linter, never inferred. The
escape hatch that makes the whole Bible reachable today: the `(w "…")`
verbatim leaf — every verse starts fully verbatim and round-trips by
construction, and progress is lifting leaves into analyzed nodes, which
succeeds only when the crate's output matches the attested surface
exactly. What cannot be verified stays verbatim; nothing is invented.

`cargo xtask build-treebank` auto-lifts all 77 books / 34,470 verses in
~10 s; `cargo xtask check-treebank` re-renders every stored tree against
the print (zero mismatches) and prints the coverage table. First-run
baseline over 631,946 tokens (2026-09-01):

| Slice | Tokens | Share |
|---|---|---|
| Analyzed (unambiguous crate match) | 107,837 | 17.1% |
| Closed-class (attested function words) | 171,560 | 27.1% |
| Ambiguous (recorded, never guessed) | 95,371 | 15.1% |
| Verbatim (the crate-vocabulary frontier) | 256,161 | 40.5% |
| Apparatus (variant marks, footnotes) | 1,017 | 0.2% |

Hand-lifted annotation (committed under `data/treebank-hand/`) reports
its own ceiling row — Genesis 1:1–8 stands at 76.7% lifted with zero
ambiguity, every remaining verbatim leaf carrying its reason.

### `church-slavonic-core`

> The compact rule engine: ending tables per declension/conjugation class,
> recension conditions where the two recensions genuinely differ, the
> Synodal accent rule and the orthographic realisation rules.

* Logic-only; no data dependency (only `unicode-normalization`).
* Synodal lemmas are the accented citation forms the sources print (`ра́бъ`,
  `рꙋка̀`, `свѧты́й`, `твори́ти`): a stem-stressed lemma keeps its stress, an
  ending-stressed one stresses every ending, and the print's marks — oxia,
  varia, the psili, the wide `ѡ`/`є` and the kamora that tell a plural from
  the singular it looks like — are placed by rule (`рꙋкѝ`, `свѧта́гѡ`,
  `творю̀`, `рабѡ́въ`, `рабы̑`).
* Used by the extractor to drop regular forms, preserving only irregulars.

### `extractor`

> Processes the pinned sources into the generated tables.

* Parses the Wiktionary/Kaikki Old Church Slavonic dump, the Alypy grammar
  tables, Polyakov's corpus-based grammatical dictionary (every
  corpus-attested Synodal form with its analysis and frequency; the
  frequency picks each cell's primary) and the Russian Wiktionary's 39
  structured Church Slavonic tables (Kaikki), and the UD PROIEL train
  split (normalised, titlo-free tokens attested ≥3 times; the corpus
  majority is each cell's primary among the split's variants); the UD
  dev/test splits and Syntacticus are loaded for evaluation only.
* Uses `church-slavonic-core` to blank every cell the rules already predict,
  so the tables hold exactly the attested exceptions.
* Numbers homograph senses and variants **deterministically** by a pure sort
  of their emitted forms — no lockfile, no identity table, no human review.
* Generates the static tables used in `church-slavonic`.

### `xtask`

Exactly three commands: `cargo xtask refresh-data` (sources -> tables),
`cargo xtask check-registry` (source-free CI gate: keys unique and
well-formed, fixed arity, rule/table layering holds) and `cargo xtask
accuracy` (with the sources; prints the two tables above).

## Table schema

One sorted static slice per part of speech in
`crates/church-slavonic/generated/`,
keyed `"<recension>:<key>"` (`ocs:градъ`, `syn:ра́бъ_2` — the Synodal key is
the accented lemma), each row the attested `(cell, form)` pairs of a
fixed-arity row in cell order; a cell not listed is served by the bare
row (for a `_n` key) and then by the rule. Only lemmas with at least one
cell the rules do not predict get a row, and a `_n` row carries only the
cells that differ from the bare row. Generated source: nouns 921 KB (8,267
rows: 1,169 OCS, 7,098 Synodal), adjectives 1,414 KB (7,544 rows: 311 OCS,
7,233 Synodal), verbs 810 KB (5,253 rows: 526 OCS, 4,727 Synodal), pronouns
5 KB — 3.1 MB in all, against 17.9 MB before the accent rule.

| Table | Cells | Order |
|-------|-------|-------|
| nouns | 21 | `number * 7 + case` (Singular, Dual, Plural × Nom, Gen, Dat, Acc, Ins, Loc, Voc) |
| adjectives | 126 | `((degree * 3 + gender) * 3 + number) * 7 + case` (Positive, Comparative × Masc, Fem, Neut × …) |
| verbs | 38 | finite blocks Present, Imperfect, Aorist, Imperative at `block * 9 + number * 3 + person`; 36 present active participle; 37 past active participle |
| pronoun | 90 | the lemma-less personal matrix: 1st `number * 6 + case`, 2nd `18 + …`, 3rd `36 + (gender * 3 + number) * 6 + case`; keyed `personal`, its variants `personal_2`… |

The personal pronoun has no lemma, so its variants are numbered on the
constant key `personal` exactly like a lemma's: `ChurchSlavonic::pronoun`
reads the primary row and `ChurchSlavonic::pronoun_sense("personal_2", …)`
the variants — the corpus's enclitics and minority spellings, the grammar's
own nominatives (`ѻ҆́нъ` beside the corpus's anaphoric `и҆̀`). That is how
the Alypy pronoun row is at 90/90.

## 📦 Obtaining Data & Running the Extractor

| Source | Recension | Pinned artifact (sha256) | Role |
|--------|-----------|--------------------------|------|
| English Wiktionary Old Church Slavonic ([Kaikki/Wiktextract](https://kaikki.org/dictionary/Old%20Church%20Slavonic/)) `kaikki.org-dictionary-OldChurchSlavonic.jsonl` | OCS | `fb20336e716d8f29d0c53bb4cc32f35065ad973ef8b496654c72bf542f876a83` | inflection tables (unaccented) |
| Archbishop Alypy (Gamanovich), *Grammar of the Church Slavonic Language*, web edition: the 198 `.htm` pages | Synodal | `41dac82d5eb14342c3c158e86b6fc790a6b1b2f76a894d29db103a32604d51a4` (sha256 of the pages concatenated in sorted file-name order) | printed paradigm tables (accented) |
| A. E. Polyakov, *Grammatical dictionary of Church Slavonic (corpus-based)*, tagged web edition ([dic.feb-web.ru](http://dic.feb-web.ru/slavonic/dicgram/)): the 43 `.htm` pages (`flexslav.htm` legend, indexes, `1/*.htm`, `2/*.htm`) | Synodal | `6fe3c1f0094c1624493f2b4a384b1fe56201392dc0c45314e928e7bc50f61c5d` (sha256 of the pages concatenated in sorted path order) | corpus-derived paradigms with frequencies (accented); table source under the institutional grant in `references/TERMS.md` |
| Russian Wiktionary, Церковнославянский section ([Kaikki/Wiktextract](https://kaikki.org/ruwiktionary/Церковнославянский/)) `kaikki.org-dictionary-Церковнославянский.jsonl` | Synodal | `5fa83de2fc23e14ad7062e84bcb4a208006352002545df88359699274e893ec7` | the 39 entries with structured inflection tables (accented); CC BY-SA 4.0 |
| UD_Old_Church_Slavonic-PROIEL r2.18 (`UD_Old_Church_Slavonic-PROIEL-64eddf87….tar.gz`) | OCS | `579b20edb50366e66168bb4d9f74bee0ce782f8e5b282bad8ebb2d8d870bd65c` | **train split**: inflection tables from the annotated corpus (institutional grant, `references/TERMS.md`; CC BY-NC-SA 4.0 upstream); **dev/test splits**: held-out corpus recall, never a table cell |
| Syntacticus treebank data 2023-04-28 (`syntacticus-treebank-data-525cee4f….tar.gz`): the PROIEL XML texts marked `language="chu"` | OCS | `e32844093cc173edf9241868fdad7167dfb63fb7f105d146af02a645ff382fec` | evaluation (institutional grant covers it; its texts overlap the train split, so its rows measure spelling robustness, not generalisation): corpus recall, never a table cell |

1. Run `scripts/fetch-sources.sh` (downloads every pinned table source and
   verifies it against `references/SHA256SUMS`; per-source download scripts
   live alongside it in `scripts/`). The two treebank tarballs are placed
   by hand under `references/downloads/<name>/` and unpacked by the
   accuracy harness into `data/intermediate/treebanks/`; the extractor
   cannot publish them — their loader is compiled only with the `checks`
   feature the `refresh-data` build never enables, and its record type has
   no path into the table generator.
2. Run `cargo xtask refresh-data`.
3. Generated tables land in `crates/church-slavonic/generated`; the filtered
   sources in `data/intermediate` (gitignored, regenerable).
4. Review `git diff crates/church-slavonic/generated/`, run
   `cargo xtask check-registry`, then `cargo xtask accuracy`.

Raw text corpora are not table sources: this library extracts from labelled
full-form data only. Accented Synodal coverage is whatever the labelled
sources give — Alypy's few dozen exemplar paradigms, Polyakov's 31,098
corpus lexemes (200,906 analysed cells mapped, ru.wiktionary's included;
the below-gate participle declensions,
an imperfective's periphrastic future, the short adjective series of the
fleeting-vowel classes and 1,798 titlo spellings whose entry never
abbreviates the citation form are outside the schema and skipped, counted
by reason on every refresh) and ru.wiktionary's 39 tables — and the table
reports it honestly. The cells are stored in the print's canonical
typography (`ꙋ` for the corpus edition's `у`, `ѧ` for its `я`, the psili
on an initial vowel, oxia inside the word and varia on a final vowel), the
one spelling the rules also produce.

## Deterministic sense numbering

Homographs and attested variants share a lemma and are disambiguated by a
numeric suffix (`lemma_2`, `lemma_3`). The suffix is assigned by a pure sort of
the emitted forms; when a source attests the paradigm the rules already
predict, the bare key is reserved for the rule and the variants start at `_2`.
There is no lockfile and no frozen identity. Keys are deterministic but
**not immutable**: a source refresh can renumber a lemma's `_n` keys. Do not
persist them as stable IDs across refreshes.

## Disclaimer

Source data is subject to upstream change. The generated tables in
`crates/church-slavonic/generated/*_phf.rs` are the source of truth for a given
revision. The rules are a compact approximation of productive morphology,
measured by the tables above; they are not a guarantee of correct inflection
for arbitrary out-of-vocabulary words.

## 📄 License

- Code: dual licensed under MIT and Apache-2.0 © gold-silver-copper.
- Data: the `ocs:` rows derive from Wiktionary content (CC BY-SA 4.0 / GFDL);
  the `syn:` rows reproduce the inflected forms printed in the Alypy grammar,
  Polyakov's dictionary (institutional grant) and the Russian Wiktionary
  (CC BY-SA 4.0). See `crates/church-slavonic/ATTRIBUTION.md`.
