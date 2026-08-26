# Prompt: replace the wave system with gold-source oracles

You are working in the church-slavonic workspace. Read `docs/REWRITE_PLAN.md`
(executed) first: the OCS side already made this exact transition — it dropped
frontier metrics and review queues in favour of replaying **every** attested
cell of its source as a hard 100% gate (`cargo xtask rewrite-pilot-accuracy`).
Your task is to do the same for the Synodal family, and to retire the wave
system it replaces.

## The problem being solved

The current synodal harness measures progress against an open-ended corpus:
sampled coverage fixtures (10 passages, 155 tokens), a top-200
family-review queue, a holdout-memorisation ratchet, per-wave generalisation
ledgers, and marginal-recovery accounting. That design made sense while the
question was "how much of an unbounded language do we cover?" It has three
costs we no longer accept:

1. **Arbitrary cutoffs.** "Top 200 proposals" and "10 fixture passages" are
   samples; passing them proves nothing about token 201 or passage 11.
2. **Ratchet bureaucracy.** Every engine improvement forces baseline
   re-keying, ledger appends, and queue re-ranking — bookkeeping that now
   dominates the commit stream.
3. **Environment splits.** The gates mean different things locally (full
   3 GB corpus) and in CI (fixtures), so "green" is ambiguous.

## The new contract

**Declare a closed set of gold sources. The library is correct when it
reproduces 100% of every gold source. Nothing else is measured.**

### Gold sources (initial set — confirm each is already checksummed under `references/`)

1. **The Elizabeth Bible (Ponomar)** — `ponomar-elizabeth-bible-2026-08-09`,
   already the evidence backbone. Gold means: every token of every verse.
2. **Wiktionary/Kaikki Church Slavonic** — the same pinned dump the
   dictionary senses come from. Gold means: every inflection-table cell of
   every entry.

Do not add more sources in this pass. A source is either gold (100%,
CI-enforced) or it is not consulted at all; there is no "partially trusted"
tier.

### What 100% means, per source shape

- **Corpus tokens (Bible):** for each token, `analyze_text` must produce at
  least one reading whose lemma+cell the reviewed registry attests for that
  surface, AND the generation direction must round-trip: inflecting that
  reading's lemma+cell must reproduce the printed surface exactly —
  accents, positional letter choices, titlo expansion state included.
  Define one canonical normalization for comparison (NFC + the existing
  lookup projections) and document it in the report header.
- **Paradigm cells (Wiktionary):** exact variant-list match per cell, the
  way `rewrite-pilot-accuracy` already defines it for OCS: rules first, and
  a residue table for whatever the rules cannot reproduce verbatim.
  Cells the source itself leaves empty are absent, not failures; typed
  defects (`HistoricallyAbsent` vs `EvidenceIncomplete`) remain first-class.

### The gap is a worklist, not a queue

Build `cargo xtask synodal-gold --check` to replay both oracles and, on any
shortfall, emit the **complete enumerated gap** —
`reports/synodal-gold-gap.tsv`, one row per failing token/cell with the
current best analysis and the reason class (no-reading, wrong-accent,
wrong-surface, unregistered-lemma, …). No ranking, no top-N, no sampling.
Human review still exists — accepting a new lexeme or an exact form is
still a curated decision — but reviewers work down a finite list that only
shrinks, instead of a re-rankable queue. 100% on both oracles makes the
gap file empty and the check green; that is the only definition of done.

### Execution plan

1. **Freeze and measure.** Build the two oracle replays (reuse the OCS
   pilot's loaders and gate style; the Bible side builds on `analyze_text`
   and the coverage machinery's tokenizer, minus its sampling). Commit the
   first full gap report as the baseline — expect it to be large; that is
   the honest starting point the wave system was hiding.
2. **Make CI and local identical.** Gold sources are pinned and small
   enough to fetch in CI (or commit their extracted oracle TSVs the way
   `data/extracted` works for OCS). The fixture bootstrap, coverage floors,
   and the local-vs-CI corpus split all go away with the wave system.
3. **Burn down the gap** in whatever order is convenient — by book, by
   POS, by reason class. Each PR must strictly shrink
   `synodal-gold-gap.tsv` and keep every other gate green. Engine fixes,
   new reviewed lexemes, exact forms, and accent paradigms are all valid
   moves; use the one-override-precedence channel only (merged irregular
   table → defects → rule kernel).
4. **Retire the wave machinery** once the gap gate is wired into
   `check-structure`: delete `synodal-waves`, `synodal-wave-close`,
   `synodal-admit-check` and the holdout baseline, the family/lexical/
   evaluation review queues, `synodal-marginal-recovery`,
   `synodal-coverage`'s fixture/floor modes, `synodal-accent-fit`'s
   report ratchet, and their ledgers under `reports/` and
   `data/synodal/` (`held_out_types.tsv`,
   `holdout_memorisation_baseline.tsv`, `synodal-waves.tsv`, …).
   Keep: the sources/checksum machinery (`synodal-sources`), the
   extractor, `synodal-evaluate` only if its rows are re-expressed as gold
   cells, and the immutable archive (append a closing entry noting the
   wave program ended and why). Follow the phase-1 deletion precedent:
   pure removal, one commit, docs to `docs/history/`.
5. **Hard rules carried over from the rewrite:** every slice lands green;
   no sampling anywhere in a gate; no gate that means different things in
   different environments; the gap file is regenerated by the check itself
   so it can never go stale.

### Out of scope

- Adding gold sources beyond the two named (design the source list to be
  extensible — adding one later means adding its oracle loader and
  accepting a temporarily non-empty gap — but do not do it now).
- Predictive/exploratory tiers (`synodal-predict`): keep the code if it
  helps burn-down triage, but it gates nothing.
- Any OCS-side change: that family is already on this model.

### Report back

The baseline numbers (gap size per source and per reason class), what was
deleted vs kept from the wave machinery, and the shape of
`synodal-gold --check` output.
