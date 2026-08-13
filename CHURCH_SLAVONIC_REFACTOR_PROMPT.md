# Church Slavonic LOC Reduction and Consolidation Prompt

Perform a broad, implementation-focused refactoring pass over this Church
Slavonic Rust workspace. Cover every workspace crate listed in the root
`Cargo.toml`: the Old Church Slavonic core, facade, dictionary, and extractor;
the Synodal Russian Church Slavonic core, facade, dictionary, and extractor;
and `xtask`. Enumerate the actual crate list from the workspace manifest at the
start rather than relying on this description.

Your primary goal is to reduce hand-written production lines of code and
structural duplication while improving clarity, maintainability, type safety,
and idiomatic Rust usage. Do not merely produce recommendations: inspect the
code, implement worthwhile refactors, add or update tests, regenerate derived
artifacts when necessary, and verify the result.

This is a refactoring task, not a corpus-coverage or linguistic-expansion task.
Do not admit new paradigms, exact forms, lexical identities, mappings, or source
claims merely to make code paths look more uniform. Preserve current linguistic
behavior unless a deliberate simplification is well supported and documented.
Backward compatibility and semver are not constraints, but linguistic truth,
provenance, deterministic output, and explicit failure behavior are.

## Objectives

1. Reduce repeated hand-written implementation patterns.
2. Consolidate recension-neutral behavior shared across crates without merging
   linguistically distinct Old Church Slavonic and Synodal rules.
3. Replace verbose or imperative code with clear, idiomatic Rust.
4. Use Rust's type system to make invalid cells, unsupported systems,
   contradictory metadata, and provenance mistakes difficult or impossible to
   represent.
5. Keep generation, lookup, analysis, and extraction behavior deterministic.
6. Achieve a meaningful net reduction in non-test, non-generated production
   LOC while making call sites and invariants simpler.

## Establish the real scope first

Before editing:

- Read `AGENTS.md` if present and follow it.
- Inspect the root workspace manifest and every member manifest.
- Read `README.md`, `docs/ARCHITECTURE.md`, `docs/API_DESIGN.md`,
  `docs/MORPHOLOGY_SPEC.md`, `docs/ORTHOGRAPHY.md`,
  `docs/SYNODAL_MORPHOLOGY.md`, `docs/SYNODAL_ORTHOGRAPHY.md`,
  `docs/DATA_PIPELINE.md`, `docs/SYNODAL_DATA_PIPELINE.md`, and the crate
  READMEs relevant to code you change.
- Inspect repository status and the complete existing diff. Preserve unrelated
  user changes.
- Identify which Rust files are generated and which source tables or generators
  own them. Never count deletion from generated registries as a refactoring win.
- Record a behavioral baseline with targeted tests before changing high-risk
  resolution, morphology, normalization, or extraction paths.

## LOC accounting

Record baseline and final LOC using one documented, reproducible method. Report
each workspace crate separately and provide workspace totals for:

1. hand-written production Rust;
2. tests, examples, benches, and fixtures;
3. generated Rust; and
4. generator, extractor, and `xtask` code.

The primary success metric is net reduction in hand-written production Rust.
Keep generated Rust, TSV/JSON data, source fixtures, reports, prompts, and audit
documents out of that primary number. Do not improve the metric by moving logic
into build scripts, generated output, opaque tables, macros, tests, or `xtask`.
If code moves between categories, report it explicitly.

## Areas to investigate

Search systematically across crate boundaries for:

- Repeated grammar-cell inventory construction and paradigm iteration.
- Duplicate case/number/gender/person/animacy traversal logic.
- Repeated exact, irregular, defective, inherited, and productive precedence
  handling.
- Parallel free-function, stable-ID, handle, explicit-specification, provider,
  paradigm, batch, and reverse-analysis paths that duplicate generation logic.
- Similar resolver branches differing only by lexical category or provenance
  source.
- Repeated `FormSet`, variant ordering, evidence, warning, and rule-trace
  assembly.
- Repeated error-to-status, error-to-code, and error-to-report mappings.
- Duplicated accent-scope matching, orthographic rendering, normalization,
  combining-mark validation, and lookup-key preparation.
- Repeated parsing of grammar cells, class codes, formations, recensions,
  metadata fields, and source identifiers.
- Duplicated generated-registry row types, table sorting, foreign-key checks,
  closed-code validation, and Rust emission.
- Similar extractor pipelines, atomic-output logic, source-order preservation,
  quarantine handling, and deterministic ID generation.
- Repeated dictionary indexing, analysis ranking, ambiguity preservation,
  vocabulary linting, and coverage/report plumbing.
- Repeated `xtask` command dispatch, audit rendering, report currency checks,
  hashing, package checks, and source-boundary validation.
- Manual loops that can become readable iterator pipelines.
- Repeated `match` expressions differing in only one or two operations.
- Redundant wrappers, intermediate collections, clones, allocations, boxing,
  string construction, and repeated normalization.
- Large modules containing multiple copies of the same algorithm.
- Public APIs that expose parallel ways to do the same work without a clear
  ergonomic or semantic distinction.
- Test helpers duplicated across crates when a small, appropriately scoped test
  utility would improve clarity.

Compare all similar implementations before introducing a shared abstraction.
Some duplication is an intentional recension boundary rather than accidental
boilerplate.

## Preferred techniques

Use the smallest abstraction that removes proven duplication. Consider:

- Small pure helper functions.
- Generic functions with precise, comprehensible trait bounds.
- Functions accepting closures for the genuinely category- or
  recension-specific operations.
- `From`, `TryFrom`, `AsRef`, `Borrow`, and `IntoIterator` where they simplify
  real call sites.
- Closed enums, newtypes, and owned summaries that encode invariants.
- Associated types or constrained internal traits for shared resolver,
  paradigm, table-validation, or emission algorithms.
- Shared iteration helpers for canonical grammar inventories.
- Table-driven logic when it is clearer than repeated branches and does not
  hide source-defined linguistic distinctions.
- Iterator combinators when they improve readability and allocation behavior.
- Existing error, provenance, capability, provider, paradigm, and generation
  types before inventing parallel abstractions.

Use advanced Rust features only when they produce a simpler API and a
comprehensible implementation. Avoid type-system cleverness that merely moves
complexity into difficult bounds or compiler diagnostics.

Prefer functions and traits over macros. Introduce a macro only when functions,
closures, traits, and data tables cannot express the repeated structure cleanly.
Do not add a dependency solely to save a small number of lines.

## Important linguistic and architectural constraints

- Old Church Slavonic and Synodal Russian Church Slavonic are separate language
  targets. Never collapse their lexicons, endings, accents, normalization, or
  source authority into one inferred system.
- Share only recension-neutral machinery. Keep genuine linguistic rules and
  source-defined seams in their owning target crate.
- The core crates remain pure, deterministic, filesystem-free morphology,
  orthography, Unicode, and result-modeling engines.
- Facades own lexical identity, registry/provider resolution, provenance,
  precedence policy, public handles, and application ergonomics.
- Dictionaries remain semantic lookup and reverse-analysis layers, not second
  morphology engines.
- Extractors and `xtask` own filesystem, source ingestion, data validation,
  report generation, and code generation. Runtime crates must not gain implicit
  file, network, JSON, TSV, XML, Lua, or database access.
- Preserve the exact-first and irregular-before-productive contracts. Preserve
  caller-supplied, normative, attested, inherited, analogical, and generated
  provenance as distinct categories.
- Preserve stable lexical identity, ambiguity, source order, ordered variants,
  recension mappings, confidence, evidence IDs, warnings, assumptions,
  contradictions, and rule traces.
- Preserve typed distinctions among unknown, ambiguous, unsupported, defective,
  historically invalid, evidence-incomplete, missing-metadata, contradictory,
  and orthographic-metadata failures.
- Do not infer stress, extended stems, principal parts, animacy, number
  inventory, semantic identity, or Synodal forms from spelling merely to enable
  a shared abstraction.
- Do not treat Old Church Slavonic evidence as direct proof of a Synodal surface
  form.
- Preserve Unicode normalization, Church Slavonic combining-mark order,
  breathing/accent ordering, positional-letter behavior, transliteration loss
  reporting, and collation semantics.
- Preserve deterministic generated registries and reports. Change generators
  and reviewed source data first, then regenerate; do not hand-maintain derived
  Rust as an independent implementation.
- Preserve no-default-feature and `wasm32-unknown-unknown` support for runtime
  crates.
- Preserve `#![forbid(unsafe_code)]` and the workspace bans on `unwrap`, `todo`,
  and debug macros. Do not add production `expect`, `panic!`, `unreachable!`,
  placeholders, or stringly typed error paths.
- Do not delete source data, evaluation fixtures, historical audits, or tests to
  reduce LOC. Do not rewrite historical audit documents.
- Do not optimize top-k corpus coverage. Coverage may be rerun only as a
  downstream regression signal.
- Do not modify unrelated areas or discard existing user changes.
- Do not commit, stage, push, publish crates, create branches, or open a PR
  unless explicitly requested.

## Working method

Start by establishing a baseline:

1. Enumerate workspace crates and architectural layers.
2. Record the LOC categories described above.
3. Run focused existing tests for the areas most likely to change.
4. Search for structural duplication using `rg`, manifest inspection, and direct
   comparison of all instances.
5. Produce a short ranked list of the strongest refactoring opportunities.

Rank candidates by:

- Expected hand-written production LOC reduction.
- Number of duplicated implementations removed.
- Reduction in future linguistic or engineering maintenance burden.
- Confidence that forms, ordering, errors, and provenance remain unchanged.
- Risk of crossing a recension or evidence boundary incorrectly.
- Risk of abstraction leakage, worse diagnostics, or reduced readability.

Then implement the highest-value candidates incrementally:

1. Understand every duplicated implementation and document its semantic
   differences.
2. Add or identify regression coverage for affected behavior.
3. Extract the smallest shared abstraction that captures only proven
   commonality.
4. Migrate every appropriate call site.
5. Remove obsolete duplicated code and types.
6. Regenerate derived artifacts when their source generator changes.
7. Run targeted formatting, compilation, linting, and tests.
8. Review resulting call sites, public APIs, error messages, traces, and
   generated output to confirm the repository is genuinely simpler.
9. Continue to the next high-value opportunity while meaningful, low-risk
   consolidation remains.

Do not stop after one trivial helper extraction. Conversely, do not force Old
Church Slavonic and Synodal behavior, unrelated parts of speech, or semantically
different data pipelines into a universal abstraction merely because their code
has a similar shape.

You may change public or internal APIs when doing so substantially simplifies
the design. Update all in-repository callers, examples, README snippets,
documentation, generated code owners, and tests affected by such changes.

## Testing and verification

Add or update regression tests whenever a refactor touches behavior not already
covered. Prefer table-driven source goldens, typed negative cases, exact
precedence tests, ambiguity tests, hostile Unicode tests, deterministic
generation tests, and public doctests.

Run targeted checks during development, then complete at minimum:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
cargo test --workspace --doc
cargo xtask synodal-engine-audit --check
cargo xtask synodal-check
cargo xtask check-all
```

Also run, when affected:

- native no-default-feature builds for every changed runtime crate;
- `wasm32-unknown-unknown` no-default-feature builds for every changed runtime
  crate;
- `cargo doc --workspace --no-deps` for public API or documentation changes;
- consecutive regeneration runs with byte-for-byte comparison when generators,
  source tables, registries, reports, or audits change;
- package-content checks and `cargo publish --dry-run` for crates whose public
  or packaged source shape changes; and
- frozen corpus/evaluation commands only when needed to detect a behavioral
  regression, never as the refactoring objective.

Do not weaken tests or update expected forms merely to make a mismatch pass.
Validate every changed expectation against the current documented contract and,
where linguistic output is involved, the pinned target source.

Do not silently ignore pre-existing or unrelated failures. Distinguish them
clearly from failures introduced by the refactor.

## Separate final review

After implementation and before reporting completion, review the complete diff
against its merge base as a separate pass. Check for:

- Changed forms, variant ordering, syncretism, animacy, number inventory, or
  accent behavior.
- Old Church Slavonic and Synodal logic unified across an invalid recension
  boundary.
- Exact/irregular/productive precedence changes.
- Lost identity, ambiguity, evidence, provenance, confidence, warnings, or rule
  trace information.
- Dictionary and morphology logic diverging into duplicate generation paths.
- Generated files changed without corresponding source/generator ownership.
- Non-deterministic map/set iteration or output ordering.
- Runtime filesystem/network/serialization-format leakage.
- WASM-incompatible bounds or feature coupling.
- New panic, unwrap, expect, unsafe, TODO, or placeholder paths.
- Excessive generics, opaque macros, or abstractions with only one meaningful
  caller.
- Public APIs or compiler diagnostics that became harder to use.
- Missing tests and documentation.
- LOC that moved categories rather than disappearing.

Validate each finding against the current code rather than accepting it
blindly. Correct every confirmed issue, rerun affected checks, and repeat the
review if a fix materially changes the design.

## Completion report

At the end, report:

- The enumerated workspace crates and layers reviewed.
- Baseline and final LOC for every changed crate in all four LOC categories,
  plus workspace totals.
- Net hand-written production LOC added or removed.
- The major duplication patterns discovered.
- The abstractions introduced and why each is appropriately scoped.
- Which crates, categories, resolver paths, or generators now share
  implementations.
- Candidate refactors deliberately rejected because they would cross a
  linguistic boundary or harm clarity.
- Public API or behavioral changes.
- Tests added or updated.
- Regenerated artifacts and proof of deterministic output.
- Every verification command and result.
- Review findings fixed or rejected with rationale.
- Remaining risks and promising follow-up opportunities.

The final result should contain less repeated hand-written production code,
clearer linguistic and architectural boundaries, stronger invariants, and
simpler call sites—not merely fewer physical lines.
