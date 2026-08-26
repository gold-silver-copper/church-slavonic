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

### ѹ exploitation wave `v0.14-uk-exploitation`

Seven admissions that the fold unified across three spellings each:
`оустна` (second-hard, dual-dominant — a manual end-stress row closes the
held-out dual nominative ѹ҆стнѣ̀), `оупованїе` and `оутвержденїе`
(first-soft-ie-n), `оутвердити` (second perfective; manual aorist/imperative
singular end-stress rows beside the fitted stem-stress block), `оуразꙋмѣти`
(first-palatalized, разꙋмѣти's prefixed twin), and the a/и pair
`оуготовати`/`оуготовити`. The preflight caught one evaluation-passage
collision before any build, and two wikisource candidates initially cited as
ponomar were corrected (with a script sweep verifying every remaining v14
candidate against its corpus). Ledger: held-out generalised 14,346 →
**14,515** (+169), corpus top-k 993,530 → **994,992** (+1,462), memorised
flat. The ѹ fold plus its exploitation is worth +279 generalised and +3,798
top-k so far.

### Fold 2 — ѻ (broad on), rekey wave `v0.14-rekey-broad-on`

`fold_digraph_uk` now also folds ѻ/Ѻ to о/О, unifying the word-initial broad
on with its plain twin in both lookup projections (unit-tested). The rekey
moved the holdout composition sharply: held-out generalised 14,515 →
**15,238** (+723) and memorised 14,998 → **14,974** (−24) purely from
re-keyed type membership — ѻ-spelled twins of productively reviewed о-words
joined the held slice already resolved by rule. The fold revealed one frozen
pin (the ѻчесъ v0.7 identity with mislabeled cells, recorded in the
allowlist for the phase-5 merge with cell-correction retractions) and one
genuine homonymy (ѻ҆ныхъ: онъ's historical pin vs оный's genitive plural).
Three re-keyed queue proposals decided: оный's ѻ prints marked moot against
the existing reviewed demonstrative; обои и ѻ҆тцы̑ (kamora) deferred to
their phases. The memorisation baseline re-derived at 94 types. No
admissions; corpus top-k unchanged at 994,992 pending exploitation.
