# Synodal v0.13 — Iteration velocity and process hardening

## Why this program

The v0.12 generalisation program (complete at held-out generalised 14,236;
see `docs/SYNODAL_V12_GENERALISATION_AUDIT.md`) proved the wave workflow
works, but measured against wall-clock it is dominated by avoidable rework:

- **Seal-time failure discovery.** Seven duplicated lexeme identities, several
  held-out-citation violations, and repeated eval-passage collisions were each
  discovered only *after* a ~4-minute canonical coverage run or a late
  `synodal-check`, forcing merge → reseal → re-close loops. Every one of these
  was detectable from the committed TSVs before any build.
- **Full-corpus recomputation.** `synodal-coverage --offline` re-analyzes all
  1,313,344 tokens even when a wave touches ten lexemes.
- **A 16-command manual closing suite** whose ordering is load-bearing
  (`synodal-lexical-union` last; `synodal-marginal-recovery` after queue
  decisions). Mis-ordering broke CI four separate times in v0.12
  (`5f2c61f`-era stale unions, wave-13's stale `synodal-marginal-recovery.json`).
- **Silent staleness.** `synodal-regenerate` without a rebuild is a no-op
  because the registry compiles into the binaries; evaluation against a stale
  binary wasted full cycles.
- **Undocumented accent-scope grammar.** Hand-inserting
  `accent_paradigms.tsv` rows takes guesswork because the scope syntax exists
  only in `crates/synodal-church-slavonic/src/registry.rs`.

The objective of v0.13 is to make one admission wave — from candidate list to
green CI — take **under 15 minutes of wall clock and zero failed seals** in
the common case, without weakening a single existing guard. Guards stay
exactly as strict; they just fire earlier and all at once.

## Ground rules

- No guard, floor, ceiling, or check may be weakened, bypassed, or made
  optional. Preflight *duplicates* checks earlier; the authoritative late
  checks all remain.
- Determinism rules stand: no timestamps or randomness in generated artifacts.
- Every new xtask command gets: a `--check` mode where meaningful, guard
  witnesses if it gates anything, unit tests, and a section in
  `docs/SYNODAL_DATA_PIPELINE.md`.
- Update `docs/SYNODAL_V13_ITERATION_AUDIT.md` after every phase (create it
  with a baseline section first: current per-wave timings, measured).
- Commit per phase; watch CI to green before the next phase.

## Phase 1 — Admission preflight (`synodal-admit-check`)

A command that validates the *data tree as it stands* (so it also works after
rows are drafted) in seconds, with **all** violations reported in one pass,
not first-failure:

1. **Duplicate-identity detection.** For every lexeme, derive its generable
   surface set signature (cheap: stems + class endings, no accent engine) and
   report any pair of lexemes whose surfaces overlap, cross-referenced against
   `lexical_reviews.tsv` and `exact_forms.tsv` so the message names the
   existing id to merge onto. Must catch all seven v0.12 cases
   (поразити/v07-eb0cb660…, вѣровати/wikt-cab2350c…, лꙋкавый/v07-5ac21ff6…,
   гробъ/wikt-ccfc0110…, хранити/wikt-4110b1c6…, десный/v07-754731df…,
   ѡставити/wikt-e0c91b5e…) when replayed against the pre-merge tree — write
   a regression test that replays one of them.
2. **Held-out citation sweep.** Normalize (accent-strip + ᲂ→о fold) every
   print cited in `reviewed_evidence.tsv`, `exact_forms.tsv`, `accents.tsv`
   and report any that is a held-out type. One normalization function, shared
   with the existing checks — not a reimplementation.
3. **Passage-disjointness sweep.** Report every evaluation row whose passage
   appears in *any* runtime evidence source (`reviewed_evidence.tsv`,
   `lexical_reviews.tsv`, fitted accent evidence), listed exhaustively. In
   v0.12 these surfaced one at a time across four separate `synodal-check`
   runs.
4. **Class/lemma/formation consistency.** Run the same consistency predicates
   the runtime uses (lemma↔stem shape per class, principal-part formation
   validity, family-review arm existence for the admitted class) against every
   lexeme, reporting per-lexeme. Must catch the беззаконный
   double-n-reduction/mobile-e mismatch and the паѵелъ missing-class case.

Wire `synodal-admit-check` into `synodal-check` (so CI enforces it) and into
the guard-witness harness with at least one injected failure per category.

## Phase 2 — One-command wave close (`synodal-wave-close`)

A single command that runs the entire closing suite in the canonical order,
regenerating stale derived artifacts instead of failing on them:

- family-review-queue (regenerate, then `--check`; on an incomplete top-200,
  print the undecided proposals as ready-to-review TSV stubs and stop),
- accent-fit (regenerate report, then `--check`),
- fixture bootstrap, coverage fixture, predict, coverage-floors,
- marginal-recovery (regenerate, then `--check`),
- lexical-union **last** (regenerate, then `--check`),
- synodal-check, check-structure, guard witnesses, archive `--check`,
- `cargo fmt --all --check`, clippy, workspace tests, doc tests.

Output: one pass/fail table with timings. A `--fix` flag regenerates what can
be regenerated; without it the command is read-only and CI-safe. Replace the
CI `structural` job's inline command list with `synodal-wave-close --check`
so local and CI ordering can never diverge again.

## Phase 3 — Staleness tripwires

1. Embed the SHA-256 of `generated/registry.rs` into the binaries at build
   time. `synodal-evaluate`, `synodal-coverage`, and `synodal-accent-fit`
   refuse to run (with the exact rebuild command in the message) when the
   on-disk generated file no longer matches the compiled hash.
2. `synodal-regenerate` prints a loud rebuild banner whenever it changed any
   generated file.
3. Guard witness for the tripwire itself (corrupt the generated file, expect
   refusal).

## Phase 4 — Incremental coverage for iteration

Keep the canonical full run as the only sealable measurement. Add
`synodal-coverage --offline --delta`, which:

- caches per-normalized-type resolution results keyed by (registry hash,
  profile, resolver version),
- on a data change, recomputes only types whose analyses could be affected
  (conservative invalidation: any type sharing a normalized stem-prefix with a
  changed lexeme, plus all previously-unresolved types), and
- prints the projected deltas for the ledger columns (holdout generalised /
  memorised, top-k, cross-lexeme ambiguous, top-1) with an explicit
  "PROJECTION — not sealable" banner.

Acceptance: a replay of a v0.12-sized wave shows the delta projection within
±0 of the subsequent full run on the holdout columns, and the delta run
completes in under 30 seconds. Sealing (`--seal-wave`, `--reseal-floors`)
must remain impossible from a delta run — enforce and witness-test that.

## Phase 5 — Accent tooling and documentation

1. Document the complete `accent_paradigms.tsv` scope grammar (every arm of
   the parser in `registry.rs`: `noun:…`, `adjective:…`,
   `adjective-agreeing:…`, `finite:…`, `participle:…`, placements,
   marks) with examples, in `docs/SYNODAL_MORPHOLOGY.md`.
2. `synodal-accent-fit --suggest <lexeme-id> <cell>`: print the exact
   paradigm row (correct paradigm_id, block-consistent evidence, scope
   string) that would realize the requested cell's print, or explain which
   witness is missing — including detecting when every available witness for
   the needed placement is itself a held-out type (the лакте́й case), which
   must be reported as "unfittable without memorisation", not as a row.
3. `synodal-accent-fit --apply` learns to widen an existing scope (e.g.
   `finite:present:singular` → `singular,plural`) when new evidence licenses
   it, instead of leaving the cell unscopable — with the same block-evidence
   discipline it already has for insertions.

## Phase 6 — Measure, document, hand off

- Re-run the timed baseline from Phase 1's audit section on a real replayed
  wave; record before/after wall-clock and failed-seal counts in the audit.
- Add a "wave cookbook" section to `docs/SYNODAL_DATA_PIPELINE.md`: the exact
  command sequence for one wave using the new tooling
  (`synodal-admit-check` → regenerate+build → accent-fit → evaluate →
  `--delta` projection → full seal → `synodal-wave-close` → commit), plus the
  known deferral list pointers.
- Move this prompt to `docs/goals/` and index it, per repo convention.

## Completion gate (all must hold)

1. `synodal-admit-check` exists, is CI-enforced, has guard witnesses for all
   four categories, and a regression test replays at least one real v0.12
   duplicate-identity and one passage-collision case.
2. `synodal-wave-close` reproduces the full closing suite; the CI structural
   job invokes it; local run and CI are order-identical by construction.
3. The staleness tripwire refuses stale-binary evaluation and is
   witness-tested.
4. `--delta` projection matches the full run on holdout columns for a
   replayed wave, runs <30s, and provably cannot seal.
5. The accent scope grammar is documented and `--suggest` produces a correct,
   insertable row for a live unscopable cell (and the correct refusal for
   лакте́й).
6. A timed end-to-end demonstration wave (may use deferred easy admissions,
   e.g. пе́рвенецъ once the mobile-е class exists, or any clean noun batch)
   goes from candidate list to all-green local close in under 15 minutes with
   zero failed seals, recorded in the audit.
7. No floor/ceiling weakened; all 31 sealed bounds hold; full workspace suite,
   clippy, fmt, doc tests, and CI green on main.
