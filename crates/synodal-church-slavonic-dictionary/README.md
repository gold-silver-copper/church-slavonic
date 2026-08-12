# synodal-church-slavonic-dictionary

Semantic lookup, reverse morphological analysis, source concordance, and
application-vocabulary validation for **Synodal Russian Church Slavonic**.

```rust
use synodal_church_slavonic_dictionary::{analyze, search_gloss};

let entries = search_gloss("city")?;
assert!(entries.iter().any(|entry| entry.lexeme.lemma() == "градъ"));

let analyses = analyze("є҆́смь")?;
assert!(analyses.iter().any(|analysis| analysis.lexeme.lemma() == "быти"));
# Ok::<(), synodal_church_slavonic_dictionary::core::Error>(())
```

`analyze` returns every compatible typed cell admitted by the default `Strict`
policy; it does not choose one lemma silently. `analyze_with` accepts a configured
`Inflector` when inherited `Productive` or `Exploratory` analyses are wanted, and
each such analysis retains its recension-mapping ID. Analyses distinguish exact
Synodal attestation, normative tables, productive target rules, inherited and
analogical predictions, and semantic abbreviation expansion. `lookup`,
`lookup_by_id`, `lemmatize`, `lemmatize_with`, `search_gloss`, and `concordance`
keep stable identity, source recension, passage, partition, and semantic-review
status visible.

`lint_vocabulary` validates a serializable game/application manifest for Unicode
orthography, known morphology, expected part of speech, required semantic sense,
and surface ambiguity. The crate accepts typed manifests only; applications may
choose their own serialization layer outside the runtime linguistic crate.

The bundled semantic registry contains 855 reviewed target lexemes and senses.
The public `families` and `show_family_by_id` operations expose exact members,
principal parts, supported systems, and missing family metadata without accepting
diagnostic candidate IDs as runtime facts. The `synodal-dict` binary adds
`search`, `show`, `families`, `show-family`, `analyze`, `lint`, `check-text`, and
corpus `coverage --by-family` commands; see
`docs/SYNODAL_CLI_AND_COVERAGE.md`. Mixed
historical D'yachenko meanings and OCS Wiktionary semantics enter only after an
explicit Synodal-target review; raw dictionary scans and corpora are not
packaged. Runtime operation is offline and the library still builds for
`wasm32-unknown-unknown` without the CLI feature.
