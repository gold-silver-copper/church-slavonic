# Synodal v0.13 iteration-velocity audit

Program prompt: `SYNODAL_V13_ITERATION_VELOCITY_PROMPT.md`. Objective: one
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
