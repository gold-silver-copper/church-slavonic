# Implement Synodal v0.5 Evidence-Backed Top-k Coverage

Continue the Synodal Russian Church Slavonic implementation in the
`church-slavonic` workspace. Maximize real-corpus `Strict` top-k coverage while
preserving the existing recension, evidence, provenance, abstention, Unicode,
offline-runtime, and reproducibility contracts.

Backwards compatibility and breaking semver are not concerns.

The locked v0.4 baseline uses the identical pinned 1,313,344-token corpus under
the `Strict` policy and `SynodalLiturgical` profile:

- 506 reviewed lexemes and senses;
- 774 generated exact forms;
- 445 passage-held-out morphological evaluation cells;
- 430,470 top-1 analyzed tokens, or 32.78%;
- 569,418 top-k analyzed tokens, or 43.36%;
- 742,721 unresolved tokens, or 56.55%;
- 13,510 ambiguous tokens;
- 457 exact-only lexemes, 46 fully classed lexemes, and 3 represented but
  partial lexemes.

Exceeding 60% top-k coverage on this fixed corpus requires at least 788,007
analyzed tokens, a gain of 218,589 over v0.4. The current diagnostic recovery
routes are:

| Route | Potential unresolved tokens |
|---|---:|
| Spelling/accent/orthographic variants | 60,053 |
| Typed abbreviation registry | 42,115 |
| Exact evidence | 8,655 |
| Existing reviewed class | 1,092 |
| Remaining ungrouped unknowns | 645,324 |

The first four routes total 111,915 tokens. Even perfect recovery from them
would produce only approximately 51.88% top-k coverage, so this milestone must
also review at least 106,674 tokens of defensible analyses from currently
ungrouped unknown families to cross 60%.

These figures are prioritization inputs, not permission to count proposals as
analyses. Do not improve coverage by weakening `Strict`, accepting an
unreviewed candidate identity, stripping meaningful marks, silently importing
OCS forms, guessing a grammatical cell, or calling a generated form attested.

## 1. Establish and lock the v0.4 baseline

Read before editing:

- `SYNODAL_V03_CORPUS_DRIVEN_COVERAGE_PROMPT.md`
- `SYNODAL_V04_MORPHOLOGICAL_FAMILY_COVERAGE_PROMPT.md`
- `docs/SYNODAL_V03_IMPLEMENTATION_AUDIT.md`
- `docs/SYNODAL_V04_MORPHOLOGICAL_FAMILY_AUDIT.md`
- `docs/SYNODAL_CLI_AND_COVERAGE.md`
- `docs/SYNODAL_MORPHOLOGY.md`
- `docs/SYNODAL_ORTHOGRAPHY.md`
- `docs/SYNODAL_DATA_PIPELINE.md`
- `docs/SYNODAL_REQUIREMENTS.md`
- `reports/synodal-coverage.json`
- `reports/synodal-evaluation.json`
- `reports/synodal-family-review-queue.json`
- `data/synodal/family_reviews.tsv`
- `data/synodal/lexical_reviews.tsv`
- all Synodal core, facade, dictionary, extractor, CLI, and `xtask` code.

Run the existing deterministic checks before changing reviewed data. Confirm
that the corpus source IDs, revisions, partitions, passage count, token count,
and denominator exactly match v0.4. Fail rather than compare coverage across a
different corpus, policy, profile, tokenizer, or normalization contract.

Record a machine-readable locked baseline containing all registry, coverage,
gap, review, and evaluation counts used by the final comparison.

## 2. Add overlap-adjusted marginal recovery analysis

Extend the deterministic coverage tooling with a counterfactual marginal-gain
report. Raw family frequency is insufficient because proposals can overlap,
several rows can belong to one paradigm, and one token can have multiple
candidate recovery routes.

For every unresolved candidate family or reviewable batch, report:

- stable candidate or batch ID;
- proposed lexeme identities and typed cells;
- recovery route;
- raw token and document frequency;
- unique unresolved tokens affected;
- overlap with every higher-ranked batch;
- marginal new top-k tokens under `Strict` and `SynodalLiturgical`;
- cumulative greedy marginal recovery;
- expected top-1, ambiguity, and abstention changes;
- compatible reviewed lexemes;
- evidence already available;
- exact missing evidence or metadata;
- evidence-readiness level;
- deterministic review-effort band;
- confidence, assumptions, contradictions, and blockers.

Rank work primarily by:

```text
marginal unique Strict top-k tokens * evidence readiness / review effort
```

Use integer or categorical inputs so ordering is deterministic. Frequency may
break ties but may not substitute for evidence. Keep pre-review recovery as a
clearly labeled diagnostic counterfactual. Only realized resolver output may
enter actual coverage.

Generate committed Markdown, JSON, and TSV reports, and add a reproducible
`xtask` command with `--check` support. Test overlaps, competing routes,
ambiguous candidates, zero-marginal proposals, stable ordering, and corpus
denominator drift.

## 3. Exhaust high-value existing-identity variants

First address unresolved surfaces already linked to reviewed runtime lexemes.
Prioritize the largest marginal families, including:

- `весь`;
- `иже` and its enclitic or gendered forms;
- `сынъ`;
- `сей`, including `сїѧ`;
- `домъ`;
- `царь`;
- `градъ`;
- `быти`;
- other known noun, pronoun, determiner, numeral, adjective, and verb families
  ranked above them by realized marginal gain.

For each recovered surface, independently establish the applicable items:

- target lexeme identity;
- exact morphological cell or all compatible typed cells;
- accent and breathing behavior;
- positional-letter and printed spelling;
- number, animacy, gender, or other restrictions;
- exact target witness or Synodal normative citation;
- semantic continuity where identity is not already established.

If the source supports only individual cells, add exact reviewed rows. Do not
invent a complete paradigm to avoid repetitive exact data. If a complete
Synodal table is independently cited, encode the full typed table and its
restrictions once, then prove every enabled cell.

Reclassify every remaining spelling-variant diagnostic explicitly as admitted,
deferred, rejected, or false grouping. Do not silently leave a high-impact row
unchanged.

## 4. Expand typed abbreviation coverage

Apply the typed, mark-sensitive registry model already used for `господь` to
the highest-marginal unresolved abbreviation families, including:

- `гл҃-`;
- `бг҃-` and `бж҃и-`;
- `нн҃ѣ`;
- `ии҃л-` and its adjectival or case forms;
- `іи҃с-`;
- contracted Jerusalem forms;
- any higher-marginal family identified by the new report.

Every admitted abbreviation row must specify:

- semantic lexeme identity;
- exact expanded form;
- exact typed grammatical cell or every independently supported alternative;
- required titlo, superscript letters, pokrytie, and combining-mark order;
- capitalization and positional restrictions;
- ambiguity and contextual restrictions;
- reversibility and reverse-registry behavior;
- target-recension evidence and exact citation.

Do not implement blind string replacement, normalize malformed contractions
into valid ones, or let a contraction establish an otherwise unsupported
declension or conjugation. Preserve all independently supported analyses in
top-k and retain negative controls for deceptive surface matches.

Adjudicate every abbreviation candidate needed to account for the 42,115-token
diagnostic recovery route. Report the exact realized recovery, duplicates,
false groupings, and evidence-blocked remainder.

## 5. Use ambiguity safely to improve top-k

Top-k coverage does not require an unsafe top-1 choice. When multiple lexical
identities or cells are genuinely supported by independent target evidence,
return all of them with their own stable identities, cells, evidence, and
traces.

Start with the 13,077-token `ꙗко` proposal. Independently investigate the
adverb and conjunction identities. If both are supported in the target
recension, admit both and preserve the ambiguity. If only one or neither is
supported, admit only what the evidence licenses and retain the rest as an
explicit deferred or rejected candidate.

Apply the same approach to:

- `иже` family alternatives;
- `мои` / `моими` and other cell-homographic possessives;
- `нашего` / `нашими` / `нашь`;
- pronoun, determiner, numeral, function-word, and marked/unmarked homographs;
- every higher-impact ambiguous family from the marginal report.

Candidate dictionary membership, accentless equality, and corpus frequency are
not sufficient evidence. Never convert ambiguity into coverage by returning
unsupported possibilities. Add tests proving that genuine alternatives stay
visible and false surface collisions such as `со`/`соти`, `ли`/`лити`, and
`юже`/`югъ` remain rejected.

## 6. Add complete high-frequency closed-class paradigms

Review closed-class systems before low-yield open-class morphology. The current
top-200 queue contains approximately 34,800 diagnostic tokens across forms of
`мой`, `твой`, `свой`, `наш`, and `ваш`, in addition to 7,567 tokens proposed
under `сь`.

Prioritize complete Synodal normative tables for:

- personal and reflexive pronouns not already complete;
- demonstrative, relative, interrogative, indefinite, and negative pronouns;
- possessive pronoun/adjective families `мой`, `твой`, `свой`, `наш`, and
  `ваш`;
- frequent determiners and their mixed or restricted behavior;
- common indeclinable conjunctions, adverbs, particles, and prepositions whose
  lexical identities can be established exactly;
- numeral families only where their agreement and government are explicitly
  modeled.

One complete family review may cover several currently separate surface
clusters. Merge them only after identity and paradigm evidence agree. Each
admitted table must record all cells, printed forms, accents, restrictions,
variants, exceptions, and normative citations. Unsupported cells must continue
to return typed failures.

## 7. Review high-frequency ungrouped exact cells

After the structured spelling and abbreviation routes, work down the
overlap-adjusted ungrouped queue. Begin with high-frequency families such as:

- `сотвор-`;
- `ꙗкож-`;
- `день`;
- `нимъ`;
- `такѡ`;
- `сердц-`;
- `приид-`;
- `глагол-`;
- `лице`;
- `из̾`;
- `человѣкъ`;
- `рук-`, `люди-`, `врем-`, `пут-`, and other higher-marginal families.

Prefer reviewed exact lexical and cell rows when they are sufficient to analyze
the observed forms. A corpus occurrence may establish a target surface witness,
but it does not by itself establish lemma identity, semantics, a morphological
cell, accent class, principal parts, or a productive rule. Obtain the missing
independent evidence or defer the row with the exact blocker.

Continue review until one of these conditions is met:

1. `Strict` top-k coverage exceeds 60% on the fixed corpus; or
2. every candidate in the structured 111,915-token recovery pool and the
   descending overlap-adjusted ungrouped batches whose cumulative unique
   potential is needed to supply the remaining 106,674 tokens have explicit
   evidence-backed admissions, deferrals, or rejections, and the audit proves
   that the target cannot be reached from the scoped evidence.

Do not stop merely because a previously reviewed top-200 row was deferred.
Revisit it when the new work supplies the exact missing evidence. Preserve the
old decision and reason in history when the blocker remains.

## 8. Add productive morphology only when it wins

Do not prioritize a new rule merely because it is linguistically broad. Before
implementation, the marginal report must show that the rule will recover more
unique unresolved tokens than the next available exact-review batch.

Implement a productive rule only when:

1. repeated corpus gaps establish meaningful marginal value;
2. a Synodal normative source specifies the full rule;
3. required lexical identity and input metadata are explicit;
4. stems, alternants, accents, restrictions, and exceptions are documented;
5. multiple real target lexemes exercise the rule where the rule is claimed to
   be productive;
6. passage-disjoint evaluation covers representative outputs;
7. unsupported inputs continue to fail explicitly.

Do not derive past stems from present classes, choose an aorist from aspect,
create participles from an undifferentiated stem, infer accent from frequency,
or import OCS endings as target forms. Use exact irregular tables and explicit
principal parts when generalization is not justified.

For every new class, rule, principal part, or exact-table batch, report predicted
and realized marginal coverage separately. Explain every discrepancy.

## 9. Preserve policy precision and evaluation integrity

Add passage-disjoint evaluation rows for every admitted family, abbreviation,
exact irregular batch, and productive rule. Evaluation passages may not provide
generation, identity, semantic, accent, or abbreviation evidence.

Test at minimum:

- exact expanded and printed lookup;
- `Strict`, `Productive`, and `Exploratory` behavior;
- all independently supported top-k alternatives;
- deterministic top-1 ordering without requiring top-1 agreement where a real
  variant is documented;
- reverse analysis and registry round trips;
- abbreviation expansion and contraction lookup;
- marked, unmarked, incorrectly marked, and positionally invalid spellings;
- unsupported cells and missing metadata;
- malformed combining sequences, missing titla, private-use characters,
  mixed-script input, and hostile Unicode;
- false family and false-abbreviation controls;
- overlap-adjusted coverage attribution.

No existing reviewed exact or normative evaluation row may disappear from
top-k. Any changed top-1 ordering must be explained by explicit evidence and
reported before and after. `Strict` must not gain coverage through exploratory
or inherited-only candidates that fail its policy contract.

Continue to report exact attestation, normative prediction, productive rule,
abbreviation, spelling variant, ambiguity, and unresolved status separately.

## 10. Keep APIs, reports, and packages deterministic

Expose marginal recovery and review readiness through the existing dictionary
API and `synodal-dict` without adding filesystem or corpus dependencies to the
runtime crates. Candidate generation, corpus simulation, and review workflows
belong in the CLI, extractor, or `xtask`.

Support human-readable and machine-readable inspection of:

- marginal recovery by family, route, and review batch;
- cumulative recovery and overlap;
- current versus proposed analyses;
- evidence readiness and missing metadata;
- admitted, deferred, and rejected decisions;
- realized coverage attributable to each admission;
- remaining tokens required to reach 60%.

All generated data and reports must be byte-deterministic. Add stale-output
checks to fixture CI and the full offline workflow. Runtime crates must remain
filesystem-free, network-free, `no_std`-compatible where documented, and
WASM-compatible.

## 11. Coverage objectives and truthfulness gate

The primary objective is:

- more than 60% `Strict` top-k coverage on the identical pinned corpus under
  `SynodalLiturgical`, without a precision or evidence-contract regression.

Secondary objectives are:

- fewer than 40% unresolved tokens;
- exhaust or explicitly adjudicate the 60,053-token spelling-variant route;
- exhaust or explicitly adjudicate the 42,115-token abbreviation route;
- recover the highest-value closed-class and possessive families;
- materially reduce the top 100 unresolved surfaces and top 100 unresolved
  probable families;
- preserve all real ambiguities in top-k;
- introduce no false target-recension attestations;
- introduce no silent guesses or generic unresolved buckets.

Do not alter the denominator, tokenizer, corpus selection, policy, profile,
normalization, candidate cutoff, or meaning of “analyzed” to reach a target. Do
not count a probable family, spelling heuristic, or abbreviation proposal until
the canonical resolver returns a policy-valid analysis.

If evidence is insufficient to exceed 60%, report the truthful result. A
below-target completion audit must include:

- every scoped admission, deferral, and rejection;
- the exact realized gain and overlap for each admitted batch;
- every remaining candidate's unique marginal potential;
- the exact missing source, identity, cell, accent, semantic, abbreviation, or
  principal-part evidence;
- the maximum defensible coverage available from completed reviews;
- the additional token count still required;
- proof that no easier higher-marginal evidence-ready batch was skipped.

## 12. Reproducibility and verification

Add deterministic commands that regenerate and check:

- the locked v0.4 baseline;
- marginal-gain JSON, TSV, and Markdown reports;
- lexical and family review queues;
- reviewed registries;
- abbreviation coverage;
- full and fixture corpus coverage;
- evaluation and candidate queues;
- the final v0.5 audit.

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
cargo xtask synodal-family-review-queue --check
cargo xtask synodal-v04-audit --check
cargo xtask synodal-v05-audit --check
cargo check -p synodal-church-slavonic-core --no-default-features
cargo check -p synodal-church-slavonic --no-default-features
cargo check -p synodal-church-slavonic-dictionary --no-default-features
cargo check -p synodal-church-slavonic-core --target wasm32-unknown-unknown --no-default-features
cargo check -p synodal-church-slavonic --target wasm32-unknown-unknown --no-default-features
cargo check -p synodal-church-slavonic-dictionary --target wasm32-unknown-unknown --no-default-features
cargo package --list --allow-dirty -p synodal-church-slavonic-core
cargo package --list --allow-dirty -p synodal-church-slavonic
cargo package --list --allow-dirty -p synodal-church-slavonic-dictionary
cargo publish --dry-run --no-verify --allow-dirty -p synodal-church-slavonic-core
cargo publish --dry-run --no-verify --allow-dirty -p synodal-church-slavonic
cargo publish --dry-run --no-verify --allow-dirty -p synodal-church-slavonic-dictionary
cargo xtask synodal-bootstrap --offline --cache references/downloads
```

Add the actual marginal-report command to this gate once named. Run targeted
tests first, then the complete gate. The fixture bootstrap must reconstruct
twice from empty temporary caches. The full bootstrap must verify every locked
artifact and reproduce the reviewed registries and committed reports byte for
byte.

Inspect CI workflow definitions and update them for every new bounded fixture
check. Do not broaden default CI to require the multi-gigabyte source cache;
keep full-source reconstruction explicitly scheduled or manual.

After verification, perform a separate full-diff review for correctness,
regressions, unsafe ambiguity handling, false attestations, Unicode hazards,
coverage double-counting, leakage, missing tests, package contamination, and
stale documentation. Validate each finding against current code, fix confirmed
issues, and rerun affected checks.

## 13. Completion audit

Create `docs/SYNODAL_V05_TOP_K_COVERAGE_AUDIT.md` and a deterministic
`cargo xtask synodal-v05-audit --check` command.

The audit must contain:

- the locked v0.4 and final v0.5 registry counts;
- exact corpus, policy, profile, tokenizer, and denominator identity;
- top-1, top-k, ambiguous, and unresolved counts and percentages;
- the exact number of tokens gained and still needed for 60%;
- diagnostic, predicted, and realized recovery by route;
- overlap-adjusted recovery for every admitted batch;
- spelling-variant, abbreviation, ambiguity, closed-class, exact-cell, and
  productive-rule results;
- every new lexeme, sense, cell table, class, principal part, abbreviation, and
  rule with target evidence and normative citations;
- all admissions, deferrals, rejections, false groupings, and blockers;
- top unresolved surfaces and families before and after;
- evaluation by policy, system, status, regularity, and provenance path;
- all top-1 changes and all abstentions;
- leakage and hostile-input test results;
- all verification commands and results;
- CI state if available;
- remaining risks and the next highest-marginal evidence-ready work.

The milestone is complete only when reviewed data, generated registries,
canonical resolver behavior, CLI/API output, corpus reports, evaluation,
documentation, fixture reconstruction, and full offline reconstruction agree.
Passing tests alone is not proof of coverage correctness, and a coverage number
alone is not proof of valid evidence.
