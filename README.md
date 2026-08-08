# Old Church Slavonic inflection for Rust

`old-church-slavonic` is a fast, offline inflector for canonical **Old Church
Slavonic** (`cu`/`chu`). It does not silently mix Russian, Serbian, Croatian, or
another later Church Slavonic recension. The public facade combines source-ordered
English-Wiktionary dictionary tables with a conservative pure-Rust rule engine.

Dictionary forms are template-generated dictionary claims, not automatically
manuscript-attested forms. Every result says whether it came from a table, a manual
override, dictionary metadata, explicit caller metadata, or an OOV prediction.

## Install

The minimum supported Rust version is 1.85.

```toml
[dependencies]
old-church-slavonic = "0.1"
```

The crate performs no runtime file access, network access, JSON parsing, or Lua
execution. Its generated dictionary is compiled into the package.

## Supported v0.1 surface

- nouns: seven cases × singular/dual/plural from dictionary tables; OOV rules for
  hard o/a, soft jo/ja, i-, u-, and n/nt/r/s/v consonant stems;
- adjectives: dictionary and OOV hard/soft, short/simple and long/compound agreement;
  dictionary-listed comparative citations;
- verbs: safely extracted present, imperfect, aorist variants, imperative,
  infinitive, supine, l-participle, citation participles, and verbal nouns; OOV
  present forms require a class and present stem, while regular supine and
  l-participle components require explicit metadata;
- dictionary-backed pronoun, determiner, and numeral cells, including person where
  a source table combines personal pronouns.

The malformed Wiktextract verb rows are not “repaired” by guessing. Rows with
`error-unrecognized-form` and declined participles that cannot be safely assigned to
a kind are excluded and counted. Productive imperfect/aorist/imperative and
participial-stem derivation remain unsupported in the core.

## Forms, ambiguity, and provenance

```rust
use old_church_slavonic::{noun, Case, NounCell, Number};

let forms = noun(
    "обѣдъ",
    NounCell { case: Case::Dative, number: Number::Dual },
)?;
assert_eq!(
    forms
        .primary_source_order()
        .expect("dictionary cell contains a variant")
        .text,
    "обѣдома"
);
assert_eq!(forms.variants.len(), 1);
# Ok::<(), old_church_slavonic::InflectionError>(())
```

`FormSet::variants` preserves source order and never joins alternatives with `/`.
`primary_source_order()` is explicitly a source-order policy, not a claim that the
first spelling is linguistically superior. Predicted results include a stable
`RuleId` and compact trace.

A bare lemma can identify multiple lexical records. Convenience calls then return
`InflectionError::AmbiguousLexeme { candidates }`; use `lookup()` and a by-ID cell
getter. Unsupported cells, invalid input, unknown lemmas, and missing lexical
metadata are distinct typed outcomes. Missing forms are never `-`, an empty string,
or an em dash.

Whole noun, adjective, finite-verb, imperative, and l-participle paradigms enumerate
the same typed cell resolver as individual calls. `dictionary_paradigm_by_id()`
returns every safely extracted table feature, including non-finite verb cells.

## Explicit OOV metadata

```rust
use old_church_slavonic::noun::NounLexeme;
use old_church_slavonic::{
    noun_with, Animacy, Case, Gender, NounCell, NounClass, Number, NumberRestriction,
};

let predicted = noun_with(
    &NounLexeme {
        lemma: "роботъ".into(),
        class: NounClass::OMasculineHard,
        gender: Gender::Masculine,
        animacy: Animacy::Inanimate,
        number_restriction: NumberRestriction::All,
    },
    NounCell { case: Case::Locative, number: Number::Plural },
)?;
assert_eq!(predicted.variants[0].text, "роботѣхъ");
assert!(!predicted.trace.is_empty());
# Ok::<(), old_church_slavonic::InflectionError>(())
```

Bare infinitives do not select a verb class or present/aorist stem. Supply a
`verb::VerbLexeme`; an omitted required stem is returned as
`MissingLexicalMetadata`, not hidden by the most frequent class.

## Unicode and scripts

Display spelling is NFC and otherwise lossless. Lookup uses the identical shared
NFC-plus-Unicode-lowercase function in the extractor and runtime. It does not strip
yers, jat, nasal vowels, accents, titla, or palatalization marks. Page words,
canonical heads, and source-listed alternatives are explicit aliases. Cyrillic and
Glagolitic display forms are kept distinct and never mechanically transliterated
into purported source forms. Source romanization remains metadata.

See [the orthography contract](docs/ORTHOGRAPHY.md) for input limits and exact versus
normalized evaluation.

## Measured pinned snapshot

The current committed snapshot contains 3,081 accepted lexemes, 134,436 public
feature cells, and 137,081 ordered variants. All 137,081 variants round-trip through
the typed public facade in source order. The deterministic held-out OOV split reports
a 91.88% macro exact average across represented noun, adjective, and verb rule slices
(also 91.88% under the conservative NFC/lowercase metric). Per-class and per-cell
denominators and development/test results are in [reports/accuracy.md](reports/accuracy.md); extraction drops are in
[reports/extraction-coverage.md](reports/extraction-coverage.md). These metrics are
not combined: registry round-trip measures pipeline integrity, while OOV recall
measures rule generalization against the same table target.

## Data, attribution, and maintenance

The normalized Kaikki/English-Wiktionary registry and generated Rust are committed.
The raw JSONL is not. Source URL, dates, byte length, Wiktextract revision, and SHA-256
are in `data/SOURCES.toml` and `data/extracted/source.json`. This distribution uses
CC BY-SA 4.0 for Wiktionary-derived data; code is MIT OR Apache-2.0. See
[ATTRIBUTION.md](ATTRIBUTION.md).
UD OCS PROIEL is CC BY-NC-SA and is accepted only as optional local evaluation input;
it is not bundled in the runtime or published package.

```bash
cargo xtask refresh-data --dump /path/to/pinned-ocs.jsonl
cargo xtask check-registry
cargo xtask extraction-report
cargo xtask accuracy
cargo xtask accuracy --dump /path/to/pinned-ocs.jsonl
cargo xtask accuracy-ud --path /path/to/UD_Old_Church_Slavonic-PROIEL
cargo xtask dump-paradigms before
cargo xtask diff-paradigms target/paradigm-fingerprint/before.tsv after.tsv
cargo xtask examples
cargo xtask speed
cargo xtask guard-witnesses
cargo xtask check-all
```

## Workspace and non-goals

- `old-church-slavonic-core`: shared types, Unicode keys, and pure rules;
- `old-church-slavonic`: static dictionary-backed facade;
- `old-church-slavonic-extractor`: streaming offline normalizer/generator;
- `xtask`: checks, reports, examples, speed, and reviewable paradigm diffs.

V0.1 does not perform morphological analysis, syntax or phrase realization,
compound tense construction, clitic placement, manuscript transcription/OCR,
abbreviation expansion, reconstructed accent placement, automatic script conversion,
or later-recension normalization.
