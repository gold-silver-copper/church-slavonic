/goal Achieve 100% evidence-qualified strict top-k coverage of the locked canonical Synodal Church Slavonic corpus by implementing every necessary lexical, morphological, orthographic, source, analyzer, and validation capability described below. Continue autonomously until the final completion gate is satisfied; do not stop at an intermediate percentage or merely report the remaining work.

# Synodal Church Slavonic 100% Strict Top-k Coverage Prompt

Expand this Church Slavonic Rust workspace until the canonical full Synodal
corpus report has 100% evidence-qualified top-k coverage under the strict
runtime policy. Do not merely make recommendations or prepare a queue: inspect
the implementation and sources, acquire and review evidence, add the necessary
lexical data and grammatical capabilities, regenerate owned artifacts, and
continue through successive coverage waves until the completion gate below is
actually satisfied.

This is a long-running implementation goal. Do not stop at 75%, 80%, 90%, 95%,
99%, or after exhausting the current top-200 queue. Recompute the frontier,
discover additional sources and candidate families, implement the next
evidence-backed wave, and keep going. Potentially unattested paradigm cells are
allowed when they follow an explicitly reviewed productive rule and are labeled
as generated or inherited rather than attested. Unattested lexical identities,
principal parts, accents, spelling variants, or irregular forms must not be
invented.

The primary acceptance target is deliberately corpus-bounded. Reaching it means
that every token in the locked canonical Synodal coverage corpus has at least
one valid strict top-k analysis. It does not prove that a finite library knows
every Church Slavonic word that has ever existed or could be coined. State this
distinction plainly in the final report.

## Non-negotiable definition of 100%

At completion, rerunning the canonical full report with

```bash
cargo xtask synodal-coverage --offline
```

under `GenerationPolicy::Strict` and
`OrthographyProfile::SynodalLiturgical` must show all of the following:

1. `summary.top_k_analyzed == summary.total_tokens`;
2. top-k coverage is exactly 10,000 basis points;
3. `summary.unresolved == 0`;
4. the complete `top_k_uncovered_frequency_by_surface` map is empty or sums to
   zero;
5. every corpus and every source slice independently has
   `top_k_analyzed == total_tokens`;
6. every partition represented in the input is independently covered at 100%;
7. no token receives coverage from an exploratory-only, guessed, cross-
   recension, or candidate-only analysis; and
8. all committed reports, registries, review ledgers, audits, and generated
   outputs are current and deterministic.

Do not change the denominator, corpus selection, tokenizer, default sources,
coverage formula, policy, profile, or definition of `is_top_k_analyzed` merely
to make the percentage rise. Do not delete difficult passages, silently filter
tokens, merge distinct tokens, strip meaningful marks, move records out of the
target recension, relabel unresolved forms as punctuation, or count a
diagnostic proposal as an analysis.

If a source adapter demonstrably emitted markup, mojibake, OCR corruption, or a
transcription artifact, correct the adapter or pinned source interpretation and
record an auditable before/after denominator change. Preserve the raw source,
quarantine record, source identity, and rationale. Such corrections must be
reviewed as source-quality fixes, never used as unexplained denominator
reduction.

Recognized Cyrillic numerals, abbreviations, proper names, foreign names,
indeclinables, particles, and nonstandard but genuine printed spellings are not
free exclusions. Give them an appropriate typed, source-accountable analysis.
In particular, if a recognized numeral currently has no `Analysis`, implement
the real numeral analysis path rather than changing the coverage counter.

## Establish the real baseline first

Before editing:

- Read `AGENTS.md` if present and follow it.
- Inspect repository status and the complete existing diff. Preserve unrelated
  user work.
- Enumerate every workspace member from the root `Cargo.toml` and identify the
  core, facade, dictionary, extractor, and `xtask` ownership boundaries.
- Read `README.md`, `docs/ARCHITECTURE.md`, `docs/API_DESIGN.md`,
  `docs/MORPHOLOGY_SPEC.md`, `docs/ORTHOGRAPHY.md`,
  `docs/SYNODAL_MORPHOLOGY.md`, `docs/SYNODAL_ORTHOGRAPHY.md`,
  `docs/SYNODAL_DATA_PIPELINE.md`, `docs/SYNODAL_CLI_AND_COVERAGE.md`, and all
  current Synodal coverage and morphology audit documents.
- Read the source manifests, checksum locks, revision locks, recension mapping
  tables, review ledgers, generated-file ownership comments, evaluation
  corrections, and current extraction, evaluation, coverage, family, marginal,
  completion, and source-frontier reports.
- Identify the exact locked corpus inputs and reproduce their hashes, passage
  count, token count, type count, source/partition split, and current coverage.
- Record the complete starting values for top-1, top-k, ambiguity, unresolved
  tokens, every gap category, every recovery route, and every per-source and
  per-corpus slice.
- Record the current generated morphology/dictionary hashes and verify that a
  clean regeneration is byte-identical before relying on the reports.
- Run targeted baseline tests for normalization, mark-sensitive lookup,
  exact-before-productive precedence, ambiguity, source partitioning, lexical
  review, family review, generated registries, and coverage accounting.

Do not rely on percentages quoted in this prompt or an old audit. The current
repository and freshly reproduced locked reports are authoritative.

## Protect the metric from false success

Top-k coverage can be gamed by returning broad or spurious candidate sets. That
is forbidden. A token counts only when at least one analysis is a linguistically
plausible, target-recension analysis supported by the repository's evidence
contract. Preserve the following safeguards throughout the work:

- Do not add a catch-all lexeme, universal indeclinable, unknown-word analysis,
  suffix-only fallback, arbitrary accentless fallback, or analysis that accepts
  every Cyrillic token.
- Do not attach all unresolved surfaces to a high-frequency lexeme or generate
  all grammar cells as candidates merely to ensure that one is present.
- Do not weaken explicit mark matching. Marked input must match reviewed
  breathing, accent, kamora, grave, titlo, superscript, and positional-letter
  behavior.
- Do not collapse homographs or homonyms. Preserve every justified ambiguity
  with stable lexical identities and meaningful ranking.
- Do not make `Strict` behave like `Productive` or `Exploratory`.
- Do not count candidate reports, probable-family groupings, source guesses,
  diagnostic projections, or deferred review rows as runtime analyses.
- Do not use the held-out evaluation occurrence being fixed as its own sole
  lexical, semantic, class, principal-part, or accent evidence.
- Do not treat generated output as attestation for its own admission.
- Do not treat Old Church Slavonic, modern Russian, another Church Slavonic
  recension, or a translation as direct proof of a Synodal surface. Such
  sources may support etymology, semantics, or grammar only through an explicit
  reviewed mapping.

Track precision as well as recall. At every substantial wave, report:

- held-out top-1 and top-k correctness;
- the number and distribution of analyses per token;
- new ambiguities and resolved ambiguities;
- collision counts by normalized and mark-sensitive key;
- false-positive negative-control results;
- exact, normative, irregular, productive, inherited, and abbreviation gains;
  and
- coverage gained by source, corpus, partition, part of speech, morphological
  system, and evidence route.

Do not accept a coverage gain that causes an unjustified precision regression,
analysis explosion, lexical-identity collision, or loss of mark sensitivity.
Any intentional top-1 change must be supported by source evidence and added to
the evaluation contract.

## Source inventory and evidence acquisition

Inventory every source already registered in `references/SOURCES.toml`,
`references/SOURCE_LOCK.tsv`, `references/SHA256SUMS`, exact-revision tables,
and the audit documents. For each source, record:

- authority, edition, revision, URL, checksum, format, and accessibility;
- target recension and permitted evidence roles;
- license, redistribution, package-inclusion, and generated-output constraints;
- coverage of lexical identity, meaning, part of speech, inflection class,
  principal parts, accent, orthography, abbreviations, proper names, and exact
  forms;
- whether the source is admissible runtime evidence, comparison-only,
  evaluation-only, metadata-only, or blocked; and
- adapter completeness, quarantine rate, parse ceilings, and known omissions.

Search for additional authoritative grammars, dictionaries, paradigmatic
lexica, liturgical editions, and machine-readable target-recension corpora when
the existing sources cannot close a frontier. Prefer primary editions,
publisher or institutional scans, stable scholarly repositories, and official
documentation. Compare multiple independent sources when possible. Record
source provenance and licensing before importing data. Never scrape around
access controls, accept an unreviewed mutable page, silently change a checksum,
or incorporate a source whose terms do not permit the intended use.

Inaccessible or restricted sources may remain metadata-only and may guide
manual comparison, but they cannot be cited as inspected evidence unless their
relevant content was actually available. When a source conflicts with another,
retain the conflict and its recension/edition boundary; do not choose the form
that produces the largest coverage gain.

## Evidence contract for admissions

Every new runtime analysis must have a durable evidence path appropriate to its
claim. At minimum:

- A new lexical identity needs a stable lemma, part of speech, target-recension
  or explicitly mapped identity, semantic review, and source accountability.
- An exact form needs a direct target-recension form witness with cell scope, or
  an explicitly documented normative table that licenses that exact form.
- A spelling or accent variant needs a bounded relationship to a reviewed
  identity plus direct target or normative orthographic evidence.
- A productive noun, adjective, determiner, pronoun, or numeral needs a reviewed
  inflection class, compatible stem shape, required gender/animacy/number
  metadata, and any independent accent or positional metadata.
- A productive verb needs a reviewed class and every principal part required by
  the systems it claims. Do not derive independent past, aorist, imperative,
  participial, or verbal-noun stems from spelling unless the reviewed rule
  explicitly licenses that derivation.
- An irregular or defective item needs exact overrides and typed defects at the
  smallest justified scope, with productive fallback only where independently
  licensed.
- An abbreviation needs a reviewed semantic base, contraction structure,
  titlo/superscript behavior, grammatical analysis, and passage-disjoint
  expansion/reverse-analysis tests.
- A proper name or foreign form needs an identity and either a justified
  declensional profile, an indeclinable classification, or exact attested cells;
  unfamiliar spelling alone never implies indeclinability.
- A Cyrillic numeral needs a typed numeric value, valid notation, and a real
  numeral analysis rather than an artificial dictionary lexeme.

Potentially unattested cells may be generated only after these lexical and
class facts are established. Label them with the correct productive,
analogical, inherited, or caller-supplied provenance; never label them exact or
attested. Add negative tests for invalid cells, incompatible metadata,
defectivity, and unsupported formations.

## Workstreams to close the frontier

### 1. Corpus and tokenizer integrity

Audit every remaining surface in context, beginning with high-frequency and
high-document-frequency gaps. Distinguish genuine words from adapter artifacts,
markup, broken combining sequences, numeral notation, abbreviations, token
boundary errors, editorial symbols, and source corruption. Fix tokenizer or
adapter behavior only when it is linguistically and source-format correct.
Add hostile and regression fixtures for every newly discovered failure class.

### 2. Existing reviewed-family and spelling-variant recovery

Resolve the highest-confidence gaps already associated with a reviewed family.
Implement narrowly scoped accent, breathing, positional-letter, inflectional,
and abbreviation metadata where evidence supports them. Preserve exact-before-
variant and mark-sensitive precedence. Do not turn diagnostic accentless keys or
prefix similarity into lexical identity.

### 3. Unknown-lexeme acquisition

The unknown-lexeme frontier will not be closed by the current family queue
alone. Generate a complete, untruncated queue of top-k-uncovered surfaces with
contexts and true document unions. Cross-reference every usable registered
source, add adapters or indexes for relevant omitted material, and create
durable reviewed lexical identities. Handle homonyms, senses, alternate lemmas,
proper names, compounds, particles, prepositions, conjunctions, interjections,
and indeclinables explicitly.

### 4. Missing grammatical metadata

For known identities, fill only source-supported declension classes, gender,
animacy, number inventories, accent paradigms, stem alternations, principal
parts, formations, variant policies, and defectivity. When the current type
system cannot express a genuine source distinction, extend it with a closed,
typed representation and exhaustive tests rather than encoding the fact in a
string or broad exception.

### 5. Missing productive systems and rule seams

If contextual review reveals a grammatical formation not represented by the
current completion inventory, add it to the authoritative morphology taxonomy,
implement it in the owning core/facade layer, add typed public access and
reverse analysis, document its recension and period, and update the completion
matrix. Compare all relevant known sources. A `53/53` inventory is not proof
that the taxonomy itself is exhaustive.

### 6. Exact, irregular, defective, and suppletive paradigms

Add the smallest source-backed table or override necessary for truly irregular
behavior. Preserve ordered variants, syncretism, restrictions, and explicit
missing cells. Test every licensed cell and every typed defect. Never duplicate
an exact table when a reviewed productive background plus a few overrides is
the more truthful representation.

### 7. Reverse index and analyzer integration

Every newly generated or exact surface must reach the dictionary analyzer with
the same stable identity, cell, provenance, evidence, warnings, and ordering as
forward generation. Regenerate through the owning generator. Add exhaustive
round-trip tests and collision audits. Ensure new analyses remain deterministic
under native, no-default-feature, and WebAssembly builds.

### 8. Evaluation and leakage control

Keep training/source, evaluation, and coverage roles explicit and passage-
disjoint wherever the repository contract requires it. Add independent held-
out examples and negative controls for new families. A corpus occurrence can
motivate investigation, but admission must be justified by the allowed source
roles. Seal new evaluation baselines so later coverage work cannot rewrite its
own test.

### 9. Source-frontier convergence

After each wave, regenerate every queue and compare the remaining unknowns
against every source's admitted and quarantined outputs. Improve adapters when
source rows are being lost. Record sources that were checked and yielded no new
admissible evidence. A frontier is converged only after two complete passes over
the then-current unresolved set and source inventory produce no unexplained
source-backed candidates. If coverage is below 100% at convergence, acquire or
implement the next legitimate evidence source rather than declaring success.

## Iterative working method

Work in deterministic, reviewable waves:

1. Reproduce and freeze the current denominator, hashes, and coverage report.
2. Build the complete top-k-uncovered surface inventory; do not limit planning
   to the rendered top 500 gaps or top 200 decisions.
3. Rank candidates by overlap-adjusted token gain, evidence readiness, source
   quality, ambiguity risk, review effort, and architectural dependency.
4. Select a coherent wave, preserving separate source/evaluation roles.
5. Inspect every cited source passage or table and record the evidence decision.
6. Add or correct reviewed source data and generators first.
7. Implement missing typed grammar or analyzer behavior where data alone is
   insufficient.
8. Regenerate registries and reports twice and require byte-identical output.
9. Run focused forward-generation, reverse-analysis, normalization, collision,
   negative-control, and evaluation tests.
10. Recompute full strict coverage from the locked corpus and attribute the
    realized gain exactly; never report a diagnostic projection as realized.
11. Independently review the wave's data, evidence, implementation, and complete
    diff. Validate findings and fix confirmed issues.
12. Seal a durable milestone report only when its evidence and generated
    artifacts are coherent; do not commit it unless the invoking request has
    separately authorized commits.
13. Generate the next frontier and continue immediately.

Use milestone reports at 75%, 80%, 85%, 90%, 95%, 97%, 98%, 99%, 99.5%, 99.9%,
and 100% when those thresholds are crossed. Each milestone must preserve the
locked denominator, actual top-k count, source and partition matrices, realized
gain by evidence route, evaluation results, registry hashes, review decisions,
verification, and remaining frontier. Historical milestone audits are immutable
after they are sealed.

Do not stop merely because a wave is large, a queue contains thousands of rows,
or an easy diagnostic ceiling is exhausted. Continue with source acquisition,
adapter improvement, grammar discovery, and lower-frequency lexical review.
Do not mass-admit candidates without row-level or family-level evidence that
actually entails each runtime claim.

## Architectural constraints

- Old Church Slavonic and Synodal Russian Church Slavonic remain separate
  targets. Share only recension-neutral machinery.
- Core crates remain deterministic and filesystem-free. They own typed
  morphology, orthography, Unicode behavior, and result modeling—not source
  acquisition.
- Facades own reviewed lexical identities, providers, precedence, public
  handles, and ergonomic generation.
- Dictionaries own semantic lookup, indexing, ranking, reverse analysis, text
  checking, and coverage reporting—not a second morphology engine.
- Extractors and `xtask` own source I/O, adapters, review queues, validation,
  code generation, reports, and audits.
- Runtime crates must not gain network, filesystem, JSON, TSV, XML, Lua, SQL, or
  hidden global-data access.
- Preserve stable IDs, source order, ordered variants, ambiguity, evidence,
  confidence, warnings, assumptions, contradictions, and rule traces.
- Preserve typed distinctions among unknown, ambiguous, unsupported, defective,
  invalid, evidence-incomplete, missing-metadata, contradictory, and malformed-
  orthography outcomes.
- Preserve exact and normative tables before irregular overrides, and irregular
  overrides before productive fallback.
- Preserve Unicode normalization, combining-mark order, breathing/accent order,
  titlo and superscript semantics, positional letters, transliteration loss,
  and collation.
- Preserve `#![forbid(unsafe_code)]`, no-default-feature operation, and
  `wasm32-unknown-unknown` support.
- Do not add production `unwrap`, `expect`, `panic!`, `unreachable!`, `todo!`,
  debug macros, unsafe code, or placeholder analyses.
- Do not hand-edit generated Rust. Change its reviewed data or generator and
  regenerate.
- Do not rewrite historical audits, delete difficult tests, weaken invariants,
  or update a golden merely because the new output differs.
- Backward compatibility and semver are not constraints, but every API change
  must be migrated throughout the workspace and documented.
- Do not discard unrelated user changes.
- Do not commit, push, change branches, publish crates, or mutate a pull request
  unless the invoking request explicitly authorizes those actions.

## Required data and code validation

Extend validation as necessary so malformed coverage admissions fail before
code generation. Require closed codes, NFC, target-recension compatibility,
stable foreign keys, unique runtime tuples, compatible parts of speech and
cells, valid source roles, nonempty evidence, source-lock integrity, correction
lineage, collision safety, and deterministic ordering.

For every admitted family, test:

- all exact reviewed cells;
- the complete licensed productive paradigm, including dual and vocative where
  grammatically applicable;
- animacy, gender, number, person, tense, aspect, voice, participle kind,
  adjective form, comparison, environment, and clitic distinctions that apply;
- exact-before-productive and override precedence;
- ordered spelling/accent variants;
- typed invalid and defective cells;
- forward/reverse round trips;
- mark-sensitive lookup and canonical Unicode equivalence;
- ambiguity preservation and negative identities; and
- deterministic generated order and stable IDs.

## Verification during development

Run targeted checks after every wave. Before each milestone and at final
completion, run at minimum:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
cargo test --workspace --doc
cargo xtask synodal-fixture-bootstrap
cargo xtask synodal-coverage --fixture --offline --check
cargo xtask synodal-coverage --offline --check
cargo xtask synodal-coverage --offline --check --require-complete
cargo xtask synodal-lexical-review-queue --check
cargo xtask synodal-evaluation-queue --check
cargo xtask synodal-family-review-queue --check
cargo xtask synodal-marginal-recovery --check
cargo xtask synodal-v07-apply --check
cargo xtask synodal-engine-audit --check
cargo xtask synodal-check
cargo xtask check-all
```

If a listed command does not currently support `--check`, implement a
non-mutating currentness mode or run the owning deterministic regeneration
twice and compare bytes. Include every newer versioned audit or apply gate that
exists by the time the work runs; do not stop verification at v0.7 merely
because it is the newest command named here.

Also run, when affected:

- native no-default-feature builds for every changed runtime crate;
- `wasm32-unknown-unknown` no-default-feature builds for every changed runtime
  crate;
- `cargo doc --workspace --no-deps` for public API/documentation changes;
- package-content checks and `cargo publish --dry-run` for changed published
  crates;
- source adapter fixtures, offline reconstruction, and quarantine/parse-ceiling
  checks for every changed source;
- consecutive full registry/report generations with exact byte comparison;
- a clean-checkout structural run proving ignored intermediates are not hidden
  prerequisites; and
- a full generated-tree cleanliness check.

Do not silently ignore pre-existing failures. Establish whether they reproduce
at the merge base, avoid broad unrelated fixes without authorization, and state
any genuine blocker precisely. Never call the goal complete while a required
check is failing or pending.

## Independent review loop

After every material admission wave and before final completion, have a fresh
reviewer that did not implement the wave inspect the complete diff against the
merge base. The review must cover:

- evidence entailing every new identity, form, class, principal part, accent,
  abbreviation, and recension mapping;
- accidental use of held-out or diagnostic data as admission evidence;
- spurious analyses, catch-all fallbacks, normalization overreach, and top-k
  inflation;
- incorrect forms, missing variants, wrong ordering, lost ambiguity,
  syncretism, defectivity, or accent behavior;
- source, license, revision, checksum, and redistribution mistakes;
- generated files without source/generator ownership;
- non-determinism, runtime I/O leakage, unsafe or panic paths, and portability;
- stale reports, inconsistent counts, denominator drift, and missing tests; and
- the entire intended change set, including staged, unstaged, and untracked
  files.

Validate every finding against current code and sources. Fix every confirmed
P0/P1 issue and address in-scope lower-severity findings. Rerun affected checks
and repeat the independent review whenever a fix materially changes the result.

If a pull request is in scope, follow the repository PR completion gate: inspect
all required CI checks and unresolved actionable review threads, fix failures
caused by the work, and do not declare completion while required checks are
pending or failing.

## Final completion gate

Do not stop until every item below is true:

- The locked full Synodal corpus has exactly 100% strict top-k coverage under
  the Synodal liturgical profile.
- `cargo xtask synodal-coverage --offline --check --require-complete` passes its
  locked-input, locked-denominator, per-slice, and empty-frontier checks.
- `top_k_analyzed == total_tokens`, coverage is 10,000 basis points, unresolved
  is zero, and no top-k-uncovered surface remains.
- Every corpus, source, and partition independently reaches the same standard.
- Every token's analysis is target-recension compatible and evidence-qualified;
  there are no catch-all, placeholder, guessed, or exploratory-only successes.
- Ambiguous tokens retain every justified identity and do not contain spurious
  padding candidates. Top-1/evaluation quality has not been dishonestly traded
  for recall.
- Every new lexical identity, exact form, productive class, principal part,
  accent rule, spelling variant, abbreviation, numeral, and irregular override
  has durable reviewed provenance.
- Every implemented productive family realizes every licensed grammatical cell
  and fails explicitly for invalid or defective cells.
- The source frontier has completed two full no-unexplained-candidate passes.
- Generated registries and all current reports/audits reproduce byte-for-byte.
- The entire required verification suite passes.
- The final independent review has no unresolved P0/P1 finding.
- If a PR is requested, required CI is green and actionable review threads are
  resolved.

If an external source, license, unavailable artifact, or required user decision
temporarily blocks a particular family, document it and continue every other
independent workstream. Do not reinterpret a blocker as coverage. Report the
goal as blocked only when the repository's goal/blocking policy permits it and
no meaningful in-scope work remains.

## Final report

At actual completion, report:

- the exact corpus identities, hashes, passages, tokens, types, partitions, and
  unchanged or explicitly justified denominator changes;
- baseline and final top-1, top-k, ambiguity, unresolved, per-gap, per-route,
  per-source, per-corpus, per-partition, part-of-speech, and system metrics;
- every milestone crossed and its immutable audit artifact;
- all sources inspected or acquired, their evidence roles, licenses, locks,
  adapter results, conflicts, and inaccessible limitations;
- lexical identities, families, exact forms, classes, principal parts,
  orthographic rules, abbreviations, numerals, and grammatical systems added;
- potentially unattested cells generated by rule, clearly separated from
  directly attested forms;
- precision, top-k-size, collision, held-out evaluation, leakage, and negative-
  control results;
- all data, generator, runtime, API, registry, report, documentation, and test
  changes;
- deterministic regeneration evidence and registry/report hashes;
- every verification command and result;
- independent-review findings fixed or rejected with rationale;
- CI and review-thread status when applicable; and
- any residual limitation outside the locked corpus.

The final report must explicitly say that 100% is coverage of the named locked
corpus under the named strict policy—not proof of a literally exhaustive
lexicon for all historical, regional, hypothetical, or future Church Slavonic.
