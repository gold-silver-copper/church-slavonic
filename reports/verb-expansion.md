# Verb expansion implementation report

## Outcome

The core now has independently declared typed APIs for imperfect, imperative,
asigmatic aorist, new *ox*-aorist, and all four non-l participles. `VerbClass` no
longer implies any of those systems, and `VerbAspect` never chooses a past
formation. Missing principal parts fail as typed metadata errors; the represented
sigmatic formation fails explicitly instead of guessing.

Known dictionary cells still win. Productive calls require `VerbLexeme` metadata and
return stable rule IDs plus derivation traces. Participial stem formation is traced
separately from the shared adjective agreement rule.

## Authorities and frozen data

- Productive rules: University of Texas *Old Church Slavonic Online*, lessons 1–3
  and 6–7, with section links and contracts in `docs/MORPHOLOGY_SPEC.md`.
- Dictionary target: Kaikki/English Wiktionary OCS JSONL, 46,091,411 bytes,
  SHA-256 `5bd61e747aa7aeb677af92b4e32c65476e5c6ee74bff146269460c962be5456c`,
  Wiktionary dump 2026-07-06, Wiktextract `d9fa233`.
- Facade corpus: UD OCS PROIEL `r2.18`, commit
  `64eddf87abfaa51e7f5acf0bef1bebcdaca1559f`.
- Native past corpus: Syntacticus `20230428`, commit
  `525cee4fb40590d7d514376c11acaed1bdd91c15`.

Every corpus file hash is in `data/evaluation-sources.json`. The corpus text and
token-level mismatch details remain external under CC BY-NC-SA 4.0.

## Baseline versus final dictionary evaluation

The clean baseline had no productive dictionary OOV slice for imperfect,
imperative, aorist, or the four participles. Its held present slices were IA1
230/240, IA2 6/8, II1 70/72, II2 46/48, and II3 15/16; infinitive was 146/146,
l-participle 1,216/1,216, and supine 139/145.

The final held results are:

| Productive slice | Exact / eligible |
|---|---:|
| IA1 present | 230 / 240 |
| IA2 present | 6 / 8 |
| II1 present (non-metadata cells) | 63 / 63 |
| II2 present (non-metadata cells) | 42 / 42 |
| II3 present (non-metadata cells) | 14 / 14 |
| imperfect | 384 / 384 |
| imperative | 157 / 188 |
| present active, `YeshtSoft` | 17 / 17 |
| present active, `YushtHard` | 25 / 31 |
| present passive, `Im` | 15 / 15 |
| present passive, `Om` | 19 / 23 |
| past active, `Ush` | 14 / 25 |
| past active, `Vush` | 20 / 25 |
| past passive, `En` | 8 / 23 |
| infinitive | 146 / 146 |
| l-participle | 1,216 / 1,216 |
| supine | 139 / 145 |

Second-conjugation 1sg present cells are now excluded from this score because the
source 2sg cannot supply their required allomorph. The smaller denominator is a
stricter metadata contract, not an accuracy gain. Dictionary aorist tables do not
provide a safe diagnostic new-aorist principal part in this snapshot, so no
dictionary aorist percentage is invented; native evaluation supplies that evidence.

The extraction audit safely adds 325 complete present participle citations, taking
the registry from 134,436 to 134,761 cells and from 137,081 to 137,406 ordered
variants. All 134,761 cell variant lists and all 137,406 variants round-trip in exact
source order. Declined participle and malformed-aorist rejection counts remain
153,310 and 17,912.

## Real-world corpus result

The old UD-only baseline admitted 29,036 bundles and unioned ambiguous dictionary
lemmas: 4,153/19,432 diplomatic-any and 4,268/19,432 lookup-any. Schema 2 now excludes
ambiguous lemmas, fused negative finite forms, passive finite labels, incomplete
resultatives, and imperative cells outside the typed historical inventory. It also
distinguishes top-1 from any: the facade is 3,811/18,712 diplomatic-any and
3,909/18,712 lookup-any. These evaluator denominators differ and are not presented
as a morphology regression.

Native PROIEL/TOROT retains the past subtype. It exposes 14,393 compatible
imperfect/aorist tokens. After one diagnostic oracle principal-part cell per lemma is
excluded, the core attempts 4,368 tokens and matches 1,971 diplomatically and 2,058
under project lookup. New-aorist lookup is 1,682/2,643; imperfect lookup is
376/1,725. The lemma-disjoint final view is 324/623 diplomatic and 341/623 lookup.
The low imperfect corpus result is preserved as evidence of contraction, spelling,
suppletion, and formation limits rather than hidden by lossy normalization.

## Reviewed fixes and deliberate breaks

Fixed witnesses include complete `несѣахъ … несѣахѫ` imperfect generation,
`рекохъ/рече` new-aorist palatalization, separate `моли-/ведѣ-` imperative series,
and `несꙑ/несомъ/несъ/несенъ` participial stem paths. Present participle citations
such as the safely voiced present-active and present-passive cells are now extracted
before finite-tense parsing.

No dictionary table cell was broken: table-first round-trip is complete. Two
productive behaviors changed deliberately:

- a second-conjugation 1sg without its explicit allomorph now returns
  `MissingLexicalMetadata` instead of applying a broad consonant mutation; and
- productive imperative 1du follows the grammar's final `-вѣ`. The pinned template's
  `-ве` spelling remains available and takes precedence as a dictionary variant.

The final review also tightened the UD mapper after corpus witnesses exposed fused
negative, passive-finite, incomplete-resultative, and unsupported-imperative bundles
that the first evaluator draft had admitted. Those tokens now fail with explicit
incompatibility reasons rather than inflating either successes or failures.

After those fixes, the follow-up full diff review found no remaining confirmed
P0/P1 defect. Lower-severity residuals are the explicit unsupported areas below
rather than hidden fallback behavior.

## Residual unsupported or oracle-dependent areas

- Sigmatic aorist alternations and stem loss are typed but not generated.
- Suppletive/root paradigms such as `бꙑти`, `дати`, `ѣсти`, `вѣдѣти`, `хотѣти`,
  and motion verbs remain dictionary-backed unless an audited caller supplies every
  required past principal part.
- Optional imperfect contraction and imperative `-ꙗмъ/-ꙗте` variants remain
  table-backed.
- Past passive formation selection is lexically sparse; its held `En` citation score
  is reported rather than generalized to other formations.
- The primary `-ьш-` past-active formation of transformed i-stems and automatic
  final-j/`ov` seams before `-въш-` remain table-backed.
- The 153,310 declined participle rows still lack row-local tense/voice identity. A
  future positional parser must validate complete pinned block shapes atomically.
- Analytic perfects, pluperfects, futures, passives, conditionals, and reflexive
  phrases are outside the word inflector.

## Verification

The final verification gate runs formatting, workspace clippy with warnings denied,
all workspace/unit/integration/doc tests, semantic registry regeneration and guard
witnesses through `cargo xtask check-all`, both accuracy commands, corpus hash/schema
and threshold checks, and `git diff --check`. Exact commands and their final state are
reported in the handoff.
