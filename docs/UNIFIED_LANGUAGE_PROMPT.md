# Prompt: merge OCS and Synodal into one language with recension styles

You are working in the church-slavonic workspace. Read first:
`docs/REWRITE_PLAN.md` (executed — its target was already "one kernel,
recension as a parameter, not a crate family", achieved down to the shared
vocabulary and orthography crates), and
`docs/SYNODAL_GOLD_ORACLE_PROMPT.md` (the gold-oracle gates; if not yet
executed, execute it first — this merge relies on its gates to arbitrate
every point where the recensions differ).

## The thesis

Old Church Slavonic and Synodal Church Slavonic are one written language
observed at two points of its history, not two languages. Most of what
separates them is systematic: orthographic reform (jer loss and
simplification, ѫ→у/ю, letter inventory, accentuation becoming explicit),
regular sound correspondences, and stylistic norms. The current codebase
treats them as parallel families (`old-church-slavonic-core` +
`synodal-church-slavonic-core`, two facades, two lexicons, disjoint data),
which forfeits the main prize: **every data source should be evidence for
the one language.** A Wiktionary OCS paradigm and an Elizabeth Bible token
should both constrain the same lexeme.

## The target model — be precise about what is shared and what is not

One `Recension` axis (already a kernel type). Three layers, with a strict
rule about which layer a difference lives in:

1. **Lexeme identity (shared).** One lemma space keyed by an
   abstract/etymological citation form. A lexeme has one identity across
   recensions, with per-recension surface citation forms derived by the
   correspondence rules or, where irregular, stored. The existing
   `mapping.rs`/`RecensionMapping` machinery stops being a bridge between
   families and becomes the identity layer itself.
2. **Morphology (shared, recension-conditioned at named points).** One
   inflection kernel: shared declension/conjugation classes, shared cell
   space (the kernel grammar enums already are shared). Where an ending or
   formation genuinely differs by recension (e.g. instr.sg. variants,
   participle stems, aorist formations available), the rule carries an
   explicit recension condition — a *named divergence*, enumerated in one
   registry, never an implicit fork. The starting inventory of named
   divergences is discovered empirically in phase 1, not asserted.
3. **Realization (per-recension).** Orthographic projection (jers,
   nasals, letter inventory), accentuation (mandatory in Synodal
   liturgical output, absent in OCS output), positional typography, and
   titlo presentation. This is `church-slavonic-orthography`'s job and is
   where MOST differences must land. The null hypothesis for any observed
   difference is "realization, not morphology"; promoting a difference to
   layer 2 requires oracle evidence that projection cannot explain it.

**What this is NOT:** a claim that the recensions agree everywhere. Duals,
aorists, vocabulary, and accent systems have real differences. The model
does not erase them — it forces each one to be either a projection rule, a
named morphological divergence, or a per-recension lexical fact, and each
claim is checked by the oracles.

## The prize: cross-recension evidence

Once identity is shared, attestation transfers with a confidence
discount, never silently:

- An OCS Wiktionary cell projects through the correspondence rules to a
  *predicted* Synodal cell (and vice versa). Predictions from
  cross-recension projection are a distinct provenance class — they rank
  below same-recension attestation in `analyze_text` (the
  attested-before-predicted ordering generalizes to
  same-recension > cross-recension-projected > rule-predicted) and they
  never satisfy a gold gate by themselves.
- The cross-recension candidate queues that today "require human
  confirmation" become mostly mechanical: confirmation is needed only
  where projection is ambiguous or a named divergence is in play.

## Gates — nothing weakens

All existing oracles survive and gate the ONE merged engine:

- OCS Wiktionary cell oracle (134,761 cells, 100%) — via the OCS
  recension parameter.
- Synodal Bible token oracle + witness adjudication + Alypy paradigm
  oracle (from the gold-oracle prompt) — via the Synodal recension
  parameter.
- Paradigm self-consistency and dictionary round-trip gates.
- **New gate: projection coherence.** For every lexeme attested in both
  recensions, project each recension's attested cells into the other and
  count matches. This is a measurement first (phase 1) and a
  no-regression gate after — it is the empirical test of the thesis, and
  its residue list IS the named-divergence registry.

The merge is behavior-preserving by construction: at every phase, both
recensions' oracle replays must be bit-identical to the pre-merge state
unless a change is a gap-shrinking fix in its own right.

## Execution plan

Phased like the rewrite: every slice lands green; the two old kernels are
strangled, not big-banged.

1. **Measure the overlap (no code moves).** Build the projection study as
   an xtask command: apply candidate correspondence rules
   (orthography-level first: jer reflexes, nasal reflexes, letter folds —
   several already exist in the orthography crate's lookup projections)
   to map OCS oracle lemmas/cells onto Synodal ones and vice versa.
   Report: lexemes identifiable across recensions, cells that project
   exactly, cells that need a morphological divergence, cells that cannot
   be related. Commit the report. **Decision point: the plan below
   assumes the exact-projection rate on shared lexemes is high (the
   thesis); if the study says otherwise, stop and re-litigate the thesis
   with the numbers rather than forcing the merge.**
2. **Shared identity layer.** One lexeme registry keyed by abstract
   identity with per-recension citation surfaces; the two families'
   lexicons become views of it. Existing IDs remain resolvable (the
   dictionary crates' public keys must not break).
3. **Merge the realization layer fully.** All projection/correspondence
   rules move into `church-slavonic-orthography` as the recension
   projection module; the two cores' remaining orthography adapters
   shrink accordingly.
4. **Merge the inflection kernels POS by POS**, smallest first (the
   rewrite's pronoun-first precedent), into a single kernel crate —
   grow `church-slavonic-core` from vocabulary crate into the rule
   kernel, absorbing `old-church-slavonic-core` and
   `synodal-church-slavonic-core` module by module. Each POS merge:
   unify the class inventories, encode the named divergences from the
   phase-1 registry as explicit recension conditions, keep both oracle
   replays bit-identical. The per-family core crates end as thin
   re-export shims, then are deprecation-released and deleted (the
   rewrite's phase-5 precedent).
5. **One facade, one dictionary.** `church-slavonic` gains the recension
   parameter per the ruthenian rule: recension selects realization, so it
   rides on functions as a profile/parameter, not as parallel function
   sets — decide `noun(lemma, case, number, recension)` vs a
   `Recension`-scoped handle once, in the phase-2 design doc, and apply
   uniformly. `synodal-church-slavonic`'s Inflector/analyze layer merges
   into `church-slavonic-dictionary`'s analyze layer with recension-aware
   readings. The synodal crate names get the same deprecation-release-
   then-delete treatment.
6. **Unify the extractors and data tree.** One extractor, one
   `data/` layout with per-source, recension-tagged inputs; the
   cross-recension evidence tables collapse into the identity layer.

## Hard rules

- Named divergences live in exactly one registry with per-entry oracle
  evidence; an unexplained recension difference is a gap row, never a
  silent fork.
- Cross-recension projection is a provenance class, ranked below
  same-recension attestation, and never satisfies a gold gate alone.
- Every slice: `cargo test --workspace` green, `check-structure` exit 0,
  both oracle replays unchanged (or strictly improved, with the gap
  diff in the commit message).
- The rewrite's standing rules hold: one override precedence, no module
  over ~1,500 lines, no generated `.rs` over ~5k lines, typed defects and
  provenance preserved, deletion only after a published deprecation
  release.

## Out of scope

- New gold sources (the treebanks PROIEL/Syntacticus/CCMH/DIACU become
  *usable* by the unified model — queue them as candidate oracles for a
  later pass once the merge lands).
- Reconstruction beyond the two attested recensions (no Proto-Slavic
  layer; "abstract identity" means a keying convention, not a
  reconstructed form — pick a convention in phase 2 and document it).

## Report back (per phase)

Phase 1 especially: the overlap numbers (shared lexemes, exact-projection
rate, divergence count by POS and by suspected layer), and the
recommendation at the decision point. Later phases: the named-divergence
registry size as it evolves, oracle deltas (should be zero), and the
crate-consolidation state.
