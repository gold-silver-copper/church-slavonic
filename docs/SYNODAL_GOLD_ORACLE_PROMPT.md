# Prompt: replace the wave system with a gold-source oracle

You are working in the church-slavonic workspace. Read `docs/REWRITE_PLAN.md`
(executed) first: the OCS family already made this transition — it replays
**every** attested cell of its pinned Wiktionary source as a hard gate
(`cargo xtask rewrite-pilot-accuracy`, 100%, no sampling). Your task is to
bring the Synodal family onto the same model and retire the wave system.

## The problem being solved

The current synodal harness measures progress against an open-ended corpus:
sampled coverage fixtures (10 passages, 155 tokens), a top-200 family-review
queue, a holdout-memorisation ratchet, per-wave generalisation ledgers, and
marginal-recovery accounting. Three costs are no longer accepted:

1. **Arbitrary cutoffs.** "Top 200 proposals" and "10 fixture passages" are
   samples; passing them proves nothing about token 201 or passage 11.
2. **Ratchet bureaucracy.** Every engine improvement forces baseline
   re-keying, ledger appends, and queue re-ranking.
3. **Environment splits.** Gates mean different things locally (full 3 GB
   corpus) and in CI (fixtures), so "green" is ambiguous.

## The new contract

**One gold source. The entire source is enumerated in every check. The gap
is a committed, finite worklist that may only shrink. Green means "the full
corpus was replayed and nothing regressed"; the program's end state is an
empty gap.**

Two deliberate differences from a naive "100% or red" gate, and from the old
ratchets:

- The reviewed lexicon (937 lexemes) is small against the Bible's
  vocabulary, so the gap will be large for a long time. A gate that stays
  red until it empties trains everyone to ignore CI. Instead the gate is a
  **full-enumeration no-regression check**: it replays the whole corpus,
  regenerates the gap, and fails if the gap differs from the committed
  baseline in any direction except shrinking. Unlike the old ratchets this
  samples nothing and ranks nothing — every token is checked every time.
- The old system's numbers ("65% top-k") were statistics over samples. The
  only numbers here are exact counts over the full enumeration.

### The gold source

**The Elizabeth Bible** — `ponomar-elizabeth-bible-2026-08-09` (pinned and
checksummed under `references/`; 37,211 verse records, ~53 MB JSONL). Gold
means: every token of every verse.

**Explicitly not gold for this family:** the Wiktionary/Kaikki dump
(`english-wiktionary-ocs-kaikki-2026-08-07`) is an *Old Church Slavonic*
source. It is already the OCS family's gold oracle, and in the synodal data
it appears only as cross-recension evidence whose identities require human
confirmation. It stays what it is: evidence feeding reviewed admissions,
never a synodal truth source. (The reviewed `synodal-evaluate` rows — 2,499
curated passages — are already-confirmed gold material: fold them into the
oracle as a third input rather than keeping a separate command.)

Do not add further sources in this pass. Design the source list to be
extensible — adding one later means adding its loader and accepting a
temporarily larger gap — but a source is either gold (fully enumerated,
gated) or not consulted by any gate at all. No partially-trusted tier.

### What 100% means for a Bible token

For each token: `analyze_text` must produce at least one reading whose
lemma+cell the reviewed registry attests for that surface, AND generation
must round-trip — inflecting that reading's lemma+cell under the
liturgical orthography profile must reproduce the printed surface.

Define the comparison contract precisely, in one place, before building
anything (`docs/SYNODAL_GOLD_ORACLE.md`, normative):

- One canonical normalization (NFC + the existing lookup projections) for
  both sides of every comparison.
- **Equivalence classes for typography that is not morphology**: a token
  printed under a titlo compares against the expansion the abbreviation
  module produces for it; verse-initial capitalization and the initial-uk
  presentation compare case-insensitively/in presented form; punctuation
  and versification marks are not part of any token. Every equivalence
  class must be enumerated in the normative doc — anything not listed is
  compared exactly, accents and positional letter choices included.
- Non-lexical tokens (letter-numerals, foreign-script insertions) get an
  explicit typed classification, not silent exclusion: they appear in the
  oracle with a `non-lexical:<kind>` tag and are gated on classification
  stability rather than morphology.

### The oracle artifact must make CI and local identical

Do not read the 53 MB JSONL in the gate. Phase 1 derives and commits a
**type-level oracle** the way `data/extracted` works for OCS: one row per
distinct surface type with its attestation count and a bounded list of
verse references (evidence pointers), generated deterministically by an
xtask command from the pinned source, with the generator command named in
the file header and a staleness check (regenerate-and-compare) in the gate.
Estimate the size first; if the full type inventory with references exceeds
what the repo policy tolerates (~20 MB, the OCS precedent), trim reference
lists, never types. After this lands, delete the fixture-bootstrap path:
there is no fixture tier any more.

### The gap is a worklist, not a queue

`cargo xtask synodal-gold --check` replays the full type-level oracle and
writes `reports/synodal-gold-gap.tsv`: one row per failing type, with the
current best analysis and a **reason class chosen so that each class has one
remediation path** — at minimum: `unregistered-lemma` (needs a curated
admission), `unreviewed-cell` (lemma known, cell lacks evidence review),
`engine-wrong-form` (rules produce a different surface — a code bug),
`engine-wrong-accent`, `abbreviation-unexpanded`, `non-lexical-unclassified`.
No ranking, no top-N. `--check` fails if the regenerated gap is not a
subset of the committed one; shrinking requires committing the smaller file
(`--fix` rewrites it). Wire `--check` into `check-structure` and CI.

Budget: the full replay must run in under ~5 minutes in CI (the OCS oracle
replays 134k cells in seconds; the type-level Bible inventory is the same
order of magnitude — if it is not, fix the lookup path, do not sample).

### Execution plan

1. **Normative doc + oracle extraction.** Write the comparison contract,
   build the type-level oracle artifact, commit it with its staleness
   check. Fold the `synodal-evaluate` rows in as confirmed readings.
2. **The gate.** Build `synodal-gold --check/--fix`, commit the first full
   gap report as the baseline — expect it to be large; that is the honest
   starting position the sampled fixtures were smoothing over. Wire into
   `check-structure` and CI. From this commit on, CI green means
   "full-corpus, no regression".
3. **Retire the wave machinery** in the same change-set that lands the
   gate (not when the gap empties): delete `synodal-waves`,
   `synodal-wave-close`, `synodal-admit-check` and the holdout baseline,
   the family/lexical/evaluation review queues, `synodal-marginal-recovery`,
   the coverage fixture/floor modes, the accent-fit report ratchet, and
   their ledgers under `reports/` and `data/synodal/`
   (`held_out_types.tsv`, `holdout_memorisation_baseline.tsv`,
   `synodal-waves.tsv`, …). Keep: `synodal-sources` and the checksum
   machinery, the extractor, `synodal-coverage`'s plain analysis mode if
   the dictionary CLI uses it, and the immutable archive (append a closing
   entry recording that the wave program ended and this gate replaced it).
   Anything wave-adjacent inside `synodal::check`'s guard witnesses is
   rewritten to witness the new gate instead. Follow the phase-1 and
   phase-5 deletion precedents: pure removal, docs to `docs/history/`.
4. **Burn-down begins** (ongoing program, outside this prompt's scope):
   PRs shrink the gap by reason class — curated admissions for
   `unregistered-lemma`/`unreviewed-cell` through the one-override-
   precedence channel (merged irregular table → defects → rule kernel),
   code fixes for the `engine-*` classes. Every PR strictly shrinks
   `synodal-gold-gap.tsv` and keeps all other gates green.

### Hard rules

- Every slice lands with `cargo test --workspace` green and
  `check-structure` exit 0.
- No sampling anywhere in any gate; no gate that means different things in
  different environments; no ranked queues.
- The gap file is regenerated by the check itself and can never go stale;
  it can never grow.
- Typed defects, provenance traces, and attested-before-predicted analysis
  ordering are preserved throughout.

### Out of scope

- Additional gold sources (Wikisource Bible, Ponomar corpus 2016, UD/
  Syntacticus, Dyachenko) — candidates for later passes only.
- Predictive/exploratory tiers (`synodal-predict`): keep as a burn-down
  triage tool if useful; it gates nothing.
- Any OCS-side change; that family is already on this model.

### Report back

Baseline numbers (oracle type count; gap size per reason class), the
oracle artifact's size and generation time, the gate's CI runtime, what
was deleted vs kept from the wave machinery, and the normative doc's list
of typography equivalence classes.
