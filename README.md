# Historical Church Slavonic libraries for Rust

This workspace contains two deliberately separate language targets:

- `old-church-slavonic`: dictionary-backed canonical Old Church Slavonic; and
- `synodal-church-slavonic`: the normalized Russian Synodal liturgical
  recension, with Synodal morphology, accents, typography, semantic
  abbreviations, analytic phrases, and explicit reviewed OCS inheritance.

The Synodal library is not an orthographic alias for the OCS engine. It has an
independent target registry and rule tables, and every OCS-derived prediction
passes through a stable recension mapping and a Synodal realization rule.

The consumer entry point analyses a passage the registry has never seen —
here Acts 8:30 from the held-out evaluation partition — and returns, for every
token, its readings with lemma, cell, provenance, and confidence, attested and
normative readings always ahead of predictions:

```rust
use synodal_church_slavonic::Inflector;
use synodal_church_slavonic_dictionary::analyze_text;

let passage = "ᲂу҆слы́ша є҆го̀ чтꙋ́ща прⷪ҇ро́ка";
let analysis = analyze_text(passage, Inflector::default())?;
let reading = &analysis.tokens[0].readings[0];
assert_eq!(reading.lexeme.lemma(), "оуслышати");
assert!(matches!(
    reading.cell,
    Some(synodal_church_slavonic::GrammarCell::FiniteVerb(_))
));
# Ok::<(), synodal_church_slavonic::Error>(())
```

The same call is on the command line as `synodal-dict analyze-text TEXT`
(`--policy exploratory` additionally offers typed segmentation hypotheses,
clearly separated from reviewed readings, for tokens with no reading at all).
Single-cell generation stays available on the facade crate:

```rust
use synodal_church_slavonic::{present, Number, Person};

let form = present("быти", Person::First, Number::Singular)?;
assert_eq!(form.primary_text(), "єсмь");
assert_eq!(form.target_recension(), synodal_church_slavonic::Recension::SynodalRussian);
# Ok::<(), synodal_church_slavonic::Error>(())
```

The full source inventory, including 321 individually locked and checksum-verified
machine-readable artifacts (about 4.6 GB), is in `references/`. Raw source bytes
are gitignored and excluded from packages. See `docs/SYNODAL_RECENSION.md`,
`docs/SYNODAL_MORPHOLOGY.md`, `docs/SYNODAL_ORTHOGRAPHY.md`, and
`docs/SYNODAL_DATA_PIPELINE.md` and `reports/synodal-evaluation.md` for the
implemented boundary, reproducible commands, and measured held-out coverage.

## Synodal inflection engine

The engine accepts typed `NounSpec`, `AdjectiveSpec`, `PronounSpec`, and `VerbSpec`
metadata without requiring a dictionary row. Explicit and registry-backed words
share one productive kernel, while exact and irregular cells retain deterministic
precedence. Grammar-backed rules cover complete Synodal noun, adjective,
pronoun, short-comparison, and participial declensions. Pronouns include the
source-defined suppletive, regular, derived, clitic, and prepositional
constructions in Alypy §§45–48. A typed, reusable accent-paradigm model realizes
reviewed stress across multiple generated cells.

Complete specialized paradigms retain structured failures for historical
invalidity, incomplete evidence, missing principal parts/formations, missing
accent metadata, and unsupported systems. See
[`docs/SYNODAL_V08_INFLECTION_ENGINE_AUDIT.md`](docs/SYNODAL_V08_INFLECTION_ENGINE_AUDIT.md)
and the explicit API examples in
[`crates/synodal-church-slavonic/README.md`](crates/synodal-church-slavonic/README.md).

The public `irregular_verb_inventory()` also exposes all 98 verb entries in
Alypy §104 in source order. Closed archaic tables, productive principal-part
contracts, historically impossible cells, and merely incomplete evidence remain
separate, inspectable outcomes.

## Synodal dictionary and coverage checkpoints

The current Synodal registry contains 937 reviewed lexemes, 940 reviewed senses,
and 3,677 generated exact normative or target-attested forms. The `synodal-dict` executable
searches and displays the registry, performs ambiguity-preserving reverse
analysis, displays reviewed and proposed morphological families, validates
application vocabulary, checks rendered text, and creates typed corpus-coverage
reports:

```bash
cargo install --path crates/synodal-church-slavonic-dictionary
synodal-dict search "king" --pos noun
synodal-dict show synodal:verb:byti
synodal-dict analyze 'бꙋ́детъ' --profile printed
synodal-dict families 'весь' --json
synodal-dict show-family family:synodal:determiner:ves --json
synodal-dict check-text rendered.txt --strict
synodal-dict coverage passages.tsv --by-family --json-out coverage.json
```

Coverage is only a downstream regression signal for registered behavior. The
committed live report is reproduced with
`cargo xtask synodal-coverage --offline`; review queues are reproduced with
`cargo xtask synodal-lexical-review-queue` and
`cargo xtask synodal-evaluation-queue`; the locked 1,258-claim lexical union is
reproduced and checked with `cargo xtask synodal-lexical-union`; the family
queue, overlap-adjusted
marginal-recovery report, and audits use
`cargo xtask synodal-family-review-queue` and
`cargo xtask synodal-marginal-recovery`,
`cargo xtask synodal-v04-audit --check`, and
`cargo xtask synodal-v05-audit`; v0.6 family packets and its completed 65% audit
use `cargo xtask synodal-v06-review-packets` and
`cargo xtask synodal-v06-audit --check`. The v0.7 exact-surface acquisition
queue, reviewed-application gate, and completed 70% audit use
`cargo xtask synodal-v07-review-packets`,
`cargo xtask synodal-v07-apply --check`, and
`cargo xtask synodal-v07-audit --check`. That v0.7 audit is an immutable
historical checkpoint; it does not describe the corrected live registry. The locked
corpus now has 919,436 of 1,313,344 tokens in canonical `Strict` top-k coverage
(70.007%), 95 tokens above the 70% gate. See
[`docs/SYNODAL_CLI_AND_COVERAGE.md`](docs/SYNODAL_CLI_AND_COVERAGE.md) for command
and input formats, gap precedence, thresholds, and the evidence-review workflow.
The 2,136-cell morphology evaluation (all expected variants present in top-k),
14 analytic phrases, and 74 typed abbreviation cases remain registered-form
regression suites, not claims of language-wide accuracy.

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
    noun("амемоурмнии", Case::Nominative, Number::Singular),
    Err(InflectionError::AmbiguousLexeme { .. }),
));
assert!(matches!(
    imperative("благословити", Person::Third, Number::Dual),
    Err(InflectionError::HistoricallyInvalidCell { .. }),
));
```

## Paradigms

Lemma-oriented `noun_paradigm`, `adjective_paradigm`, `ordinal_numeral_paradigm`,
`collective_numeral_paradigm`, `present_paradigm`, `finite_paradigm`,
`imperative_paradigm`, `l_participle_paradigm`, and `participle_paradigm`
enumerate the same by-ID resolver as one-cell calls. Typed
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
| Nouns | `noun`, `Noun`, `noun_paradigm` | Exact tables first; dictionary metadata, all 108 reviewed deformed-twofold members, all 37 fixed-gender class-0 substantives with typed reconstruction evidence, or explicit productive rules |
| Adjectives | `long_adjective`, `short_adjective`, `Adjective` | Exact tables first; hard/soft metadata rules; citation comparatives only |
| Determiners | `determiner`, `Determiner`, typed paradigm | Exact pinned dictionary cells only |
| Pronouns | `personal_pronoun_with`, `reflexive_pronoun`, `anaphoric_pronoun`, compatible ordinary functions, `Pronoun` | Complete reviewed personal/reflexive/anaphoric tables; other pronouns use exact source cells |
| Numerals | `numeral`, `cardinal_numeral_identity`, `compound_cardinal`, `ordinal_numeral`, `collective_numeral`, typed paradigms, `Numeral` | Reviewed cardinals through 10,000, all ten simple ordinal adjective paradigms, and the inherited collective series two through ten with pronominal/adjectival cells and direct versus reconstructed evidence; exact fallback for other source-table numeral types |
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
palatalization marks. Cyrillic and Glagolitic remain distinct and transliteration is
never automatic. The explicit `realize_glagolitic` and
`transliterate_glagolitic_to_cyrillic` APIs use the normalized Jagić/TN41 profile;
every non-reversible mapping is reported or rejected. See
[docs/ORTHOGRAPHY.md](docs/ORTHOGRAPHY.md).

OCS stress reconstruction is deliberately separate from canonical output. A
caller may apply an explicit, complete `AccentParadigm` to any generated Cyrillic
cell with `reconstruct_accent`; the returned `ReconstructedAccent` carries its
comparative or disputed status, source citation, paradigm identity, and rule
trace. Existing source accent or breathing marks are preserved only by exact
dictionary lookup and are never overwritten or generalized.

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
does not perform implicit script conversion, diplomatic manuscript transcription,
OCR, or abbreviation expansion. Syntax and phrase-valued morphology use separate
typed construction APIs rather than the single-word inflector.
