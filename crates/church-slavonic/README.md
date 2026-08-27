# church-slavonic

Church Slavonic inflection — nouns, adjectives, verbs and the personal
pronoun — in both recensions of the language (Old Church Slavonic and the
Synodal print), backed by source-derived lookup tables with a rule-engine
fallback.

```rust
use church_slavonic::*;

assert_eq!(
    ChurchSlavonic::noun("градъ", &Case::Genitive, &Number::Singular, &Recension::OldChurchSlavonic),
    "града"
);
assert_eq!(
    ChurchSlavonic::noun("рабъ", &Case::Dative, &Number::Singular, &Recension::Synodal),
    "рабꙋ̀"
);
```

See the [repository README](https://github.com/gold-silver-copper/church-slavonic)
for the architecture, the accuracy tables, the data provenance and the
licence of the bundled tables (`ATTRIBUTION.md`).
