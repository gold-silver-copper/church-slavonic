/goal Harden the Synodal coverage contract so that strict top-k coverage cannot be satisfied without real morphology, make the held-out split actually measure generalization, remove the architectural ceilings that block a large reviewed lexicon, and then close the verb frontier. Continue autonomously through the ordered phases until the completion gate below is satisfied; do not reorder the phases and do not substitute raw coverage growth for the integrity work that phase 1 requires.

# Synodal v0.11: coverage integrity, generalization measurement, and verb morphology

This goal succeeds `CHURCH_SLAVONIC_100_PERCENT_TOP_K_PROMPT.md`. That program
is not withdrawn: 100% evidence-qualified strict top-k coverage of the locked
canonical Synodal corpus remains the destination. This prompt changes what has
to be true *along the way*, because the current acceptance test can be satisfied
by work that leaves the library no better than it is today.

Read `docs/SYNODAL_ACCENT_PARADIGM_FIT.md` first. It records the most recent
realized wave and the measured state this prompt starts from.

## Why this goal exists

Three defects in the present contract, each reproducible in this repository:

1. **The cheapest path to 100% teaches the engine nothing.** An analysis whose
   grammar cell is `LexicalForm` commits to no morphology at all, yet satisfies
   `is_top_k_analyzed` in
   `crates/synodal-church-slavonic-dictionary/src/coverage.rs`. That route
   already accounts for **94,026 of 963,251 covered tokens (9.76%)**, and the
   overwhelming majority of the remaining frontier surfaces are reachable the
   same way. Every such row can carry genuine dual-source, passage-disjoint
   evidence and still add nothing a user can inflect, generate, or reverse.

2. **The held-out split barely measures generalization.** It is passage-
   disjoint, not type-disjoint. Most frontier surfaces occur in *both*
   partitions, so an exact row sourced from a `source` passage closes its own
   held-out twin. Only about **2,308 surfaces / 3,502 tokens — 0.27% of the
   corpus** — can ever test whether the engine generalizes rather than
   memorizes.

3. **The correctness contract does not grow with the lexicon.**
   `data/synodal/evaluation.tsv` holds 2,270 rows of which exactly **one** uses
   `policy=productive`; `data/synodal/linguistic_evaluation.tsv` is pinned at
   **12** rows by a test assertion. Coverage can travel from 73% to 100% while
   measured productive behaviour stays fixed at a single expectation.

## What this goal is deliberately NOT

**Do not target raw top-1.** `is_top_1_analyzed`
(`crates/synodal-church-slavonic-dictionary/src/coverage.rs:2800`) requires
`analysis.analyses.len() == 1`. Church Slavonic syncretises heavily and
truthfully: `чи́стъ` is nominative singular masculine animate, nominative
singular masculine inanimate, *and* accusative singular masculine inanimate,
all from one lexeme. Such a token can never be top-1 no matter how good the
engine becomes. Measured now:

| Measure | Tokens | Share |
|---|---:|---:|
| top-k (any analysis) | 963,251 | 73.34% |
| lemma-unique (top-k minus ambiguous) | 953,853 | 72.63% |
| top-1 (exactly one analysis) | 613,949 | 46.75% |
| cell-ambiguous but lemma-unique | 339,904 | 25.88% |

The 26-point gap between lemma-unique and top-1 is almost entirely genuine
syncretism. The only way to move raw top-1 is to delete justified readings,
which the predecessor program forbids and which would make the library wrong.
Treat top-1 as a **regression guard**, never as an objective.

## Phase ordering is mandatory

Phases 1 and 2 change what the numbers mean. Phase 3 removes a hard ceiling.
Doing lexical work before them produces results that are unmeasurable,
unshippable, or both. Do not begin phase 4 until phases 1–3 are sealed.

---

## Phase 1 — Harden the coverage contract

Make `cargo xtask synodal-coverage --offline --check --require-complete`
un-satisfiable by morphology-free admission.

Add to the completion gate, and to the committed report so each is auditable
per wave:

- **Morphology-free ceiling.** The share of covered tokens whose only analyses
  carry cell `LexicalForm` must not rise, and the gate must name a declining
  ceiling. Record the current value (94,026 tokens; 9.76% of coverage) as the
  sealed starting bound.
- **Per-system floors.** `by_morphological_system` counts must not regress for
  any system. Aggregate growth may not be purchased by losing a system.
- **Top-1 and lemma-unique regression guards.** Neither
  `top_1_analyzed` nor `top_k_analyzed - ambiguous` may fall *unless* the
  wave's report attributes the fall to newly modelled syncretism, cell by cell,
  in a committed ledger. An unexplained fall fails the gate.
- **Ambiguity composition.** Report newly ambiguous tokens split into
  within-lexeme (syncretism, expected) and cross-lexeme (homonymy, must be
  justified per identity pair). `TokenStatus::Ambiguous` is currently set only
  for cross-lexeme ambiguity; the within-lexeme case is invisible today and
  must be measured.

Do not change `is_top_k_analyzed`, the tokenizer, the denominator, the corpus
selection, the policy, or the profile. This phase adds constraints; it never
relaxes one to make a number move.

Extend `cargo xtask synodal-coverage` so every new quantity is emitted in
`reports/synodal-coverage.json`, rendered in the Markdown report, and verified
by `--check`.

## Phase 2 — Make the held-out split measure generalization

Introduce a **type-disjoint** evaluation split alongside the existing
passage-disjoint one. Do not delete or weaken the passage-disjoint partition:
add the second axis and report both.

- Hold out a reviewed set of *normalized token types*, chosen deterministically
  and sealed, such that no runtime evidence row may cite any occurrence of a
  held-out type in any passage or either edition.
- Report coverage over the type-disjoint holdout separately, and require the
  completion gate to state it. A wave that raises corpus coverage while
  type-disjoint coverage stays flat is memorizing, and the report must say so.
- Expect the headline numbers to fall when this lands. That is the intended
  outcome; do not tune the split to protect a percentage.
- Preserve every existing leakage check. `synodal-accent-fit` already excludes
  sealed passages by passage name across both editions because the corpora are
  verse-parallel with independent partition assignments; apply the same
  discipline to every new tool.

## Phase 3 — Remove the architectural ceilings

None of these are optional, and all of them block a large lexicon.

- **Registry lookups are linear scans of `&'static` arrays.**
  `crates/synodal-church-slavonic/src/registry.rs` scans `EXACT_FORMS`,
  `ACCENTS`, `ACCENT_PARADIGMS`, `PRINCIPAL_PARTS`, and `NOUN_RESTRICTIONS` per
  lexeme, and `Analyzer::new` re-enters `resolve_cell` per cell and profile.
  Cost is roughly Θ(L·E + L·C·K·E). Replace with ordered or perfect-hash
  indices keyed on `lexeme_id` and `(lexeme_id, cell)`. Preserve stable IDs,
  source order, ordered variants, and determinism exactly.
- **Generated Rust size and packaging.**
  `crates/synodal-church-slavonic/generated/registry.rs` is already ~2.6 MB as
  one `Raw…([…])` literal per row. A registry at target scale is well past the
  crates.io per-crate limit that CI's `cargo publish --dry-run` enforces. Move
  the bulk payload to a compact embedded representation the runtime crates can
  read without gaining filesystem, network, JSON, or TSV access, and keep
  `#![forbid(unsafe_code)]`, no-default-feature operation, and
  `wasm32-unknown-unknown` support.
- **Positional-letter realization does not exist on the registry path.**
  `resolve_cell` applies accent metadata only; `apply_positional_paradigm` runs
  solely for caller-supplied specs, and `data/synodal/positional_rules.tsv` is
  read-only introspection. Wide `є`, `ѡ`, and antistich `ѣ` appear in ending
  tables only where hard-coded. Surfaces distinguished by the number-antistich
  rule — `двє́ри` alone is 211 tokens — cannot be generated productively today
  and must not be budgeted as productive coverage until this is implemented.
- **Quadratic validators and frozen constants.** `validate_candidate_links`
  reads every intermediate JSONL in full; `validate_abbreviation_families` and
  `validate_exact_form_attestation_evidence` are quadratic in table size. The
  wave-frozen digests and counts in `crates/xtask/src/synodal_v07_*.rs` require
  a Rust edit per data wave. Fix the complexity and move the frozen values into
  reviewed data.

Prove each fix with a measurement, not an assertion: record before/after
wall-clock for `Analyzer::new`, the canonical coverage run, `synodal-check`,
and the slowest workspace test.

## Phase 4 — Close the verb frontier

This is where the remaining linguistic value is.

`data/synodal/lexemes.tsv` registers **17 productive verbs** and
`principal_parts.tsv` holds **123 rows**, while the finite systems already
carry aorist 39,987, present 22,974, future 15,990, and imperfect 6,158 covered
tokens, and the high-frequency uncovered head is overwhelmingly verbal:
`возврати́сѧ`, `ᲂу҆́мре`, `собра́шасѧ`, `ца́рствова`, `ѡ҆полчи́шасѧ`,
`ꙗ҆ви́сѧ`, `предадѐ`, `глаго́лаша`.

Verbs are also the hardest item to fake, because each one needs every principal
part the systems it claims actually require
(`crates/synodal-church-slavonic-core/src/morphology.rs`,
`missing_principal_parts`). Use that: a verb admitted with real principal parts
is evidence the engine learned something; a verb admitted as a `lexical-form`
row is not.

For each admitted verb:

- a reviewed lexical identity, conjugation class, aspect, and present stem;
- every principal part required by each system it claims, each independently
  evidenced — never derived from spelling unless a reviewed rule licenses that
  derivation;
- an accent contract via `cargo xtask synodal-accent-fit` or explicit
  `accents.tsv` rows;
- typed defects at the smallest justified scope for defective or suppletive
  behaviour, with negative tests proving invalid cells fail explicitly;
- held-out expectations added to `evaluation.tsv` with `policy=productive`, not
  `strict`, so the productive engine is actually measured.

Grow `evaluation.tsv` productive rows and `linguistic_evaluation.tsv` in
proportion to the lexicon. Lift the 12-row assertion by raising the number with
the data, never by deleting the assertion.

## Protect the metric from false success

Everything the predecessor prompt forbids still applies. In addition:

- Do not raise top-1 or lemma-unique by removing a justified reading, merging
  homographs, or narrowing a cell inventory. Syncretism is a fact about the
  language, not a defect.
- Do not satisfy a per-system floor by relabelling a `lexical-form` admission
  as a typed cell without the metadata that cell requires.
- Do not choose a type-disjoint holdout that avoids hard types.
- Do not widen an accent or positional scope beyond its evidence to make a cell
  realizable. `synodal-accent-fit` already rejects rules that overlap an
  existing paradigm, cannot realise a claimed cell, contradict a
  source-partition print, or place a kamora on a singular-only scope; keep
  those guards and add equivalents for any new generator.
- Do not treat "not proposed by the tool now" as "rejected". Once a rule takes
  effect its cells leave the gap. Re-derivation means resetting the generated
  rows and re-applying from a clean state.

## Baseline to reproduce before editing

Reproduce and freeze these before the first change. Do not trust the numbers
quoted here; the repository is authoritative.

| Measure | Value |
|---|---:|
| Passages / tokens / types | 74,130 / 1,313,344 / 57,476 |
| top-k analyzed | 963,251 (73.34%) |
| lemma-unique | 953,853 (72.63%) |
| top-1 analyzed | 613,949 (46.75%) |
| ambiguous | 9,398 |
| unresolved | 349,064 |
| `lexical-form` covered tokens | 94,026 |
| `unknown-lexeme` gap | 338,901 |
| `missing-accent-or-orthographic-metadata` gap | 10,048 |
| productive verbs / principal parts | 17 / 123 |
| held-out evaluation rows (productive) | 2,270 (1) |

Also record the generated registry hashes, verify a clean regeneration is
byte-identical, and confirm the full verification suite is green at the merge
base before attributing any failure to your own work.

## Verification

Every command below must pass before each milestone and at completion.
`--require-complete` will fail until 100% coverage is reached; that is the
predecessor program's gate and remains outstanding. Every *other* command must
be green, and phase 1 adds new assertions to the first of them.

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
cargo test --workspace --doc
cargo xtask synodal-fixture-bootstrap
cargo xtask synodal-coverage --fixture --offline --check
cargo xtask synodal-coverage --offline --check
cargo xtask synodal-lexical-review-queue --check
cargo xtask synodal-evaluation-queue --check
cargo xtask synodal-family-review-queue --check
cargo xtask synodal-marginal-recovery --check
cargo xtask synodal-accent-fit --check
cargo xtask synodal-v07-apply --check
cargo xtask synodal-engine-audit --check
cargo xtask synodal-check
cargo xtask check-all
```

Also, when affected: native and `wasm32-unknown-unknown` no-default-feature
builds for every changed runtime crate; `cargo doc --workspace --no-deps` for
public API changes; package-content checks and `cargo publish --dry-run` for
changed published crates; consecutive regenerations compared byte-for-byte; and
a clean-checkout structural run.

## Independent review

After every material wave, have a fresh reviewer that did not implement it
inspect the complete diff against the merge base, including untracked files.
The review must specifically attempt to **refute** each claimed gain by
re-deriving it from the locked corpus, and must check for: coverage bought with
morphology-free rows, unexplained top-1 or lemma-unique regression, holdout
leakage across the two verse-parallel editions, rules contradicted by prints
the fitting stage could not see, homograph collapse, and stale or
non-deterministic reports. Fix every confirmed P0/P1 before sealing.

Reviewers must not modify the working tree. If a reviewer needs to build a
probe, it belongs in a scratch directory outside the repository.

## Completion gate

- Phase 1 assertions exist, are exercised by `--check`, and have caught at
  least one deliberately introduced violation in a test.
- A type-disjoint holdout exists, is sealed, is reported separately, and no
  runtime evidence row cites a held-out type in either edition.
- Registry lookups are no longer linear scans; the published crate is within
  its size limit; positional-letter realization works on the registry path;
  measured timings are recorded before and after.
- The verb inventory and its principal parts have grown substantially, every
  admitted verb realizes every licensed cell and fails explicitly for invalid
  or defective ones, and `evaluation.tsv` productive rows have grown with it.
- Strict top-k coverage has risen, the `lexical-form` share of coverage has
  **fallen**, and no morphological system has regressed.
- Generated registries and every committed report reproduce byte-for-byte.
- The full verification suite passes except `--require-complete`.
- The final independent review has no unresolved P0/P1 finding.

## Final report

Report the corpus identity and hashes; baseline and final values for every
measure in the baseline table; the `lexical-form` share before and after;
per-system deltas; type-disjoint holdout results; the measured performance
before and after each phase-3 fix; every verb admitted with its principal parts
and evidence; potentially unattested cells generated by reviewed rule, kept
separate from attested forms; every verification command and result; and every
independent-review finding fixed or refuted with rationale.

State plainly that coverage figures describe the named locked corpus under the
named strict policy, and that a type-disjoint holdout measures generalization
to unseen *types within that corpus* — not to all historical, regional, or
future Church Slavonic.
