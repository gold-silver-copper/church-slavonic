# old-church-slavonic

Dictionary-backed Old Church Slavonic inflection starts with a lemma and direct
grammatical dimensions:

```rust
use old_church_slavonic::{noun, verb, Case, Number, Person};

let dual = noun("обѣдъ", Case::Dative, Number::Dual)?;
assert_eq!(dual.primary_text(), "обѣдома");

let present = verb("благословити", Person::First, Number::Singular)?;
assert_eq!(present.primary_text(), "благословлѭ");
# Ok::<(), old_church_slavonic::InflectionError>(())
```

Dictionary determiners use the same direct grammatical dimensions:

```rust
use old_church_slavonic::{determiner, Case, Gender, Number};

let which = determiner("кꙑи", Case::Accusative, Number::Singular, Gender::Feminine)?;
assert_eq!(which.primary_text(), "кѫѭ");
# Ok::<(), old_church_slavonic::InflectionError>(())
```

For repeated calls, bind a unique dictionary identity once:

```rust
use old_church_slavonic::{Case, Noun, Number, Person, Verb};

let meal = Noun::new("обѣдъ")?;
assert_eq!(meal.form(Case::Dative, Number::Dual)?.primary_text(), "обѣдома");

let bless = Verb::new("благословити")?;
assert_eq!(
    bless.present(Person::First, Number::Singular)?.primary_text(),
    "благословлѭ",
);
# Ok::<(), old_church_slavonic::InflectionError>(())
```

A successful `FormSet` is nonempty. `primary_text()` means the first deterministic
source-order spelling, while `variants()` and `texts()` preserve every alternative.
Romanization, source, warnings, traces, and competing analyses remain accessible.
Ambiguity, unknown lemmas, missing metadata, unsupported formations, and invalid
historical cells are distinct `InflectionError` values.

Resolved `Participle` handles retain the verb's ordered source-backed analyses and
independent oblique stems:

```rust
use old_church_slavonic::{Animacy, Case, Gender, Number, Verb};

let participle = Verb::new("благословити")?.past_active_participle()?;
let forms = participle.short(
    Case::Genitive,
    Number::Singular,
    Gender::Masculine,
    Animacy::Inanimate,
)?;
assert_eq!(
    forms.texts().collect::<Vec<_>>(),
    ["благословл҄ьша", "благословивъша"],
);
# Ok::<(), old_church_slavonic::InflectionError>(())
```

The crate root is the ordinary API. Generic cells and tooling are in
`advanced::cells`; stable IDs in `advanced::by_id`; explicit caller metadata and
`*_with` rules in `advanced::rules`; audited principal parts in
`advanced::metadata`; generic dictionary features in `advanced::raw_features`; and
diagnostics in `trace`.

| Former call | Current call |
|---|---|
| `noun(lemma, NounCell { ... })` | `noun(lemma, case, number)` |
| generic long/short adjective cell | `adjective(...)` / `short_adjective(...)` |
| present finite cell | `verb(lemma, person, number)` |
| `noun_paradigm(id)` | `noun_paradigm(lemma)` / `advanced::by_id::noun_paradigm_by_id(id)` |
| `primary_source_order().unwrap()` | `primary()` / `primary_text()` |

The package includes its pinned generated dictionary and attribution. Runtime code
performs no file, network, JSON, TSV, XML, or Lua access. Original code is MIT OR
Apache-2.0; English-Wiktionary-derived data is CC BY-SA 4.0. See
[ATTRIBUTION.md](ATTRIBUTION.md).
