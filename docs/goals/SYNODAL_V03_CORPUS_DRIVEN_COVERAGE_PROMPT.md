# Implement Synodal v0.3 Corpus-Driven Coverage

Implement the next major coverage milestone for the Synodal Russian Church
Slavonic workspace in `church-slavonic`.

The objective is to turn the existing 61-lexeme seed library into a practical,
evidence-backed dictionary and inflection system driven by real Synodal texts.
Preserve the existing recension boundaries, provenance model, typed failures,
generation policies, deterministic data pipeline, offline runtime, and
WASM-compatible library crates.

Backwards compatibility and breaking semver are not concerns.

## 1. Inspect the existing architecture

Before changing code, read:

- `README.md`
- `docs/SYNODAL_RECENSION.md`
- `docs/SYNODAL_MORPHOLOGY.md`
- `docs/SYNODAL_ORTHOGRAPHY.md`
- `docs/SYNODAL_DATA_PIPELINE.md`
- `docs/SYNODAL_REQUIREMENTS.md`
- `docs/SYNODAL_V02_IMPLEMENTATION_AUDIT.md`
- `reports/synodal-evaluation.md`
- the existing `ocs-dict` implementation
- all Synodal core, facade, dictionary, extractor, and `xtask` crates

Reuse the useful ergonomics of `ocs-dict`, but do not import Old Church Slavonic
forms as Synodal attestations. OCS evidence may only enter through the existing
reviewed inheritance and recension-mapping system.

Do not weaken `Strict`, silently guess missing metadata, or classify generated
forms as attested.

## 2. Build the `synodal-dict` CLI

Add a production-quality `synodal-dict` executable. It may live in
`synodal-church-slavonic-dictionary` or a dedicated workspace crate, provided the
runtime libraries remain free of filesystem and network access and continue to
compile for `wasm32-unknown-unknown`.

Implement these commands:

### `search`

Search reviewed English glosses and Synodal lemmas.

Support:

- exact and fuzzy gloss search;
- part-of-speech filtering;
- deterministic ranking;
- stable lexeme and sense IDs;
- human-readable and JSON output;
- source, recension, semantic-review, and attestation information.

### `show`

Display one lexeme by lemma or stable ID.

Include:

- senses and examples;
- lexical class;
- number restrictions, animacy, aspect, and other applicable metadata;
- verb principal parts;
- accent and positional-spelling metadata;
- exact forms and irregular overrides;
- source citations and recension mappings;
- supported and unsupported morphological systems.

Ambiguous lemma requests must return all candidates or require a stable ID.

### `analyze`

Analyze a Synodal surface form without silently choosing one interpretation.

Support:

- `Strict`, `Productive`, and `Exploratory` policies;
- expanded and printed/liturgical orthographic profiles;
- exact, normative, inherited, analogical, abbreviation, and spelling-variant
  analyses;
- all compatible lexemes and typed cells;
- confidence, assumptions, contradictions, evidence, and rule traces;
- human-readable and JSON output.

### `lint`

Validate application or game vocabulary manifests.

Check:

- Unicode and Church Slavonic orthography;
- known lexeme identity;
- expected part of speech and semantic sense;
- requested morphological cells;
- ambiguity;
- unsupported formations;
- missing accents or printed-spelling metadata;
- Latin or later-language fallback text.

Provide deterministic diagnostics, source locations, useful exit codes, and
machine-readable output.

### `check-text`

Tokenize and analyze arbitrary rendered Church Slavonic text.

For every lexical token, report whether it is:

- an exact reviewed form;
- a normative generated form;
- an inherited prediction;
- an abbreviation expansion;
- a recognized spelling variant;
- ambiguous;
- unresolved.

Preserve original text, normalized text, byte/span locations, combining marks,
titla, punctuation, capitalization, and passage boundaries.

Offer configurable thresholds such as `--max-unknown`, `--max-ambiguous`, and
`--strict`.

### `coverage`

Analyze normalized corpora or passage-bearing TSV/JSONL input and produce
frequency-ranked coverage reports.

Report:

- token and type coverage;
- exact, normative, inherited, and exploratory coverage;
- top-1 and top-k analysis;
- ambiguity and abstention;
- counts by morphological system, lexeme, corpus, source, and policy;
- the most frequent unresolved forms;
- a proposed review queue;
- human-readable Markdown plus deterministic JSON/TSV.

Do not duplicate raw-source adapters inside the CLI. Use the existing extractor
and `xtask` pipeline to convert locked sources into normalized passage-bearing
coverage input.

## 3. Classify every unresolved token

Create a typed, serializable gap model. Every unresolved lexical token must
receive one deterministic primary classification:

- `UnknownLexeme`
- `MissingDeclensionOrClass`
- `MissingVerbPrincipalPart`
- `UnsupportedFormation`
- `MissingAccentOrOrthographicMetadata`
- `AmbiguityOrSpellingVariant`

Allow secondary reasons, but define and document a stable precedence rule when
multiple problems apply.

Each gap record must retain:

- original and normalized surface text;
- corpus, source, edition, passage, and partition;
- token position;
- candidate lexeme IDs;
- requested or inferred morphological system;
- policy and orthographic profile;
- resolver trace;
- missing metadata fields;
- frequency and document frequency;
- suggested next review action.

Punctuation, whitespace, and recognized numeral tokens may be classified
separately as non-lexical. A lexical token may not disappear into a generic
“other” bucket.

## 4. Run coverage over real pinned texts

Use only sources already pinned by immutable URL/revision, size, format, and
checksum, or add new sources through the existing lock-preserving source
workflow.

Run coverage over available Synodal biblical and liturgical texts. Keep:

- corpus and edition identity;
- passage boundaries;
- source recension;
- raw and normalized forms;
- training, review, and evaluation partitions.

Do not treat OCS, modern Russian, Serbian, or mixed historical dictionary
material as Synodal surface attestation.

Generate committed, deterministic reports containing:

- overall coverage;
- frequency-ranked gap categories;
- top unresolved lemmas and surface forms;
- missing principal parts and lexical classes;
- unsupported formations by frequency;
- accent and printed-orthography gaps;
- ambiguity and variant clusters;
- source-specific differences;
- the ordered lexical review queue.

Add an `xtask` command that reproduces these reports from the pinned source
cache.

## 5. Expand the reviewed registry

Grow the reviewed Synodal registry from 61 lexemes toward at least 500
high-frequency, useful lexemes.

Never pad the registry with weakly supported entries merely to reach the
numerical target. Every admitted lexeme must have:

- stable identity;
- normalized Synodal lemma;
- part of speech;
- independently reviewed semantic sense;
- target-recension evidence or an explicit reviewed inheritance decision;
- lexical class and restrictions where applicable;
- provenance and citations;
- sufficient metadata for every enabled productive rule.

Prioritize entries according to real corpus frequency and repeated coverage
failures, especially:

- irregular, suppletive, and defective verbs;
- independently sourced verb principal parts;
- mixed, consonantal, and heteroclitic nouns;
- stem alternants and number-restricted nouns;
- reflexive, relative, interrogative, indefinite, and negative pronouns;
- irregular and velar-stem determiners;
- collective, compound, and irregular numerals;
- productive and irregular accent classes;
- breathing marks and positional-letter realization;
- common semantic abbreviations and nomina sacra;
- high-frequency spelling variants.

Automatically extracted candidates must remain candidates until their evidence,
recension, semantics, and morphology have been reviewed. Preserve rejected
candidates and reasons.

## 6. Add productive rules conservatively

Implement a new productive rule only when:

1. the coverage report demonstrates a repeated, meaningful gap;
2. a Synodal normative source specifies the rule;
3. the rule has a stable ID and exact citation;
4. its input metadata contract is explicit;
5. alternations, accent behavior, restrictions, and exceptions are documented;
6. representative real-world lexemes test it;
7. unsupported inputs continue to return typed failures.

Do not derive past stems from present classes, choose an aorist from aspect,
infer participles from an undifferentiated verb stem, or substitute OCS endings
for missing Synodal rules.

Rules must generate sourced predictions, never false attestations.

## 7. Expand real-world evaluation

Increase passage-disjoint evaluation from 38 cells to several hundred
independently reviewed cells.

The evaluation must include:

- nouns from every implemented class;
- short and long adjectives;
- comparison;
- all supported pronoun, determiner, and numeral systems;
- present, imperfect, aorist, imperative, infinitive, and l-participles;
- all supported active and passive participles;
- irregular and defective verbs;
- analytic future, perfect, pluperfect, conditional, and passive constructions;
- accents, breathing, positional spelling, abbreviations, and numerals;
- expected abstentions and hostile Unicode cases;
- examples under all three generation policies.

Prevent leakage by separating passages and, where practical, lemmas from
training and review inputs. Add guards proving evaluation passages cannot become
generation evidence.

Report:

- returned-form coverage;
- top-1 and top-k accuracy;
- ambiguity;
- abstention;
- exact versus predicted performance;
- performance by morphological system;
- policy-specific precision and coverage;
- inheritance-specific results and confidence calibration;
- printed/liturgical-profile accuracy;
- results by source and corpus.

Do not present the evaluation as language-wide accuracy. Report raw counts and
limitations.

## 8. Testing and CI

Add:

- CLI integration tests for every command;
- deterministic human and JSON golden outputs;
- tokenizer tests with combining marks, titla, punctuation, abbreviations,
  numerals, and hostile Unicode;
- gap-classification tests covering every category and precedence combination;
- corpus fixture tests with stable hashes;
- data leakage and recension-boundary guards;
- registry integrity and provenance tests;
- exact-form round trips;
- generated-tree cleanliness checks;
- native, no-default-feature, and WASM builds;
- package-content and `cargo publish --dry-run` checks.

Keep default CI bounded and deterministic. Full-source reconstruction should
remain manual or scheduled. Update deprecated GitHub Actions dependencies
encountered while modifying CI.

Run at minimum:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
cargo test --workspace --doc
cargo xtask synodal-fixture-bootstrap
cargo xtask synodal-check
cargo xtask synodal-coverage --offline
```

Also run the clean-cache full bootstrap when the complete pinned source cache is
available.

## 9. Documentation and completion report

Update the public README and Synodal documentation with:

- CLI installation and examples;
- command and file-format reference;
- coverage-category definitions;
- data-review workflow;
- supported morphology;
- current lexical and evaluation coverage;
- remaining unsupported systems;
- reproduction commands;
- source and licensing constraints.

Produce a final implementation audit recording:

- registry size before and after;
- exact-form and sense counts;
- corpus token/type coverage;
- gap frequencies by category;
- evaluation results;
- productive rules added and their normative citations;
- rejected or deferred rule proposals;
- remaining high-frequency gaps;
- all verification commands and results.

The implementation is complete only when the CLI, reproducible coverage
workflow, expanded reviewed registry, real passage-disjoint evaluation, tests,
reports, and documentation agree with one another and can be regenerated without
modifying committed source locks.
