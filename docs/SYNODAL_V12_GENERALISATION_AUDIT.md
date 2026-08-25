# Synodal v0.12 generalisation audit

This records the v0.12 program (`SYNODAL_V12_GENERALISATION_AND_PREDICTIVE_LEXICON_PROMPT.md`)
as it is realised: what each phase found, changed, and deliberately left. It
is updated per sealed wave and is not a completion claim until the final
section says so.

## Baseline (merge base `781e2e6`, reproduced 2026-08-24)

Reproduced with `cargo xtask synodal-coverage --offline --check` under
`Strict` / `SynodalLiturgical` on the locked corpus.

| Measure | Value |
|---|---:|
| Passages / tokens / types | 74,130 / 1,313,344 / 57,476 |
| top-k analyzed | 964,791 (73.46%) |
| top-1 analyzed | 614,583 (46.79%) |
| `morphology-free` covered tokens | 50,151 |
| `unknown-lexeme` gap | 336,662 |
| held-out types / tokens | 2,929 / 44,425 |
| held-out **generalised** | 9,467 (21.31%) |
| held-out memorised | 15,048 |
| held-out unresolved | 18,840 |
| registry lexemes / verb lexemes | 999 / 169 |
| `principal_parts.tsv` rows | 149 |
| `evaluation.tsv` productive rows | 1 |

One correction to the prompt's baseline table: it quoted "26 verb lexemes"
from `data/synodal/lexemes.tsv` alone. The registry also admits verbs through
the reviewed source-decision tables and the irregular inventory, and the wave
ledger counts the registry, which is what the engine actually runs on: 169
verb lexemes of 999. The 149 principal-part rows are the same either way.

## Phase 1 — generalisation is the headline

- `reports/synodal-coverage.md` now opens with the type-disjoint holdout: an
  outcome table (generalised / memorised / ambiguous / unresolved / top-k /
  top-1), the per-status table, and a new per-system table
  (`held_out_type_status_by_system` in the JSON, additive to schema 4).
  Corpus-wide figures follow under their own heading.
- `reports/synodal-waves.tsv` is the per-wave ledger. `--seal-wave` appends;
  `--check` and `synodal-coverage-floors` (the CI path) verify the last row
  against the live report and lexicon; consecutive rows must ratchet
  (`holdout_generalised` never falls, `holdout_memorised` never rises). Two
  guard witnesses inject a falsified last row and a regressing row and prove
  both are rejected.
- `CoverageReport::held_out_generalised` / `held_out_memorised` are the single
  definition of the two measures; the floors read them too.
- Sealed wave `v0.12-baseline`: generalised 9,467, memorised 15,048.

What the per-system split shows at baseline, and what phase 2 therefore has
to move:

| System | Held-out | Generalised | Memorised |
|---|---:|---:|---:|
| `aorist` | 1,304 | 95 | 1,209 |
| `imperfect` | 272 | 0 | 270 |
| `determiner` | 147 | 0 | 147 |
| `indeclinable` | 1,561 | 0 | 1,534 |
| `lexical-form` | 7,679 | 0 | 6,881 |
| `noun` | 6,480 | 2,058 | 4,061 |
| `present` | 749 | 459 | 270 |
| `future` | 683 | 458 | 161 |
| `infinitive` | 1,027 | 1,027 | 0 |
| `pronoun` | 4,210 | 4,188 | 0 |

The finite past systems are almost entirely memorised: the engine has aorist
and imperfect *rows*, not aorist and imperfect *rules* that reach unseen
types. That is the verb frontier stated precisely.

## Phase 2, wave 1 — the reflexive rule and three verbs

Sealed as `v0.12-wave-1`. Reproduced with `cargo xtask synodal-coverage
--offline` before and after.

| Measure | Baseline | Wave 1 | Δ |
|---|---:|---:|---:|
| held-out **generalised** | 9,467 | 9,868 | +401 |
| held-out memorised | 15,048 | 14,998 | −50 |
| held-out unresolved | 18,840 | 18,315 | −525 |
| held-out tokens | 44,425 | 44,251 | see §digraph |
| corpus top-k | 964,791 | 968,073 | +3,282 |
| corpus unresolved | 347,524 | 344,207 | −3,317 |
| `morphology-free` covered | 50,151 | 50,138 | −13 |
| lexemes / verb lexemes | 999 / 169 | 1,002 / 172 | +3 / +3 |
| `principal_parts.tsv` rows | 149 | 170 | +21 |
| productive evaluation rows | 1 | 29 | +28 |
| `linguistic_evaluation.tsv` | 12 | 19 | +7 |

Held-out generalisation by system: aorist 95 → 354, imperative 0 → 57,
future 458 → 502, present-active participle 117 → 150, present 459 → 465.

### What was built

- **Alypy §73 reflexive voice** (`SYN-VERB-REFLEXIVE-ALYPY-73`). Two
  mechanisms share one core rule (`reflexive_surface`: host + `сѧ`, final jer
  deleted). A verb registered under a lemma in `-сѧ` stores bare stems and
  the resolver attaches the enclitic to every generated cell *before* accent
  realisation, so exact accent rows, fitted paradigms, and corpus prints all
  describe the same surface. A surface in `-сѧ` with no registered reading is
  analysed as the reflexive/passive of a registered active verb when the host
  (jer restored where needed) has a verbal reading; the analysis carries
  `reflexive: true`, the rule in its trace, and `SynodalProductiveRule` as its
  source — no row cites the surface, so it is generalisation by construction.
  The registry held **zero** reflexive verbs before this wave; reflexive forms
  were 1,862 of the 18,840 held-out unresolved tokens and 29,488 corpus-wide.
- **Alypy §93 j-series imperative** (`бо́йсѧ`, `сто́йте`): the vowel-stem case
  where `-и-` passes into `-й-`, stated by Alypy and previously unmodelled.
- **The `ᲂу` digraph fold.** The reverse-lookup projections never folded the
  printed digraph lead (U+1C82) back to the expanded `о`, so every lowercase
  `ᲂу`-initial corpus token (10,348 unresolved tokens; 642 held-out) was
  invisible to the accentless index and therefore to the accent fitter; only
  sentence-initial `Оу…` prints reached it. The fold is now in
  `normalize_lookup` and `normalize_lookup_accentless`; the accent engine
  recognises the expanded `оу` as the digraph (psili and first stem vowel on
  its `у`, `ᲂу҆́мре` not `о҆́умре`); and generated liturgical prints apply
  `present_initial_uk_digraph` so they print as reviewed exact rows do.
  **Consequence for the holdout:** 121 sealed held-out types were keyed under
  the unfolded presentation and could match nothing after the fold. The file
  was regenerated by the *unchanged* selector on the corrected keys
  (`cargo xtask synodal-type-holdout`): 2,929 → 2,924 types, 44,425 → 44,251
  tokens. No parameter was touched; the identity of 126 `оу`-initial types
  changed because two presentations of the same type were unified. Generalised
  and memorised are compared across that change above and both moved in the
  intended direction.
- **Sibilant participle series** (`SYN-ADJ-LONG-SIBILANT-ALYPY-57-58`). Long
  participles on a hard stem in `ш/щ/ж/ч` were declined with `-ыхъ`, which the
  corpus never prints (2,604 tokens in `-ихъ`, 0 in `-ыхъ`; 319 in `-ими`, 0
  in `-ыми`), while the plural dative `-ымъ` (1,541) and plural feminine `-ыѧ`
  (856) keep `ы`. The class is derived from the stem at declension time.
- **Typed-first attribution.** A token is now attributed to its first *typed*
  reading. Before, a reviewed lexical-form row that outranked a noun reading
  in source precedence made the token count as `lexical-form`; 44,458 such
  tokens moved to their typed systems (noun +31,340). `morphology-free`
  (every reading lexical-form) is the measure that guards the cheap route and
  did not move. `system:lexical-form`, `system:indeclinable`, and
  `system:present-passive-participle` (−2, same cause) were re-sealed with
  that justification.
- **Fitter routes derived reflexives to their host** so a reflexive print
  attests the host's cell (`возврати́сѧ` → `возврати́`).

### Admissions

`боѧтисѧ` (imperfective, second, reflexive-only), `оубоѧтисѧ` (perfective),
`оумрети` (perfective, first-unpalatalized; consonant aorist, mobile-vowel
l-participle base `оумер-`). Every principal part cites a source-partition
print by candidate id; the held-out types `бо́йтесѧ`, `ᲂу҆боѧ́тсѧ`, and
`ᲂу҆́мретъ` are deliberately not cited and are reached by rule. Accent
contracts were fitted; within-scope mobility that the scope grammar cannot
state (`ᲂу҆мрꙋ̀` vs `ᲂу҆́мреши`, `ᲂу҆мро́хъ` vs `ᲂу҆́мре`, `ᲂу҆́мерлъ` vs
`ᲂу҆мерла̀`) is carried by explicit `accents.tsv` rows and one hand-authored
future-singular paradigm row mirroring the fitted present-singular rule.

### Floors

- `integrity:cross_lexeme_ambiguous` 9,398 → 9,502 (+104), attributed
  surface by surface: `имꙋщихъ/имꙋщемꙋ/имꙋщее/имꙋщїѧ/имꙋщей/имꙋщемъ/имꙋщими`
  (86) are long participles the sibilant series now generates for *both*
  `имати` and `имѣти`, a shared identity that predates this wave; `имꙋтсѧ`,
  `сотворилсѧ`, `сотворихомсѧ` (18) are reflexive derivations whose hosts
  already span two lexemes. No identity was added; the `имати`/`имѣти` pair is
  recorded for review.
- The remaining `missing-accent` gap grew 10,747 → 11,638: reflexive forms of
  exact-only hosts (`възвратити`, `ѡбратити`) now resolve accentless but their
  hosts have no productive class for the fitter to work on. Those hosts are the
  next wave's admissions, not a rule gap.

### Verification at seal

`cargo fmt`, `clippy -D warnings`, `cargo test --workspace --all-targets`,
doc tests, `synodal-check`, `synodal-guard-witnesses` (11 witnesses),
`synodal-coverage-floors` (31 bounds, ledger current), `synodal-accent-fit
--check`, `synodal-engine-audit --check`, all three review-queue checks,
`synodal-marginal-recovery --check`, `synodal-fixture-bootstrap`, and the
fixture coverage check all pass. `--reseal-floors` ratcheted every bound to
the sealed values; `holdout:generalised_analyzed` is now sealed at 9,868.

## Handoff (state at the end of wave 1)

Phase 1 is complete. Phase 2 has one sealed wave. Phases 3–5 have not been
started. What the next session should pick up, in order:

1. **Exact-only reflexive hosts.** `възвратити` (`synodal:verb:v07-ff04037d9da0c605`),
   `ѡбратити` (`wikt-3ca5f600ecca`), and `събрати` (`v07-553feeb14b8be67e`)
   have exact rows but no productive class, so their reflexive forms
   (`возврати́сѧ` 348, `Ѡ҆брати́сѧ` 140, `собра́шасѧ` 238 corpus tokens) resolve
   accentless only. Admitting them productively — class, principal parts,
   fitted accents, productive evaluation rows — is the highest-value next
   wave, and the fitter now derives host accents from reflexive prints.
2. **Deferred `оу`-initial families** surfaced by the digraph fold:
   `оуслышати` (`synodal:family-candidate:5c1b0900fb0f4c7bd64f0e12`),
   `оузрѣти` (`950fff394b9d42d3af913a9b`), `оубити`
   (`50bc23c0c780ef4e9ad05818`), the plural-only noun `оуста`
   (`9845b516c81680201ffeb607`); plus `вавѷлонскїй`
   (`275a15b9d4368ec457e74052`) and `ме́рзость` (`ed7467dd3d77d260a7ea07c5`).
3. **The `имати`/`имѣти` shared identity**: both generate the `имꙋщ-` long
   participle; 86 tokens are cross-lexeme ambiguous because of it. Decide
   whether one lexeme owns that stem.
4. **The `missing-accent` gap (11,638 tokens)** is now visible to the fitter
   for `оу`-initial and reflexive tokens; a plain `cargo xtask
   synodal-accent-fit --apply` after each admission wave is cheap and should be
   routine. Note the fitter now inserts rows inside an existing block and
   reuses the block's evidence; an earlier append had broken `нести` entirely.
5. **Phase 3** (predictive tier on `FormSource::AnalogicalPrediction` under
   `Exploratory`), **phase 4** (consumer `analyze-text` API + README), and
   **phase 5** (archive v04–v07 audits, move prompts to `docs/goals/`) are
   untouched.

Working notes for whoever continues: use `cargo build --release -p xtask` and
`./target/release/xtask …` (the canonical coverage run is ~4 min in release,
much longer in debug); run `synodal-regenerate` after *every* data change and
rebuild before probing with `synodal-dict analyze … --profile
synodal-liturgical`; seal a wave with `synodal-coverage --offline
--reseal-floors --seal-wave <label> --note "…"`; and never cite a held-out
type in `reviewed_evidence.tsv` (`data/synodal/held_out_types.tsv`).

## Phase 2, wave 2 — the exact-only reflexive hosts become productive

Sealed as `v0.12-wave-2`.

| Measure | Wave 1 | Wave 2 | Δ |
|---|---:|---:|---:|
| held-out **generalised** | 9,868 | 9,968 | +100 |
| held-out memorised | 14,998 | 14,998 | 0 |
| held-out unresolved | 18,315 | 18,215 | −100 |
| corpus top-k | 968,073 | 970,579 | +2,506 |
| `morphology-free` covered | 50,138 | 50,122 | −16 |
| `principal_parts.tsv` rows | 170 | 193 | +23 |
| productive evaluation rows | 29 | 46 | +17 |
| `linguistic_evaluation.tsv` | 19 | 23 | +4 |

Promoted `возвратити` (`synodal:verb:v07-ff04037d9da0c605`), `ѡбратити`
(`wikt-3ca5f600ecca`), and `собрати` (`v07-553feeb14b8be67e`) from exact-only
identities to productive verbs — second conjugation with the §80 dental
alternation attested directly (`возвращꙋ̀` Amos 9:14, `ѡ҆бращꙋ̀` Ezek 16:53),
and first conjugation with the suppletive present stem `собер-` against the
aorist base `собра-`, including `собрати`'s §100 passive participles
(`со́брани`, `со́бранныхъ`). Their Synodal lemma spellings replace the OCS
headwords, licensed by attested infinitive `lexical-form` rows (the upgrade
validator requires exactly that). The reflexive frontier heads this unlocked:
`возврати́сѧ` (348), `Ѡ҆брати́сѧ` (140), `собра́шасѧ` (238) now resolve in the
liturgical profile by the §73 derivation.

Mechanics this wave added or corrected:

- `reflexive_base_candidates` also offers the isolated grave for a host whose
  pre-enclitic print carries a final acute (`возврати́сѧ` ↔ `возвратѝ`).
- Future-singular accent scopes mirror the fitted present-singular rules
  (hand-authored rows, same witness), since the fitter's attestations label
  those cells present.
- A mislabelled v0.7 exact cell — the aorist print `Возвратѝ` filed as
  `future:third:singular` — is retracted through
  `v10_exact_cell_corrections.tsv` with its held-out expectation; the
  productive engine now generates the genuine `возврати́тъ`.
- `system:lexical-form` became an **at-most** ceiling (49,568): under
  typed-first attribution it is the cheap residue and fell (−16) as reviewed
  headwords gained real morphology, which an at-least floor cannot express.
- Held-out aorist by resolver status is now 372 by rule / 1,536 memorised
  (was 95 / 1,209 at baseline).

Deferred with reasons (`family_reviews.tsv`): `послꙋшати` (j-series verb
wave), `предати` (needs an archaic table like `дати`), `зане́же`
(conjunction sweep), `саꙋ́лъ` (proper-noun wave).

## Phase 2, wave 3 — four more verbs; the ѹ monograph duplicate found

Sealed as `v0.12-wave-3`.

| Measure | Wave 2 | Wave 3 | Δ |
|---|---:|---:|---:|
| held-out **generalised** | 9,968 | 10,048 | +80 |
| held-out memorised | 14,998 | 14,998 | 0 |
| corpus top-k | 970,579 | 971,946 | +1,367 |
| lexemes / verb lexemes | 1,002 / 172 | 1,005 / 175 | +3 / +3 |
| `principal_parts.tsv` rows | 193 | 224 | +31 |
| productive evaluation rows | 46 | 65 | +19 |
| `linguistic_evaluation.tsv` | 23 | 27 | +4 |

Admitted `оуслышати` (full paradigm including §100 passives), `оузрѣти`
(promoting the v0.7 exact-only identity `оузьрѣти` under its Synodal lemma via
the attested infinitive; the held-out `ᲂу҆́зрѧтъ` reached by a hand-authored
future-plural scope witnessed by the non-held-out `ᲂу҆́зрите`), `оубити`
(j-series imperative `ᲂу҆бі́й`, palatalized future on the stem `оубї-`, both
passive stems), and `оумертвити` (labial epenthesis `ᲂу҆мерщвлю̀` attested at
Ex 9:15). Cited nothing held-out (`ᲂу҆слы́шавше`, `ᲂу҆́зрѧтъ` excluded from
evidence).

**Duplicate identity found:** the registry holds `оумьрѣти`
(`synodal:verb:v07-789965b2445975f1`) with exact rows spelled with the
*monograph* uk `ѹ` (U+0479: `ѹмре`, `ѹ҆́мреши`) — the same verb as wave 1's
`v12-umreti`. Those rows are dead keys today because the lookup fold covers
only the two-character `ᲂу` presentation, not `ѹ`. Folding `ѹ` without first
merging the identities would create live cross-lexeme collisions, so the fold
and the merge must land together; `оузьрѣти`'s rows have the same property and
its identity is now productively owned. Recorded for the identity-merge wave
alongside `имати`/`имѣти`.

Deferred with reasons: `ѿпꙋстити` (needs a ст→щ first-singular print),
`приближитисѧ` (reflexive-lexeme pattern; held-out `прибли́жишасѧ` must stay
uncited), `колесни́ца` (nominal wave; broad-е antistich belongs with the
phase-3 positional design).

## Phase 2, wave 4 — the §86 -тъ rule, клѧтисѧ, приближитисѧ, царствовати

Sealed as `v0.12-wave-4`.

| Measure | Wave 3 | Wave 4 | Δ |
|---|---:|---:|---:|
| held-out **generalised** | 10,048 | 10,266 | +218 |
| held-out memorised | 14,998 | 14,998 | 0 |
| corpus top-k | 971,946 | 973,202 | +1,256 |
| lexemes / verb lexemes | 1,005 / 175 | 1,008 / 178 | +3 / +3 |
| `principal_parts.tsv` rows | 224 | 244 | +20 |
| productive evaluation rows | 65 | 77 | +12 |

New rule `SYN-VERB-AORIST-VOWEL-T-ALYPY-86` (`vowel-t` formation): Alypy §86
names the closed list `ꙗти, начати, вити, пити, клѧти` whose 2nd/3rd singular
aorist takes `-тъ` beside the bare stem, as ordered variants. That is how the
held-out `клѧ́тсѧ` (144 tokens) is reached by rule from an aorist stem whose
evidence never cites it. Admissions: `клѧтисѧ` (reflexive-only, suppletive
present `клен-` against aorist `клѧ-`; the imperfective productive-only class
now requires *one* attested past system rather than specifically the
imperfect), `приближитисѧ` (held-out `прибли́жишасѧ`, `прибли́житсѧ`,
`прибли́житисѧ` all reached by rule), and `царствовати` (`-ова-`/`-ꙋ-` stem
suppletion; held-out `ца́рствꙋю` licensed by the §82 ending). Also retracted a
second mislabelled v0.7 exact cell — the imperfect print `Глаго́лаше` filed as
`aorist:third:plural` of `глаголати` — unblocking the genuine `глаго́лаша`
(192 tokens).

Running total: held-out generalised 9,467 → 10,266 (+799, +8.4%) and corpus
top-k 964,791 → 973,202 (+8,411) across four waves, with memorisation capped
at 14,998 throughout.
