# Synodal Dictionary Test-Performance Optimization Prompt

You are working in the `church-slavonic` Rust workspace. Optimize the Synodal
dictionary library and CLI tests without weakening correctness, linguistic
coverage, determinism, provenance, or public API behavior.

Backwards compatibility and breaking semver are not concerns. Preserve all
currently intended behavior and test assertions unless a test is demonstrably
duplicating coverage or exercising the wrong boundary.

## Baseline

Measured locally in Cargo's normal test profile:

- `synodal-church-slavonic-dictionary` library: 30 tests in 5m32.88s.
- `synodal-church-slavonic-dictionary` CLI: 6 tests in 4m17.55s.

When other CPU-heavy Cargo processes ran concurrently inside `cargo xtask
check-all`, those components increased to:

- Dictionary library: 8m19.62s.
- Dictionary CLI: 6m57.31s.

One standalone `synodal-dict analyze` invocation took approximately 27.8s in
the debug profile and 5.787s in release mode on the same machine, a roughly
4.8x execution-time difference. The cold release build took 10.32s.

## Confirmed bottleneck

The dominant cost is repeated construction or emulation of the complete
reverse-analysis index, not compilation or the number of assertions.

Relevant code:

- `crates/synodal-church-slavonic-dictionary/src/lib.rs`: `analyze_with` loops
  over every lexeme and every cell returned by `candidate_cells`, twice for the
  expanded and printed orthography profiles.
- `crates/synodal-church-slavonic-dictionary/src/coverage.rs`: `Analyzer::new`
  performs the same complete paradigm enumeration to build its indexes.
- `crates/synodal-church-slavonic-dictionary/tests/cli.rs`: nearly every helper
  call starts a new `synodal-dict` process, so analyzer state cannot be reused.

The current registry has approximately:

- 833 lexemes.
- 165 verbs.
- 449 nouns and 22 proper nouns.
- 54 adjectives.
- 30 pronouns.
- 15 numerals.

The theoretical candidate-cell cross product causes approximately 550,994
form-resolution attempts per complete two-profile index construction. Verbs
alone account for about 368,280 attempts. The library tests trigger roughly 26
complete or equivalent scans, and the CLI tests trigger roughly nine fresh
process/index builds.

## Objective

Make the dictionary library and CLI test suites fast enough for routine local
and CI execution. Prefer removing unnecessary work over merely hiding it with
more hardware or parallelism.

Aim for these warm-cache targets on the same development machine:

- Dictionary library tests: under 60 seconds.
- Dictionary CLI tests: under 60 seconds.
- Combined dictionary test execution: under 2 minutes.
- No individual ordinary CLI query should rebuild an avoidably exhaustive
  theoretical morphology inventory.

If a target cannot be reached safely, report the measured result, remaining
bottleneck, and next concrete optimization.

## Required work

### 1. Establish reproducible measurements

Before changing behavior:

- Record library and CLI suite timings separately.
- Measure analyzer construction separately from a cached lookup.
- Count analyzer/index constructions in each suite.
- Avoid concurrent Cargo processes while benchmarking.
- Use at least three warm runs for before/after comparisons and report the
  median. Keep cold compilation time separate from test execution time.
- Add lightweight instrumentation or a benchmark harness if needed, but do
  not leave noisy unconditional logging in the public library.

### 2. Evaluate a moderately optimized test profile

Measure the effect of this workspace configuration:

```toml
[profile.test]
opt-level = 2
```

Use the lowest optimization level that provides most of the runtime benefit
without making incremental test compilation unreasonably slow. Preserve test
assertions, overflow checks, and debug information appropriate for diagnosis.
Document the compile-time/runtime tradeoff with actual measurements.

Do not stop at profile tuning if redundant index construction still dominates.

### 3. Reuse the default analyzer in library tests and APIs

- Create a thread-safe, lazily initialized default reverse analyzer or index.
- Reuse it across compatible default-policy/default-profile library calls.
- Replace repeated `Analyzer::new(Inflector::default())` construction in unit
  tests with a shared fixture.
- Ensure custom generation policies, orthography profiles, mapping thresholds,
  and provider/configuration state cannot accidentally reuse an incompatible
  index.
- Avoid global mutable state and test-order dependencies.
- Cache initialization errors safely; do not panic in public fallible APIs
  merely because a lazy cache is convenient.

Consider routing `analyze`, `families`, `lemmatize`, and `lint_vocabulary`
through a reusable analyzer where their semantics permit it. Batch operations
must build or obtain one analyzer and reuse it for every item rather than
calling an exhaustive single-word path repeatedly.

### 4. Make CLI tests reuse process-local state

Refactor the CLI boundary so command dispatch can be tested in-process, for
example through a function that accepts arguments, input/output streams, and a
reusable analyzer provider or cache.

- Convert most CLI assertions to in-process tests sharing one analyzer.
- Retain a small number of subprocess integration tests for binary wiring,
  stdin/stdout/stderr behavior, and exit-status contracts.
- Do not replace end-to-end coverage entirely with unit tests.
- Preserve deterministic human-readable and JSON golden output.

### 5. Stop enumerating impossible theoretical cells

Replace the universal Cartesian products in `candidate_cells` with the
smallest complete per-lexeme inventory that preserves all valid analyses:

- Always include exact registered cells.
- Include cells implied by genuinely supported productive systems.
- Respect lexeme subtype, defectiveness, available principal parts, class,
  person/gender restrictions, numeral kind, comparison support, participle
  tense/voice/form, verbal-noun support, and other compatibility metadata.
- Do not silently omit exact-only cells or sparse reviewed overrides.
- Do not overclaim unsupported productive cells.
- Keep deterministic ordering.

Prefer generating a compact supported-cell inventory from the authoritative
registry/data pipeline over probing thousands of cells and interpreting typed
errors as ordinary control flow.

### 6. Consider a generated strict reverse index

The extractor already knows the reviewed exact forms. Evaluate generating a
compact mapping from normalized expanded/printed surfaces to exact analysis
records.

The runtime analyzer should then:

- Load exact analyses directly.
- Generate only genuinely productive cells.
- Preserve mark-sensitive matching and explicit accentless fallback rules.
- Preserve every analysis's stable lexeme identity, typed grammar cell,
  evidence IDs, recension provenance, source classification, confidence,
  assumptions, contradictions, warnings, and rule trace.
- Preserve legitimate ambiguity while rejecting unreviewed lexical-identity
  contamination.

The generated reverse index must be deterministic, regenerated by the normal
pipeline, validated against source registries, and checked for staleness.

### 7. Reduce allocation only after eliminating redundant morphology work

Once profiling shows that indexing allocations matter:

- Build reusable `Inflector` values outside inner cell loops.
- Avoid cloning complete `Analysis` objects into several maps when stable IDs,
  indices, or shared immutable values suffice.
- Consider hash-based construction/lookup if it measures faster, while sorting
  externally visible results to retain deterministic output.
- Avoid speculative unsafe code.

## Correctness requirements

The optimization must preserve:

- All exact and productive analyses admitted by current policy.
- Mark-sensitive homograph behavior and accentless fallback semantics.
- Strict versus inherited/exploratory generation-policy boundaries.
- Stable lexeme and family identities.
- Typed grammar cells and complete represented system inventory.
- Evidence, source recension, target recension, confidence, mapping, and rule
  trace metadata.
- Abbreviation expansion.
- Deterministic ordering and serialization.
- CLI error handling and exit thresholds.
- Native and no-default/wasm build boundaries.

Add regression tests comparing cached/indexed results with an independent slow
reference implementation across every registry lexeme, supported cell,
orthography profile, and generation policy that the public API represents.
Compare full analysis values, not only returned surface strings or counts.

Test concurrent access to the lazy cache and prove that construction happens
once per compatible configuration.

## Verification

Start with targeted checks and then run the repository's complete required
gate. At minimum run:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test -p synodal-church-slavonic-dictionary --lib --all-features
cargo test -p synodal-church-slavonic-dictionary --test cli --all-features
cargo test --workspace --all-targets --all-features
cargo test --workspace --doc
cargo xtask check-all
cargo xtask check-structure
cargo xtask synodal-check
cargo check -p synodal-church-slavonic-dictionary --no-default-features
cargo check -p synodal-church-slavonic-dictionary \
  --target wasm32-unknown-unknown --no-default-features
```

Run heavy Cargo commands sequentially during performance measurements. CI may
parallelize independent jobs on separate runners, but do not run duplicate
full suites concurrently on one host.

After implementation, report:

- Before/after median timings and speedups for library tests, CLI tests,
  analyzer construction, and cached lookup.
- Analyzer construction counts before and after.
- Compile-time impact of the chosen test profile.
- Memory impact and generated artifact size, if a reverse index is added.
- Which work was eliminated versus merely made faster.
- Any remaining hot path and recommended follow-up.

## Review gate

Inspect the complete intended diff against the merge base, including relevant
staged, unstaged, and untracked files. Have a fresh independent reviewer inspect
the full diff for correctness, regressions, security issues, unsafe edges, and
missing tests. Validate findings against the current tree, fix every confirmed
P0/P1 issue, rerun affected checks, and repeat the review loop until no
confirmed P0/P1 finding remains.

Do not weaken or delete expensive correctness tests merely to improve the
number. Optimize the implementation and test architecture while retaining an
independent exhaustive equivalence check at an appropriate boundary.
