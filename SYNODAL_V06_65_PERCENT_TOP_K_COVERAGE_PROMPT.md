# Implement Synodal v0.6 Evidence-Backed 65% Top-k Coverage

Continue the Synodal Russian Church Slavonic implementation in the
`church-slavonic` workspace. Increase real-corpus `Strict` top-k coverage from
the completed v0.5 result to strictly more than 65%, while preserving the
existing recension, evidence, provenance, abstention, Unicode, offline-runtime,
evaluation-leakage, and reproducibility contracts.

Backwards compatibility and breaking semver are not concerns.

Do not commit, push, publish, open a pull request, or mutate remote state unless
the user explicitly requests it. Preserve unrelated staged, unstaged, and
untracked work. The v0.3, v0.4, and v0.5 implementation may still be present as
uncommitted workspace changes; treat that work as the baseline rather than
discarding or reconstructing it from `HEAD`.

## Locked v0.5 baseline and target

The locked v0.5 result uses the identical pinned corpus, tokenizer, policy, and
profile:

- target recension: `synodal-russian`;
- policy: `Strict`;
- orthography profile: `SynodalLiturgical`;
- tokenizer: `synodal-dictionary-tokenize-v1`;
- 74,130 passages;
- 1,313,344 tokens;
- 57,476 token types;
- source IDs `ponomar-elizabeth-bible-2026-08-09` and
  `wikisource-church-slavonic-bible-2026-08-09` at their pinned revisions;
- 580 reviewed lexemes and 580 reviewed senses;
- 1,707 generated exact forms;
- 537 passage-held-out evaluation cells;
- 534,615 top-1 analyzed tokens, or 40.706%;
- 792,421 top-k analyzed tokens, or 60.336%;
- 14,599 ambiguous tokens, or 1.112%;
- 519,542 unresolved tokens, or 39.559%.

Strictly more than 65% on this denominator requires at least 853,674 analyzed
tokens, a realized gain of at least 61,253 top-k tokens over v0.5. The stretch
goal is at least 919,341 tokens, or 70%, requiring a gain of 126,920.

The current overlap-adjusted marginal artifact contains 996 diagnostic batches
and 190,042 unique potential tokens. If every current diagnostic batch were
validly admitted, the counterfactual projection would be 982,463 tokens, or
74.806%. That is a diagnostic ceiling, not an attainable-coverage claim. It
would still be 2,545 tokens short of 75%.

Current modeled marginal recovery is:

| Route | Batches | Overlap-adjusted diagnostic tokens |
|---|---:|---:|
| Typed abbreviation registry | 36 | 11,405 |
| Spelling/orthographic variants | 23 | 2,969 |
| Ungrouped exact or morphological families | 937 | 175,668 |

Current evidence-readiness bands are:

| Readiness / effort | Batches | Overlap-adjusted diagnostic tokens |
|---|---:|---:|
| Ready / small | 18 | 1,828 |
| Partial / medium | 46 | 13,595 |
| Partial / large | 158 | 36,870 |
| Weak / large | 773 | 137,288 |
| Blocked / small | 1 | 461 |

In the current greedy ordering, rank 176 reaches a cumulative diagnostic gain
of 61,288, just enough to cross 65%. Do not assume those 176 batches are valid,
independent, or still optimally ordered after each admission. Recompute the
canonical resolver, gaps, overlaps, and ranking after every material review
wave.

The uncovered route pools in the canonical coverage report are larger than the
modeled marginal batches and currently contain 21,204 abbreviation-registry
tokens, 1,393 exact-evidence tokens, 9,169 spelling-variant tokens, and 488,960
ungrouped-unknown tokens. Explain any difference between route-pool totals and
marginal-batch totals; never add them together as if they were disjoint.

These values prioritize review. They do not authorize admitting a proposal,
guessing a lexical identity or cell, broadening `Strict`, stripping meaningful
marks, importing generic OCS morphology as Synodal morphology, or counting a
generated form as attested.

## 1. Read and reproduce the completed v0.5 state

Before editing, read completely:

- `SYNODAL_V03_CORPUS_DRIVEN_COVERAGE_PROMPT.md`;
- `SYNODAL_V04_MORPHOLOGICAL_FAMILY_COVERAGE_PROMPT.md`;
- `SYNODAL_V05_TOP_K_COVERAGE_PROMPT.md`;
- `docs/SYNODAL_V03_IMPLEMENTATION_AUDIT.md`;
- `docs/SYNODAL_V04_MORPHOLOGICAL_FAMILY_AUDIT.md`;
- `docs/SYNODAL_V05_TOP_K_COVERAGE_AUDIT.md`;
- `docs/SYNODAL_CLI_AND_COVERAGE.md`;
- `docs/SYNODAL_MORPHOLOGY.md`;
- `docs/SYNODAL_ORTHOGRAPHY.md`;
- `docs/SYNODAL_DATA_PIPELINE.md`;
- `docs/SYNODAL_REQUIREMENTS.md`;
- `data/synodal/README.md`;
- `reports/synodal-coverage.json`;
- `reports/synodal-evaluation.json`;
- `reports/synodal-marginal-recovery.json`;
- all current Synodal reviewed registries and review queues;
- all relevant core, facade, dictionary, extractor, CLI, and `xtask` code.

Inspect the complete working tree before editing and preserve unrelated changes.
Run the deterministic v0.5 checks needed to prove that the committed/generated
reports match the current runtime behavior. Confirm the exact source revisions,
partitions, passage count, token count, tokenizer, normalization contract,
policy, profile, and denominator. Fail rather than compare against a drifted
corpus or changed meaning of “analyzed.”

Create a machine-readable locked v0.5 baseline, for example
`reports/synodal-v05-baseline.json`, containing every registry, coverage,
status, gap, route, evaluation, and identity field needed for the final v0.6
comparison. Add deterministic generation and `--check` support.

## 2. Preserve the evidence and leakage boundary

Every new runtime analysis must be licensed by explicit reviewed evidence.
Maintain or strengthen the v0.5 evidence-integrity checks:

- direct corpus evidence must use the exact pinned source ID and citation;
- a target occurrence must contain the NFC whole token, not a substring;
- target evidence must identify the target recension explicitly;
- evaluation passages and passage-overlapping held-out passages may not license
  runtime lexical, semantic, accent, abbreviation, cell, principal-part, or
  productive-rule facts;
- rejected or deferred evidence may not license runtime facts;
- semantic alignment and target identity must each be reviewed when both are
  required;
- generic OCS dictionaries provide candidate identity or form evidence only,
  not an automatic Synodal paradigm;
- corpus frequency or shared stems may propose a family but cannot establish a
  lemma, meaning, grammatical cell, accent class, principal part, or rule.

When new external sources are necessary, acquire them through the existing
source-governance pipeline. Record stable source IDs, versions, checksums,
licenses, extraction boundaries, citations, recension scope, and evidence
roles. Pin and cache them for deterministic offline reconstruction. Do not add
a runtime network or filesystem dependency.

For every admission, distinguish:

1. lexical and semantic identity evidence;
2. target-recension surface evidence;
3. exact typed-cell or full-table evidence;
4. accent, breathing, positional-letter, and printed-orthography evidence;
5. abbreviation expansion and mark-order evidence where applicable;
6. principal-part or productive-rule evidence where applicable.

If the available sources establish only some of those facts, encode only the
supported exact facts and keep all unsupported facts deferred with precise
blockers.

## 3. Improve family-level review packets before bulk review

The remaining queue is dominated by repeated surfaces whose shared lemma or
paradigm has not been proved. Extend the deterministic review tooling so a
reviewer can adjudicate a complete probable family without manually
re-discovering its related rows.

For every high-priority family packet, include:

- stable family and member-candidate IDs;
- all top-k-uncovered surfaces and their exact frequencies;
- total raw and overlap-adjusted token gain;
- document frequency and representative target passages;
- all candidate lexeme identities and senses;
- all proposed typed cells, including ambiguity rather than a forced merge;
- existing reviewed runtime lexemes that may be extended;
- available evidence grouped by evidence role;
- missing evidence, contradictions, false-grouping risks, and assumptions;
- current resolver result and trace for each surface;
- predicted top-1, top-k, ambiguity, and abstention changes;
- evaluation passages excluded from runtime evidence;
- a deterministic admission, partial-admission, deferral, rejection, or split
  decision with reviewer reason.

Consolidate only when identity evidence supports consolidation. Similar stems,
accentless equality, prefixes, endings, transliteration, or corpus proximity are
not enough. Split mixed families whenever distinct lemmas, meanings, parts of
speech, or inflectional systems remain possible.

Rank packets by overlap-adjusted `Strict` top-k gain per unit of missing
evidence and review effort. Recompute rankings after admissions because route
precedence, ambiguity, and overlaps may change. Keep all pre-review values
explicitly labeled diagnostic; only canonical resolver output is realized
coverage.

Generate deterministic JSON, TSV, and Markdown packet/queue artifacts with
`--check` support. Test family consolidation, family splitting, overlap,
zero-marginal rows, stable ordering, candidate ambiguity, and stale-output
detection.

## 4. Wave A: adjudicate the highest-return typed abbreviations

Review all 36 currently modeled abbreviation batches, beginning with:

| Family or surface | Current overlap-adjusted diagnostic tokens |
|---|---:|
| `нн҃ѣ` | 1,792 |
| Israel/Jerusalem `ии҃л-` and `іи҃л-` contraction families | approximately 5,339 |
| `заⷱ҇` | 585 |
| `мцⷭ҇-` | 431 |
| `агг҃лъ` | 349 |
| `сн҃ъ / сꙑнъ` | 261 |
| `спⷭ҇ни-` | 245 |

Continue through the remaining batches, including `блгⷣть`, `блгⷭ҇венъ`,
`іи҃съ`, `цр҃ь`, `сп҃с-`, `првⷣн-`, `бг҃ъ`, `учн҃ц-`, `цр҃кв-`,
`млⷭ҇тивъ`, `ст҃ын-`, `чл҃вѣческїй`, `гл҃-`, `прⷭ҇нѡ`, and `дв҃дꙋ`.

Do not assume what an abbreviated surface expands to from familiarity. Every
admitted row must specify and independently support:

- exact marked surface in canonical combining-mark order;
- exact expanded surface;
- stable lexeme and sense identity;
- exact typed cell or all independently supported alternative cells;
- superscript letters, titlo, pokrytie, breathings, accents, and required mark
  order;
- capitalization, position, and contextual restrictions;
- reversibility and reverse-registry behavior;
- target-recension witness and normative expansion/cell citations;
- ambiguity behavior and deterministic ranking.

Do not implement blind string replacement or normalize malformed contractions
into valid ones. A contraction may not license an unsupported expanded lemma or
paradigm. Add negative controls for missing titla, wrong superscripts, reordered
marks, deceptive substrings, mixed scripts, and positionally invalid forms.

Adjudicate each batch as admitted, partially admitted, split, deferred,
rejected, or false grouping. Report predicted and realized gain separately.

## 5. Wave B: close evidence-ready spelling and orthographic variants

Adjudicate all ready/small spelling batches before lower-readiness work. The
current 1,828-token set includes surfaces associated with:

- `дати` (`давати / данъ / даꙗти` diagnostics);
- `дѣлати`;
- `лице`;
- `тои` (`тыѧ`);
- `азъ` (`мы`);
- `народъ`;
- `жизнь`;
- `село`;
- `сила`;
- `любити`;
- `сей`;
- `свѣтъ`;
- `глава`;
- `сотворити`;
- `день`;
- `любовь`;
- `бо`.

The current queue commonly lacks printed accent or positional-orthography
evidence and an accent-class decision. Obtain and cite that evidence; do not
turn the diagnostic spelling relation into a blanket normalization rule.

For each surface, independently establish exact identity, every returned cell,
accent and breathing behavior, case and positional-letter behavior, and any
capitalization restrictions. Prefer an explicit reviewed variant row when the
evidence covers only one printed form. Introduce a general transformation rule
only when a normative source specifies it and multiple held-out lexemes prove
its intended scope and exclusions.

Add hostile controls for visually similar but unrelated forms and for forms
that differ only after unsafe accent stripping. Preserve all genuine
evidence-backed alternatives in top-k.

## 6. Wave C: review high-frequency exact noun and verb families

Wave A and Wave B cannot supply the 61,253-token target alone. The main v0.6
gain must come from evidence-backed open-class families. Review complete family
packets in descending recomputed marginal order, beginning with the families
represented by these current diagnostics:

- `видъ`, approximately 1,111 tokens;
- `вѣко / вѣкъ`, approximately 1,002 tokens;
- `господь`, approximately 948 top-k-uncovered family tokens;
- `посла`, approximately 908 tokens;
- `врата / вратъ`, approximately 748 tokens;
- `родити`, approximately 733 tokens;
- `поставити`, approximately 709 tokens;
- `взѧ`, approximately 682 tokens;
- `братиꙗ`, approximately 669 tokens;
- `рещи` and its related exact forms, at least approximately 621 tokens;
- `имѧ`, approximately 642 tokens;
- `очи`, approximately 579 tokens;
- high-frequency forms of `изыти`, `душа`, `судъ`, `риза`, `заповѣдь`,
  `градъ`, `братъ`, `кнѧзь`, `кровь`, `слово`, `тысѧща`, and any higher-gain
  family produced by the recomputed queue.

The labels and counts above are diagnostics, not established identities or
guaranteed gains. Validate every grouping and count against the current report.

For nouns, establish only what the evidence supports:

- lemma and sense identity;
- declension class or explicit exact table;
- gender and animacy;
- exact singular, dual, and plural cells;
- stem, vowel, consonant, and suppletive alternants;
- accent class and printed accent variants;
- positional-letter and capitalization variants;
- indeclinability or number restrictions;
- target witnesses and normative citations.

For verbs, establish only what the evidence supports:

- lemma and sense identity;
- aspect and valency only where represented by the public model;
- present, infinitive, aorist, imperfect, imperative, participial, l-form,
  future, and other cells only when independently supported;
- exact principal parts rather than guessed stem derivation;
- prefix, augment, consonant, vowel, and suppletive alternants;
- accent class and printed variants;
- reflexive and non-reflexive identity boundaries;
- target witnesses and normative citations.

If a full paradigm is not supported, add the high-frequency exact cells that
are supported and defer the remainder. Do not manufacture a complete class to
avoid repetitive exact data. A target occurrence proves a printed occurrence,
not the proposed lemma, sense, cell, or paradigm by itself.

## 7. Introduce productive morphology only from proved paradigms

The largest remaining modeled pool is 137,288 tokens in weak/large batches.
This creates strong pressure to add productive rules, but coverage value is not
evidence. Add a class, principal-part mechanism, or transformation rule only
when all of the following hold:

1. a Synodal normative source defines the rule and its restrictions;
2. input lexemes have explicit identities and required metadata;
3. stems, principal parts, accents, alternants, and exceptions are recorded;
4. multiple independent target lexemes exercise the claimed productive scope;
5. the rule recovers more unique unresolved tokens than the next practical
   exact-review packets;
6. passage-disjoint evaluation covers representative cells and lexemes;
7. negative tests prove that unsupported inputs and cells still fail;
8. exact irregular data override productive outputs deterministically;
9. reverse analysis returns the same evidence-qualified interpretations;
10. predicted and realized marginal gains are reported separately.

Do not derive past stems from present classes, infer aspect from a surface,
choose an aorist automatically, generate participles from an undifferentiated
stem, infer accent from frequency, or copy generic OCS endings into the Synodal
runtime. Prefer exact principal parts and exact irregular tables whenever the
generalization is incomplete.

When several reviewed paradigms reveal a genuinely shared rule, implement it
once with explicit applicability metadata and migrate redundant exact rows only
if the resulting output and provenance remain identical or intentionally
improve with documented evidence.

## 8. Use ambiguity to improve top-k without corrupting top-1

Top-k coverage requires at least one valid analysis, not a forced unique
answer. When two or more lexical identities or cells are independently
supported for one surface, return all supported analyses with stable IDs,
typed cells, evidence, provenance, and deterministic ordering.

Apply this to genuinely ambiguous families such as `вѣкъ`, `врата`, `дати`,
and any noun/verb homographs identified during review. Do not erase ambiguity
to inflate top-1. Conversely, do not add an unsupported alternative merely to
turn an unresolved token into an ambiguous analyzed token.

`ꙗкѡ` is already top-k-covered and therefore offers no material top-k gain.
Reviewing its adverb/conjunction semantics may improve correctness or top-1
quality, but it must not be counted as recovery unless a currently top-k-
uncovered surface becomes analyzed.

Report top-1 gain, top-k gain, newly ambiguous tokens, resolved ambiguity, and
abstention reduction separately. Keep already top-k-covered ambiguity out of
uncovered-route totals and marginal-recovery totals.

## 9. Maintain exact coverage accounting

Coverage must be computed only by the canonical resolver under `Strict` and
`SynodalLiturgical`. Preserve and test the partition invariant:

```text
top-k analyzed + Cyrillic numerals + top-k-uncovered = total corpus tokens
```

Resolver status totals must remain mutually exclusive. Distinguish at least:

- abbreviation expansion;
- exact Synodal attestation;
- Synodal normative table;
- Synodal productive rule;
- spelling variant;
- ambiguous analysis;
- Cyrillic numeral;
- unresolved.

Do not count a probable family, candidate, spelling heuristic, abbreviation
proposal, review admission, generated form, or evaluation expectation unless
the canonical runtime actually returns a policy-valid analysis for that token.
Do not double-count overlaps, already covered ambiguity, duplicated corpus
sources, capitalization variants, or multiple analyses for one token.

For every review wave, retain:

- predicted raw and overlap-adjusted gain;
- realized token-level resolver delta;
- status and route transitions;
- top-1, top-k, ambiguity, numeral, and unresolved deltas;
- discrepancies caused by overlaps, route precedence, invalid grouping,
  Unicode normalization, or unsupported cells.

Add regression tests for every accounting bug discovered. `check-text
--strict` must continue to reject every non-numeral token that remains
top-k-uncovered, including spelling-variant and abbreviation diagnostics.

## 10. Expand evaluation without leakage

Add passage-disjoint evaluation for every admitted abbreviation, exact family,
new table, principal part, spelling rule, and productive rule. Select evaluation
passages before selecting runtime evidence, record the partition explicitly,
and enforce zero passage overlap in tooling.

Test at minimum:

- exact expanded and printed lookup;
- `Strict`, `Productive`, and `Exploratory` policies;
- deterministic top-1 and complete evidence-backed top-k;
- genuine lexical and cell ambiguity;
- reverse analysis and registry round trips;
- abbreviation expansion, printed contraction lookup, and malformed marks;
- accent, breathing, positional-letter, capitalization, and mark-order variants;
- unsupported cells, missing metadata, and false family matches;
- malformed combining sequences, missing titla, private-use characters, mixed
  scripts, substring collisions, and hostile Unicode;
- masked-leakage reconstruction in both expanded and printed lookup;
- overlap-adjusted coverage attribution and status partition invariants.

No existing reviewed exact or normative evaluation row may disappear from
top-k. Explain every changed top-1 ordering with before/after traces and
evidence. Do not require top-1 equality where genuine supported variation or
ambiguity exists.

## 11. Runtime, CLI, reports, and package constraints

Keep corpus discovery, evidence extraction, review simulation, and report
generation in `xtask`, the extractor, or development CLI. Runtime crates must
remain deterministic, filesystem-free, network-free, `no_std`-compatible where
documented, and WASM-compatible.

Expose only the additional APIs or CLI output necessary to inspect:

- family review packets and member candidates;
- current versus proposed analyses;
- evidence roles, readiness, blockers, and review decisions;
- overlap-adjusted predicted and realized recovery;
- resolver traces and route transitions;
- remaining tokens required for 65% and 70%.

Keep generated registries and reports byte-deterministic. Do not package raw
corpora, reports, review queues, reference caches, or source artifacts into the
runtime crates. Update documentation and CI stale-output checks for every new
artifact.

## 12. Coverage objectives and stopping condition

The primary objective is:

- at least 853,674 `Strict` top-k-analyzed tokens, strictly more than 65% of the
  identical 1,313,344-token corpus under `SynodalLiturgical`, with no evidence,
  precision, leakage, Unicode, abstention, runtime, or reproducibility
  regression.

The stretch objective is:

- at least 919,341 top-k-analyzed tokens, or 70%, only if the required evidence
  is available and all primary completion checks remain satisfied.

Secondary objectives are:

- reduce unresolved tokens below 35%;
- explicitly adjudicate all 36 modeled abbreviation batches;
- explicitly adjudicate every ready/small spelling batch;
- replace isolated high-frequency surface review with evidence-backed family
  packets;
- add complete exact tables or productive rules only where independently
  supported;
- materially reduce the top 100 top-k-uncovered surfaces and probable families;
- preserve genuine alternatives in top-k;
- introduce no false target attestations, untyped expansions, or silent guesses;
- leave every reviewed but unadmitted batch with an exact blocker.

Continue until one of these conditions is met:

1. the canonical full-corpus result reaches at least 853,674 top-k-analyzed
   tokens and every verification and audit requirement passes; or
2. every evidence-ready and partial batch needed by the recomputed marginal
   ordering has been explicitly admitted, partially admitted, split, deferred,
   rejected, or proved a false grouping, and the audit demonstrates why the
   target cannot be reached from available governed evidence.

Do not stop because one source or family is difficult while other higher-value
evidence-ready work remains. Do not lower the target by changing the corpus,
denominator, tokenizer, policy, profile, normalization, candidate cutoff, or
definition of analyzed. If the defensible result remains below 65%, report it
truthfully with the exact remaining token deficit and blockers.

## 13. Deterministic verification

Run targeted checks after each implementation wave, then the full repository
gate. Add the v0.6 commands created by this work to the gate and to appropriate
bounded CI workflows.

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
cargo xtask synodal-marginal-recovery --check
cargo xtask synodal-v04-audit --check
cargo xtask synodal-v05-audit --check
cargo xtask synodal-v06-audit --check
cargo xtask check-all
cargo xtask guard-witnesses
cargo xtask synodal-guard-witnesses
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
git diff --check
```

If a listed command does not yet exist, implement its deterministic v0.6
equivalent rather than silently omitting the check. Reconstruct the fixture
twice from empty temporary caches and prove byte-identical output. The full
offline bootstrap must verify every locked artifact and reproduce all reviewed
registries and committed reports byte for byte.

Inspect package contents and confirm that no raw corpora, reports, review
queues, references, caches, or unintended generated artifacts ship in any
runtime crate. Expected Cargo dirty-worktree or unused-patch warnings are not
test failures, but record them accurately.

Keep default CI bounded and fixture-based. Put multi-gigabyte, pinned-source
verification in the existing manual or scheduled full-source workflow. Do not
claim remote CI status unless it was actually inspected.

## 14. Independent review and correction loop

After implementation and local verification, perform a separate full-diff
review against the intended base branch and merge base. Include all relevant
staged, unstaged, and untracked files in the review. Review for:

- false lexical identities, senses, cells, attestations, and citations;
- substring-backed or wrong-recension evidence;
- evaluation leakage or passage overlap;
- unsafe abbreviation expansion or mark normalization;
- over-broad productive rules and missing applicability restrictions;
- Unicode normalization, combining-order, positional-letter, and mixed-script
  hazards;
- coverage double-counting, route overlap, and covered-ambiguity leakage;
- top-1 regressions, hidden alternatives, and weakened abstention;
- stale or nondeterministic reports;
- runtime filesystem/network dependencies;
- `no_std`, WASM, packaging, and publication regressions;
- missing tests and inaccurate documentation.

Validate each finding against current code rather than accepting it blindly.
Fix every confirmed P0/P1 issue and every in-scope lower-severity issue whose
fix is safe. Rerun affected checks and perform a final independent review.
Continue the review-and-fix loop if that pass discovers a new confirmed P0/P1.
Document rejected findings with concrete rationale.

## 15. Completion audit

Create `docs/SYNODAL_V06_65_PERCENT_TOP_K_COVERAGE_AUDIT.md` and a deterministic
`cargo xtask synodal-v06-audit --check` command.

The audit must contain:

- exact v0.5 baseline and final v0.6 registry counts;
- proof of corpus, source revision, partition, tokenizer, normalization,
  policy, profile, passage, token, and denominator identity;
- top-1, top-k, ambiguous, numeral, top-k-uncovered, and unresolved counts and
  percentages;
- exact realized gain and the margin above or deficit below 65% and 70%;
- route-pool totals and overlap-adjusted marginal totals without conflating
  them;
- predicted and realized gain for every admitted or partially admitted batch;
- all family splits, overlaps, route transitions, and zero-gain admissions;
- abbreviation and spelling adjudication tables, including every deferral,
  rejection, and false grouping;
- every new lexeme, sense, exact cell, table, principal part, abbreviation,
  spelling rule, transformation, and productive class with precise citations;
- top uncovered surfaces and probable families before and after;
- evaluation by policy, morphological system, attestation status, regularity,
  and provenance route;
- all expanded and printed top-1 disagreements and changes;
- masked-leakage and passage-overlap results;
- hostile-input, Unicode, reverse-analysis, and accounting test results;
- deterministic reconstruction and stale-output results;
- formatting, linting, tests, native, WASM, package, and publish-dry-run results;
- full-diff review scope, confirmed fixes, rejected findings, and remaining
  risks;
- CI and review-thread state only if actually inspected;
- the next highest-marginal evidence-ready work.

The milestone is complete only when reviewed data, generated registries,
canonical resolver behavior, CLI/API output, coverage and marginal reports,
evaluation, documentation, fixture reconstruction, full offline bootstrap, and
the v0.6 audit agree. Passing tests alone is not proof of evidence integrity;
crossing 65% alone is not proof of correctness.
