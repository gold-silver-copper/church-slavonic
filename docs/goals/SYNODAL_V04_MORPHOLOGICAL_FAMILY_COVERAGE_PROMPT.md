# Implement Synodal v0.4 Morphological-Family Coverage

Continue improving the Synodal Russian Church Slavonic implementation in the
`church-slavonic` workspace.

The v0.3 baseline contains:

- 505 reviewed lexemes and senses;
- 754 exact forms;
- 437 passage-disjoint evaluation cells;
- 1,313,344 analyzed corpus tokens;
- 404,452 top-1 analyses;
- 530,005 top-k analyses;
- 782,134 unresolved tokens.

The principal limitation is no longer merely missing headwords. Most recently
added lexemes have only one exact `LexicalForm`, so they do not generalize to
related inflected forms. The v0.4 objective is to convert high-frequency exact
entries and unresolved surfaces into independently specified, evidence-backed
morphological families.

Do not improve metrics by silently importing OCS paradigms, guessing classes,
weakening `Strict`, or treating generated forms as attestations.

## 1. Audit the current implementation

Read before editing:

- `SYNODAL_V03_CORPUS_DRIVEN_COVERAGE_PROMPT.md`
- `docs/SYNODAL_V03_IMPLEMENTATION_AUDIT.md`
- `docs/SYNODAL_CLI_AND_COVERAGE.md`
- `docs/SYNODAL_MORPHOLOGY.md`
- `docs/SYNODAL_DATA_PIPELINE.md`
- `reports/synodal-coverage.json`
- `reports/synodal-evaluation.json`
- `data/synodal/lexical_reviews.tsv`
- all Synodal core, facade, dictionary, extractor, and `xtask` code.

Record the exact baseline before changing generated data.

## 2. Build a morphological-family review queue

Add a deterministic `xtask` workflow that groups unresolved surface forms into
probable lexeme families.

The grouping must consider:

- normalized and printed spelling;
- accentless spelling, while preserving explicit marks;
- positional letter variants;
- prefixes and common derivational boundaries;
- possible nominal case/number endings;
- possible finite-verb, imperative and participial endings;
- known stems and principal parts;
- abbreviation expansions;
- candidate dictionary identities;
- corpus context and passage identity.

Every proposed family must include:

- a stable candidate ID;
- proposed lemma and part of speech;
- all associated corpus surfaces;
- token and document frequencies;
- source, corpus, edition, passage and partition;
- possible morphological cells;
- compatible existing lexemes;
- evidence supporting or contradicting the grouping;
- missing metadata;
- confidence and assumptions;
- a review status and explicit reason.

Surface similarity must never automatically establish lexical identity.
Preserve ambiguous and rejected groupings.

Add coverage dimensions for:

- unresolved tokens by probable family;
- tokens recoverable through exact evidence;
- tokens recoverable through a reviewed class;
- tokens recoverable through a reviewed principal part;
- abbreviation and spelling-variant families;
- remaining ungrouped unknowns.

## 3. Review the highest-impact families

Start with the most frequent repeated gaps, especially:

- `рещи`, including `Рече́` and its independently sourced aorist system;
- `весь`, including forms such as `всѧ̀`;
- `сынъ`, including consonantal and plural forms such as `сы́нове`;
- `землѧ`, including its declension and printed accent behavior;
- `ꙗкѡ` and `ꙗкоже`, preserving adverb/conjunction ambiguity;
- common forms of `быти`, `имати`, `дати`, motion verbs and irregular verbs;
- common pronoun, determiner and numeral families;
- high-frequency nomina sacra and contractions such as the `господь` family;
- high-frequency positional, breathing and accent variants.

Review at least the top 200 family proposals. Aim to admit approximately 100
useful, fully specified families, but do not pad the registry when evidence is
insufficient.

For every admitted inflectable family, require the applicable metadata:

- declension or conjugation class;
- independently identified stems;
- gender, animacy and number restrictions;
- stem alternants;
- aspect and defectiveness;
- verb principal parts for each enabled system;
- accent class or exact cell accents;
- printed positional spelling;
- irregular overrides and exceptions;
- stable source citations;
- target-recension status.

If only one surface is supported, retain it as exact `LexicalForm` evidence
rather than inventing a paradigm.

## 4. Improve abbreviation support

Create an evidence-backed abbreviation registry for frequent Synodal
contractions and nomina sacra.

Each abbreviation entry must specify:

- semantic lexeme identity;
- expanded and printed forms;
- grammatical restrictions;
- reversibility;
- required titla and superscript letters;
- contextual or positional restrictions;
- ambiguity;
- exact evidence and source recension.

Do not implement blind string replacement. Expansion must preserve possible
lexical and morphological analyses.

Coverage must report abbreviation families separately from unknown lexemes and
ordinary spelling variants.

## 5. Implement productive morphology conservatively

Implement a productive rule only when:

1. repeated corpus gaps demonstrate its value;
2. a Synodal normative source specifies it;
3. required input metadata is explicit;
4. alternations, accent behavior and exceptions are documented;
5. multiple real lexemes test the rule;
6. passage-disjoint evaluation tests its output;
7. unsupported inputs continue to fail explicitly.

Prefer exact irregular tables when no safe productive rule exists.

Do not:

- derive past stems from present classes;
- infer an aorist from aspect;
- create participles from an unspecified generic stem;
- infer accent position from surface frequency alone;
- import OCS endings as Synodal forms;
- interpret a corpus occurrence as proof of an entire paradigm.

Generated forms must retain rule IDs, citations, confidence, traces and
prediction status.

## 6. Extend the API and CLI

Add family-oriented capabilities to `synodal-dict` and the public dictionary
API.

Useful additions include:

- `families QUERY` — show known and proposed family members;
- `show-family ID` — display surfaces, cells, evidence and missing metadata;
- `coverage --by-family`;
- family IDs and family summaries in JSON;
- probable-family diagnostics for unresolved tokens;
- recovery estimates showing which reviewed metadata would resolve the most
  tokens.

Never silently choose a family when multiple candidates remain.

Keep library crates offline, filesystem-free and WASM-compatible. Candidate and
corpus workflows must remain in the CLI or `xtask`.

## 7. Expand real-world evaluation

Add passage-disjoint tests for every newly admitted family and rule.

Include:

- unseen forms of newly classed nouns and adjectives;
- irregular and consonantal noun families;
- irregular verbs and independently sourced principal parts;
- pronouns, determiners and numerals;
- abbreviations and their expanded forms;
- marked, unmarked and incorrectly marked spellings;
- positional letter variants;
- ambiguous homographs;
- expected abstentions;
- malformed combining marks and hostile Unicode.

Separate tests for:

- exact attestation lookup;
- productive generation;
- reverse analysis;
- printed orthography;
- family grouping;
- abbreviation expansion;
- strict-policy abstention.

Prevent passages used as lexical or morphological evidence from entering
evaluation.

## 8. Coverage objectives

Aim for, but do not manipulate data to guarantee:

- top-k corpus coverage above 60%;
- unresolved-token coverage below 40%;
- at least 100 additional fully specified families;
- meaningful reduction in the top 100 unresolved surfaces;
- no loss of strict-policy precision;
- no increase in false target-recension attestations.

If evidence is insufficient to reach a target, report the truthful result and
explain the blocker.

Report both absolute counts and percentages. Compare v0.3 and v0.4 using
identical pinned corpus input.

## 9. Reproducibility and CI

Add deterministic commands that regenerate:

- the family review queue;
- reviewed family registries;
- coverage by family;
- abbreviation coverage;
- expanded evaluation;
- the final audit.

Run at minimum:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
cargo test --workspace --doc
cargo xtask synodal-fixture-bootstrap
cargo xtask synodal-check
cargo xtask synodal-coverage --fixture --offline --check
cargo xtask synodal-coverage --offline --check
cargo xtask synodal-lexical-review-queue --check
cargo xtask synodal-evaluation-queue --check
cargo check -p synodal-church-slavonic-core --no-default-features
cargo check -p synodal-church-slavonic --no-default-features
cargo check -p synodal-church-slavonic-dictionary --no-default-features
```

Also run WASM checks, package-content checks, publish dry-runs and the complete
offline bootstrap from the pinned source cache.

## 10. Completion audit

Create `docs/SYNODAL_V04_MORPHOLOGICAL_FAMILY_AUDIT.md` containing:

- before-and-after registry counts;
- exact-only versus fully classed lexeme counts;
- family review decisions and rejections;
- new principal parts, classes and abbreviation entries;
- every new productive rule and normative citation;
- corpus coverage before and after;
- coverage recovered by each family or rule;
- evaluation results by system and policy;
- remaining high-frequency gaps;
- deferred proposals and reasons;
- all verification commands and results.

The milestone is complete only when reviewed data, generated registries,
CLI/API behavior, corpus reports, evaluation, documentation and clean offline
reconstruction agree.
