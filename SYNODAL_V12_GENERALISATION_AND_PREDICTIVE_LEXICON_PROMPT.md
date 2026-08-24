/goal Re-anchor the Synodal program on measured generalisation instead of corpus recall, finish the verb frontier for the engine's sake, add an honest predictive layer for the unknown-lexeme tail that never counts as reviewed coverage, make the library usable end to end by a real consumer, and cut the verification overhead that no longer protects anything. Continue autonomously through the ordered phases until the completion gate below is satisfied; do not reorder the phases, and do not let the corpus top-k percentage become the objective again.

# Synodal v0.12: generalisation, a predictive lexicon tier, and a usable library

This goal succeeds `SYNODAL_V11_COVERAGE_INTEGRITY_AND_VERB_MORPHOLOGY_PROMPT.md`.
Phases 1–3 of that program are sealed; its phase 4 (verbs) is carried into this
goal as phase 2 below with a changed objective. The 100% strict top-k target of
`CHURCH_SLAVONIC_100_PERCENT_TOP_K_PROMPT.md` is **retired as a driving
objective** by this prompt. It did its job: it forced a real engine, real
evidence rules, sealed floors, and a type-disjoint holdout. It is not
withdrawn as a *constraint* — every floor in `data/synodal/coverage_floors.tsv`
still holds and strict top-k may still never fall — but no wave in this
program is justified by the corpus percentage alone.

Read `docs/SYNODAL_V11_PHASE3_ARCHITECTURE.md` and
`reports/synodal-coverage.md` first. They record the state this prompt starts
from, including the open positional-realization defect and the confirmed data
error in `грѣхѡ́мъ` that phase 3 found.

## Why this goal exists

Four facts, each reproducible from the committed reports:

1. **The remaining gap is lexicon, not morphology.** Of 347,524 unresolved
   tokens, 336,662 are `unknown-lexeme`; only 115 are blocked by a missing
   class or principal part. The registry holds 937 lexemes against 57,476
   corpus types. Strict top-k rose 65% → 73.5% between 11 and 16 August when
   waves admitted thousands of tokens each; the four verb-admission commits
   that followed under v0.11 evidence rules added between 40 and 1,149 tokens
   each. The frontier is Zipfian (`reports/synodal-coverage-frontier.tsv`:
   rank 1 is 336 tokens; 54,609 gap types sit below it). Every further point
   of corpus recall costs more than the last, and the number no longer
   distinguishes a better engine from a longer lookup table.

2. **The engine generalises poorly, and the holdout proves it.** On the sealed
   type-disjoint holdout, 25,574 of 44,425 tokens are covered, but 15,048 of
   those are `exact-synodal-attestation` — a row citing the held-out type
   itself, grandfathered memorisation. Only **9,467 tokens (21.3%)** reach
   coverage by normative table, productive rule, or irregular override. That
   is the truthful measure of what the library knows, and it is the number
   this program exists to move.

3. **The library has no measured consumer.** Top-1 is 46.8% and is correctly
   not an objective (syncretism). But nobody has demonstrated the crate doing
   the one thing a user would actually ask of it — analysing a passage it has
   never seen and returning something usable — and the README's worked example
   is a single present-tense cell of `быти`. Coverage bookkeeping has consumed
   nearly all recent effort; API fitness has consumed none.

4. **Verification overhead has outgrown its protective value.** CI runs
   fifteen `xtask` checks, four of which re-derive immutable historical audits
   (v04–v07) that by their own definition can never change. Twenty-two prompt
   files sit in the repository root. The most recent CI failure on `main` was
   a stale checksum in a report, not a defect. Each wave now spends more effort
   proving it did not cheat than doing linguistics.

## What this goal is deliberately NOT

- **Not a relaxation of any floor.** `cargo xtask synodal-coverage-floors`
  stays in CI unchanged. `summary:top_k_analyzed` may not fall.
  `integrity:morphology_free_analyzed` may only fall. No per-system floor
  moves down. The type-disjoint holdout stays sealed and its
  `holdout:memorised_analyzed` ceiling stays.
- **Not a top-1 program.** Everything the v0.11 prompt says about syncretism
  still applies. Top-1 is a regression guard.
- **Not a licence to admit `lexical-form` rows.** The cheapest route to
  coverage remains closed. The predictive tier in phase 3 is the *typed*
  alternative to that route, and it is walled off from strict coverage by
  construction.
- **Not a rewrite.** Registry storage stays generated Rust until the 40 MiB
  tripwire in `check_package_metadata` fires; the tokenizer, denominator,
  corpus selection, and `is_top_k_analyzed` are unchanged.

## Phase ordering is mandatory

Phase 1 changes what every later wave is measured by; a wave done before it
cannot be scored. Phase 2 is the engine work the predictive tier in phase 3
will lean on, so its rules must be right first. Phase 4 needs the API that
phases 2–3 produce. Phase 5 is last because deleting verification before the
new measures exist would remove protection without replacing it.

---

## Phase 1 — Make generalisation the headline

Change what "progress" means before changing anything else.

- Promote the holdout to the top of `reports/synodal-coverage.md` and
  `synodal-coverage.json`. The first table any reader sees must be the
  held-out split: `generalised` (normative table + productive rule +
  irregular override), `memorised` (exact attestation of the held-out type),
  `ambiguous`, `unresolved`. Corpus-wide top-k moves below it.
- Add a per-wave **generalisation ledger** at `reports/synodal-waves.tsv`:
  one row per sealed wave with `holdout:generalised_analyzed`,
  `holdout:memorised_analyzed`, `summary:top_k_analyzed`,
  `integrity:morphology_free_analyzed`, lexeme count, principal-part count,
  productive evaluation rows. `--check` verifies the last row against the live
  report. A wave that raises corpus top-k while `generalised` stays flat is
  memorising, and the ledger must show it.
- Raise the floor `holdout:generalised_analyzed` from its sealed value at the
  end of every wave that moves it. It is a ratchet, never a target set in
  advance.
- Split the holdout report by morphological system so a verb wave can be
  seen landing in `aorist`/`present`/`imperfect` rather than somewhere else.
- Record here, and reproduce before editing: 9,467 generalised, 15,048
  memorised, 18,840 unresolved, over 2,929 held-out types.

Do not change how the holdout is selected. Do not reseed it.

## Phase 2 — Finish the verb frontier for the engine, not for the corpus

This is v0.11 phase 4, carried forward with its evidence rules intact and its
objective replaced. The unit of success is *held-out verb tokens reached by
rule*, not corpus tokens reached at all.

Order the work by the frontier head, which is overwhelmingly verbal
(`возврати́сѧ` 336, `ᲂу҆́мре` 239, `собра́шасѧ` 231, `ца́рствова` 224,
`ѡ҆полчи́шасѧ` 213, `ꙗ҆ви́сѧ` 198, `предадѐ` 195), but admit each verb only
when it brings the whole paradigm:

- a reviewed lexical identity, conjugation class, aspect, and present stem;
- every principal part each claimed system requires, independently evidenced
  (`crates/synodal-church-slavonic-core/src/morphology.rs`,
  `missing_principal_parts`) — never derived from spelling unless a reviewed
  rule licenses it;
- an accent contract via `cargo xtask synodal-accent-fit` or explicit
  `accents.tsv` rows;
- typed defects at the smallest justified scope, with negative tests proving
  invalid cells fail explicitly;
- at least one held-out expectation per claimed system in `evaluation.tsv`
  with `policy=productive`.

Grow `evaluation.tsv` productive rows (currently **1**) and
`linguistic_evaluation.tsv` (currently pinned at **12**) with the lexicon.
Lift the assertion by raising the number, never by deleting it.

Prefer productive **classes** over individual verbs where the grammar
licenses them. Ten verbs admitted through one reviewed class rule that also
resolves held-out members of that class are worth more than ten verbs admitted
as ten exact tables. Measure this: for each admitted class rule, report how
many held-out tokens it newly resolves.

Resolve, as part of this phase, the open positional/accent ordering defect
from phase 3 of v0.11: the registry path resolves exact accent rows by expanded
form, `PositionalParadigm::apply` rejects any prosodic mark, and the caller
paths apply positional before accent. Design the ordering, wire the resolver
so `positional_paradigms.tsv` is consumed, prove with a test that a populated
row no longer breaks unrelated cells, and only then populate the table where
lexical review demands it. Until that lands the table stays empty.

## Phase 3 — A predictive lexicon tier for the tail

The 325,526 `ungrouped-unknown` tokens will not be hand-reviewed. Give them an
honest, typed, lower-confidence analysis that a consumer can use and that the
coverage contract cannot mistake for evidence.

`FormSource::AnalogicalPrediction { model }` already exists in
`crates/synodal-church-slavonic-core/src/evidence.rs` and
`GenerationPolicy::Exploratory` already exists in `policy.rs`. Neither is
populated by anything. Build the layer they anticipate:

- **Segmentation against the reviewed engine.** For a surface with no
  registry match, enumerate (stem, ending) splits where the ending is a
  licensed cell ending of some reviewed class and the residual stem is
  phonotactically admissible for that class. Score candidates by how many
  reviewed lexemes of the class share the stem shape and by corpus
  co-occurrence of sibling cells of the same hypothesised stem (a hypothesised
  aorist stem gains support if its 3pl and 3sg both occur). Emit each surviving
  reading as `AnalogicalPrediction` with a named `ModelId`, a numeric
  confidence, and the sibling evidence that supported it.
- **Policy walls.** Predictions are returned only under
  `GenerationPolicy::Exploratory`. Under `Strict` and `Productive` the surface
  stays unresolved. `is_top_k_analyzed` is unchanged and never sees them.
- **Report separately.** Add a `predicted` slice to the coverage report and
  ledger: tokens reachable only by prediction, by system, with a confidence
  histogram. This slice never adds to `summary:top_k_analyzed` and the floors
  do not read it.
- **Measure precision, not recall.** Mask every reviewed lexeme in turn: hide
  its registry rows, run the predictor on its corpus surfaces, and score the
  predicted lemma and cell against the reviewed truth. Report precision by
  class and by confidence bucket in `reports/synodal-prediction-precision.md`.
  A confidence bucket whose precision is below a stated threshold is not
  emitted at all. This masked evaluation is the gate for the tier; do not ship
  a predictor that has not been scored this way.
- **Feed the review queue.** High-confidence, high-frequency predictions with
  sibling-cell support become ranked candidates in
  `cargo xtask synodal-lexical-review-queue`, each carrying the proposed
  class, the proposed principal parts, and the contexts that supported them.
  The predictor's job is to make phase 2 cheaper, not to replace it. A
  prediction promoted to a reviewed row must pass every v0.11 evidence rule;
  its origin does not shortcut review.

## Phase 4 — Prove a consumer can use the library

Pick one concrete consumer scenario and make it work end to end:

> Given a Synodal passage the registry has never seen, return for every token
> its readings (lemma, cell, provenance, confidence), with attested and
> normative readings before predicted ones, in one call, with stable
> serialisation.

- Implement it as a documented public function on `synodal-church-slavonic`
  (and a `synodal-dict analyze-text` subcommand) with a typed result, not a
  string. Provenance must be visible per reading: `is_attested`,
  `is_prediction`, policy tier, evidence id.
- Write the README's leading example around that scenario. Replace the single
  `быти` cell.
- Add a doc-tested example per public entry point that a consumer would reach
  for; `cargo test --workspace --doc` gates it.
- Audit the public API against the scenario and fix the ergonomics the audit
  finds. Record what was changed and why in `docs/SYNODAL_CONSUMER_API.md`.
  Breaking changes are acceptable at 0.x and preferable to shipping a wrong
  shape; note every one in a `CHANGELOG.md`.
- **Do not** build a contextual disambiguator in this phase. Ranking within a
  token is provenance order only. If a later goal wants context-sensitive
  top-1, it will have a stable analysis API to build on; that is the
  deliverable here.

## Phase 5 — Cut verification that no longer protects anything

Do this last, after phases 1–4 have replaced the protection being removed.

- **Freeze the historical audits.** `synodal-v04-audit`, `synodal-v05-audit`,
  `synodal-v06-audit`, `synodal-v07-audit`, `synodal-v06-review-packets`,
  `synodal-v07-baseline`, and their baselines are immutable by definition.
  Replace their re-derivation in CI with a single `synodal-archive --check`
  that verifies the sha256 of each archived artifact against one committed
  manifest. Keep the xtask commands so the archive can still be re-derived on
  demand; remove them from the CI structural job.
- **Move the prompt files.** Every `*_PROMPT.md` and the OCS gaps file move to
  `docs/goals/` with a one-paragraph index in `docs/goals/README.md` stating
  which are complete, which are superseded, and by what. The repository root
  keeps `README.md`, licences, attribution, and this file's successor only.
- **Keep** the floors, the holdout, `synodal-check`, `synodal-guard-witnesses`,
  the fixture bootstrap, and the accent-fit check. These guard live failure
  modes.
- Measure and record CI wall-clock before and after.

## Protect the metric from false success

Everything the v0.11 prompt forbids still applies. In addition:

- Do not move `holdout:generalised_analyzed` by adding an exact row for a
  held-out type and then a rule that happens to match it. The holdout ceiling
  `holdout:memorised_analyzed` exists to catch this; it does not rise.
- Do not let a prediction reach `Strict` or `Productive` output by any path,
  including through the review queue, without a reviewed row that passes the
  v0.11 evidence rules.
- Do not tune the predictor's confidence threshold on the corpus it is scored
  against. The masked-lexeme evaluation is the only tuning signal, and the
  held-out types are excluded from it.
- Do not choose the consumer scenario in phase 4 to flatter the engine. Use a
  held-out passage.
- Do not delete a check in phase 5 whose failure mode is not covered by a
  remaining check. Name the covering check in the commit message for each
  removal.

## Baseline to reproduce before editing

Reproduce and freeze these before the first change. Do not trust the numbers
quoted here; the repository is authoritative.

| Measure | Value |
|---|---:|
| Passages / tokens / types | 74,130 / 1,313,344 / 57,476 |
| top-k analyzed | 964,791 (73.46%) |
| top-1 analyzed | 614,583 (46.79%) |
| `morphology-free` covered tokens | 50,151 |
| `unknown-lexeme` gap | 336,662 |
| `ungrouped-unknown` tokens | 325,526 |
| held-out types / tokens | 2,929 / 44,425 |
| held-out top-k | 25,574 (57.56%) |
| held-out **generalised** | 9,467 (21.31%) |
| held-out memorised | 15,048 |
| held-out unresolved | 18,840 |
| lexemes / verb lexemes / principal parts | 937 / 26 / 149 |
| evaluation rows (productive) | 2,270 (1) |
| `linguistic_evaluation.tsv` rows | 12 |
| CI structural checks | 15 |

Also record the generated registry hashes, verify a clean regeneration is
byte-identical, and confirm the full verification suite is green at the merge
base.

## Verification

Every command below must pass before each sealed wave and at completion.
Phase 5 shortens this list; until then it is the v0.11 list plus the ledger
check. `--require-complete` is no longer a gate for this program and is not
run.

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
cargo test --workspace --doc
cargo xtask synodal-fixture-bootstrap
cargo xtask synodal-coverage --fixture --offline --check
cargo xtask synodal-coverage --offline --check
cargo xtask synodal-coverage-floors
cargo xtask synodal-lexical-review-queue --check
cargo xtask synodal-evaluation-queue --check
cargo xtask synodal-family-review-queue --check
cargo xtask synodal-accent-fit --check
cargo xtask synodal-engine-audit --check
cargo xtask synodal-check
cargo xtask synodal-guard-witnesses
cargo xtask check-all
```

Also, when affected: native and `wasm32-unknown-unknown` no-default-feature
builds for every changed runtime crate; `cargo doc --workspace --no-deps`;
`cargo publish --dry-run` for changed published crates; consecutive
regenerations compared byte-for-byte; and a clean-checkout structural run.

## Independent review

After every material wave, a fresh reviewer that did not implement it inspects
the complete diff against the merge base. The review must attempt to
**refute** each claimed generalisation gain by re-deriving the holdout numbers
from the locked corpus, and must specifically check for: held-out types
reached by a rule fitted to their own exact rows; predictions leaking into
strict output; a predictor threshold tuned on scored data; a consumer example
that only works on a `source`-partition passage; and a removed check whose
failure mode is now uncovered. Fix every confirmed P0/P1 before sealing.

Reviewers must not modify the working tree.

## Completion gate

- The holdout leads the coverage report, the wave ledger exists and is
  verified by `--check`, and `holdout:generalised_analyzed` has been ratcheted
  at least once.
- `holdout:generalised_analyzed` has risen by at least 50% over its baseline
  (≥ 14,200 tokens), with the gain attributable per wave in the ledger, and
  `holdout:memorised_analyzed` has not risen.
- Every admitted verb realises every licensed cell and fails explicitly for
  invalid ones; productive evaluation rows number at least one per admitted
  verb per claimed system; the positional/accent ordering defect is resolved
  and tested.
- The predictive tier exists, is reachable only under `Exploratory`, is
  reported in its own slice, has a committed masked-lexeme precision report,
  and feeds ranked candidates into the review queue.
- The consumer scenario works on a held-out passage through one documented
  public call and one CLI subcommand, with per-reading provenance, and the
  README leads with it.
- Historical audits are archived behind one manifest check; prompt files live
  under `docs/goals/`; CI wall-clock is recorded before and after.
- No floor has fallen; strict top-k has not fallen; `morphology-free`
  coverage has not risen.
- Generated registries and every committed report reproduce byte-for-byte.
- The full verification suite passes.
- The final independent review has no unresolved P0/P1 finding.

## Final report

Write `docs/SYNODAL_V12_GENERALISATION_AUDIT.md` recording: the baseline and
final rows of the wave ledger; each verb wave with the held-out tokens it
newly resolved by rule; the predictor's precision table and the threshold it
ships with; the consumer scenario, its passage, and its output; every check
removed and the check that now covers its failure mode; and what this program
deliberately left for its successor — including whether contextual
disambiguation is now worth building on the API this program produced.
