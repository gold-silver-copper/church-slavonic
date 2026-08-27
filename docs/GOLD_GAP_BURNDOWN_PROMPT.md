# Prompt: burn down the Synodal gold gap — fast

You are working in the church-slavonic workspace. Read first:
`docs/SYNODAL_GOLD_ORACLE.md` (the normative comparison contract — it
governs every pass/fail), `docs/SYNODAL_GOLD_ORACLE_PROMPT.md` (executed:
the gate and its semantics), `docs/UNIFIED_LANGUAGE_PROMPT.md` and
`docs/UNIFIED_FACADE.md` (executed: merged kernel, identity layer, §5's
ordering of the final analyze-layer merge behind this burn-down).

## Iteration speed is the design goal

The gap is 53,879 rows. A program that verifies each change with the full
gate battery (~2–3 minutes: `check-structure` ≈100 s, the gold replay
20–36 s in debug, a `synodal-regenerate` + recompile of the generated
registry) cannot burn that down. So this program has two loops, and the
first deliverable is the tooling that makes the inner one fast:

- **Inner loop (per edit, target ≤10 s):** admit a batch → replay only
  what the batch can affect → read the delta. No full battery, no
  regeneration of unrelated artifacts, no CI.
- **Outer loop (per commit/push):** full `check-structure`, CI. Batches of
  inner-loop work land as one commit with the regenerated gap file.

Everything below is organised around keeping the inner loop tight.

## Slice 0 — build the fast loop before burning anything

*Status (2026-08-26): delivered — `cargo gold` alias plus dev-profile
opt-level for the replay's hot crates; `--only`/`--lemma`/`--types-from`
scoped replay; the morphology registry moved to the `generated/registry.dat`
artifact with an xtask-only in-process override; `synodal-gold propose`,
`admit`, and `loop`. Usage is documented in `docs/SYNODAL_DATA_PIPELINE.md`
("The gold-gap inner loop"); the measured numbers are in that slice's report.*

Deliverables, each with a measured time you report:

1. **A release-profile xtask path.** Add a cargo alias (e.g.
   `cargo gold` → `cargo run --release -p xtask -- synodal-gold`) or a
   `[profile.dev.package.*]` opt-level bump for the crates the replay
   spends its time in, so the full gold replay runs in single-digit
   seconds. Measure debug vs release; pick the cheaper of "faster
   profile" vs "fewer allocations in the replay" if either alone meets
   the budget.
2. **Scoped replay.** `synodal-gold --check --only <class>` and
   `--lemma <key>` / `--types-from <file>` so a batch replays only its
   own rows and prints the per-class delta; full replay stays the
   authority for `--fix`.
3. **Incremental registry regeneration.** Measure what `synodal-regenerate`
   plus the recompile of `crates/synodal-church-slavonic/generated/
   registry.rs` costs per admission batch. If it dominates the inner
   loop, do the rewrite plan's deferred move first: the registry's
   lexeme/exact-form data becomes a loaded artifact (or a much smaller
   generated table) so admissions do not recompile a 12k-line file. This
   is allowed to be the biggest piece of Slice 0 — it pays for itself
   within the first thousand admissions.
4. **Hypothesis tooling.** `synodal-gold propose [--class ...]` — for
   every gap type, run the merged kernel's analysis to emit (lemma,
   class) hypotheses, cluster types by hypothesis, and rank clusters by
   *attested cells cleared per admission*. Output is a TSV an admit
   step consumes directly. Ranking orders work; it never filters the
   gate.
5. **Batch admission.** `synodal-gold admit <hypotheses.tsv>` writes the
   admissions (lexeme rows with class + provenance from the gold source:
   verse references or Alypy section) into the curated data, then the
   scoped replay arbitrates: hypotheses whose class reproduces **every**
   attested cell of their cluster (accents included where attested) stay;
   the rest are reverted automatically and written to
   `reports/synodal-gold-rejected-hypotheses.tsv` with the failing cells.
   Humans are not in this loop; the oracle is the reviewer.
6. **A single inner-loop command** that chains 4→5→2 and prints one
   line: rows cleared per class, rules-vs-residue split of what landed,
   elapsed seconds.

Report Slice 0's numbers before proceeding: full replay time (before/
after), scoped replay time, regeneration cost per batch, and a dry run of
propose→admit on the engine-bug-free classes.

## The contract (unchanged from the gold program, restated)

- **The gap file is the only progress metric.** `--fix` produces a
  strictly smaller `reports/synodal-gold-gap.tsv`; `--check` is subset-
  only. No percentages over samples, no queues, no ratchets.
- **Rules first, residue second.** An admission is a lexeme *with a
  class*; exact forms enter the merged irregular table only for genuine
  irregularity. The inner-loop line reports the rules-vs-residue split;
  a batch that clears rows mostly by exact forms is a smell.
- **Review-by-oracle is legitimate; invention is not.** Gold attestation
  (Bible cells with printed accents; Alypy tables) reproduced by the
  assigned class *is* the review. Every admission carries provenance. A
  form with no gold attestation is never written down as fact.
- **The accent asymmetry stands.** Projection-seeded admissions supply
  identity and class; accent facts come only from Synodal evidence.
  Unevidenced accented cells stay in the gap honestly.
- **One override precedence**; no provider/caller override channels.
- **The comparison contract is off-limits to slices.** It changes only
  through its own reviewed revision in `docs/SYNODAL_GOLD_ORACLE.md`.

## Order of work — class by class, batches within a class

Starting position (exact): token `unregistered-lemma` 51,235 ·
`abbreviation-unexpanded` 1,080 · paradigm `unregistered-lemma` 934 ·
token `engine-wrong-form` 295 · paradigm `unreviewed-cell` 250 ·
paradigm `engine-wrong-form` 36 · token `engine-wrong-accent` 35 ·
paradigm `engine-wrong-accent` 14. Plus 6 defect candidates, 1,631
identity candidates, and 5,118 projection-reachable types.

1. **Engine bugs** (380 rows). Code-only; group by the divergence
   registry entry / kernel module implicated; fix root causes. A fix that
   would move an OCS oracle cell is a merge-layer bug — route through the
   named-divergence registry, never a family fork. Scoped replay on
   `--only engine-wrong-form --only engine-wrong-accent`.
2. **Alypy headwords and cells** (934 + 250). The grammar's own words are
   normative: batch-admit headwords with the class their table proves;
   review the 250 cells against table rows. Target: paradigm oracle at 0
   gap rows — tag it.
3. **Abbreviations** (1,080). A closed inventory (nomina sacra and their
   inflected titlo forms): extend the abbreviation review data so each
   titlo surface expands to a reviewed reading round-tripping under §3
   of the contract. Includes the `сн҃а`-as-numeral misclassification.
4. **Projection-seeded admissions** (≤5,118 types). Feed the 599
   confirmed identity entries' unregistered Synodal cells, then the
   candidates file, through propose→admit; ambiguous candidates resolve
   by which projection matches attested cells. Each landed admission
   also extends `data/unified/identity.tsv` (the coherence gate measures
   it).
5. **The long tail.** Loop: `propose` → take the top clusters by cells-
   cleared → `admit` → scoped replay → next. Batches of hundreds, not
   ones. Partial reproduction means "look for a subclass or a genuine
   irregular", never "force it". Proper names, foreign transliterations,
   hapaxes dominate the end — classify honestly (indeclinable / foreign-
   stem classes) rather than skipping.
6. **Defect ledger.** Adjudicate the 6 candidates under the two-witness
   rule; confirmed ones move to `data/synodal/gold_source_defects.tsv`
   with witness evidence. Re-run the sweep after any slice that changes
   engine output.
7. **Terminal step, gated on the above:** when `unregistered-lemma` is
   gone or reduced to the documented honest residue and the identity
   table covers the identified lexicon, execute the analyze-layer merge
   per `docs/UNIFIED_FACADE.md` §5 and deprecation-release the synodal
   crate names per the established precedent.

## Slice discipline

- Inner loop freely; commit when a class-batch is done or the gap has
  shrunk by a meaningful amount (thousands of rows or a class emptied).
  One commit = the data change + the regenerated gap file + whatever
  downstream artifacts that change touched — nothing else.
- Commit message = the inner-loop line (per-class delta, rules-vs-residue
  split). Full tables only at class boundaries.
- Push in batches; let CI be the outer loop. Never wait on CI inside the
  inner loop.
- Where a class is exhausted of oracle-reviewable moves and the rest
  needs human linguistic judgment (a sense split, a disputed reading, a
  witness split), write those rows to
  `reports/synodal-gold-human-review.tsv` with the specific question per
  row and move on. That file must never contain anything an oracle could
  have settled.

## Out of scope

- New gold sources (Ponomar 2016 is queued; add it only after the Bible
  gap is substantially burned down, as its own baseline slice).
- Re-ranking, sampling, or top-N in any *gate*. (Ranking inside
  `propose` orders work; it never filters what the gate replays.)

## Report back

After Slice 0: the measured loop times. After each class: the exact gap
table, rules-vs-residue split of admissions, identity-table growth and
coherence movement, rejected-hypothesis count and dominant rejection
reasons, human-review file size and question categories.
