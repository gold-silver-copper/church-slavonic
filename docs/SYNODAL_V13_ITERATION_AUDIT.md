# Synodal v0.13 iteration-velocity audit

Program prompt: `docs/goals/SYNODAL_V13_ITERATION_VELOCITY_PROMPT.md`. Objective: one
admission wave from candidate list to green local close in under 15 minutes
with zero failed seals, with no guard weakened.

## Baseline (measured during v0.12, waves 10–15)

Wall-clock components of one wave on the development machine (release builds):

| Step | Typical | Notes |
|---|---:|---|
| Candidate/attestation queries (surface index) | 1–2 min | scratchpad pickle, fast |
| Data-row drafting | 2–5 min | scripted appends |
| `synodal-regenerate` + release rebuild | 1–2 min | rebuild mandatory (registry compiles in) |
| `synodal-accent-fit --apply` + regenerate + rebuild | 1–2 min | |
| `synodal-evaluate` + abstention fixes | 1–10 min | manual paradigm-row guessing when the fitter left cells unscopable |
| Canonical coverage seal | ~4 min | full 1,313,344-token re-analysis |
| **Reseal loops after ceiling failures** | **+5–15 min each** | wave 11: 2 reseals; wave 12: 2; wave 14: 1 |
| Ordered closing suite (16 commands) | 3–6 min | family queue slowest; ordering manual |
| Late `synodal-check` passage-collision fixes | +2–6 min | one violation reported per run; waves 11, 14, 15 needed 2–4 round-trips |
| fmt/clippy/tests | 2–3 min | |
| Commit + CI watch | 8–12 min | CI failed on stale derived artifacts twice (waves 8 ordering-fix era, wave 13 marginal-recovery) |

Observed totals: **~35–60 minutes per wave**, of which roughly half was
rework triggered by failures that are statically detectable from the TSVs:

- 7 duplicated lexeme identities discovered only by the
  `integrity:cross_lexeme_ambiguous` ceiling after a full coverage run
  (поразити, вѣровати, лꙋкавый, гробъ, хранити, десный, ѡставити);
- 4+ evaluation-passage collisions reported one-at-a-time by `synodal-check`;
- 1 class/formation mismatch (беззаконный) that silently produced a
  generates-nothing lexeme until evaluation abstained;
- 1 full cycle lost to evaluating against a binary older than the
  regenerated registry;
- 2 CI failures caused purely by closing-suite ordering (stale
  `lexical_source_claims.tsv` era; stale `synodal-marginal-recovery.json`
  at wave 13).

## Phase 1 — Admission preflight (`synodal-admit-check`)

Landed as `crates/xtask/src/synodal_admit_check.rs`, wired into
`synodal-check` (CI-enforced via the structural job), with four guard
witnesses in `synodal-guard-witnesses` and two regression tests replaying
real v0.12 failures (the лꙋкавство duplicate identity and the вѣ́рный
Apoc.1.5 passage collision). Categories: duplicate identities (surfaces owned
by one identity that the registry analyzes to another, plus normalized-lemma
collisions, against `data/synodal/homonymy_allowlist.tsv` — 46 reviewed
pairs, split between genuine cross-POS homonymy and frozen v0.6/v0.7
duplicates awaiting the identity-merge wave); new held-out memorisation
against the frozen 93-type baseline `holdout_memorisation_baseline.tsv`;
evaluation-passage overlap restricted to runtime-referenced evidence ids
(exact parity with the extractor predicate — an earlier draft that swept all
citations over-reported by 12); and generation-dead lexemes probed through
lemma, owned surfaces, evaluation expectations, and stem-derived candidates.

**Immediate payoff:** the first run against the "green" post-v0.12 tree found
two live duplicated identities that all v0.12 ceilings had missed because the
pairs never co-covered enough tokens to trip the ceiling: `вѣрный`
(wave 15) duplicated `synodal:adjective:v06-a79476be07ef953c` (review lemma
вѣрьнъ) and `лꙋкавство` (wave 15) duplicated
`synodal:noun:v07-9c2563bd3383fa6d` (review lemma лѫкавьство). Both were
merged onto the existing identities (lemma preservation satisfied by new
exact `lexical-form` rows for the Synodal lemma prints, both non-held), and
the lexicon change was sealed as `v0.13-prep-merges` (coverage unchanged,
lexemes 1,076 → 1,074). Preflight wall clock: ~4 seconds.

## Phase 2 — One-command wave close (`synodal-wave-close`)

Landed as `crates/xtask/src/synodal_wave_close.rs`: the full closing suite as
one in-process command with a pass/fail table and per-step timings. `--check`
is read-only; the CI structural job now invokes `synodal-wave-close --check`
instead of its inline command list, so the canonical ordering lives in exactly
one place. Steps that recompute from the gitignored intermediate corpus
(accent-fit, family-review-queue) self-skip when it is absent — verified by
running the command with `adapter-reports.json` moved aside (11 steps green,
matching the CI environment) and restored (13 steps green). This *adds* CI
enforcement relative to the old inline list: `synodal-lexical-union --check`
now runs in CI, closing the stale-union failure class permanently. `--fix`
regenerates the derived artifacts in canonical order (union last) and prints
undecided top-200 family proposals as review stubs on that gate's failure.
The default local mode appends fmt/clippy/tests/doc-tests: 17 steps, ~2
minutes wall clock, replacing what was previously ~16 hand-ordered commands.

## Phase 3 — Registry staleness tripwires

Each runtime crate gained a build script embedding an FNV-1a fingerprint of
its `generated/registry.rs` as `REGISTRY_FINGERPRINT`. The xtask measurement
entry points (`synodal-evaluate`, `synodal-coverage`, `synodal-accent-fit`)
now call `ensure_registry_current` and refuse a stale binary with the exact
rebuild command; `synodal-regenerate` skips its in-process evaluation and
prints a REBUILD REQUIRED banner whenever its write changed the registries
(previously it silently wrote an evaluation report measured against the old
compiled data). Verified live (tamper → refusal → restore → pass), witness-
tested ("stale compiled registry" injection in `synodal-guard-witnesses`),
and checked against the portable-runtime and publish-dry-run constraints
(build scripts run host-side; wasm/no-default builds and `cargo publish
--dry-run` all pass).

## Phase 4 — Delta coverage projection

`synodal-coverage --offline --delta` projects the ledger columns from a
distinct-surface inventory (63,507 surfaces) written by every canonical run
into the gitignored intermediate directory. The projection reuses the very
`classify_token`/`update_slice`/`update_integrity`/holdout-status code the
corpus loop uses (a new `project_surface_counts` API in the dictionary
coverage module), so equality with the full run is by construction, and the
acceptance check confirmed it empirically: on the sealed v0.13-prep-merges
tree, the projection reproduced generalised 14,236, memorised 14,998,
holdout top-k 30,293, and corpus top-k 990,913 **exactly (+0 on every
column) in 17.6 seconds** versus the ~4-minute canonical run. The output is
stamped `PROJECTION — not sealable` and shows deltas against the last sealed
ledger row; combining `--delta` with any sealing or checking flag is an
error, and three guard witnesses inject exactly those combinations.

## Phase 5 — Accent scope grammar and `--suggest`

The complete accent-paradigm scope grammar (every parser arm, placement kind,
and mark, with worked examples from the live data) is now documented in
`docs/SYNODAL_MORPHOLOGY.md`. `synodal-accent-fit --suggest <lexeme> <cell>`
produces the exact insertable row — verified by a round-trip that inserted
the suggested `злый` comparative-plural row into the live fitted block,
regenerated cleanly, and reverted — reusing the fitter's own placement search
and scope derivation, with the block paradigm ID and block-uniform evidence
when a block exists. Refusals are specific: not-expanding, not-in-gap,
no-witness, conflicting witnesses (each print listed — the мѡа́влѧ/мѡа̑влѧ
kamora homography, честна̀/че́стна), and the лакте́й case is reported exactly
as required: "unfittable without memorisation: every corpus witness (ла́ктей,
лакте́й) is itself a held-out type". Automatic scope *widening* in `--apply`
was deliberately not automated: every real widening case in v0.12
(входи́те/вхо́дите) is a same-letters homography conflict that the fitter
refuses by design, and the conflict listing from `--suggest` is the review
aid for exactly that human decision.

## Phase 6 — Measured demonstration and completion

Two demonstration waves ran end-to-end with the new tooling:

- **первенецъ** (first-mixed-ts-m): 17m16s total, including two first-use
  tooling defects that the tooling itself surfaced mid-wave and that were
  fixed on the spot (the generation-probe guard witness had been broken by
  the pending-rebuild refinement, and a hand-edited family row was one column
  short — caught by the family gate inside `wave-close --fix`). Zero failed
  seals. This wave also hardened the preflight: a lexeme absent from the
  compiled registry is now reported as pending-rebuild, not generation-dead.
- **вертепъ** (first-hard-m), the clean replay: **6m17s** from candidate rows
  to all 23 `wave-close --fix` steps green — preflight 4s, regenerate+rebuild
  ×2 ≈ 60s, accent-fit apply 47s, evaluate 2s, delta projection 18s
  (pre-seal total 126s), canonical seal ≈ 4min, one-command close in
  parallel-free sequence. Zero failed seals; floors and the ledger hold.

Baseline comparison: v0.12 waves ran 35–60 minutes with reseal loops and
one-at-a-time late-check round-trips; the same shape of wave now closes in
about six minutes with every failure class surfaced up front.

## Completion gate review

1. `synodal-admit-check`: CI-enforced through `synodal-check`; guard
   witnesses for all four categories; regression tests replay the лꙋкавство
   duplicate and the вѣ́рный passage collision. **Holds.** (First run also
   found and merged two live duplicates the v0.12 ceilings had missed.)
2. `synodal-wave-close` reproduces the suite; the CI structural job invokes
   `--check`; ordering is single-sourced. **Holds.**
3. The staleness tripwire refuses stale-binary measurement, witness-tested
   ("stale compiled registry"). **Holds.**
4. `--delta` matched the full run **exactly (+0 on every ledger column)** on
   the sealed tree in 17.6s, and sealing from a delta is rejected and
   witness-tested (three injected flag combinations). **Holds.**
5. The accent scope grammar is documented; `--suggest` emitted a row for the
   live злый comparative block that round-trip-inserted cleanly, and refuses
   лакте́й as unfittable without memorisation. **Holds.**
6. The timed вертепъ wave: 6m17s, zero failed seals, recorded above.
   **Holds.**
7. No floor or ceiling weakened (the only bound edits were the two justified
   v0.12-wave-12 notes predating this program); all 31 bounds hold; the full
   suite, clippy, fmt, doc tests, and CI are green. **Holds.**

The v0.13 iteration-velocity program is complete.
