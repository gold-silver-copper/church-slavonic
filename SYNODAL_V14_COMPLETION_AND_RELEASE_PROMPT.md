# Synodal v0.14 — Finish the deferred work, fix everything fixable, release

## Why this program

v0.12 proved generalisation (held-out generalised 9,467 → 14,236, memorised
capped) and v0.13 made a wave cost ~6 minutes. What remains is a known,
finite inventory: every item below is already named in a `family_reviews.tsv`
deferral, the v0.12 audit handoff, the accent-fit conflict report, or the
`homonymy_allowlist.tsv`. The objective of v0.14 is to drive that inventory
to zero-or-explicitly-out-of-scope, and then ship: **nothing releases until
all work is done, and this prompt defines "all work".**

Two measures anchor the program. The held-out generalised ratchet continues
from 14,236 (every phase that touches coverage must move it or hold it —
never lower it). The deferral inventory is the second: at completion, every
`deferred` row in `family_reviews.tsv` and every allowlist entry must be
either resolved or re-justified as permanently out of scope by a stated
criterion, not by fatigue.

## Ground rules

- The v0.13 workflow is mandatory: `synodal-admit-check` before and after
  data rows, `--delta` projection before any canonical seal,
  `synodal-wave-close --fix` before every commit, CI watched to green per
  phase. No guard, floor, or ceiling may be weakened; ceilings may move only
  with the established justification discipline.
- Holdout discipline is unchanged: never cite a held-out type in runtime
  evidence, exact rows, or accent rows; the memorisation baseline may only
  ratchet down.
- **Presentation folds re-key the holdout.** Changing normalization changes
  which normalized types the content-hash selector chooses. Each fold
  therefore lands in its own wave that ONLY re-keys (`synodal-type-holdout`
  regeneration, baseline/allowlist re-derivation, floors re-justification
  where the denominators move) with no admissions in the same wave, followed
  by separate admission/fold-exploitation waves. A fold whose rekey would
  *lower* the sealed generalised floor is landed with the floor held at the
  new honest value and the delta explained in the floors note and audit.
- Every engine rule gets: unit tests in core, registration in
  `data/synodal/engine_capabilities.tsv`, a rule id, documentation in
  `docs/SYNODAL_MORPHOLOGY.md`, and at least one productive evaluation row
  proving it on a held-out cell where one exists.
- Update `docs/SYNODAL_V14_COMPLETION_AUDIT.md` after every phase (create it
  first with the opening inventory: current deferral rows, allowlist entries,
  accent-fit conflict count, marginal-recovery route table).
- Commit per wave or phase; a phase is not done until CI is green.

## Phase 1 — Presentation folds (ѻ, є, ѹ)

Three folds, strictly one at a time, each as **rekey wave → exploitation
waves**:

1. **ѹ monograph** (ѹ҆мре́ти, ѹ҆гото́ва, ѹ҆боѧ́тсѧ, ѹ-spelled twins of
   existing оу/ᲂу lexemes): fold ѹ→оу (or the established internal target)
   in `normalize_lookup`/`normalize_lookup_accentless`, with digraph-aware
   accent and printing parity (the ᲂу machinery from v0.12 is the model).
   After the rekey, merge the ѹ-spelled duplicate identities the fold
   reveals (оумрети/ѹмрети, имати-family ѹ-pins) — `admit-check` will name
   them.
2. **ѻ broad-on** (ѻ҆дрѣ̀, ѻ҆слы̀, ѻ-initial nouns already admitted with
   о-lemmas): fold ѻ→о on lookup; teach printing the `broad-on` initial
   presentation where the corpus demands it (the `InitialPresentation`
   enum already has `BroadOn`).
3. **є wide-e** (фарїсє́й, воє́въ, словесє́мъ, і҆ере́є): fold є→е on lookup;
   print є where the positional `wide-e` machinery licenses it (ending
   position after vowel, plural-marking uses). This is the fold the family
   queue keeps resurfacing; словесє́мъ closes an es-stem dative and voєвъ a
   soft genitive plural.

Per fold: measure the projected yield with `--delta` immediately after the
fold lands (before admissions) and record it; then admit or scope-fix what
the fold unblocked. Acceptance for the phase: all three folds live under
every profile (lookup, accent, print), holdout re-keyed per fold in its own
sealed wave, the fold-specific deferral rows resolved, and held-out
generalised strictly above 14,236.

## Phase 2 — Kamora plural/dual marking

Model the ◌̑ (kamora) as the systematic plural/dual disambiguation mark it
is (кни̑ги, жи̑лы, вели̑кїѧ, лꙋка̑ваѧ, мѡа̑вли, стражбы̑, пꙋ̑сты,
і҆а̑кѡвли, лꙋка̑вства, достоѧ̑нїѧ):

- an accent-engine mechanism by which a cell's print carries kamora instead
  of the paradigm's acute/grave exactly when the cell is plural/dual and
  homographic with a singular cell of the same lexeme (Alypy's stated
  distributional rule), with the existing `kamora` mark and scope grammar as
  the substrate — prefer one derivational rule over per-lexeme rows;
- re-run `synodal-accent-fit`: the 258-conflict count and the "Unfitted
  scope families" section must collapse substantially, and every remaining
  conflict must be re-attributable to something other than kamora homography
  (record the before/after counts);
- close the deferred kamora items: кни̑ги (family-queue deferral), the
  стражбы̑ plural, the лакте́й/ла́ктей genitive if the mechanism makes the
  variant prints expressible — otherwise write its final disposition.

Acceptance: the mechanism is normative-cited, tested, documented; accent-fit
conflicts materially reduced with numbers in the audit; the kamora deferrals
resolved.

## Phase 3 — Remaining nominal classes and titlo heads

1. **Hard mobile-е masculine class** (па́ѵелъ, ѻ҆се́лъ, and the telets-style
   validator arm for it): admit па́ѵелъ (deferred since wave 10; its dative
   is held-out and must be reached productively) and ѻселъ once the ѻ fold
   from phase 1 is live.
2. **Titlo abbreviation heads** (дв҃дъ, сп҃си́, блгⷭ҇ви́тъ, ѻ҆ц҃е́мъ,
   і҆и҃съ, нбⷭ҇ный, ᲂу҆чн҃къ, хрⷭ҇то́съ): the abbreviation registry and the
   `exact-typed-abbreviation-cells` family arm exist. Extend the abbreviation
   layer so a titlo head maps to its expanded lexeme's *paradigm* (title
   contraction over a productively declined base), so held-out titlo types
   are reached by rule rather than by memorising exact rows — the memorised
   ceiling and baseline must not move. The marginal-recovery report
   attributes ~10,700 diagnostic tokens to the abbreviation route; record
   how much of it this actually recovers.
3. Sweep the remaining deferred nominal rows in `family_reviews.tsv`
   (собранїе-adjacent -їе nouns, ethnonyms, any second-tier items the folds
   unblocked) in ordinary waves until no nominal deferral remains that an
   existing class can admit.

## Phase 4 — Irregular and athematic verbs

1. **Athematic ꙗсти compounds** (поѧсти — поѧдѐ/поѧдѧ́тъ/поѧдо́ша, 114
   tokens deferred) and **дати compounds** (предати, воздати): the archaic
   athematic paradigms exist for быти/дати/ꙗсти; generalise them to
   prefixed compounds instead of new hand tables.
2. **возмощи** (velar imperative возмогꙋ́тъ family, 70+ tokens): model the
   velar alternation in the imperative/present the way the existing velar
   classes do elsewhere.
3. **пойти / прейти** (по́йдꙋтъ 100, пре́йдꙋтъ 41): suppletive й-stem
   compounds of ити — the изити/взыти/внити precedents show the shape.
4. **стоѧти** and any remaining verb deferrals in the family queue.

Acceptance: each verb's held-out cells reached productively with evaluation
rows; the corresponding deferral rows resolved.

## Phase 5 — The identity-merge wave

Collapse every pair in `homonymy_allowlist.tsv` whose justification says
"frozen historical duplicate": послꙋ́ша (v06 pin), рабы, чꙋдеса, чесѡ, ѧ,
сѧ, свои, твои, бѣсте, имава, сотвори, былъ, вїно, аарѡнъ, краѧ, каѧ,
инѣмъ, писахъ, взыдоша, Блаженъ. For pins held by immutable v0.6/v0.7
records, the merge preserves the historical record (ownership ledgers stay
intact) while the *runtime* stops shipping two identities — follow the
established merge procedure, and where an immutable record genuinely blocks
a merge, write the permanent disposition into the allowlist justification
instead. Acceptance: every allowlist row is either gone (merged) or reads as
genuine homonymy / permanently-pinned-with-reason; the
`integrity:cross_lexeme_ambiguous` ceiling ratchets *down* to the new honest
value.

## Phase 6 — Queue burndown and inventory zero

- Extend family-review decisions beyond the top-200 until every proposal
  that phases 1–5 made admissible is admitted and everything else carries a
  reasoned deferral naming the concrete blocker ("needs X, which is out of
  scope because Y") — no bare deferrals.
- Re-derive the memorisation baseline (`--write-baseline`) and confirm it
  only shrank.
- Sweep `docs/` for stale statements (counts, workflows, deferral lists) and
  fix them.
- Final canonical seal; the audit records the closing inventory next to the
  opening one.

## Phase 7 — Release

Only after phases 1–6:

1. Version the three crates together (choose v1.0.0 if the deferral
   inventory is genuinely closed; otherwise v0.14.0 with the residual list
   in the release notes) and write `CHANGELOG.md` from the v0.12–v0.14
   audits.
2. README: a real consumer section — `analyze`, `analyze_text`, the CLI,
   the coverage numbers with their honest definitions (held-out generalised
   vs corpus top-k), and what the library does not do.
3. `cargo publish` the three crates in dependency order (core → morphology →
   dictionary) after dry-runs; tag the release; verify the published crates
   build in a clean scratch project with a smoke test that analyzes a
   held-out passage.
4. Record the release (versions, checksums, date) in the audit.

## Completion gate (all must hold)

1. All three presentation folds live under lookup, accent, and print, each
   re-keyed in its own sealed wave; held-out generalised strictly above
   14,236 with the ratchet intact throughout.
2. Kamora marking is a tested, documented engine rule; accent-fit conflicts
   reduced with before/after counts recorded; no kamora deferral remains.
3. The mobile-е class exists and па́ѵелъ resolves productively; titlo heads
   reach their paradigms by rule with the memorised ceiling and baseline
   unmoved.
4. The athematic compounds, возмощи, пойти/прейти, and стоѧти resolve their
   held-out cells productively.
5. `homonymy_allowlist.tsv` contains only genuine homonymy or
   permanently-pinned entries with stated reasons; the cross-lexeme ceiling
   ratcheted down accordingly.
6. Zero bare deferrals: every `deferred` row in `family_reviews.tsv` names
   its concrete blocker and why it is out of scope; the memorisation
   baseline only shrank; docs carry no stale claims.
7. The three crates are published, tagged, and smoke-tested from the
   registry; the release is recorded in
   `docs/SYNODAL_V14_COMPLETION_AUDIT.md`.
8. All 31+ sealed bounds hold, `synodal-wave-close --fix` is fully green,
   and CI is green on main at the release tag.
