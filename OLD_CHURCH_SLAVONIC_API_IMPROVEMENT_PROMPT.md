# Improve the Old Church Slavonic API

Improve the public Rust API of the `church-slavonic` workspace, using
`../ruthenian` as an API-design reference. Ruthenian is inspiration for
discoverability, consistency, validated orthography, documentation, and
single-path paradigm generation—not for its bare-string returns, sentinel
errors, or linguistic fallback behavior.

Start by reading the complete public APIs, documentation, tests, and workspace
instructions in:

- `../church-slavonic`;
- `../ruthenian/crates/ruthenian-core`; and
- `../ruthenian/crates/ruthenian-orthography`.

Preserve unrelated uncommitted changes. Backwards compatibility is not required;
prefer one coherent API over deprecated aliases.

## Core principles

1. Keep Old Church Slavonic source-backed and fail-closed.
2. Never return `"?"`, silently substitute another grammatical cell, or
   manufacture unsupported historical forms.
3. Preserve all source variants, romanizations, warnings, provenance, analyses,
   and rule traces.
4. Keep runtime morphology free of file, network, JSON, TSV, XML, and Lua access.
5. Do not use Interslavic, Slovowiki, Russian, or another later Slavic language
   as lexical authority.
6. Regenerate dictionary artifacts through the existing extraction/xtask
   pipeline; never edit them manually.

## Target root API

Give ordinary callers a flat, discoverable “lemma plus grammar” facade:

```rust
noun(...)
long_adjective(...)
short_adjective(...)
determiner(...)
present(...)
imperfect(...)
aorist(...)
finite(...)
imperative(...)
l_participle(...)
infinitive(...)
supine(...)
verbal_noun(...)
```

Rename ambiguous existing functions:

- `verb` → `present`;
- `adjective` → `long_adjective`;
- `finite_verb` → `finite`;
- `verb_paradigm` → `present_paradigm`; and
- `comparative` → `comparative_citation`.

Use corresponding explicit method names on resolved handles. Remove old aliases
unless retaining one is demonstrably cleaner; compatibility is not a requirement.

## Resolved identities

Rename fallible dictionary constructors from `new` to `resolve`:

```rust
let noun = Noun::resolve("обѣдъ")?;
let verb = Verb::resolve("благословити")?;
```

Continue supporting stable dictionary identities through `from_id`, `id`, and
`lemma`. Add coherent resolved handles for `Determiner` and source-supported
closed classes. Add `Pronoun` and `Numeral` handles only after auditing the
generated feature inventory and designing grammatical cell types that accurately
represent the available OCS systems.

Do not create a catch-all API dominated by unrelated `Option` arguments. Prefer
separate typed cell structures or functions for genuinely different systems,
such as personal, reflexive, demonstrative, and interrogative pronouns.

## Validated input

Introduce a validated normalized lemma type inspired by
`ruthenian-orthography`:

```rust
let lemma = Lemma::parse("благословити")?;
assert_eq!(lemma.script(), Script::Cyrillic);
```

It should:

- enforce NFC normalization or return a precise normalization error;
- reject empty input, control characters, markup, and malformed combining marks;
- detect Cyrillic and Glagolitic;
- reject unintended mixed-script input;
- preserve historically meaningful characters and diacritics; and
- expose the normalized spelling and detected script.

Keep ordinary calls convenient. Avoid awkward generic signatures merely to
accept both `&str` and `&Lemma`. Design the smallest coherent interface, and
ensure all raw-string entry points pass through the same validation path.

Do not promise bijective Cyrillic–Glagolitic transliteration unless pinned
evidence supports it.

## Structured results

Keep `Result<FormSet, InflectionError>` as the ordinary return type. Improve
`FormSet` ergonomics without silently reducing it to a string:

```rust
forms.primary_text()
forms.unique_text()?
forms.variants()
forms.select(VariantPolicy::SourceFirst)
```

`unique_text()` must fail when multiple source variants exist. Variant selection
must be explicit and deterministic. Do not implement a misleading `Display`
conversion that silently discards alternatives.

## Contextual errors

Redesign underspecified errors so applications can report them directly. Errors
should retain the rejected lemma, part of speech, lexeme identity, and requested
cell where relevant:

```rust
UnknownLemma {
    lemma: String,
    part_of_speech: PartOfSpeech,
}

UnsupportedCell {
    lexeme_id: String,
    cell: RequestedCell,
}

HistoricallyInvalidCell {
    lexeme_id: String,
    cell: RequestedCell,
}
```

Continue distinguishing invalid input, unknown and ambiguous lemmas, missing and
contradictory metadata, unsupported formations, historically invalid cells, and
unsupported but conceptually valid cells. Do not collapse these into a string
message.

## Paradigm ergonomics

Preserve typed paradigm structures, but remove the awkward ordinary access
pattern:

```rust
Option<&Result<FormSet, InflectionError>>
```

Provide APIs such as:

```rust
paradigm.form(case, number)
paradigm.iter()
paradigm.successes()
paradigm.failures()
paradigm.into_rows()
```

A fixed paradigm's `form` method should return the represented cell's outcome
directly. Absence and failed generation must remain distinguishable where a
paradigm genuinely has a restricted inventory.

Every paradigm must be assembled by calling the same canonical one-cell resolver
used by direct calls and resolved handles. There must be no duplicate inflection
implementation.

## Closed-class coverage

Audit dictionary coverage for pronouns, determiners, and numerals. Add ordinary
typed root functions and paradigms wherever the source data supports a coherent
contract. Keep unsupported systems explicit rather than guessing.

No public game-facing workflow should require raw feature strings such as
`"pron:gen:sg"`. Keep raw feature-key access under `advanced::raw_features` for
extraction, diagnostics, and specialist tools.

## Prelude and documentation

Add a restrained `prelude` containing only common grammar types, resolved
handles, structured result types, and ordinary root functions. Do not put
metadata internals or raw feature APIs in it.

Rewrite the crate-level documentation and README around:

1. one direct cell call;
2. resolving a lemma once;
3. handling multiple variants;
4. inspecting an error;
5. walking a paradigm; and
6. using advanced explicit metadata for an out-of-vocabulary lemma.

Include a concise capability table covering nouns, adjectives, closed classes,
finite tenses, imperatives, non-finite forms, and participles. Clearly distinguish
dictionary-table forms, metadata-generated forms, reviewed overrides, and
explicit caller-supplied productive rules.

## Required tests and guards

Add or update tests proving:

- every root function has a compiling doctest;
- raw strings and `Lemma` values follow the same validation rules;
- mixed-script and hostile inputs fail without panicking;
- unknown-lemma errors retain the input and part of speech;
- unsupported-cell errors retain the requested cell;
- ambiguous lookup preserves all candidate identities;
- `unique_text()` accepts one form and rejects multiple variants;
- direct calls, handles, by-ID calls, and paradigm cells produce identical
  outcomes;
- paradigms use the canonical cell resolver;
- grammar `ALL` inventories remain exhaustive;
- ordinary APIs do not expose raw normalized feature keys;
- runtime crates perform no file or network access; and
- existing dictionary, registry, attribution, and accuracy checks remain current.

Run at minimum:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
cargo test --workspace --doc
cargo xtask check-all
cargo publish -p old-church-slavonic --dry-run --allow-dirty
```

If a new public package is affected, also run its crates.io dry run. Do not
commit, push, tag, or publish unless separately requested.

## Deliverable

Report:

- the resulting public API;
- deliberate breaking changes;
- which Ruthenian ideas were adopted;
- which Ruthenian behaviors were rejected and why;
- verification commands and results; and
- remaining source-data limitations, especially for pronouns and numerals.
