# Prompt: burn down the Synodal gold gap

You are working in the church-slavonic workspace. Read first:
`docs/SYNODAL_GOLD_ORACLE.md` (the normative comparison contract — it
governs every pass/fail), `docs/SYNODAL_GOLD_ORACLE_PROMPT.md` (executed:
the gate and its semantics), `docs/UNIFIED_LANGUAGE_PROMPT.md` and
`docs/UNIFIED_FACADE.md` (executed: the merged kernel, identity layer, and
§5's ordering of the final analyze-layer merge behind this burn-down).

## The starting position (exact, from `reports/synodal-gold-gap.tsv`)

| oracle | reason class | rows | remediation path |
|---|---|---:|---|
| token | `unregistered-lemma` | 51,235 | curated lexeme admission |
| token | `abbreviation-unexpanded` | 1,080 | titlo/abbreviation review |
| paradigm | `unregistered-lemma` | 934 | admit Alypy's own headwords |
| token | `engine-wrong-form` | 295 | code fix |
| paradigm | `unreviewed-cell` | 250 | cell evidence review |
| paradigm | `engine-wrong-form` | 36 | code fix |
| token | `engine-wrong-accent` | 35 | code fix |
| paradigm | `engine-wrong-accent` | 14 | code fix |

Plus 6 witness-adjudicated defect candidates in
`reports/synodal-gold-defect-candidates.tsv`, 1,631 identity candidates
in `data/unified/identity-candidates.tsv`, and the projection study's
finding that 5,118 of the unregistered token types are reachable from an
OCS-attested lexeme.

## The contract of this program

- **The gap file is the only progress metric.** Every slice ends with
  `cargo xtask synodal-gold --fix` producing a strictly smaller
  `reports/synodal-gold-gap.tsv`; `--check` remains subset-only. There are
  no percentages over samples, no queues, no ratchets — only the exact
  enumerated remainder, per class.
- **Rules first, residue second.** An admission is a *lexeme with a
  declension/conjugation class* so the merged kernel generates its cells;
  exact forms enter the merged irregular table only for genuine
  irregularity the rules cannot reach. Track the rules-vs-residue split the
  way the OCS pilot does and report it per slice; a slice that clears gap
  rows mostly by exact-form dumping is a smell, not a win.
- **Review-by-oracle is legitimate; invention is not.** The gold sources
  are the evidence: a lexeme admitted because its attested Bible cells
  (with their printed accents) and/or its Alypy table are reproduced by
  the assigned class is reviewed *by the oracle*. Every admission carries
  provenance (source id, verse references or Alypy section). A form with
  no gold attestation is never written down as fact.
- **The accent asymmetry stands.** Projection-seeded admissions supply
  identity and class; accent facts come only from the Bible's printing or
  the accent paradigms. An admission whose accented cells are not yet
  evidenced leaves those cells in the gap honestly.
- **One override precedence** (merged irregular table → typed defects →
  rule kernel). Never reintroduce provider/caller override channels.
- **Every slice lands green**: `cargo test --workspace`, `check-structure`
  (which runs `synodal-gold --check`, `rewrite-pilot-accuracy`,
  `unified-identity --check`), clippy `-D warnings`, fmt. The OCS oracle
  stays at 100% and the identity coherence baseline may only improve.

## Order of work — by remediation class, then by leverage within a class

Ordering by leverage is not sampling: every row remains in the gate at
all times; ordering only decides what to fix first.

1. **Engine bugs first** (380 rows: `engine-wrong-form` + `engine-wrong-
   accent`, both oracles). Code-only, zero curation, and each fix may
   clear rows outside its own class. Group by the divergence registry
   and the kernel module they implicate; fix root causes, not surfaces.
   Any fix that would change an OCS oracle cell is a merge-layer bug —
   route it through the named-divergence registry, never a family fork.
2. **Alypy headwords and cells** (934 + 250 paradigm rows). The grammar's
   own example words are normative: admit each headword with the class
   its table proves, and review the 250 cells against the table rows.
   This also grows the paradigm oracle's servable coverage in cells the
   Bible never attests. Target: the paradigm oracle at 0 gap rows — a
   fully reproduced grammar is a milestone worth its own tag.
3. **Abbreviations** (1,080 token rows). A closed liturgical inventory
   (nomina sacra and their inflected titlo forms). Extend the abbreviation
   review data so each titlo surface expands to a reviewed reading, with
   the expansion round-tripping under the contract's §3 equivalence class.
   Includes the known `сн҃а`-as-numeral misclassification.
4. **Projection-seeded admissions** (up to 5,118 token types). For each
   identity candidate (start with the 599 confirmed entries' unregistered
   Synodal cells, then the candidates file), assign the class the merged
   kernel needs, generate, and admit where the Bible's attested cells —
   accents included — are reproduced. Ambiguous candidates resolve by
   which projection actually matches attested cells; unresolved ones stay
   candidates. Each admission also grows `data/unified/identity.tsv`
   (the coherence gate then measures it).
5. **The long tail** (the remaining `unregistered-lemma` types). Cluster
   gap types by lemma hypothesis: run the merged kernel's analysis over
   each type to propose (lemma, class) hypotheses, group types by
   hypothesis, and order hypotheses by *attested cells cleared per
   admission* (a lexeme whose 40 attested cells all reproduce under one
   class is worth more than a hapax). Admit a hypothesis only when its
   class reproduces every attested cell of the cluster; partial
   reproduction is a signal to look for a subclass or a genuine
   irregular, not to force it. Proper names, foreign transliterations,
   and hapaxes will dominate the end of this tail — classify them
   honestly (indeclinable, foreign-stem class) rather than skipping.
6. **Defect ledger.** Adjudicate the 6 candidates under the two-witness
   rule; move confirmed ones to `data/synodal/gold_source_defects.tsv`
   with witness evidence. Re-run the sweep after each slice that touches
   engine output; new candidates get the same treatment.
7. **The terminal step, gated on the above:** when the identity table
   covers the identified lexicon and the gap's `unregistered-lemma` mass
   is gone or reduced to the documented honest residue, execute the
   analyze-layer merge per `docs/UNIFIED_FACADE.md` §5 — the synodal
   `analyze_text` and the unified dictionary's `lemmatize_in` become one
   recension-aware analysis with attested > cross-recension-projected >
   rule-predicted ordering — and deprecation-release the synodal crate
   names per the established precedent.

## Slice discipline

- One slice = one class (or one cluster within a class), one PR-sized
  commit, message stating the per-class gap delta and the rules-vs-
  residue split of what was admitted.
- Commit the regenerated gap file with the slice; never regenerate it
  separately from the change that shrank it.
- Regenerate downstream artifacts in the same slice (`synodal-regenerate`,
  the gold oracles' `--check`, `unified-identity`, `rewrite-emit-residue`
  where OCS-side data moved) so `check-structure` stays green at every
  commit.
- Where a class is exhausted of oracle-reviewable moves and what remains
  needs human linguistic judgment (a sense split, a disputed reading, a
  source pair the witnesses cannot settle), write those rows to a
  `reports/synodal-gold-human-review.tsv` with the specific question per
  row, and move on. That file is the only thing this program hands to a
  person; it must never contain anything an oracle could have settled.

## Out of scope

- New gold sources (the Ponomar 2016 corpus is queued in the gold prompt;
  add it only after the Bible gap is substantially burned down, and
  accept the larger gap it brings as a new baseline in its own slice).
- Any change to the comparison contract to make rows pass. The contract
  changes only through its own reviewed revision, documented in
  `docs/SYNODAL_GOLD_ORACLE.md`, never inside a burn-down slice.
- Re-ranking, sampling, or top-N of any kind in any gate.

## Report back (per slice and at each class boundary)

The gap table above, updated with exact counts; rules-vs-residue split of
admissions; identity-table growth and coherence-gate movement; the
human-review file's size and its question categories; and, at class
boundaries, whether the next class's leverage ordering still holds.
