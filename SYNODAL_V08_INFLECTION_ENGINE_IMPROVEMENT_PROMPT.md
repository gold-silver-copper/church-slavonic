# Implement Synodal v0.8 as an Inflection-Engine Release

Continue the Synodal Russian Church Slavonic implementation in the
`church-slavonic` workspace, but change the optimization target completely.
Build a better inflection engine. Do not pursue a higher corpus-coverage
percentage.

The result must improve productive morphology, irregular and defective
paradigms, accent realization, public API ergonomics, and grammar-backed
behavioral tests. It must let a caller inflect a lexeme from explicit typed
linguistic metadata without requiring that the lexeme already exist in the
reviewed dictionary.

Backwards compatibility and breaking semver are not concerns. Prefer a smaller,
coherent, linguistically honest API over compatibility shims.

Do not commit, push, publish, open or modify a pull request, delete branches, or
otherwise mutate remote state unless the user explicitly requests it. Preserve
unrelated staged, unstaged, and untracked work. The local `.git` metadata may be
read-only or stale even when the equivalent source tree has already been pushed.
Do not reset or discard work based only on a stale local `HEAD`.

## Objective

Deliver a v0.8 engine milestone with these properties:

1. A caller can provide a typed noun, adjective, or verb specification and ask
   for one form or a complete paradigm without first adding a dictionary row.
2. Productive rules operate on explicit class, stem, principal-part, aspect,
   animacy, comparison, and accent metadata. They never infer a class merely
   from a suffix or corpus frequency.
3. Exact and irregular forms override productive rules deterministically, while
   partial irregular paradigms fall back only when a regular background is
   explicitly licensed.
4. Accent behavior can be expressed as reusable, source-backed paradigms rather
   than only as thousands of unrelated per-cell strings.
5. Every newly supported rule is tested through complete historically valid
   cell inventories and authoritative golden paradigms.
6. Unsupported, historically invalid, ambiguous, and metadata-deficient cells
   remain distinct typed outcomes.
7. Corpus coverage is, at most, a downstream regression signal. It is not a
   success metric, prioritization input, or license to add lexical facts.

## Fixed target and linguistic boundary

The target is Synodal Russian Church Slavonic, represented in the repository as
`Recension::SynodalRussian` and the `synodal-church-slavonic-*` crates.

Do not silently mix it with Old Church Slavonic. The OCS crates may be inspected
for architecture or historical comparison, but an OCS ending, class, lexeme,
accent, or surface spelling is not Synodal evidence. Any reused historical
analysis must cross the repository's explicit recension-mapping boundary and
must still be realized through a sourced Synodal rule.

Keep these representations separate:

- lexical expanded spelling;
- inflectional stems and principal parts;
- accent and breathing realization;
- positional printed spelling;
- abbreviation or contraction; and
- lookup normalization.

Do not use a presentation rewrite to conceal a morphological error.

## Frozen corpus-coverage boundary

Treat commit `aa4e693136ef094aab0da6ab166e1f23f49f9792` and the v0.7 audit as the
completed corpus-coverage checkpoint. Its locked result is 919,752 top-k tokens
out of 1,313,344, or 70.031%.

For this task:

- do not set a new top-k target;
- do not work the marginal-recovery queue;
- do not bulk-add exact corpus forms, abbreviations, senses, or lexical
  identities;
- do not rank morphology work by token-frequency gain;
- do not weaken `Strict`, change the tokenizer, change the denominator, or
  broaden lookup normalization;
- do not describe exact-form growth as productive morphology;
- do not regenerate v0.3-v0.7 review packets merely to make metrics move; and
- do not make corpus-report churn part of the minimum deliverable.

If a legitimate engine change alters analysis output for an already reviewed
lexeme, run the existing coverage tools as regression checks and explain the
effect. A coverage decrease is a defect only when it reveals a real behavioral
regression. A coverage increase is incidental and does not prove that the
engine improved.

Keep the v0.7 audit and baselines historically reproducible. If an old audit
command currently conflates a frozen historical snapshot with the live v0.8
engine, separate those concepts rather than rewriting history.

## Current engine baseline

Verify the baseline from the current source rather than trusting this summary,
but begin with these known facts:

- the pure core exposes seven productive noun declensions;
- hard and soft short/long positive adjectives are productive;
- long comparative and superlative forms require an independent comparison
  stem;
- verbs store independent present edges, medial present stem, imperfect stem
  and formation, aorist stem and formation, imperative stem and formation,
  l-participle stem, four tense/voice-specific participial specifications, and
  optional verbal-noun metadata;
- present, imperfect, aorist, imperative, infinitive, l-participle, and declined
  participle generation exist for their documented input contracts;
- simple future is exact-only;
- productive supine and verbal-noun realization are still unsupported;
- special short active-participle nominatives are exact-only;
- pronouns and most cardinal numerals are intentionally closed-class and
  exact-table driven;
- exact cells precede productive generation in the canonical resolver;
- `SynodalLiturgical` currently requires a matching accent registry row for a
  generated form;
- the pure core performs no registry or runtime I/O; and
- the convenient facade resolves dictionary identities before generating a
  form, even though the pure core accepts explicit lexeme structures.

Do not assume every currently implemented table is correct or complete. Audit
the code against its cited normative source before extending it.

## 1. Read and reproduce the engine baseline

Before editing, read completely:

- `docs/SYNODAL_MORPHOLOGY.md`;
- `docs/SYNODAL_ORTHOGRAPHY.md`;
- `docs/SYNODAL_REQUIREMENTS.md`;
- `docs/SYNODAL_RECENSION.md`;
- the relevant source and evidence documentation;
- `crates/synodal-church-slavonic-core/src/grammar.rs`;
- `crates/synodal-church-slavonic-core/src/morphology.rs`;
- `crates/synodal-church-slavonic-core/src/orthography.rs`;
- `crates/synodal-church-slavonic-core/src/error.rs`;
- `crates/synodal-church-slavonic-core/src/result.rs`;
- `crates/synodal-church-slavonic/src/inflector.rs`;
- `crates/synodal-church-slavonic/src/resolver.rs`;
- `crates/synodal-church-slavonic/src/registry.rs`;
- `crates/synodal-church-slavonic/src/handles.rs`;
- `crates/synodal-church-slavonic/src/paradigm.rs`;
- relevant generated-registry inputs, accent rows, principal parts, irregular
  overrides, and exact forms;
- all unit, integration, doctest, and guard tests that define engine behavior;
  and
- applicable repository instructions, including `AGENTS.md`.

Inspect the complete worktree and preserve unrelated changes. Run targeted
baseline tests before editing. Record the exact baseline behavior of every
public engine entry point and the exact inventory of implemented productive
rules.

Do not begin by reading or processing giant corpus queues. They are outside the
optimization target of this task.

## 2. Build a linguistic capability matrix

Create a deterministic, reviewable capability matrix for the engine. The
machine-readable form should be concise enough to review and should generate a
human-readable section in `docs/SYNODAL_V08_INFLECTION_ENGINE_AUDIT.md`.

For every morphological system or subtype, record:

- grammatical category and stable rule ID;
- target recension;
- exact-only, irregular, productive, inherited, or unsupported status;
- complete historically valid cell inventory;
- historically invalid cells;
- required lexical metadata and principal parts;
- stem alternations and seam transformations;
- accent input contract;
- normative source and precise citation;
- at least one positive golden lexeme where available;
- at least one boundary, defective, or negative example where applicable;
- current implementation location;
- current test location; and
- the exact typed failure used when the rule cannot run.

The matrix must distinguish:

- a grammatical category represented by types;
- a productive rule implemented for arbitrary caller-supplied metadata;
- a registered lexeme that happens to use the rule;
- an exact or irregular paradigm;
- a per-cell accent witness; and
- a complete reusable accent paradigm.

Do not mark a row complete because the enum variant exists or because one exact
word form exists.

Use general linguistic value, missing system coherence, and quality of available
normative evidence to choose implementation work. Do not include corpus
frequency or top-k recovery in the prioritization score.

## 3. Add an explicit typed-specification API

Design and implement a first-class public route for inflecting caller-supplied
lexical metadata.

The API should support typed specifications equivalent to the core's noun,
adjective, and verb inputs while presenting an ergonomic, coherent public
surface. A reasonable design may use `NounSpec`, `AdjectiveSpec`, `VerbSpec`,
`AccentSpec`, and builders, but choose names and ownership based on the complete
API rather than this suggestion.

The public route must support:

- generating one requested `GrammarCell`;
- generating a complete specialized paradigm;
- selecting `Expanded`, `ExpandedAccentless`, or `SynodalLiturgical` output;
- attaching explicit source/provenance information to caller-supplied metadata;
- returning stable rule traces;
- validating contradictory class, gender, aspect, formation, and principal-part
  combinations before generation; and
- distinguishing missing metadata from unsupported and historically invalid
  formations.

A user must be able to inflect an unregistered regular word without fabricating
a dictionary identity. The result must say that the output is a sourced or
caller-specified prediction, never an attestation.

The registry-backed convenience API should delegate to the same generation
kernel after identity resolution. Do not maintain two subtly different
inflectors.

Avoid stringly typed public class names and feature keys. Parsing serialized
metadata may use strings at the boundary, but validated engine state must use
closed types.

Add concise README examples for:

1. an unregistered regular noun;
2. an unregistered adjective;
3. a verb with independently supplied principal parts;
4. a complete paradigm with retained failures; and
5. an accented liturgical result with explicit accent metadata.

At least one example must require no dictionary lookup or corpus data.

## 4. Improve productive morphology

Use the capability matrix to select a coherent source-backed implementation
tranche. Do not declare the task complete after only creating the matrix or
refactoring existing functions.

The minimum substantive morphology scope is:

- at least one nominal or adjectival productive gap; and
- at least one verbal productive gap.

Choose gaps that extend reusable behavior across lexemes. Examples worth
investigating include documented stem alternation subtypes, declensional
subclasses not expressible by the current seven noun classes, short comparison,
special participial formations, supine formation, and typed verbal-noun
formation. These are investigation candidates, not permission to implement a
rule without complete Synodal evidence.

If one candidate lacks enough normative evidence for a complete input and output
contract, leave it unsupported, document the precise blocker, and continue to
the next source-backed gap. Do not fill missing evidence with analogy.

A productive rule is complete only when it has:

- a stable rule ID;
- an exact target-recension citation;
- a typed input contract;
- validation for contradictory metadata;
- every historically valid cell in its declared inventory;
- typed historically invalid and unsupported outcomes;
- all documented alternations and ordered variants;
- explicit accent behavior or an explicit requirement for accent metadata;
- provenance and a rule trace on every variant;
- at least one complete golden paradigm; and
- seam-focused negative and boundary tests.

Do not add a rule that only works for one hard-coded lexeme. That belongs in the
irregular layer.

Do not select a past stem from present conjugation or aspect. Preserve
independent verb principal parts. Do not derive a participial stem from a generic
verb stem when the normative morphology requires an independent formation.

## 5. Implement reusable accent paradigms

Replace the assumption that liturgical generation must always find a separate
precomputed accent string for every generated cell with a typed, source-backed
accent mechanism.

Design the model from the normative accent evidence available in the repository.
It must be able to express, where sourced:

- fixed stem stress;
- fixed ending stress;
- cell- or number-conditioned mobility;
- lexical exceptions;
- monosyllabic acute/grave behavior;
- breathing independently from stress; and
- explicit inability to determine an accent.

Do not assume that these categories are sufficient if the sources require a
more precise model. Do not force a modern Russian stress system onto Church
Slavonic.

Resolution order should remain explicit and deterministic:

1. exact reviewed accented/printed cell;
2. reviewed lexical irregular accent override;
3. reviewed reusable accent paradigm applied to a generated expanded form;
4. typed `OrthographicMetadataRequired` or a more precise typed failure.

An accent paradigm must name its evidence and applicable morphological cells.
It must not be inferred from a corpus token or one accidental surface example.
Exact per-cell accent rows may remain when the evidence does not license a
generalization.

Preserve Church Slavonic combining-mark ordering, NFC behavior, breathing before
accent, titlo/superscript validation, and the separation between accented and
printed forms.

The minimum accent deliverable is at least one genuinely reusable reviewed
paradigm that produces multiple cells and is exercised through both the explicit
specification API and the registry-backed API. A renamed collection of per-cell
strings does not satisfy this requirement.

## 6. Strengthen irregular and defective paradigms

Audit how exact forms, irregular overrides, regular backgrounds, and defectivity
interact.

Implement or improve at least one source-backed irregular or defective paradigm
whose behavior exposes a real engine concern, such as:

- a partially irregular paradigm with an explicitly licensed regular
  background;
- a suppletive paradigm;
- a lexeme with a documented missing cell;
- a formation whose valid cell inventory differs from a regular paradigm; or
- ordered normative variants that cannot be represented by one string.

Do not manufacture a full irregular paradigm from scattered corpus forms.

The engine must distinguish:

- an attested exact cell;
- a normative exact table cell;
- an irregular override;
- a regular generated fallback;
- a historically nonexistent cell;
- a cell omitted because evidence is incomplete; and
- an ambiguous set of normative variants.

Ensure that exact and irregular precedence is identical for direct functions,
stable-ID calls, resolved handles, explicit specifications, and complete
paradigms.

## 7. Make paradigms and errors useful to callers

Review `Paradigm`, `ParadigmRow`, `ParadigmStatus`, `Capabilities`, and
`MetadataField` as a coherent public contract.

Improve them where necessary so callers can answer, without parsing error text:

- Which cells are part of this grammatical paradigm?
- Which cells succeeded?
- Which outputs are attested, normative predictions, inherited predictions, or
  caller-specified predictions?
- Which cells are historically invalid?
- Which cells are supported but missing required metadata?
- Which cells are unsupported by the engine?
- Which principal part, class, accent paradigm, or override is missing?
- Which rule and evidence produced each variant?

Prefer closed status enums and structured diagnostics. Error display text should
be useful, but it must not be the machine-readable API.

Do not hide failed paradigm cells. A complete paradigm is a complete inventory
of attempted grammatical cells with structured outcomes.

## 8. Add grammar-backed behavioral tests

Make linguistic behavior, not implementation snapshots or corpus frequency, the
main test oracle.

Add a compact golden fixture or equivalent typed test data containing the exact
normative citation for every example. Cover:

- every newly added productive rule with a complete paradigm;
- at least one representative complete paradigm for every pre-existing
  productive rule touched by the refactor;
- singular, dual, and plural where the rule licenses them;
- all seven cases for declinables, including vocative;
- animacy-dependent accusative ordering;
- every valid person/number cell for finite systems;
- historically invalid imperative and aspect/tense combinations;
- independent present edges and non-present principal parts;
- missing principal parts and contradictory metadata;
- exact/irregular/productive precedence;
- partial irregular fallback and documented defectivity;
- accent mobility and exact accent overrides;
- canonical combining-mark order and hostile Unicode;
- multiple normative variants without silent candidate loss;
- complete paradigm status classification; and
- parity between the explicit-specification and registry-backed routes.

Where useful, add invariant or property tests:

- every declared rule has a nonempty stable ID and evidence citation;
- every produced variant has a target recension, source classification, and
  trace;
- every declared cell inventory is deterministic and duplicate-free;
- a paradigm lookup agrees with direct generation for the same cell;
- exact forms always outrank productive forms;
- accentless rendering never changes historical base letters; and
- invalid or hostile inputs never panic.

Keep core engine tests fast and independent of the full corpus and semantic
dictionary index. Do not make every morphology test rebuild the million-token
coverage analysis.

## 9. Preserve architecture and runtime constraints

Maintain these boundaries:

- `synodal-church-slavonic-core` remains pure and deterministic;
- runtime crates do not read TSV, JSON, XML, Lua, corpus files, or the network;
- reviewed registries remain generated at build/development time and compiled
  into runtime code;
- morphology, accent realization, positional spelling, and abbreviation remain
  separate stages;
- one canonical resolver defines precedence;
- all public word input passes the Unicode validator;
- all output is standard Unicode with no private-use characters; and
- native, no-default-feature, and supported WASM builds remain valid.

Do not solve API ergonomics by exposing raw generated registry rows.

Do not introduce a general-purpose rule engine, plugin framework, database, or
network service unless a concrete morphological requirement needs it.

## 10. Documentation and deliverables

Deliver at least:

1. the typed explicit-specification API;
2. the selected nominal/adjectival productive improvement;
3. the selected verbal productive improvement;
4. the reusable accent-paradigm model and reviewed implementation;
5. the irregular or defective paradigm improvement;
6. grammar-backed golden and invariant tests;
7. updated public examples and package documentation;
8. an updated morphology specification;
9. a concise deterministic engine capability artifact; and
10. `docs/SYNODAL_V08_INFLECTION_ENGINE_AUDIT.md`.

The audit must state plainly:

- what the engine can productively generate;
- what remains exact-only;
- what remains unsupported;
- the metadata required for each productive system;
- every new normative citation;
- every new reusable accent paradigm;
- every irregular/defective behavior changed;
- the behavioral tests and their results;
- any incidental corpus-coverage change, clearly labeled non-goal; and
- remaining linguistic risks or source blockers.

Do not report counts of exact forms, reviewed tokens, or top-k percentage as the
headline result. Headline the new linguistic capabilities.

## 11. Verification

Start with targeted checks while implementing, then run the complete relevant
gate. At minimum, run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test -p synodal-church-slavonic-core --all-features
cargo test -p synodal-church-slavonic --all-features
cargo test -p synodal-church-slavonic-dictionary --all-features
cargo test --workspace --all-targets --all-features
cargo test --workspace --doc
cargo xtask synodal-check
cargo xtask check-all
```

If a deterministic v0.8 capability/audit generator is added, it must support a
byte-current check such as:

```bash
cargo xtask synodal-engine-audit --check
```

Also run relevant no-default-feature and `wasm32-unknown-unknown` checks for the
runtime packages when that target is installed. Inspect package file lists and
run publish dry-runs only when packaging inputs or manifests changed.

Run the v0.7 full offline coverage command only when engine behavior for
registered lexemes changed or when needed to prove there is no accidental
regression. Do not delay engine completion to improve its percentage.

If a check fails because of an unrelated pre-existing problem, demonstrate that
fact and leave unrelated work untouched. Do not weaken a guard to get green.

## Completion criteria

Do not declare completion unless all of the following are true:

- the explicit typed-specification route can inflect unregistered nouns,
  adjectives, and verbs;
- it can return complete paradigms with structured failures;
- at least one new source-backed nominal/adjectival productive gap is closed;
- at least one new source-backed verbal productive gap is closed;
- at least one reviewed accent paradigm generates multiple cells without a
  separate precomputed accented string for each cell;
- at least one irregular or defective paradigm concern is represented and
  tested correctly;
- registry-backed and explicit-specification calls share one generation kernel;
- every new rule and accent paradigm has precise normative evidence;
- exact, irregular, and productive precedence is consistent across all APIs;
- grammar-backed goldens cover every new rule's full valid inventory;
- unsupported behavior still fails honestly;
- the core remains pure and runtime packages remain offline;
- all relevant checks pass; and
- the final audit emphasizes linguistic capability rather than corpus coverage.

If the available Synodal sources cannot support one of the substantive minimum
deliverables, do not guess and do not substitute exact corpus forms. Document
the exact missing authority, complete every independently supported part, and
report the task as incomplete with the narrow blocker.

## Final handoff

In the final response, lead with what a library user can now inflect that they
could not inflect before. Then report:

- the new public API with one short example;
- the productive rules added or corrected;
- accent paradigms implemented;
- irregular and defective behavior handled;
- grammar-backed test scope;
- verification commands and results;
- incidental corpus metric movement, if any, as a secondary regression note;
- remaining unsupported morphology and source blockers; and
- whether any changes were committed or pushed.

Do not claim that the engine supports all Church Slavonic unless the capability
matrix and authoritative full-paradigm tests actually establish that claim.
