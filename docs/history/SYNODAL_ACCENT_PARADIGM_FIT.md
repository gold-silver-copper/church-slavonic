# Synodal accent-paradigm fit wave

This audit records one realized coverage wave on the continuing 100% strict
top-k program. It is not a completion claim and not a milestone: the wave
crosses no threshold named in the program (the first is 75%).

All numbers were reproduced with `cargo xtask synodal-coverage --offline` under
`GenerationPolicy::Strict` and `OrthographyProfile::SynodalLiturgical` against
the locked corpus, before and after the change.

## What the wave targeted

The `missing-accent-or-orthographic-metadata` gap is the part of the frontier
where identity and grammar cell are **already resolved**: the accentless
surface analyses correctly, but the liturgical profile cannot realise the
printed marks because the lexeme carries no reviewed accent contract.

`crates/synodal-church-slavonic-dictionary/src/coverage.rs` raises this gap
when `analyze_profile(ExpandedAccentless)` or `analyze_profile(Expanded)`
succeeds while the liturgical profile does not. The failure is otherwise
silent: `index_cell` discards any cell whose resolution errors, so a lexeme
with a class, a stem, and no accent paradigm contributes **zero** strict top-k
coverage and produces no diagnostic of its own.

At the merge base this gap held 16,859 tokens over 1,697 frontier surfaces and
427 distinct already-reviewed lexemes.

## Method

`cargo xtask synodal-accent-fit` derives the missing contract instead of
storing one accented string per cell.

1. Load only `source`-partition verse records of the two direct target corpora,
   **excluding** every passage sealed into the held-out evaluation contract
   (`evaluation.tsv`, `abbreviation_evaluation.tsv`). Both exclusions happen at
   the input boundary, so no later stage can read them. The held-out set is
   keyed by passage alone, not by `(source_id, passage)`: the two editions are
   verse-parallel with independent partition assignments, so 1,104 of the 1,439
   sealed pairs have the same verse marked `source` in the sibling edition.
   Keying on the pair would let that near-identical printing back in.
2. Run the canonical analyzer and keep every token whose gap is exactly this
   one, pairing the printed corpus token with the expanded form of each
   resolved cell. A variant is paired only when its accentless normalization
   equals the token's, so a sibling variant of the same cell can never be
   matched against the wrong surface.
3. Skip any cell that already resolves under the liturgical profile: an
   existing rule governs it, and the registry admits at most one accent
   paradigm per cell.
4. Search the closed placement space — stem/word/ending vowel offsets 0–9 ×
   {acute, grave, kamora} — applying the engine's own
   `AccentParadigm::apply`. A candidate is kept only if it reproduces **every**
   source-partition attestation in scope. The comparison is made on
   `normalize_lookup`, the exact key the reverse analyzer indexes, so
   sentence-initial capitalisation does not defeat a correct rule while every
   prosodic and positional mark stays significant.
5. Refine greedily from coarse to fine — number, then case, then gender, then
   animacy. A partition that one placement explains is settled at that
   granularity, and only the partitions that disagree are split further, so
   genuine accent mobility is localised rather than smoothed away or forced.
6. Reject any fitted rule that fails one of four guards, reporting rather than
   narrowing it:
   - it would overlap a cell an existing reviewed paradigm governs, which the
     registry rejects outright as contradictory metadata;
   - it claims a cell whose form the placement cannot address;
   - it generates a print for an in-scope cell that the source partition
     contradicts. Fitting only sees tokens still in the accent gap, so a print
     of the same accentless key that already resolved by another route — an
     exact row, or a duplicate registry entry — is invisible to it. This guard
     re-checks every emitted rule against a corpus-complete index of printed
     tokens;
   - it puts a kamora on a singular-only scope. Kamora is this print
     tradition's number-disambiguating mark, and the reverse analyzer offers
     every syncretic reading of a token, so a kamora-marked dual or plural
     print is also offered as a singular reading. Admitting that fit would
     invent an unattested singular accent and erase the distinction the print
     exists to state.

Rules that survive all six steps are written to `accent_paradigms.tsv` with one
`reviewed_evidence.tsv` row per lexeme, citing a source-partition passage of a
direct target corpus.

### What the wave deliberately did not do

- No new lexical identity, principal part, spelling variant, or irregular form
  was invented. Every rule attaches to a lexeme whose identity, class, and stem
  were already reviewed.
- No accentless or suffix-only fallback was added, and `Strict` was not
  softened toward `Productive`.
- Scope groups that no reviewed placement explains were left unfitted and
  reported with counterexamples, not forced to fit.

## Realized result

Denominator unchanged: 74,130 passages, 1,313,344 tokens, 57,476 normalized
types, same corpus/source/partition split.

| Measure | Baseline | After | Delta |
|---|---:|---:|---:|
| Top-k analyzed | 956,440 | 963,251 | +6,811 |
| Top-k basis points | 7,282 | 7,334 | +52 |
| Top-1 analyzed | 615,152 | 613,949 | −1,203 |
| Ambiguous | 7,205 | 9,398 | +2,193 |
| Unresolved | 355,875 | 349,064 | −6,811 |

The gain is attributed exactly and entirely to the targeted gap:

| Gap | Baseline | After | Delta |
|---|---:|---:|---:|
| `missing-accent-or-orthographic-metadata` | 16,859 | 10,048 | −6,811 |
| `ambiguity-or-spelling-variant` | 8,234 | 10,427 | +2,193 |
| `unknown-lexeme` | 338,901 | 338,901 | 0 |

No other gap category moved. By resolver status the gain is productive
morphology, not stored strings:

| Status | Baseline | After | Delta |
|---|---:|---:|---:|
| `synodal-productive-rule` | 26,787 | 33,350 | +6,563 |
| `exact-synodal-attestation` | 653,114 | 651,638 | −1,476 |
| `synodal-normative-table` | 186,649 | 186,180 | −469 |

### Generalisation to held-out passages

| Partition | Baseline top-k | After | Delta |
|---|---:|---:|---:|
| `source` | 762,515 | 767,969 | +5,454 |
| `evaluation` | 193,925 | 195,282 | +1,357 |

Fitting read only source-partition passages, and excluded every sealed
evaluation passage. The 1,357 evaluation-partition tokens were therefore
realised by rule rather than by a memorised string. This is the wave's
generalisation evidence.

Both corpora and both sources moved together — Elizabeth/Ponomar +3,438 and
Church Slavonic Bible/Wikisource +3,373 — so the gain is not concentrated in
one edition.

### Precision

Top-1 fell by 1,203 tokens and ambiguity rose by 2,193. This is the mechanical
consequence of correctly realising syncretic paradigms, not a precision trade.
Church Slavonic short adjectives genuinely syncretise: `чи́стъ` now returns
nominative singular masculine (animate and inanimate) *and* accusative singular
masculine inanimate, and `мꙋ́дрїи` returns nominative and vocative plural
masculine. Every newly ambiguous reading observed belongs to the **same**
lexeme; no cross-lexeme homograph was introduced. The program forbids
collapsing justified ambiguity, so these readings are kept.

The sealed held-out evaluation report is numerically **unchanged**: 2,203/2,267
expanded and 2,119/2,267 printed, with zero changed metrics.

Per-lexeme accounting over the whole registry: **0 lexemes lost top-k
coverage**; 115 gained.

## Admitted data

620 `accent_paradigms.tsv` rows over 116 lexemes, each citing one of 116 new
`reviewed_evidence.tsv` rows.

| Part of speech | Rows |
|---|---:|
| adjective | 240 |
| pronoun | 170 |
| noun | 124 |
| verb | 51 |
| determiner | 20 |
| numeral | 15 |

Scope granularity actually used, i.e. how much mobility the evidence forced:

| Granularity | Share of rows |
|---|---|
| number only | widest generalisation, used where one placement explained a whole number |
| number + case | used where case-conditioned mobility was attested |
| number + case + gender | used for agreeing paradigms with gender-conditioned stress |
| number + case + gender + animacy | rare; only where the animacy contrast itself moved the accent |

## Engine change: reusable pronoun accent scopes

`AccentScope::PronounCases` and `AccentScope::PronounAgreement` exist in the
core crate and are parsed by the runtime registry, but the reviewed data layer
rejected every `pronoun:` scope string, so no pronoun accent contract could be
authored even though the engine could compile one. Pronouns were the largest
single block of the accent gap.

`validate_accent_scope_code` now accepts exactly the two shapes the registry
parses — `pronoun:{numbers}:{cases}` and
`pronoun-agreeing:{numbers}:{cases}:{genders}:{animacies}` — and nothing more.
Closing this seam contributed 170 of the admitted rows. Positive and negative
tests cover both shapes.

## Data repairs

- `exact_forms.tsv`: the collective nominative singular `бра́тїѧ` claimed
  `synodal-attestation` while citing only the Alypy §37 table. Ten sibling rows
  citing the same evidence are `normative-table`. The row's own v0.6 lexical
  review already owns a Ponomar `Acts.9.30` witness, so the corpus witness is
  now cited alongside the table and the attestation claim is honest. This was a
  pre-existing `cargo xtask synodal-check` failure at the merge base.
- `lexical_source_decisions.tsv`: seven invariant seed identities — `внѣ`,
  `всегда`, `тѣмже`, `предо`, `вскꙋю`, `доколѣ`, `ѡбъ` — had no reviewed
  classification. All seven are genuinely uninflected and now carry the same
  `indeclinable` / `reviewed-exact-indeclinable` decision the two existing
  invariant identities use. Pre-existing `cargo xtask check-all` failure.
- `family_reviews.tsv`: eight proposals entered the reviewed top 200 as the
  frontier re-ranked. Each received a specific deferral rather than an
  admission, because each genuinely lacks target identity or morphological
  metadata — including two real identity conflicts (`пожр-` conflates the
  "devour" and "sacrifice" verbs; `і҆исꙋ́сꙋ` conflates Jesus and Joshua).
- `synodal_v07_apply.rs`: the wave-materialization step deduplicated by whole
  row, so a derived row whose stable ID a previous wave had already
  materialized with a *refined* cell was appended as a duplicate ID. It now
  dedupes by stable identifier for the four tables that have one, which both
  restores idempotency and stops a later refinement from being regressed to a
  generic `lexical-form` cell.

## One test assertion changed

`additional_fourth_declension_nouns_are_productive_and_bounded` asserted that
`камень` dative plural fails with `OrthographicMetadataRequired`. That boundary
was a statement about missing evidence, and the evidence exists: the dative
plural `ка́менємъ` is **directly printed** in the source partition at Ezek.6.3
in both editions, in an unambiguous dative chain
(`гора́мъ и҆ холмѡ́мъ, и҆ ка́менємъ и҆ де́бремъ`), and the genitive plural
`ка́менїй` is printed 16 times. This is attested behaviour, not a
generalisation to an unattested cell.

The assertion was replaced rather than deleted: the test now asserts the dative
plural realises as `ка́менємъ` from `SynodalNormativeGeneration`, and
additionally pins the genitive plural `ка́менїй`. The test's purpose — that this
family is productive *and bounded by reviewed evidence* — is preserved: `камень`
still fails with `OrthographicMetadataRequired` for its singular dative,
accusative, instrumental and vocative, and the same invariant is asserted in
three other places.

One qualification belongs on the record. The fitted `noun:dual,plural` contract
is a single acute on the first stem vowel, which matches every high-frequency
plural print — `ка́менїй` (16), `ка́менехъ`, `ка́менїемъ`, `ка́мєни` (11, the
wide-є nominative plural the engine generates). But the corpus also prints
`ка̑мени` once, at Ex.28.12, with a **kamora** on a narrow-е nominative plural
(`ка̑мени въ па́мѧть сꙋ́ть`). The contract cannot produce that variant. It is a
missing ordered variant rather than a wrong form — the form the engine does
generate for that cell is itself attested 11 times — but the paradigm is
therefore incomplete, not exhaustive, and this file previously overstated it as
having no counterexample.

## Independent review

A four-lens independent review (evidence integrity, metric integrity, code
correctness, tests and invariants) plus an adversarial verification pass ran
against the merge base. Its confirmed findings were fixed before this audit was
finalised:

- **Two rules the tool could not derive.** Re-running the fitter from a clean
  merge-base worktree produced 656 rows; the committed table held 658. The two
  extras (`multiplicative-sugubyi` singular/plural, `v06-vzeti` past-passive
  long singular) were residue of an earlier `--apply` run made before the
  realizability guard existed, and the tool's own conflict report names their
  counterexamples (`сꙋгꙋбаѧ→сꙋгꙋ̑баѧ`, `взѧтымъ→взѧ̑тымъ`). Cause: append-only
  `--apply`. Fixed by re-deriving the whole table from a reset state.
- **Kamora fitted onto singular cells from syncretic tokens.** Five rules put a
  kamora on a singular scope after the analyzer offered a singular reading of a
  dual or plural print — `бра̑та` (accusative dual, `ви́дѣ два̀ бра̑та`), `сы̑нꙋ`
  (genitive dual), `кра̑ѧ` (dual, `два̀ кра̑ѧ`), `краи̑` (plural), `ка̑ѧ` (neuter
  plural). Fixed by the singular-only-kamora guard; `synodal:noun:krai` now
  carries kamora on dual and plural only and acute on the singular, which is
  the distinction the print encodes.
- **A rule fitted on one variant contradicting its sibling variant.**
  `synodal:noun:chudo` nominative plural has variants `чꙋдеса | чꙋда`; the fit
  saw `чꙋ́да` and set a stem-initial acute, which would have realised
  `чꙋ́деса` while the corpus prints `чꙋдеса̀` 121 times. Those prints were
  invisible because a duplicate registry entry already resolved them. Fixed by
  the corpus-contradiction guard, which rejected the rule.

Verified and refuted: the metric definition is unchanged
(`coverage.rs` is untouched), the denominator is unchanged, no held-out passage
is cited by any admitted row, the emitted scope strings compile to exactly the
scopes the guards test (proved exhaustively over 5,022 combinations), the
extractor validator and registry parser accept exactly the same twelve scope
shapes, and reports are byte-deterministic.

## Residual

The wave converged: further `--apply` rounds add nothing.

- 169 scope families still have no reproducing placement at the finest scope
  the grammar can express; they are reported with counterexamples for review.
- Rules rejected by the four guards: 69 overlapping an existing reviewed
  paradigm, 8 contradicted by a source-partition print, 8 singular-only kamora,
  1 unrealizable.
- 40 fitted rows for `synodal:adjective:zlyi` are admitted and evidence-backed
  but currently yield no tokens; the lexeme's residual unresolved tokens fail
  for a different reason.
- `--apply` is append-only by design. Once a rule takes effect its cells leave
  the accent gap, so a later run no longer proposes it; "not proposed" cannot
  be read as "rejected", and the tool must not prune on that basis. Re-deriving
  the table under a stricter guard means resetting `accent_paradigms.tsv` and
  the `*-accent-fit` evidence rows and re-applying from scratch, which is how
  this wave's final table was produced.
- The `missing-accent-or-orthographic-metadata` gap retains 10,048 tokens.
- Infinitive, supine, and lexical cells carry no grammatical number, so the
  reusable scope grammar cannot address them; they need per-cell `accents.tsv`
  evidence.

The dominant remaining frontier is unchanged and untouched by this wave:
`unknown-lexeme`, 338,901 tokens over 57,623 surfaces.

## Verification

Reproduced on the working tree after the wave:

| Command | Merge base `edb3036` | After wave |
|---|---|---|
| `cargo fmt --all -- --check` | pass | pass |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | **fail** (2 errors) | pass |
| `cargo test --workspace --all-targets --all-features` | pass | pass |
| `cargo test --workspace --doc` | pass | pass |
| `cargo xtask synodal-fixture-bootstrap` | pass | pass |
| `cargo xtask synodal-coverage --offline --check` | pass | pass |
| `cargo xtask synodal-coverage --offline --fixture --check` | **fail** (stale) | pass |
| `cargo xtask synodal-coverage --offline --check --require-complete` | fail (72.82%) | fail (73.34%) |
| `cargo xtask synodal-lexical-review-queue --check` | **fail** (stale) | pass |
| `cargo xtask synodal-evaluation-queue --check` | **fail** (stale) | pass |
| `cargo xtask synodal-family-review-queue --check` | pass | pass |
| `cargo xtask synodal-marginal-recovery --check` | **fail** (stale) | pass |
| `cargo xtask synodal-v07-apply --check` | **fail** (stale) | pass |
| `cargo xtask synodal-engine-audit --check` | **fail** (stale) | pass |
| `cargo xtask synodal-accent-fit --check` | n/a | pass |
| `cargo xtask synodal-check` | **fail** (invalid row) | pass |
| `cargo xtask check-all` | **fail** (clippy, then missing decisions) | pass |

Nine of the eleven required Synodal gates were already red at the merge base;
the baseline column was reproduced in a clean `git worktree` at `edb3036`.

`--require-complete` remains failing by design: it is the 100% completion gate
of the ongoing program, and coverage is 73.3434%.

Also verified:

- `cargo xtask synodal-regenerate` three consecutive times is byte-identical
  for both generated registries.
- `cargo xtask synodal-coverage --offline` and `cargo xtask synodal-accent-fit`
  reproduce byte-identical reports on a second run.
- `--no-default-features` and `wasm32-unknown-unknown --no-default-features`
  build for `synodal-church-slavonic`, `-core`, and `-dictionary`.
