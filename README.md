# Old Church Slavonic inflection for Rust

`old-church-slavonic` is a fast, offline inflector for canonical **Old Church
Slavonic** (`cu`/`chu`). Its ordinary API is a lemma followed by direct grammatical
dimensions:

```rust
use old_church_slavonic::{noun, verb, Case, Number, Person};

let dual = noun("обѣдъ", Case::Dative, Number::Dual)?;
assert_eq!(dual.primary_text(), "обѣдома");

let present = verb("благословити", Person::First, Number::Singular)?;
assert_eq!(present.primary_text(), "благословлѭ");
# Ok::<(), old_church_slavonic::InflectionError>(())
```

The facade combines source-ordered English-Wiktionary dictionary tables with a
conservative pure-Rust rule engine. It does not silently mix Russian, Serbian,
Croatian, or another later Church Slavonic recension.

## Install

The minimum supported Rust version is 1.85.

```toml
[dependencies]
old-church-slavonic = "0.3"
```

The generated dictionary is compiled into the package. Runtime crates perform no
file, network, JSON, TSV, XML, or Lua access.

## Semantic dictionary and game vocabulary

The workspace also contains `old-church-slavonic-dictionary`, an offline
Wiktionary-backed meaning dictionary and the `ocs-dict` CLI. It searches English
concepts, displays OCS senses and examples, analyzes exact and productively
generated dictionary forms, and validates game-vocabulary manifests without a
later-language fallback.

```bash
cargo install old-church-slavonic-dictionary
ocs-dict search "gold coin"
ocs-dict show златикъ
ocs-dict lint game-vocabulary.tsv
ocs-dict check-text rendered-game.txt --max-unknown 0
```

Meaning and morphology remain separate evidence layers. A semantic sense links to
the inflector only when both snapshots resolve the same lexical identity; a lookup
hit does not silently manufacture an unsupported paradigm.

`check-text` distinguishes citations, exact source-table forms, forms generated
by the inflector from pinned dictionary metadata, tokens attested in pinned
source examples, explicit allowlist entries, and unknowns. This lets consumers
enforce a zero-unknown gate without treating a productive form as a quotation.

## Resolve once for repeated calls

`Noun`, `Adjective`, and `Verb` bind one unambiguous dictionary identity without
copying mutable class or stem metadata:

```rust
use old_church_slavonic::{Animacy, Case, Gender, Number, Person, Noun, Verb};

let meal = Noun::new("обѣдъ")?;
assert_eq!(
    meal.form(Case::Dative, Number::Dual)?.primary_text(),
    "обѣдома",
);

let bless = Verb::new("благословити")?;
assert_eq!(
    bless.present(Person::First, Number::Singular)?.primary_text(),
    "благословлѭ",
);

let participle = bless.past_active_participle()?;
let declined = participle.short(
    Case::Genitive,
    Number::Singular,
    Gender::Masculine,
    Animacy::Inanimate,
)?;
assert_eq!(
    declined.texts().collect::<Vec<_>>(),
    ["благословл҄ьша", "благословивъша"],
);
# Ok::<(), old_church_slavonic::InflectionError>(())
```

`new` resolves exactly one lexeme of the required part of speech. Ambiguous and
unknown lemmas remain typed errors. After selecting a candidate with `lookup`, use
`Noun::from_id`, `Adjective::from_id`, or `Verb::from_id` to bind its stable ID.

## Forms, alternatives, and evidence

Every successful `FormSet` is nonempty. `primary()` and `primary_text()` return the
first deterministic **source-order** variant; they do not claim that one spelling
is linguistically superior. `variants()` and `texts()` retain every alternative:

```rust
use old_church_slavonic::{aorist, FormSource, Number, Person};

let forms = aorist("бꙑти", Person::First, Number::Singular)?;
assert_eq!(forms.primary_text(), "бѣхъ");
assert_eq!(forms.texts().collect::<Vec<_>>(), ["бѣхъ", "бꙑхъ"]);
assert_eq!(forms.source(), &FormSource::DictionaryTable);
# Ok::<(), old_church_slavonic::InflectionError>(())
```

Results also retain romanization, source class, warnings, rule traces, and ordered
morphological analyses. Sources distinguish exact dictionary tables, dictionary
principal-part generation, explicit caller metadata, OOV prediction, and reviewed
overrides. Dictionary table forms are source dictionary claims, not automatically
manuscript-attested forms.

The facade never returns `?`, `-`, an empty string, or a plausible substitute for a
failure. `UnknownLemma`, `AmbiguousLexeme`, `MissingLexicalMetadata`,
`UnsupportedFormation`, `HistoricallyInvalidCell`, and `UnsupportedCell` remain
distinct.

```rust
use old_church_slavonic::{imperative, noun, Case, InflectionError, Number, Person};

assert!(matches!(
    noun("блѧдь", Case::Nominative, Number::Singular),
    Err(InflectionError::AmbiguousLexeme { .. }),
));
assert!(matches!(
    imperative("благословити", Person::Third, Number::Dual),
    Err(InflectionError::HistoricallyInvalidCell),
));
```

## Paradigms

Lemma-oriented `noun_paradigm`, `adjective_paradigm`, `verb_paradigm`,
`finite_verb_paradigm`, `imperative_paradigm`, `l_participle_paradigm`, and
`participle_paradigm` enumerate the same by-ID resolver as one-cell calls. Their
`CellOutcome` entries retain per-cell errors rather than omitting gaps. Each type
provides `lemma()`, `id()`, direct-dimension `get(...)`, `iter()`, and owned and
borrowed iteration. Stable identity alternatives are under `advanced::by_id`.

## Explicit rules and specialist APIs

Ordinary callers need only the curated crate root. Generic cells, explicit lexical
metadata, dictionary metadata, stable-ID operations, and raw feature access live in
intentional namespaces:

```rust
use old_church_slavonic::advanced::cells::NounCell;
use old_church_slavonic::advanced::rules::{
    noun_with, NounClass, NounLexeme, NumberRestriction,
};
use old_church_slavonic::{Animacy, Case, FormSource, Gender, Number};

let forms = noun_with(
    &NounLexeme {
        lemma: "роботъ".into(),
        class: NounClass::OMasculineHard,
        gender: Gender::Masculine,
        animacy: Animacy::Inanimate,
        number_restriction: NumberRestriction::All,
    },
    NounCell { case: Case::Locative, number: Number::Plural },
)?;
assert_eq!(forms.primary_text(), "роботѣхъ");
assert!(matches!(forms.source(), FormSource::ExplicitMetadataRule { .. }));
assert!(!forms.trace().is_empty());
# Ok::<(), old_church_slavonic::InflectionError>(())
```

- `advanced::cells`: generic cell structs for tools and paradigm inspection;
- `advanced::by_id`: stable dictionary-identity operations;
- `advanced::rules`: caller-supplied lexemes and `*_with` generation;
- `advanced::metadata`: audited dictionary principal parts and evaluation entry
  points;
- `advanced::raw_features`: closed-class and normalized feature-key access; and
- `trace`: provenance and rule diagnostics.

The independent `old-church-slavonic-core` crate remains available to rule-only
callers who explicitly supply lexical class and principal-part facts.

## Migration from the former facade

| Former call | New ordinary call |
|---|---|
| `noun(lemma, NounCell { case, number })` | `noun(lemma, case, number)` |
| `adjective(lemma, AdjectiveCell { form: Long, ... })` | `adjective(lemma, ...)` |
| `adjective(lemma, AdjectiveCell { form: Short, ... })` | `short_adjective(lemma, ...)` |
| `finite_verb(lemma, Present cell)` | `verb(lemma, person, number)` |
| `finite_verb(lemma, Imperfect cell)` | `imperfect(lemma, person, number)` |
| `finite_verb(lemma, Aorist cell)` | `aorist(lemma, person, number)` |
| `noun_paradigm(id)` | `noun_paradigm(lemma)` or `advanced::by_id::noun_paradigm_by_id(id)` |
| `forms.primary_source_order().unwrap().text` | `forms.primary_text()` |
| public result fields | `variants()`, `source()`, `warnings()`, `trace()`, `analyses()` |

The direct syntax is Ruthenian-like; `Result<FormSet, InflectionError>` is
intentionally not. Old Church Slavonic is an attested, dictionary-backed language
with homographs, source variants, incomplete principal parts, competing analyses,
and genuine historical gaps.

## Supported surface

- nouns: seven cases × singular/dual/plural from dictionary tables, plus explicit
  rules for hard o/a, soft jo/ja, i-, u-, and n/nt/r/s/v consonant stems;
- adjectives: dictionary and explicit hard/soft short and long agreement, plus
  dictionary-listed comparative citations;
- verbs: safe table cells and independently specified present, imperfect, aorist,
  imperative, infinitive, supine, l-participle, and four declined participle
  systems; and
- dictionary-backed pronoun, determiner, and numeral cells in
  `advanced::raw_features`.

Malformed Wiktextract verb rows are not repaired by guessing. Productive rules
expand missing-cell behavior only when source-backed or caller-supplied metadata is
sufficient.

## Unicode, data, and maintenance

Display spelling is NFC and otherwise lossless. Lookup applies shared
NFC-plus-Unicode-lowercase normalization without stripping historical letters,
accents, titla, or palatalization marks. Cyrillic and Glagolitic remain distinct;
there is no lossy automatic transliteration. See
[docs/ORTHOGRAPHY.md](docs/ORTHOGRAPHY.md).

The current pinned snapshot contains 3,081 lexemes, 134,761 public feature cells,
137,406 ordered variants, and 3,157 normalized verb metadata fields. Accuracy,
corpus, and extraction reports remain separate in [reports](reports). The source
identity and transformation record are in `data/SOURCES.toml`, generated metadata,
and the published `ATTRIBUTION.md`. Wiktionary-derived data is CC BY-SA 4.0; code is
MIT OR Apache-2.0.

```bash
cargo xtask check-registry
cargo xtask accuracy
cargo xtask examples
cargo xtask guard-witnesses
cargo xtask check-dictionary
cargo xtask check-all
```

The workspace also contains the offline extractor and `xtask`. It intentionally
does not implement productive arbitrary numerals, a feature-only pronoun API,
automatic Cyrillic/Glagolitic transliteration, syntax, phrase realization,
manuscript transcription, OCR, abbreviation expansion, or later-recension
normalization.
