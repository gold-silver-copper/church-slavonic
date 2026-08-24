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
