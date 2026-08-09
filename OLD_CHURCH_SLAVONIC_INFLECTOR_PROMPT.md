# Build the first Old Church Slavonic inflector

Create a production-quality Rust workspace in this repository for **Old Church
Slavonic** (ISO 639-1 `cu`, ISO 639-2 `chu`; Wiktionary/UD `lang_code` `cu`).
The result should be a fast, embeddable
inflection library in the same family as the sibling `english`, `interslavic-rs`,
and `ruthenian` projects: deterministic offline data extraction,
committed generated lookup data, a small rule engine for out-of-vocabulary words,
an ergonomic public facade, and reproducible accuracy reports.

This is Old Church Slavonic, not a later Russian, Serbian, Croatian, or other
recension of Church Slavonic. Do not silently mix those standards. If a source
contains a later form, exclude or label it rather than treating it as OCS.

The first version must be useful and honest. Prefer a structured “unavailable or
ambiguous” result over a plausible-looking invented form. Wiktionary tables are
dictionary-generated forms, not automatically corpus-attested forms; preserve that
distinction in types, reports, and documentation.

---

## 0. Read the sibling repositories before designing anything

Do not modify the sibling repositories. Read them as architectural evidence. At a
minimum, inspect these files and their associated tests:

### `../english`

- `README.md`, `Cargo.toml`
- `crates/english-core/src/{lib,grammar,noun,adj,adverb,verb,sense_key}.rs`
- `crates/english/src/lib.rs`
- `crates/extractor/src/{pipeline,extract,assign,checks,file_generation}.rs`
- `crates/xtask/src/main.rs`
- the core golden tests, public API tests, sense-stability tests, and generated-data
  synchronization tests

Learn from its table-first/rule-fallback split, deterministic homograph handling,
parse-failure threshold, exception compaction, generated PHF tables, any-key versus
bare-key accuracy metrics, and dump-free registry check.

### `../interslavic-rs`

- `README.md`, `INTEGRATION.md`, `Cargo.toml`
- `crates/interslavic-core/src/{types,paradigm,cells,noun,adjective,verb,pronoun}.rs`
- `crates/interslavic/src/{lib,dictionary,fingerprint}.rs`
- `crates/interslavic-extractor/src/main.rs`
- `crates/xtask/src/main.rs`
- declension, conjugation, no-panic, variant-order, metadata, OOV, and paradigm tests

Learn from its dependency-free core, dictionary-metadata facade, full-paradigm
structs, clean-cell expansion, explicit overrides, whole-dictionary fingerprints,
variant-order contract, OOV tests, and external parity benchmark. Do not copy its
slash-delimited multi-form return convention; return structure instead.

### `../ruthenian`

- `RUTHENIAN_CORE_PROMPT.md`, `DIRECTION.md`
- `docs/{RUTHENIAN,COMPARATIVE_GRAMMAR}.md`
- `crates/ruthenian-core/src/{lib,grammar,lemma,dsl,spelling,noun,adjective,verb,render}.rs`
- `crates/ruthenian-core/tests/{conformance,guards}.rs`

Learn from its explicit source authority, morphology specification, one-generation-
path law, corpus extracted separately from tests, hostile-input checks, executable
guard witnesses, and refusal to patch specification gaps with guesses. Unlike
Ruthenian, OCS is attested and lexically irregular, so do **not** inherit its
“always return a string” totality policy.

After reading, add `docs/ARCHITECTURE.md` with a short “adopted / deliberately not
adopted” comparison. This is a design record, not a tour of the sibling code.

---

## 1. Research and source authority

Before implementing morphology, create `docs/MORPHOLOGY_SPEC.md`. Every rule in the
core must have a stable rule ID and point to a section in this document. The spec
must state which source is authoritative for each claim and record conflicts instead
of silently choosing one.

Use these starting points:

- Kaikki’s machine-readable English-Wiktionary OCS dictionary:
  <https://kaikki.org/dictionary/Old%20Church%20Slavonic/index.html>
- English Wiktionary’s OCS inflection-template inventory:
  <https://en.wiktionary.org/wiki/Category:Old_Church_Slavonic_inflection-table_templates>
- noun templates (currently covering o/jo, a/ja, i, u, n, nt, r, s, and v stems):
  <https://en.wiktionary.org/wiki/Category:Old_Church_Slavonic_noun_inflection-table_templates>
- adjective templates (hard, soft, short/long, and comparative patterns):
  <https://en.wiktionary.org/wiki/Category:Old_Church_Slavonic_adjective_inflection-table_templates>
- verb templates (including IA1, IA2, II1, II2, II3, irregular/root patterns, and
  `бꙑти`):
  <https://en.wiktionary.org/wiki/Category:Old_Church_Slavonic_verb_inflection-table_templates>
- University of Texas Old Church Slavonic Online grammar:
  <https://lrc.la.utexas.edu/eieol_toc/ocsol>
- UD Old Church Slavonic PROIEL, for optional corpus evaluation only:
  <https://universaldependencies.org/treebanks/cu_proiel/index.html>

Wiktionary/Kaikki supplies the versioned dictionary-table target. A grammar source
supplies the generalization implemented by the OOV rule engine. UD supplies attested
surface evidence, not complete paradigms. Do not treat circular agreement with the
same Wiktionary template as independent validation.

The current Kaikki export is a useful bootstrap (roughly 4,600 JSONL entries and a
large number of generated forms), but coverage changes over time. Never hard-code
today’s counts as eternal facts. Generate a source inventory on every refresh.

### Licensing boundary

- Code should be dual-licensed MIT OR Apache-2.0, consistent with the sibling
  libraries.
- Wiktionary-derived data needs its own attribution and applicable CC BY-SA/GFDL
  notices. Add `ATTRIBUTION.md`, data-license notices, source URLs, dump dates,
  revision identifiers where available, and hashes.
- UD OCS PROIEL is currently CC BY-NC-SA 4.0. Do **not** bundle it in the runtime
  crates or published package. Support it as an optional local evaluation input and
  document the license. Do not silently turn the runtime artifact into a
  noncommercial dataset.
- Do not copy copyrighted grammar tables or prose wholesale. Reimplement rules from
  understood descriptions, cite them, and use small attributed examples.

---

## 2. Deliverable and workspace shape

Build this workspace:

```text
Cargo.toml
README.md
LICENSE-MIT
LICENSE-APACHE
ATTRIBUTION.md
docs/
  ARCHITECTURE.md
  MORPHOLOGY_SPEC.md
  DATA_PIPELINE.md
  ORTHOGRAPHY.md
data/
  SOURCES.toml
  overrides.tsv
  extracted/                 # compact, committed normalized source registry
reports/
  extraction-coverage.md
  extraction-coverage.json
  accuracy.md
  accuracy.json
crates/
  old-church-slavonic-core/
    Cargo.toml
    README.md
    src/
      lib.rs
      grammar.rs
      orthography.rs
      trace.rs
      noun.rs
      adjective.rs
      verb.rs
      pronoun.rs
  old-church-slavonic/
    Cargo.toml
    README.md
    build.rs
    generated/
    src/
      lib.rs
      dictionary.rs
      lookup.rs
      paradigm.rs
    tests/
    examples/
  old-church-slavonic-extractor/
    Cargo.toml
    src/
      main.rs
      schema.rs
      normalize.rs
      extract.rs
      validate.rs
      emit.rs
      report.rs
    tests/fixtures/
  xtask/
    Cargo.toml
    src/main.rs
```

Names may be split into additional cohesive modules, but keep the four-crate roles:

1. `old-church-slavonic-core`: pure rule engine; no I/O or bundled dictionary.
2. `old-church-slavonic`: public table-backed facade.
3. `old-church-slavonic-extractor`: offline source parser and deterministic code/data
   generator.
4. `xtask`: repeatable maintainer workflows.

Runtime crates must not read files, contact the network, invoke Lua, or parse JSON.
Generated data is committed and compiled into the public crate. Start with PHF or
equally deterministic static lookup structures; measure before inventing a custom
format.

Use `#![forbid(unsafe_code)]`. Keep the core dependency-free if practical. A narrowly
scoped Unicode-normalization dependency in the facade/extractor is acceptable and
must be justified in `ARCHITECTURE.md`.

---

## 3. V0.1 linguistic scope

### 3.1 Nouns

Support the seven cases and three numbers actually needed by OCS:

```rust
pub enum Case {
    Nominative,
    Genitive,
    Dative,
    Accusative,
    Instrumental,
    Locative,
    Vocative,
}

pub enum Number { Singular, Dual, Plural }
pub enum Gender { Masculine, Feminine, Neuter }
pub enum Animacy { Animate, Inanimate }
```

Dictionary-backed lookup must preserve every accepted form and its source order.
The rule engine should cover the productive and documented o/jo, a/ja, i, and u
classes first, followed by the consonant stems (n, nt, r, s, v) where the source spec
is sufficient. Do not infer a rare stem class from the final letter when more than one
class is plausible. Provide an explicit metadata override API.

Support singularia, dualia, and pluralia tantum as lexical metadata. Preserve
animate/inanimate accusative variants rather than choosing one without evidence.

### 3.2 Adjectives and adjective-like participles

Support:

- short/simple and long/compound adjective paradigms;
- hard and soft stems;
- case × number × gender agreement;
- animacy where the masculine accusative distinguishes it;
- dictionary-listed comparative forms when present.

Call the historical distinction `AdjectiveForm::Short | Long`; explain terminology
in the docs. Do not call every long form semantically “definite” without qualification.
Superlative and phrase-level comparison may remain out of scope unless the cited
sources define a single-word morphological result.

Participles that decline adjectivally must reuse the adjective inflection path. Do not
maintain a second set of adjective endings inside `verb.rs`.

### 3.3 Verbs

The first version should expose, when source data or rules support them:

- present indicative;
- imperfect;
- aorist, preserving multiple listed aorist variants rather than collapsing them;
- imperative;
- infinitive and supine;
- l-participle by number and gender;
- present active, present passive, past active, and past passive participles;
- verbal noun/citation derivative only when explicitly supplied or generated by a
  documented rule.

Use separate request types for finite verbs, imperatives, l-participles, and declined
participles. Do not create one giant Cartesian-product API in which most combinations
are meaningless.

```rust
pub enum Person { First, Second, Third }
pub enum FiniteTense { Present, Imperfect, Aorist }
pub enum ParticipleKind {
    PresentActive,
    PresentPassive,
    PastActive,
    PastPassive,
}
```

Compound perfects, pluperfects, futures, conditionals, auxiliary selection, clitic
placement, and sentence-level agreement are composition/syntax and are not required
for v0.1. Supply their morphological components without pretending the whole phrase is
one inflected word.

Bare infinitives often do not determine every required stem or conjugation class.
Expose `verb_with`/`VerbLexeme` metadata for class, present stem, aorist stem, and
documented irregular overrides. Dictionary lookup fills those facts when known. For
unknown lemmas, return `MissingLexicalMetadata` when classification is ambiguous;
never silently select the most common class and label it authoritative.

### 3.4 Pronouns and numerals

Include dictionary-backed personal, reflexive, demonstrative, relative, and numeral
paradigms when their tables can be extracted reliably. Closed-class suppletion belongs
in compact tables, not suffix heuristics. OOV pronoun/numeral generation is not a v0.1
goal.

### 3.5 Explicit non-goals

- morphological analysis (surface form → lemma/features);
- phrase or sentence realization;
- later Church Slavonic recensions;
- diplomatic manuscript transcription, OCR cleanup, or abbreviation expansion;
- automatic conversion of every Cyrillic form to Glagolitic or vice versa;
- reconstructed accent placement;
- claiming that a Wiktionary-generated table cell is attested in a manuscript;
- hiding unsupported cells behind an em dash string.

---

## 4. Public API: structured, provenance-aware, and ambiguity-safe

Design the exact names idiomatically, but preserve these semantics:

```rust
pub struct FormVariant {
    pub text: String,
    pub romanization: Option<String>,
}

pub struct FormSet {
    pub lemma: String,
    pub variants: Vec<FormVariant>,
    pub source: FormSource,
    pub warnings: Vec<InflectionWarning>,
}

pub enum FormSource {
    DictionaryTable,
    DictionaryMetadataRule { rule_id: RuleId },
    ExplicitMetadataRule { rule_id: RuleId },
    OovPrediction { rule_id: RuleId },
    ManualOverride,
}

pub enum InflectionError {
    InvalidInput { reason: String },
    UnknownLemma,
    AmbiguousLexeme { candidates: Vec<LexemeSummary> },
    MissingLexicalMetadata { needed: Vec<MetadataField> },
    UnsupportedCell,
}
```

Required operations:

- lookup all lexemes for `(lemma, part of speech)`;
- request a single noun/adjective/verb cell;
- request a complete paradigm;
- provide an explicit lexical specification for OOV or ambiguous words;
- inspect which dictionary record/class/rule produced a result;
- optionally obtain a compact rule trace for predicted forms;
- retrieve all variants without parsing punctuation embedded in a string.

`paradigm()` and single-cell calls must share one generation path. A paradigm builder
must enumerate calls to the same cell resolver, or both must call one internal
resolver. Test this exhaustively.

Do not encode missing cells as `"-"`, `"—"`, an empty string, or an empty variant.
Do not join alternatives with `/`. Do not silently choose the first homograph.

If an ergonomic convenience API returns the primary variant, name the policy, expose
the complete `FormSet`, and pin primary ordering with tests. Variant order is API and
must follow source order, with exact duplicates removed stably.

---

## 5. Orthography and Unicode are part of correctness

Write `docs/ORTHOGRAPHY.md` before implementing lookup folding.

The data contains historical Cyrillic, combining marks, abbreviations, and some
Glagolitic. For example, a page key may be `царь` while the displayed canonical form
is `цар҄ь`. Preserve both rather than destructively normalizing the display form.

Maintain at least three concepts:

1. raw source spelling;
2. canonical display spelling;
3. lookup key/aliases.

Requirements:

- choose and document NFC or another explicit Unicode normalization form;
- never strip yers, nasal vowels, jat, palatalization marks, titla, or accents merely
  because they are inconvenient;
- create aliases from explicit source relationships (page word, canonical head, and
  listed alternatives) before considering a general fold;
- keep Cyrillic and Glagolitic display forms intact; do not assume a mechanically
  transliterated string is source-attested;
- preserve source romanization as metadata; if algorithmic romanization is added,
  label it separately;
- test decomposed/composed sequences, combining-mark order, uppercase input, empty
  input, punctuation, non-OCS text, and very long input;
- make lookup normalization identical in extractor and runtime by putting the shared
  pure logic in the core or another single shared module.

No lossy normalization may be used for exact accuracy. If a looser key is useful,
report exact and folded metrics separately and detect collisions.

---

## 6. Data extraction and generated registry

The extractor must accept an explicit input path. Normal builds and tests must never
download data. A separate, explicit network refresh command is allowed, but the
downloaded source must be hashed and recorded.

Start from Kaikki’s OCS JSONL or filter the full Wiktextract JSONL on
`lang_code == "cu"`. Accept only supported parts of speech and forms whose source is
the declension/conjugation table. Keep source metadata needed to audit the decision:
lemma, page word, POS, head templates, class marker, original tags, original order,
form, romanization, and source revision/dump identity when available.

Normalize into a committed registry with a schema version, conceptually:

```text
lexeme_id  lemma  pos  class  feature_key  variant_rank  form  romanization  source
```

The exact format may be TSV or another reviewable deterministic format. One component
owns parsing and serialization. Generated Rust is a pure deterministic function of
the committed normalized registry plus `data/overrides.tsv`.

### Strict schema audit

Current OCS Wiktextract output contains a serious trap: some verb finite forms carry
spurious `l-participle` and/or `error-unrecognized-form` tags, and some lack person
tags even though table order still implies a person. Verify this against the current
snapshot; do not assume the bug remains identical forever.

Do not solve this by deleting the error tag and guessing. Implement one of these and
document why it is sound:

- parse versioned OCS conjugation-template invocations from a pinned Wiktionary source
  snapshot and generate the cells from a separately specified class system;
- use a strictly validated positional table-block mapper whose complete expected
  shape is fixture-tested and fails closed on any unknown shape;
- temporarily exclude unsafe verb blocks with explicit drop reasons and limit the
  advertised verb coverage.

Whichever path is chosen, validate representative rendered paradigms independently
against pinned Wiktionary page revisions and grammar examples. Unknown tag
signatures, class names, template shapes, missing required dimensions, duplicate
feature keys with contradictory forms, or a sudden coverage drop must fail the
refresh—not disappear silently.

The extractor must:

- stream input rather than loading a multi-gigabyte dump into memory;
- count total lines and parse failures;
- abort above a documented parse-failure threshold and always print nonzero failures;
- reject table sentinels such as `-`, `no-table-tags`, class labels, template names,
  and empty forms from the public registry;
- preserve legitimate multiple variants and their source order;
- assign deterministic lexeme IDs/sense keys based on documented content, not input
  iteration order;
- report ID changes on refresh and never promise semantic permanence that the source
  cannot provide;
- emit sorted, byte-stable output;
- write atomically so a failed refresh cannot leave half-generated tables;
- generate a machine-readable extraction report plus a concise Markdown report.

`reports/extraction-coverage.*` must include counts by POS, class, cell type, accepted
tag signature, rejected tag signature, drop reason, ambiguity, script, and source.
Pin semantic minimums or maximum drop rates only after measuring the first accepted
snapshot. Do not hide a drop with a changed denominator.

`data/SOURCES.toml` must record source URL, dump date, extraction date if distinct,
byte length, SHA-256, Wiktextract version/commit when known, registry schema, and the
command used to refresh it.

---

## 7. Rules, dictionary precedence, and overrides

Resolution order:

1. exact manual override for a known bad or genuinely lexical cell;
2. exact dictionary table cell;
3. dictionary metadata + rule engine;
4. explicit caller metadata + rule engine;
5. conservative OOV inference only when classification is unambiguous;
6. typed error explaining what is missing.

Manual overrides live in `data/overrides.tsv` with lemma, POS, complete feature key,
ordered variants, reason, source citation, and reviewer status. Keep this file small.
An override without a reason/source fails validation.

Every productive rule has:

- a stable `RuleId`;
- a cited `MORPHOLOGY_SPEC.md` section;
- direct unit tests for its seam changes/palatalizations;
- class-level golden paradigms;
- an OOV holdout measurement.

Do not port Wiktionary Lua templates line-for-line into Rust and then call agreement
with Wiktionary independent accuracy. The rules should be understandable morphology,
while the template output remains one data target.

Keep rule declarations separate from plumbing. Ordered spelling changes should be
visible as ordered data/functions, not scattered string replacements. At minimum,
make stem selection, ending selection, and morphophonemic seam changes separately
testable.

---

## 8. Correctness and evaluation

Publish separate measurements; never combine them into one inflated “accuracy”
number.

### A. Dictionary registry round-trip

For every accepted dictionary cell, call the **public facade** and require the source
variant to be reachable for the correct lexeme and features. Target 100% reachability
for accepted cells. This measures table/generator/runtime integrity, not linguistic
generalization.

Also report:

- primary-variant accuracy;
- variant-order mismatches;
- ambiguous bare-lemma calls;
- cells served by table, metadata rule, override, or fallback;
- excluded cells and exact reasons.

### B. Rule-engine OOV evaluation

Evaluate `old-church-slavonic-core` without access to the facade tables. Use a
deterministic lemma-level split so forms of one lemma never occur in both development
and test. Report exact and documented-normalized recall by POS, class, and cell.
Include macro averages so the common o-stem nouns cannot hide failure on small
classes.

Do not tune on a repeatedly inspected “sealed” test set. Record which split is
development and which is final evaluation. A grammar rule may be adopted with a known
tradeoff, but report fixed/broken cell counts and inspect regressions.

### C. Optional attested-corpus evaluation

Add `cargo xtask accuracy-ud --path <UD_DIRECTORY>` or equivalent. This command is
optional/local because of UD’s license and must not be needed to build or test the
published crate.

Compare compatible UD feature bundles to generated variants, preserving manuscript
orthography. Report raw exact and explicitly normalized recall separately. UD is not a
complete paradigm and absence is not evidence that a generated form is wrong.

### D. Curated goldens

Commit a small, manually audited fixture set with source revision IDs and cover:

- every noun class supported by the core, hard/soft and animate/inanimate contrasts;
- singular, dual, and plural in all seven cases;
- short and long hard/soft adjectives;
- at least one verb from each supported class, plus `бꙑти` and irregular/root types;
- present, imperfect, aorist, imperative, infinitive, supine, l-participle, and each
  supported participle;
- personal/reflexive pronouns;
- multiple variants in one cell;
- Cyrillic combining marks and at least one source-backed Glagolitic paradigm;
- unsupported/ambiguous queries returning typed errors.

Use examples such as the Wiktionary paradigm of `обѣдъ` only from a pinned revision,
not from a mutable live page during tests.

### E. Semantic invariants and guards

Add guards with a stated minimal failure witness. Required invariants include:

- generated registry is current and deterministic;
- no duplicate lexeme/cell/variant-rank keys;
- no empty/sentinel public forms;
- nominative singular includes the canonical citation form where the paradigm says it
  should, with an explicit exemption registry;
- paradigms and cell getters agree for every bundled lexeme;
- source variant order survives end to end;
- extraction coverage does not silently collapse;
- every public call is panic-free on hostile input;
- runtime crates perform no I/O/network access;
- public crate and generated data carry required attribution;
- committed report numbers match fresh evaluator output.

Actually apply each named witness, observe the guard fail, and revert it before
declaring the guard useful. Record this in the implementation report.

---

## 9. `xtask` workflow

Provide at least:

```text
cargo xtask refresh-data --dump <PATH>
cargo xtask check-registry
cargo xtask extraction-report
cargo xtask accuracy --dump <PATH-or-normalized-registry>
cargo xtask accuracy-ud --path <UD_DIRECTORY>    # optional/local
cargo xtask dump-paradigms [NAME]
cargo xtask diff-paradigms <BEFORE> <AFTER>
cargo xtask examples
cargo xtask speed
cargo xtask check-all
```

`check-registry` must be offline and dump-free: regenerate from the committed
normalized registry into memory/a temporary directory and compare it byte-for-byte to
committed generated files, while also running semantic registry validation.

`check-all` runs formatting, clippy with warnings denied, all workspace tests, doc
tests, registry validation, report freshness checks, and relevant examples. It must
not require a network connection or a private local corpus.

Paradigm dumps must include lexeme identity, POS, complete feature key, ordered
variant rank, and form. Their fingerprint is diagnostic; `diff-paradigms` must make
every changed cell reviewable rather than asking maintainers to bless an opaque hash.

---

## 10. Documentation and examples

The root README must clearly answer:

- what “Old Church Slavonic” means here;
- installation and minimum supported Rust version;
- which parts of speech and cells v0.1 supports;
- exact dictionary-backed versus predicted semantics;
- why a query can be ambiguous or need lexical metadata;
- how Unicode/script normalization works;
- how to request one form and a whole paradigm;
- how variants and provenance are represented;
- how data is refreshed and attributed;
- measured dictionary round-trip and OOV results, with denominators;
- known extraction/source limitations, especially verb-table tagging;
- non-goals and unsupported cells.

Include runnable examples for:

1. a regular dictionary-backed noun such as `обѣдъ`;
2. a dual form;
3. an adjective short/long contrast;
4. a dictionary-backed verb paradigm;
5. an OOV word with explicit class/stem metadata;
6. an ambiguous lookup returning candidates;
7. a multi-variant cell;
8. provenance/rule-trace inspection.

Do not advertise forms or accuracy beyond what the generated reports prove.

---

## 11. Implementation order

Keep every milestone green and reviewable:

1. **M0 — source audit and specification:** sibling comparison, source/licensing
   inventory, `MORPHOLOGY_SPEC.md`, `ORTHOGRAPHY.md`, a sampled Kaikki schema audit,
   and an initial extraction-coverage report. No morphology code yet.
2. **M1 — workspace, grammar types, Unicode keys, and result model:** hostile-input
   tests and normalization collision tests included.
3. **M2 — strict extractor and normalized registry:** nouns/adjectives first; stable
   variants, IDs, provenance, drop reasons, deterministic emission.
4. **M3 — nominal core:** noun and adjective rules, seam changes, explicit metadata,
   class goldens, OOV split evaluation.
5. **M4 — verb extraction audit and core:** solve or explicitly constrain the malformed
   Wiktextract verb-cell mapping; then implement supported classes and non-finite forms.
6. **M5 — public facade and generated tables:** dictionary precedence, homographs,
   full paradigms, traces, examples, no duplicate generation path.
7. **M6 — pronoun/numeral tables, whole-registry sweep, performance, docs, and final
   reports.**

Do not begin M4 by assuming the verb tags are trustworthy. Do not declare a POS
supported merely because its enum or stub exists.

---

## 12. Verification and definition of done

Run, at minimum:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo test --workspace --doc
cargo xtask check-registry
cargo xtask extraction-report
cargo xtask accuracy
cargo xtask examples
cargo xtask speed
cargo package -p old-church-slavonic --allow-dirty
```

V0.1 is done only when:

- the public crate inflects dictionary-backed nouns, adjectives, and the explicitly
  advertised verb subset through generated static data;
- the core predicts documented OOV classes without reading dictionary data;
- all seven cases and all three numbers work where supported;
- ambiguity, missing lexical metadata, and unavailable cells are typed outcomes;
- alternatives are structured and remain in source order;
- every accepted dictionary cell round-trips through the public API;
- OOV accuracy is measured on a lemma-disjoint split and reported per class;
- extractor failures, unsafe verb blocks, and coverage gaps are visible in reports;
- generated files and reports reproduce offline from committed inputs;
- no required verification uses the network or the noncommercial UD corpus;
- attribution and data provenance ship in the package;
- the full generated diff and reports have received a separate correctness review.

In the final implementation handoff, report architecture decisions, supported scope,
source snapshot and licensing, extraction coverage/drop reasons, dictionary round-trip
results, OOV results, verification commands, package size/speed, known risks, and the
next highest-value missing class. Do not summarize a partial extractor as a complete
Old Church Slavonic grammar.
