# old-church-slavonic

Offline, dictionary-backed Old Church Slavonic inflection with a typed
lemma-plus-grammar API. The bundled snapshot is derived from English Wiktionary;
runtime code performs no file or network access.

## One direct cell

```rust
use old_church_slavonic::{noun, present, Case, Number, Person};

let dual = noun("обѣдъ", Case::Dative, Number::Dual)?;
assert_eq!(dual.primary_text(), "обѣдома");

let blessing = present("благословити", Person::First, Number::Singular)?;
assert_eq!(blessing.primary_text(), "благословлѭ");
# Ok::<(), old_church_slavonic::InflectionError>(())
```

Every raw lemma goes through `Lemma::parse`: input is NFC-normalized, historical
characters and diacritics are preserved, Cyrillic and Glagolitic are detected,
and empty, mixed-script, markup-bearing, or malformed input is rejected.

## Resolve once

Use a handle when several cells belong to the same unambiguous dictionary identity:

```rust
use old_church_slavonic::{Number, Person, Verb};

let verb = Verb::resolve("благословити")?;
assert_eq!(
    verb.present(Person::First, Number::Singular)?.primary_text(),
    "благословлѭ",
);
assert_eq!(verb.lemma(), "благословити");

let rebound = Verb::from_id(verb.id())?;
assert_eq!(rebound, verb);
# Ok::<(), old_church_slavonic::InflectionError>(())
```

`Noun`, `Adjective`, `Determiner`, `Pronoun`, and `Numeral` provide the same
`resolve`/`from_id`/`lemma`/`id` pattern. Ambiguous lookup returns every candidate
identity instead of selecting one silently.

## Handle source variants explicitly

A successful `FormSet` is nonempty. `primary_text()` is the first deterministic
source-order spelling—not a claim that it is linguistically preferred.

```rust
use old_church_slavonic::{aorist, Number, Person, VariantPolicy};

let forms = aorist("бꙑти", Person::First, Number::Singular)?;
assert_eq!(forms.texts().collect::<Vec<_>>(), ["бѣхъ", "бꙑхъ"]);
assert!(forms.unique_text().is_err());
assert_eq!(forms.select(VariantPolicy::SourceFirst)?.text, "бѣхъ");
# Ok::<(), Box<dyn std::error::Error>>(())
```

Variants retain romanization; the set also retains its source class, warnings,
rule trace, provenance, and competing morphological analyses.

## Inspect a contextual error

```rust
use old_church_slavonic::{present, InflectionError, Number, PartOfSpeech, Person};

let error = present("несуществовати", Person::Third, Number::Singular)
    .expect_err("unknown fixture");
assert!(matches!(
    error,
    InflectionError::UnknownLemma { ref lemma, part_of_speech: PartOfSpeech::Verb }
        if lemma == "несуществовати"
));
```

Invalid input, unknown or ambiguous lemmas, missing or contradictory metadata,
unsupported formations, historically invalid cells, and unsupported cells stay
distinct. Cell errors retain the stable lexeme ID and the exact `RequestedCell`.
The crate never uses `"?"`, an empty string, or a neighboring cell as a fallback.

## Walk a paradigm

```rust
use old_church_slavonic::{noun_paradigm, Case, Number};

let paradigm = noun_paradigm("обѣдъ")?;
assert_eq!(
    paradigm.form(Case::Dative, Number::Dual)?.primary_text(),
    "обѣдома",
);
assert_eq!(paradigm.iter().count(), 21);
assert_eq!(paradigm.successes().count() + paradigm.failures().count(), 21);

let rows = paradigm.into_rows();
assert_eq!(rows.len(), 21);
# Ok::<(), Box<dyn std::error::Error>>(())
```

Every paradigm calls the same canonical one-cell by-ID resolver used by direct
calls and handles. Failed rows remain visible. `ParadigmLookupError` distinguishes
a cell outside a specialized inventory from a represented cell that failed.

## Supply explicit out-of-vocabulary metadata

The advanced rules API requires the caller to state lexical facts instead of
guessing them from a later Slavic language:

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
# Ok::<(), old_church_slavonic::InflectionError>(())
```

## Capability table

| System | Ordinary API | Evidence behavior |
|---|---|---|
| Nouns | `noun`, `Noun`, `noun_paradigm` | Tables first; supported dictionary metadata or explicit rules |
| Adjectives | `long_adjective`, `short_adjective`, `long_only_adjective`, `Adjective` | Tables first; hard/soft metadata rules; exhaustive typed long-only inventory; citation comparatives |
| Determiners | `determiner`, `determiner_identity`, `determiner_paradigm`, `Determiner` | Exhaustive 11-identity reviewed inventory over regular `2/p`, exceptional `кꙑи`, and adjectival `2/a`; explicit OOV metadata |
| Pronouns | `personal_pronoun_with`, `reflexive_pronoun`, `anaphoric_pronoun`, compatible ordinary functions, `Pronoun` | Complete reviewed personal/reflexive/anaphoric tables; other pronouns use exact source cells |
| Numerals | `numeral`, `gendered_numeral`, `cardinal_numeral_identity`, `cardinal_numeral_paradigm`, `compound_cardinal`, `compound_cardinal_paradigm`, `Numeral` | Reviewed cardinals through 99 with typed agreement/government, structured components and correlated alternatives; exact fallback for other numeral types |
| Finite verbs | `present`, `imperfect`, `aorist`, `finite`, `Verb` | Tables, independently sourced metadata, reviewed overrides |
| Imperatives | `imperative` | Six historically represented person-number cells |
| Non-finite forms | `infinitive`, `supine`, `verbal_noun`, `l_participle` | Table or independently supported rule |
| Participles | four named binders and `Participle` | Four separate verbal formations with adjective agreement |

The crate root is the ordinary API and includes a restrained `prelude`.
Specialist interfaces live under:

- `advanced::cells` for generic typed cells;
- `advanced::by_id` for stable dictionary identities;
- `advanced::rules` for explicit caller-supplied metadata;
- `advanced::metadata` for audited dictionary principal parts;
- `advanced::raw_features` for extraction and diagnostic feature keys; and
- `trace` for provenance and rule diagnostics.

The package includes its generated dictionary and attribution. Original code is
MIT OR Apache-2.0; English-Wiktionary-derived data is CC BY-SA 4.0. See
[ATTRIBUTION.md](ATTRIBUTION.md).
