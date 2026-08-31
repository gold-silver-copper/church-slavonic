# Prompt: finish church-slavonic and ship 1.0.0

Copy everything below this line into a fresh Claude Code session started in the repo root.

---

Take this project to a finished 1.0.0. "Finished" does not mean every number is 100% — the
held-out corpus-recall gap is an asymptote, not an error budget. Finished means: the schema
scope is closed, the data ceiling is measured and documented, the release is published, and
the wave process is retired. Work autonomously through every phase; do not stop to ask
questions unless you hit a licensing or credential wall you genuinely cannot resolve.

## Ground truth about this repo (verify, then trust)

- Workspace of 4 crates: `church-slavonic` (facade + generated PHF tables),
  `church-slavonic-core` (rules; one resolution engine in `core::resolution` that the
  runtime, extractor, and audits all share), `extractor`, `xtask`.
- Three commands: `cargo xtask refresh-data` (regenerate tables from pinned sources),
  `cargo xtask check-registry` (source-free structural gate), `cargo xtask accuracy`
  (scores every table source and the held-out corpus-recall splits).
- Sources are pinned by sha256 in README §"Obtaining Data"; `scripts/fetch-sources.sh`
  downloads and verifies them. The two treebank tarballs are placed by hand under
  `references/downloads/`.
- Both published crates are versioned in lockstep (currently 0.9.0). Publish order:
  `church-slavonic-core`, then `church-slavonic`.

## Invariants — violating any of these is failure, whatever else improves

1. **Held-out data never becomes a table cell.** UD PROIEL dev/test and Syntacticus are
   evaluation-only under `references/TERMS.md`. Do not ingest them, do not tune against
   them by hand, do not "fix" individual recall misses by adding rows.
2. **Every table source scores 100% with variant gap 0** after any change
   (`cargo xtask accuracy`). OCS output stays byte-identical wherever the current
   changelog/audits say it must.
3. **Judge every ending-choice or rule change by full refresh row/cell delta, never by
   counting stored-form misses.** Stored tallies see only rule misses; the covered
   majority is invisible. This lesson killed three plausible rule flips in v0.9.0
   (possessive AccSg flip, LocPl -ахъ, long LocSg -омъ). It is not optional.
4. **Before every commit, verify `crates/church-slavonic/examples/` still contains
   `speedmark.rs` and `test.rs`.** They have been stray-deleted three times.
5. Settled verdicts — do not relitigate without new table data: the accent census and the
   letters-differ census are mined out (v0.8.0/v0.9.0); mobile-stress scheme tokens,
   computed kamora markers, unconditional widen-anywhere, and coverage gate 1 were all
   measured and rejected. The remaining stored mass is irreducibly lexical at current
   source coverage.

## Phase 0 — Baseline

Run `check-registry` and `accuracy` on a clean tree. Record every number in
`NOTES.md` (create it) as the baseline block. If anything is not green, fix that first.

## Phase 1 — Close the schema scope

README §recall names the two remaining schema gaps: **non-personal pronouns** and the
**l-participle** (both currently counted under "skipped by reason" in the recall harness).
Implement both, in that order, the way every prior paradigm class was added:

- Rule first in `church-slavonic-core` (grammar sources: Alypy for Synodal, the OCS
  grammar tables already in the corpus dictionary/Wiktionary extracts), then extractor
  subtraction so only exceptions are stored, then wire the recall harness to stop
  skipping those tokens and score them.
- Gate: all table sources back to 100%/0; held-out recall must not regress for any
  already-scored part of speech; the newly scored categories' recall gets recorded in
  NOTES.md, whatever it is.
- These are additive schema widenings, not census re-mining, so invariant 5 does not
  block them.

## Phase 2 — New table sources, bounded

The recorded program verdict is that further table shrinkage needs new attested data,
not new mechanism. Spend one bounded pass (aim: about a day of work, not a week):

1. Enumerate candidate sources not yet in the pinned list. Known names to check first:
   Sobolevsky and Đorđić grammar paradigm tables, the Old Church Slavonic dictionary
   (Cejtlin/SJS) if machine-readable, any further accented Synodal grammar in the
   public domain. A candidate qualifies only if: license permits table extraction
   (or fits the existing institutional grant), forms are attested paradigms (not
   reconstructions), and it is machine-extractable with a pinnable artifact.
2. For each qualifying source: pin it (sha256 + fetch script), extend the extractor,
   refresh, and record the delta (new rows, cells un-stored, recall movement).
3. After ingesting all qualifying sources, re-run the accent and letters-differ censuses
   **once**. Implement any lever whose full-refresh delta is ≥500 rows. Reject smaller
   levers without exception — a 30-row flip is how this project fails to end.
4. When a full pass produces no qualifying source and no ≥500-row lever, the data
   ceiling is reached. Write the ceiling block in NOTES.md (what was searched, what was
   rejected and why) and stop mining permanently.

If no candidate source qualifies at step 1, say so in NOTES.md and skip to Phase 3 —
that outcome still counts as reaching the ceiling.

## Phase 3 — Retire the process

- Move the research-diary material ("Findings, stated plainly", rejected designs,
  census numbers) out of CHANGELOG.md into NOTES.md. Rewrite the changelog as a
  consumer-facing history; add the 1.0.0 entry stating the scope now covered and the
  documented ceiling.
- Regenerate every number in README.md from actual `accuracy` and `speedmark` runs —
  no hand-edited figures.
- Confirm `cargo doc --no-deps` builds clean for both published crates and
  `cargo publish --dry-run` passes for each.

## Phase 4 — Ship

- Bump both published crates to 1.0.0 (lockstep). Commit with the same style as prior
  waves (short declarative title, e.g. "Non-personal pronouns, the l-participle, and
  the 1.0.0 close").
- Tag `v1.0.0`, push commits and tag, publish `church-slavonic-core` then
  `church-slavonic` to crates.io.

## Definition of done — every box checked, verified by running the command, not by memory

- [ ] `cargo xtask check-registry` green
- [ ] `cargo xtask accuracy`: every table source 100.00%, variant gap 0
- [ ] Non-personal pronouns and l-participle inflect and are scored (not skipped) by
      the recall harness
- [ ] Held-out recall for previously scored categories at or above the Phase 0 baseline
- [ ] Phase 2 executed to its stopping rule; ceiling block written in NOTES.md
- [ ] CHANGELOG consumer-facing; NOTES.md holds the diary; README numbers regenerated
- [ ] `crates/church-slavonic/examples/` intact in the final commit
- [ ] 1.0.0 published (both crates), `v1.0.0` tag pushed

Anything genuinely impossible (source unobtainable, license refused): record it in
NOTES.md with what you tried, and continue — the checklist item then reads as
"documented as unreachable", which still counts as done.
