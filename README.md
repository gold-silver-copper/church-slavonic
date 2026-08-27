# church-slavonic

[![Crates.io](https://img.shields.io/crates/v/church-slavonic)](https://crates.io/crates/church-slavonic)
[![Docs.rs](https://docs.rs/church-slavonic/badge.svg)](https://docs.rs/church-slavonic)
![License](https://img.shields.io/crates/l/church-slavonic)

**church-slavonic** is a fast, lightweight Church Slavonic inflection library
written in Rust, covering the Old Church Slavonic and Synodal recensions of the
one language. Total bundled data is a few megabytes of generated lookup
tables. It provides noun, adjective, verb, and pronoun inflection from
processed Wiktionary and grammar-table data, making it useful for real-time
procedural text generation.

## ⚡ Speed and Accuracy

Evaluation (`cargo xtask accuracy`, against the pinned sources listed under
[Obtaining Data](#-obtaining-data--running-the-extractor)) and performance
benchmarking (`examples/speedmark.rs`, release, averaged over 10 runs):

| Part of Speech | Recension | Correct / Total | Accuracy | Throughput (calls/sec) |
|----------------|-----------|-----------------|----------|------------------------|
| **Nouns**      | OCS       | _pending_       |          |                        |
| **Nouns**      | Synodal   | _pending_       |          |                        |
| **Verbs**      | OCS       | _pending_       |          |                        |
| **Verbs**      | Synodal   | _pending_       |          |                        |
| **Adjectives** | OCS       | _pending_       |          |                        |
| **Adjectives** | Synodal   | _pending_       |          |                        |
| **Pronouns**   | OCS       | _pending_       |          |                        |
| **Pronouns**   | Synodal   | _pending_       |          |                        |

The accuracy percentages measure **recall through any published key**: the
share of attested source slots reproducible via the bare lemma **or** any
`_n` sense key. `cargo xtask accuracy` also reports bare-lemma correctness —
whether the natural bare-lemma call returns the primary attested form:

| Part of Speech | Recension | Bare Primary / Total | Bare Accuracy |
|----------------|-----------|----------------------|---------------|
| _pending_      |           |                      |               |

## 📦 Installation

```bash
cargo add church-slavonic
```

```rust
use church_slavonic::*;

fn main() {
    // Every call names the recension; realisation (jers, nasals, accents,
    // titla) is applied on output.
    assert_eq!(
        ChurchSlavonic::noun("градъ", &Case::Genitive, &Number::Singular, &Recension::OldChurchSlavonic),
        "града"
    );
    assert_eq!(
        ChurchSlavonic::noun("градъ", &Case::Genitive, &Number::Singular, &Recension::Synodal),
        "гра́да"
    );
    // Sense-numbered keys expose homographs and attested variants.
    // assert_eq!(ChurchSlavonic::noun("градъ_2", ...), ...);
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
* PHF-backed irregular lookups with regular-rule fallback; recension
  realisation applied on output.

### `church-slavonic-core`

> The compact rule engine: ending tables per declension/conjugation class,
> recension conditions where the two recensions genuinely differ, and the
> orthographic realisation rules.

* Logic-only; no data dependency.
* Used by the extractor to classify attested forms as regular or irregular.

### `extractor`

> Processes the pinned sources into the generated tables.

* Parses the Wiktionary/Kaikki Old Church Slavonic dump and the Alypy grammar
  tables (and any further labelled full-form source listed below).
* Uses `church-slavonic-core` to drop regular forms, preserving only
  irregulars.
* Numbers homograph senses **deterministically** by a pure sort of their
  emitted forms — no lockfile, no identity table, no human review.
* Generates the static PHF tables used in `church-slavonic`.

## 📦 Obtaining Data & Running the Extractor

| Source | Pinned artifact | Role |
|--------|-----------------|------|
| English Wiktionary Old Church Slavonic (Kaikki/Wiktextract) | _pending checksum_ | inflection tables |
| Archbishop Alypy, *Grammar of the Church Slavonic Language* (web edition) | _pending checksum_ | paradigm tables |

1. Download the pinned artifacts (`cargo xtask refresh-data --help` lists
   them).
2. Run `cargo xtask refresh-data --sources <dir>`.
3. Generated tables land in `crates/church-slavonic/generated`; intermediate
   artifacts in `data/intermediate` (gitignored).
4. Review `git diff crates/church-slavonic/generated/`, run
   `cargo xtask check-registry`, then `cargo xtask accuracy`.

Raw text corpora (the Elizabeth Bible) are not table sources: this library
extracts from labelled full-form data only, as `english` does.

## Deterministic sense numbering

Homographs and attested variants share a lemma and are disambiguated by a
numeric suffix (`lemma_2`, `lemma_3`). The suffix is assigned by a pure sort of
the emitted forms — no lockfile, no frozen identity. Keys are deterministic but
**not immutable**: a source refresh can renumber a lemma's `_n` keys. Do not
persist them as stable IDs across refreshes.

## Disclaimer

Source data is subject to upstream change. The generated tables in
`crates/church-slavonic/generated/*_phf.rs` are the source of truth for a given
revision.

## 📄 License

- Code: dual licensed under MIT and Apache-2.0 © gold-silver-copper.
- Data: Wiktionary content is CC BY-SA 4.0 / GFDL; see the generated tables'
  attribution headers.
