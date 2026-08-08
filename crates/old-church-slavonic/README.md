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

Original code is MIT OR Apache-2.0. The bundled English-Wiktionary-derived data is
redistributed under CC BY-SA 4.0; [ATTRIBUTION.md](ATTRIBUTION.md) records the source,
snapshot hash, transformation notice, and license links.
