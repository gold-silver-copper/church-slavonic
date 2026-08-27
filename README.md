# church-slavonic

[![Crates.io](https://img.shields.io/crates/v/church-slavonic)](https://crates.io/crates/church-slavonic)
[![Docs.rs](https://docs.rs/church-slavonic/badge.svg)](https://docs.rs/church-slavonic)
![License](https://img.shields.io/crates/l/church-slavonic)

**church-slavonic** is a fast, lightweight Church Slavonic inflection library
written in Rust, covering the Old Church Slavonic and Synodal recensions of the
one language. Total bundled data is under a megabyte of generated lookup
tables. It provides noun, adjective, verb, and pronoun inflection from
processed Wiktionary and grammar-table data, making it useful for real-time
procedural text generation.

## ⚡ Speed and Accuracy

Evaluation (`cargo xtask accuracy`, against the pinned sources listed under
[Obtaining Data](#-obtaining-data--running-the-extractor)) and performance
benchmarking (`examples/speedmark.rs`, release):

| Part of Speech | Recension | Correct / Total | Accuracy | Variant gap |
|----------------|-----------|-----------------|----------|-------------|
| **Nouns** | OCS | 40159 / 40159 | 100.00% | 0 |
| **Nouns** | Synodal | 502 / 502 | 100.00% | 0 |
| **Adjectives** | OCS | 38952 / 38952 | 100.00% | 0 |
| **Adjectives** | Synodal | 441 / 441 | 100.00% | 0 |
| **Verbs** | OCS | 18141 / 18141 | 100.00% | 0 |
| **Verbs** | Synodal | 262 / 262 | 100.00% | 0 |
| **Pronouns** | OCS | 54 / 54 | 100.00% | 1 |
| **Pronouns** | Synodal | 90 / 90 | 100.00% | 76 |

The accuracy percentages measure **recall through any published key**: the
share of attested source slots (a lemma's cell, with every form the source
lists for it) reproducible via the bare lemma **or** any `_n` sense key. The
*variant gap* counts attested forms no key produces — for the personal pronoun
the grammar's clitic and enclitic alternatives (`мѝ`, `тѧ̀`, `и҆̀`), which the
lemma-less `pronoun` call cannot address. `cargo xtask accuracy` also reports
bare-lemma correctness — whether the natural bare-lemma call returns the
primary (first-listed) attested form:

| Part of Speech | Recension | Bare Primary / Total | Bare Accuracy | Demoted to `_n` |
|----------------|-----------|----------------------|---------------|-----------------|
| **Nouns** | OCS | 39602 / 40159 | 98.61% | 557 |
| **Nouns** | Synodal | 476 / 502 | 94.82% | 26 |
| **Adjectives** | OCS | 38952 / 38952 | 100.00% | 0 |
| **Adjectives** | Synodal | 421 / 441 | 95.46% | 20 |
| **Verbs** | OCS | 17941 / 18141 | 98.90% | 200 |
| **Verbs** | Synodal | 261 / 262 | 99.62% | 1 |
| **Pronouns** | OCS | 54 / 54 | 100.00% | 0 |
| **Pronouns** | Synodal | 90 / 90 | 100.00% | 0 |

A *demotion* is a slot whose first-listed form lives at a `_n` key because the
deterministic sort put a lexicographically earlier variant on the bare key, or
because a regular paradigm was attested and reserved the bare key for the
rule (`сꙑнъ` -> `сꙑнови` by rule, `сꙑнъ_2` -> `сꙑноу`). Every attested form
stays reachable.

Throughput on an Apple M-series laptop (`cargo run --release --example
speedmark`): about 11 million calls per second for a table hit (nouns,
pronouns) and about 250 thousand per second for a rule fallback on a long
out-of-vocabulary word (the fold, the realisation and the rule itself).

## 📦 Installation

```bash
cargo add church-slavonic
```

```rust
use church_slavonic::*;

fn main() {
    // Every call names the recension. Rule output is realised in the
    // recension's spelling; table cells are returned as attested (Synodal
    // cells keep the grammar's accents).
    assert_eq!(
        ChurchSlavonic::noun("градъ", &Case::Genitive, &Number::Singular, &Recension::OldChurchSlavonic),
        "града"
    );
    assert_eq!(
        ChurchSlavonic::noun("рабъ", &Case::Dative, &Number::Singular, &Recension::Synodal),
        "рабꙋ̀"
    );
    assert_eq!(
        ChurchSlavonic::verb("бꙑти", &Person::First, &Number::Singular, &Tense::Present, &Form::Finite, &Recension::OldChurchSlavonic),
        "ѥсмь"
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
* Pure Rust; dependencies: `phf`, `unicode-normalization`, and the
  first-party `church-slavonic-core`.
* PHF-backed lookups of the attested exceptions with regular-rule fallback;
  case restoration and recension realisation applied on output.

### `church-slavonic-core`

> The compact rule engine: ending tables per declension/conjugation class,
> recension conditions where the two recensions genuinely differ, and the
> orthographic realisation rules.

* Logic-only; no data dependency (only `unicode-normalization`).
* Used by the extractor to drop regular forms, preserving only irregulars.

### `extractor`

> Processes the pinned sources into the generated tables.

* Parses the Wiktionary/Kaikki Old Church Slavonic dump and the Alypy grammar
  tables.
* Uses `church-slavonic-core` to blank every cell the rules already predict,
  so the tables hold exactly the attested exceptions.
* Numbers homograph senses and variants **deterministically** by a pure sort
  of their emitted forms — no lockfile, no identity table, no human review.
* Generates the static PHF tables used in `church-slavonic`.

### `xtask`

Exactly three commands: `cargo xtask refresh-data` (sources -> tables),
`cargo xtask check-registry` (source-free CI gate: keys unique and
well-formed, fixed arity, rule/table layering holds) and `cargo xtask
accuracy` (with the sources; prints the two tables above).

## Table schema

One `phf` map per part of speech in `crates/church-slavonic/generated/`,
keyed `"<recension>:<key>"` (`ocs:градъ`, `syn:рабъ_2`), each row a
fixed-arity array of cells; an empty cell means "not attested — the rule
serves it". Only lemmas with at least one cell the rules do not predict get a
row.

| Table | Cells | Order |
|-------|-------|-------|
| nouns | 21 | `number * 7 + case` (Singular, Dual, Plural × Nom, Gen, Dat, Acc, Ins, Loc, Voc) |
| adjectives | 126 | `((degree * 3 + gender) * 3 + number) * 7 + case` (Positive, Comparative × Masc, Fem, Neut × …) |
| verbs | 38 | finite blocks Present, Imperfect, Aorist, Imperative at `block * 9 + number * 3 + person`; 36 present active participle; 37 past active participle |
| pronoun | 90 | one lemma-less `personal` row: 1st `number * 6 + case`, 2nd `18 + …`, 3rd `36 + (gender * 3 + number) * 6 + case` |

## 📦 Obtaining Data & Running the Extractor

| Source | Recension | Pinned artifact (sha256) | Role |
|--------|-----------|--------------------------|------|
| English Wiktionary Old Church Slavonic ([Kaikki/Wiktextract](https://kaikki.org/dictionary/Old%20Church%20Slavonic/)) `kaikki.org-dictionary-OldChurchSlavonic.jsonl` | OCS | `fb20336e716d8f29d0c53bb4cc32f35065ad973ef8b496654c72bf542f876a83` | inflection tables (unaccented) |
| Archbishop Alypy (Gamanovich), *Grammar of the Church Slavonic Language*, web edition: the 198 `.htm` pages | Synodal | `41dac82d5eb14342c3c158e86b6fc790a6b1b2f76a894d29db103a32604d51a4` (sha256 of the pages concatenated in sorted file-name order) | printed paradigm tables (accented) |

1. Place the artifacts under `references/downloads/english-wiktionary-ocs/`
   and `references/downloads/alypy-grammar/` (or pass `--sources DIR`).
2. Run `cargo xtask refresh-data`.
3. Generated tables land in `crates/church-slavonic/generated`; the filtered
   sources in `data/intermediate` (gitignored, regenerable).
4. Review `git diff crates/church-slavonic/generated/`, run
   `cargo xtask check-registry`, then `cargo xtask accuracy`.

Raw text corpora are not table sources: this library extracts from labelled
full-form data only. Accented Synodal coverage is whatever the labelled
source gives — a few dozen exemplar paradigms today — and the table reports
it honestly.

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
  the `syn:` rows reproduce the inflected forms printed in the Alypy grammar.
  See `crates/church-slavonic/ATTRIBUTION.md`.
