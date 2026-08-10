# Historical Church Slavonic libraries for Rust

This workspace contains two deliberately separate language targets:

- `old-church-slavonic`: dictionary-backed canonical Old Church Slavonic; and
- `synodal-church-slavonic`: the normalized Russian Synodal liturgical
  recension, with Synodal morphology, accents, typography, semantic
  abbreviations, analytic phrases, and explicit reviewed OCS inheritance.

The Synodal library is not an orthographic alias for the OCS engine. It has an
independent target registry and rule tables, and every OCS-derived prediction
passes through a stable recension mapping and a Synodal realization rule.

```rust
use synodal_church_slavonic::{present, Number, Person};

let form = present("быти", Person::First, Number::Singular)?;
assert_eq!(form.primary_text(), "єсмь");
assert_eq!(form.target_recension(), synodal_church_slavonic::Recension::SynodalRussian);
# Ok::<(), synodal_church_slavonic::Error>(())
```

The full source inventory, including 244 locally cached and checksum-verified
machine-readable artifacts (about 4.6 GB), is in `references/`. Raw source bytes
are gitignored and excluded from packages. See `docs/SYNODAL_RECENSION.md`,
`docs/SYNODAL_MORPHOLOGY.md`, `docs/SYNODAL_ORTHOGRAPHY.md`, and
`reports/synodal-evaluation.md` for the implemented boundary and measured seed
coverage.

## Old Church Slavonic inflection

`old-church-slavonic` is a fast, offline inflector for canonical **Old Church
Slavonic** (`cu`/`chu`). Its ordinary API is a lemma followed by direct grammatical
dimensions:

```rust
use old_church_slavonic::{noun, present, Case, Number, Person};

let dual = noun("обѣдъ", Case::Dative, Number::Dual)?;
assert_eq!(dual.primary_text(), "обѣдома");

let blessing = present("благословити", Person::First, Number::Singular)?;
assert_eq!(blessing.primary_text(), "благословлѭ");
# Ok::<(), old_church_slavonic::InflectionError>(())
```

The facade combines source-ordered English-Wiktionary dictionary tables with a
conservative pure-Rust rule engine. It does not silently mix Russian, Serbian,
Croatian, or another later Church Slavonic recension.

## Install

The minimum supported Rust version is 1.85.

```toml
[dependencies]
old-church-slavonic = "0.5"
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

`Noun`, `Adjective`, `Verb`, `Determiner`, `Pronoun`, and `Numeral` bind one
unambiguous dictionary identity without copying mutable class or stem metadata:

```rust
use old_church_slavonic::{Animacy, Case, Gender, Number, Person, Noun, Verb};

let meal = Noun::resolve("обѣдъ")?;
assert_eq!(
    meal.form(Case::Dative, Number::Dual)?.primary_text(),
    "обѣдома",
);

let bless = Verb::resolve("благословити")?;
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
# Ok::<(), Box<dyn std::error::Error>>(())
```

`resolve` selects exactly one lexeme of the required part of speech. Ambiguous and
unknown lemmas remain typed errors. After selecting a candidate with `lookup`, use
the corresponding handle's `from_id` constructor to bind its stable ID.

## Forms, alternatives, and evidence

Every successful `FormSet` is nonempty. `primary()` and `primary_text()` return the
first deterministic **source-order** variant; they do not claim that one spelling
is linguistically superior. `variants()` and `texts()` retain every alternative.
Use `unique_text()` when multiple source variants must be an error, or call
`select(VariantPolicy::SourceFirst)` to make source-first selection explicit:

```rust
use old_church_slavonic::{aorist, FormSource, Number, Person, VariantPolicy};

let forms = aorist("бꙑти", Person::First, Number::Singular)?;
assert_eq!(forms.primary_text(), "бѣхъ");
assert_eq!(forms.texts().collect::<Vec<_>>(), ["бѣхъ", "бꙑхъ"]);
assert!(forms.unique_text().is_err());
assert_eq!(forms.select(VariantPolicy::SourceFirst)?.text, "бѣхъ");
assert_eq!(forms.source(), &FormSource::DictionaryTable);
# Ok::<(), Box<dyn std::error::Error>>(())
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
    Err(InflectionError::HistoricallyInvalidCell { .. }),
));
```

## Paradigms

Lemma-oriented `noun_paradigm`, `adjective_paradigm`, `present_paradigm`,
`finite_paradigm`, `imperative_paradigm`, `l_participle_paradigm`, and
`participle_paradigm` enumerate the same by-ID resolver as one-cell calls. Typed
closed-class paradigms cover the source-supported determiner, pronoun, and numeral
systems. Their `CellOutcome` entries retain per-cell errors rather than omitting
gaps. Each type provides `lemma()`, `id()`, direct-dimension `form(...)`, `iter()`,
`successes()`, `failures()`, and `into_rows()`. A missing specialized row and a
represented cell that failed are distinct `ParadigmLookupError` variants.

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
- `advanced::raw_features`: generic normalized feature-key access for diagnostics; and
- `trace`: provenance and rule diagnostics.

The independent `old-church-slavonic-core` crate remains available to rule-only
callers who explicitly supply lexical class and principal-part facts.

## Migration from the former facade

| Former call | New ordinary call |
|---|---|
| `noun(lemma, NounCell { case, number })` | `noun(lemma, case, number)` |
| `adjective(lemma, AdjectiveCell { form: Long, ... })` | `long_adjective(lemma, ...)` |
| `adjective(lemma, AdjectiveCell { form: Short, ... })` | `short_adjective(lemma, ...)` |
| `verb(lemma, person, number)` | `present(lemma, person, number)` |
| `finite_verb(lemma, Imperfect cell)` | `imperfect(lemma, person, number)` |
| `finite_verb(lemma, Aorist cell)` | `aorist(lemma, person, number)` |
| `finite_verb(lemma, tense, person, number)` | `finite(lemma, tense, person, number)` |
| `comparative(lemma)` | `comparative_citation(lemma)` |
| `verb_paradigm(lemma)` | `present_paradigm(lemma)` |
| `finite_verb_paradigm(lemma)` | `finite_paradigm(lemma)` |
| `Noun::new(lemma)` | `Noun::resolve(lemma)` |
| `noun_paradigm(id)` | `noun_paradigm(lemma)` or `advanced::by_id::noun_paradigm_by_id(id)` |
| `forms.primary_source_order().unwrap().text` | `forms.primary_text()` |
| public result fields | `variants()`, `source()`, `warnings()`, `trace()`, `analyses()` |

The direct syntax is Ruthenian-like; `Result<FormSet, InflectionError>` is
intentionally not. Old Church Slavonic is an attested, dictionary-backed language
with homographs, source variants, incomplete principal parts, competing analyses,
and genuine historical gaps.

## Supported surface

| System | Ordinary API | Evidence behavior |
|---|---|---|
| Nouns | `noun`, `Noun`, `noun_paradigm` | Exact tables first; dictionary metadata or explicit rules for supported classes |
| Adjectives | `long_adjective`, `short_adjective`, `Adjective` | Exact tables first; hard/soft metadata rules; citation comparatives only |
| Determiners | `determiner`, `Determiner`, typed paradigm | Exact pinned dictionary cells only |
| Pronouns | `pronoun`, `personal_pronoun`, `gendered_pronoun`, `Pronoun` | Separate source-backed cell shapes; no catch-all options API |
| Numerals | `numeral`, `gendered_numeral`, `Numeral` | Declension cells present in the pinned source; no arbitrary numeral generator |
| Finite verbs | `present`, `imperfect`, `aorist`, `finite` | Exact tables, independently sourced metadata, then reviewed overrides |
| Imperatives | `imperative` | Six historically represented person-number cells |
| Non-finite forms | `infinitive`, `supine`, `verbal_noun`, `l_participle` | Table or independently supported productive rule |
| Participles | four named binders plus `Participle` | Independently modeled verbal formations with adjective agreement |

Malformed Wiktextract verb rows are not repaired by guessing. Productive rules
expand missing-cell behavior only when source-backed or caller-supplied metadata is
sufficient.

## Unicode, data, and maintenance

`Lemma::parse` NFC-normalizes one word, exposes `Script::Cyrillic` or
`Script::Glagolitic`, and rejects empty, mixed-script, markup-bearing, or malformed
input. Every ordinary raw-string call passes through that validation. Lookup then
applies Unicode lowercase without stripping historical letters, accents, titla, or
palatalization marks. Cyrillic and Glagolitic remain distinct; there is no lossy
automatic transliteration. See
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
does not implement productive arbitrary numerals, a catch-all pronoun API,
automatic Cyrillic/Glagolitic transliteration, syntax, phrase realization,
manuscript transcription, OCR, abbreviation expansion, or later-recension
normalization.
