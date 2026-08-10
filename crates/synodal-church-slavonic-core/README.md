# synodal-church-slavonic-core

Pure, file-free grammar and Unicode primitives for **Synodal Russian Church
Slavonic**. This is a target-recension engine, not an Old Church Slavonic
orthography mode. Callers supply typed lexical metadata; the crate bundles no
dictionary and performs no file, network, JSON, XML, or TSV access.

```rust
use synodal_church_slavonic_core::{
    decline_noun, Animacy, Case, Gender, NounCell, NounDeclension, NounLexeme,
    Number, OrthographyProfile, SynodalWord,
};

let forms = decline_noun(
    &NounLexeme {
        lemma: SynodalWord::parse("рабъ")?,
        stem: SynodalWord::parse("раб")?,
        gender: Gender::Masculine,
        declension: NounDeclension::FirstHardMasculine,
    },
    NounCell {
        case: Case::Accusative,
        number: Number::Plural,
        animacy: Animacy::Animate,
    },
    OrthographyProfile::Expanded,
)?;

// Alypy §35: the normally nominative-like form and genitive-like variant
// remain separate, sourced candidates.
assert_eq!(
    forms.variants().iter().map(|v| v.printed.as_str()).collect::<Vec<_>>(),
    ["рабы", "рабовъ"],
);
# Ok::<(), synodal_church_slavonic_core::Error>(())
```

The current productive slice covers five reviewed noun classes; hard and soft,
short and long positive adjectives; present, imperfect, aorist, imperative,
infinitive, and l-participle formation from independent principal parts;
traditional Cyrillic numerals; validated Synodal words and rendered text;
loss-reporting transliteration; and versioned UTN #41 Synodal collation.

Every successful `FormSet` is nonempty and every variant records target
recension, source kind, confidence, evidence, assumptions, warnings, and a rule
trace. Generated forms are never called attested. Unsupported comparison stems,
declined productive participles, supines, verbal nouns, irregular paradigms, and
unreviewed declensions fail with typed errors.

`SynodalWord` rejects whitespace, other scripts, private-use code points,
controls, leading combining marks, and invalid Church Slavonic mark order.
`RenderedText` is the separate validator for phrases and punctuation. The three
presentation profiles are `Expanded`, `ExpandedAccentless`, and
`SynodalLiturgical`; the last one refuses productive rendering when accent
metadata is unavailable.

Default features enable `serde`. The crate also builds with
`--no-default-features` and for `wasm32-unknown-unknown`.

See the repository's `docs/SYNODAL_RECENSION.md`,
`docs/SYNODAL_MORPHOLOGY.md`, and `docs/SYNODAL_ORTHOGRAPHY.md` for the exact
language and evidence contracts.
