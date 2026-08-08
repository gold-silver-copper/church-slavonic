# old-church-slavonic

Dictionary-backed Old Church Slavonic inflection with structured variants, typed
ambiguity/errors, source-ordered alternatives, and a pure rule fallback.

```rust
use old_church_slavonic::{Case, NounCell, Number, noun};

let forms = noun(
    "обѣдъ",
    NounCell { case: Case::Dative, number: Number::Dual },
)?;
assert_eq!(forms.variants[0].text, "обѣдома");
# Ok::<(), old_church_slavonic::InflectionError>(())
```

The package includes the pinned generated dictionary and performs no runtime file,
network, JSON, or Lua access. See the repository README and reports for exact V0.1
coverage, constrained verb scope, OOV accuracy, and provenance semantics.

Known verbs use the same conservative order for every lemma and by-ID API: exact
dictionary table, typed dictionary principal parts, approved cell override, then
productive core. `FormSet::analyses` keeps multiple source-backed analyses ordered
with their evidence and traces. Missing or contradictory principal parts and
unsupported formations are returned as typed errors; no suffix-frequency heuristic
is used.

```rust
use old_church_slavonic::{
    participle, AdjectiveCell, AdjectiveForm, Animacy, Case, Gender, Number,
    ParticipleCell, ParticipleKind,
};

let forms = participle(
    "благословити",
    ParticipleCell {
        kind: ParticipleKind::PastActive,
        adjective: AdjectiveCell {
            case: Case::Genitive,
            number: Number::Singular,
            gender: Gender::Masculine,
            animacy: Animacy::Inanimate,
            form: AdjectiveForm::Short,
        },
    },
)?;
assert_eq!(forms.variants.len(), 2);
# Ok::<(), old_church_slavonic::InflectionError>(())
```

Original code is MIT OR Apache-2.0. The bundled English-Wiktionary-derived data is
redistributed under CC BY-SA 4.0; [ATTRIBUTION.md](ATTRIBUTION.md) records the source,
snapshot hash, transformation notice, and license links.
