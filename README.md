# Historical Church Slavonic libraries for Rust

This workspace contains two deliberately separate language targets:

- `church-slavonic`: rule-first canonical Old Church Slavonic; and
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

`church-slavonic` is a fast, offline, rule-first inflector for canonical
**Old Church Slavonic** (`cu`/`chu`): a pure rule kernel
(`church-slavonic-core` plus the `old-church-slavonic-core` rule engine) with
compact generated residue tables holding only the attested cells the rules do
not reproduce verbatim. `cargo xtask rewrite-pilot-accuracy` replays every
attested cell in `data/extracted` against the facade and requires 100% per
part of speech. Every function returns the primary form as a `String`, with a
`*_variants` companion returning every attested spelling, primary first:

```rust
use church_slavonic::{noun, present, Case, Number, Person};

assert_eq!(noun("градъ", Case::Genitive, Number::Singular)?, "града");
assert_eq!(present("нести", Person::Third, Number::Singular)?, "несетъ");
# Ok::<(), church_slavonic::Error>(())
```

The facade serves nouns, long/short adjectives, all finite tenses,
imperatives, participles (citation and fully declined), l-participles,
infinitives, supines, verbal nouns, closed-class pronouns/numerals/
determiners, value-driven numerals (1–10,000), typed phrase constructions,
and whole-paradigm enumeration. Because unseen lemmas run through the same
rules, it inflects words it has never stored. Unknown lemmas and
underdetermined cells return typed errors — no empty-string holes. See
[`crates/church-slavonic/README.md`](crates/church-slavonic/README.md) for
the full API tour.

Orthography lives in `church-slavonic-orthography`: script detection and
lookup normalization, plus the reversible normalized Jagić/TN41 Glagolitic
profile where every non-reversible mapping is reported or rejected. See
[docs/ORTHOGRAPHY.md](docs/ORTHOGRAPHY.md).

## Install

The minimum supported Rust version is 1.85.

```toml
[dependencies]
church-slavonic = "0.2"
```

The generated residue tables (under 1 MB total) are compiled into the
package. Runtime crates perform no file, network, JSON, TSV, XML, or Lua
access.

## Semantic dictionary

`church-slavonic-dictionary` is the offline Wiktionary-backed meaning layer,
re-keyed onto the facade's lemma keys (deterministic numeric homograph
suffixes included): English-concept search, OCS senses and examples, and
`lemmatize(form)` returning (lemma key, typed paradigm cell) readings built
by inverting the facade's paradigm enumeration. Meaning and morphology remain
separate evidence layers: a sense links to the inflector only when both
snapshots resolve the same lexical identity.

## Legacy crates

The pre-rewrite facades remain published on crates.io as
`old-church-slavonic` 0.6.0 and `old-church-slavonic-dictionary` 0.3.0; both
carry succession notices pointing here and receive no further releases. Their
rule kernel survives as this workspace's `old-church-slavonic-core`, and
`docs/DEPRECATION_MAP.md` maps their surface onto `church-slavonic`.

## Data and maintenance

The current pinned snapshot contains 3,081 lexemes, 134,761 public feature
cells, 137,406 ordered variants, and 3,157 normalized verb metadata fields.
The source identity and transformation record are in `data/SOURCES.toml`,
generated metadata, and each package's `ATTRIBUTION.md`. Wiktionary-derived
data is CC BY-SA 4.0; code is MIT OR Apache-2.0.

```bash
cargo xtask check-registry
cargo xtask check-dictionary
cargo xtask rewrite-pilot-accuracy
cargo xtask check-structure
cargo xtask check-all
```

The workspace also contains the offline extractor and `xtask`. It intentionally
does not perform implicit script conversion, diplomatic manuscript transcription,
OCR, or abbreviation expansion. Syntax and phrase-valued morphology use separate
typed construction APIs rather than the single-word inflector.
