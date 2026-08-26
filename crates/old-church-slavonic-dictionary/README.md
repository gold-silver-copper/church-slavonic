# old-church-slavonic-dictionary

> **Succession notice.** This crate's consumer surface is superseded by
> the rule-first [`church-slavonic`](../church-slavonic) facade (and
> `church-slavonic-dictionary`); see `docs/DEPRECATION_MAP.md` for the
> item-by-item mapping and `docs/REWRITE_PLAN.md` for the program. This
> crate remains the reference implementation until the final deprecation
> release is published.

An offline, source-attributed Old Church Slavonic semantic dictionary for Rust.
It provides OCS-to-English lookup, English-to-OCS search, attested examples, and
a vocabulary-manifest lint designed for games. It contains no modern
constructed-language or later-recension fallback data.

```rust
use old_church_slavonic_dictionary::{search, SearchOptions};

let results = search("gold coin", &SearchOptions::default())?;
assert_eq!(results[0].sense().lemma(), "златикъ");
# Ok::<(), old_church_slavonic_dictionary::DictionaryError>(())
```

Install the CLI from the same package:

```console
cargo install old-church-slavonic-dictionary
ocs-dict search "gold coin"
ocs-dict show златикъ
ocs-dict lint game-vocabulary.tsv
ocs-dict check-text rendered-game.txt --max-unknown 0
```

Dictionary entries link to the stable identities in `old-church-slavonic` when
the pinned morphology snapshot contains the same lexeme. Meaning search and
inflection remain separate operations so incomplete source data is never hidden.
The text checker reports dictionary citations, exact inflection-table forms,
metadata-generated forms, source-example attestations, and unknown tokens as
distinct statuses.
