# Synodal v0.14 completion-and-release audit

Program prompt: `SYNODAL_V14_COMPLETION_AND_RELEASE_PROMPT.md` (archives to
`docs/goals/` at completion). Objective: drive the deferred-work inventory to
zero-or-explicitly-out-of-scope, then release. Two anchors: held-out
generalised must end strictly above 14,236 with the ratchet intact, and every
deferral must resolve or name its concrete blocker.

## Opening inventory (sealed wave `v0.13-demo-vertep`, 2026-08-25)

| Measure | Value |
|---|---:|
| Held-out generalised / memorised / top-k | 14,236 / 14,998 / 30,293 |
| Corpus top-k analyzed | 991,194 |
| `family_reviews.tsv` `deferred` rows | 627 |
| `homonymy_allowlist.tsv` pairs (frozen-duplicate) | 46 (20) |
| Accent-fit conflicts / unscopable lexemes | 258 / 12 |
| Memorisation baseline held-out types | 93 |
| Marginal-recovery diagnostic routes | abbreviation-registry 10,677 · spelling-variant 14,935 · exact-evidence 437 · reviewed-class 70 · reviewed-principal-part 54 |

The `spelling-variant` route is what phases 1 (ѻ/є/ѹ folds) address; the
`abbreviation-registry` route is phase 3's titlo heads.

## Phase 1 — Presentation folds

(filled per fold)

### Fold 1 — ѹ (uk monograph), rekey wave `v0.14-rekey-uk`

`normalize_lookup` and `normalize_lookup_accentless` now expand ѹ (U+0479)
to оу, unifying the third spelling of every оу/ᲂу word; marks carried by the
monograph land on the у exactly where the digraph carries them (unit-tested).
The holdout re-keyed under the new normalization: 57,341 → 54,868 corpus
types, 2,785 held types; the memorisation baseline re-derived at 93 types
(unchanged membership). The fold immediately revealed the long-deferred
оумрети duplicate — the v0.7 ѹ-pinned identity `synodal:verb:v07-789965b2445975f1`
(review lemma оумьрѣти) — first as an `admit-check` violation and then as a
+571 cross-lexeme ceiling hit; the v12 productive admission was merged onto
it with a Synodal `lexical-form` row (ѹ҆мре́ти, Psalm 48:18, wikisource
candidate). One floor moved down with justification:
`summary:top_1_analyzed` 619,580 → 617,987 (unification adds twin readings
to previously single-reading tokens; nothing was removed). Eight family-queue
proposals re-keyed to new candidate ids and were re-deferred with
phase-specific notes. Yield of the fold alone, before any admissions:
held-out generalised 14,236 → **14,346** (+110), corpus top-k 991,194 →
**993,530** (+2,336), memorised flat at 14,998. (`wave-close --fix` also
gained coverage-fixture regeneration, a gap this wave exposed.)
