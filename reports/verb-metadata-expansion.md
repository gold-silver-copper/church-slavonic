# Verb metadata expansion

This report records the source-backed verb-principal-part milestone against the
pinned 2026-07-06 English-Wiktionary/Kaikki snapshot. Dictionary agreement,
real-manuscript recall, and corpus-derived oracle morphology remain separate
questions throughout.

## Starting point

The clean baseline contained 3,081 lexemes, 134,761 public dictionary cells, and
137,406 ordered variants. It had no normalized verb-principal-part registry and no
ordinary lemma-to-productive-verb bridge: `VerbLexeme` metadata had to be assembled
by the caller.

On the pinned UD input, 18,712 unambiguous compatible attempts returned 8,715 exact
dictionary-table results. Diplomatic any accuracy was 3,811/18,712 and shared
NFC/lowercase any accuracy was 3,909/18,712; 9,997 attempts returned no public form.
The independent native-corpus diagnostic was already 1,682/2,643 lookup-any for the
new *ox*-aorist, 376/1,725 for imperfects, and 341/623 in the lemma-disjoint final
view. Those native numbers use a principal part from another token of the same lemma
and are explicitly oracle metadata.

The legacy dictionary OOV diagnostic reported 384/384 held imperfect cells and
157/188 held imperative cells. It tested production behavior with evaluator-declared
metadata; it did not show that the public facade could construct those lexical
inputs.

## Delivered design

- Schema 2 adds `data/extracted/verb_metadata.tsv`. Its 3,157 field rows preserve
  lexeme ID, independent system, analysis rank, field/value, provenance, diagnostic
  feature/spelling, cross-check features, and authority.
- `DictionaryVerbMetadata` validates normalized codes into enums. Present,
  imperfect, aorist, imperative, l-participle, and all four non-l participle systems
  are separate ordered analysis arrays; lexical aspect is independent.
- `FormSet::analyses()` binds each generated alternative to evidence and its own
  metadata-selection/productive-rule trace. Exact source table variants remain
  unchanged and first.
- Lemma and by-ID APIs now resolve table -> metadata -> approved cell override ->
  production core. `*_with` remains dictionary-independent. A normalized
  `form_by_id` entry point lets offline evaluators exercise that same public path.
- Overrides compile into a separate static array and never remove a source form.
  Six otherwise absent `бꙑти` imperfect cells are the initial reviewed set; the
  pinned 3sg table cell still returns `DictionaryTable`.
- The core adds the primary transformed i-stem `-ьш-` formation and typed ordinary,
  final-j-deletion, and `ov -> u` paths before `-въш-`. The sigmatic aorist stays
  immediately unsupported because one generic stem cannot encode its root grade,
  singular allomorph, `s/x` seam, and optional `-тъ` policy.

## Metadata coverage

Coverage counts are distinct lexemes, not generated cells.

| Field/formation | Lexemes |
|---|---:|
| lexical aspect | 647 |
| present class + stem + first-singular field | 121 |
| imperfect stem + formation | 150 |
| imperfect `A` / `YatA` | 44 / 106 |
| imperfect `UncontractedOnly` variant policy | 150 |
| aorist stem + formation | 0 |
| aorist `Asigmatic` / `New` / sigmatic subtypes | 0 / 0 / 0 |
| imperative stem + formation | 73 |
| imperative i-series / yat-series | 73 / 0 automatically admitted |
| l-participle stem | 185 |
| present-active participle | 186 |
| present-passive participle | 134 |
| past-active participle | 185 |
| past-passive participle | 134 |
| past-active `Ish` / `Ush` / `Vush` | 44 / 96 / 89 |
| past-passive `En` / `N` / `T` | 67 / 51 / 16 |

Past-active alternatives explain why that system has 229 normalized analyses for
185 lexemes. No source citation is silently collapsed into one formation.
No aorist row in this snapshot survives both the typed 1sg diagnostic and an
independent exact cross-cell check, so dictionary-backed aorist generation remains
at zero rather than using the native corpus's same-lemma oracle metadata.

## Leakage-controlled dictionary result

The primary held-cell evaluator removes the target feature, every same-spelling
feature, and equivalent 2sg/3sg finite or imperative cells before derivation or
cross-checking. It rebuilds and validates metadata, then calls the public
dictionary-metadata resolver with exact-table lookup unavailable. FNV-1a modulo five
keeps whole normalized lemma keys in one frozen partition.

| Funnel stage | Development | Final holdout |
|---|---:|---:|
| compatible requested cells | 8,895 | 2,748 |
| unambiguous requested cells | 8,628 | 2,564 |
| metadata found and validated | 2,784 | 997 |
| generation attempts / returned | 2,784 / 2,784 | 997 / 997 |
| diplomatic top-1 / any | 2,690 / 2,690 | 964 / 964 |
| lookup top-1 / any | 2,690 / 2,690 | 964 / 964 |

Final metadata availability is 38.89% of unambiguous targets and final conditional
lookup-any correctness is 96.69%. Development availability is 32.27% and
conditional correctness is 96.62%. Availability and conditional accuracy have
independent non-regression guards so abstention cannot manufacture a gain. System,
cell, generation path, class, formation, source-cell policy, analysis kind, and
frequency slices are committed in `reports/accuracy.json` and `reports/accuracy.md`.

Citation participles intentionally do not appear in this held-target numerator:
removing their only safely typed citation leaves no independent formation selector.
Their value is instead tested by declined public requests and external real-text
recall.

## Independent-corpus result

Every UD and native input file matched the commits and SHA-256 values in
`data/evaluation-sources.json`. No token listing or excerpt is committed.

The same 18,712 UD facade attempts now return 11,063 forms, an increase of 2,348.
Diplomatic any matches rise from 3,811 to 4,711; lookup-any rises from 3,909 to
4,850. Exact dictionary-table results are unchanged at 8,715. New returned paths
are:

| Dictionary-metadata path | Returned | Diplomatic any | Lookup any |
|---|---:|---:|---:|
| multiple ordered analyses | 170 | 58 | 60 |
| past-active participle rule | 1,294 | 548 | 581 |
| past-passive participle rule | 175 | 84 | 88 |
| present-active participle rule | 646 | 157 | 159 |
| present-passive participle rule | 63 | 53 | 53 |

This is real-text dictionary-metadata recall, not a held-dictionary-target score:
the corpus surface is not used as metadata, but the dictionary principal part was
not rebuilt separately for each corpus token. The native same-lemma oracle results
remain unchanged at 1,682/2,643 new-aorist lookup-any, 376/1,725 imperfect
lookup-any, and 341/623 in the final lemma partition.

The imperfect mismatch audit found 1,349 diplomatic mismatches in 1,725 attempts:
890 in `YatA`, 459 in explicit-base `A`, and 1,100 in 3sg. `бꙑти` contributes 630
and abbreviation-heavy `глаголати` 235. Document aggregates are Suprasliensis 717,
Marianus 483, Zographensis 113, and Psalterium Sinaiticum 36. Contraction,
suppletion, abbreviation/diacritics, orthography, and unsafe principal-part choice
are interleaved, so no corpus-wide contracted variant was admitted.

## Authorities, extraction, and licensing

- Dictionary fields come only from the pinned 46,091,411-byte Kaikki/Wiktextract
  artifact with SHA-256
  `5bd61e747aa7aeb677af92b4e32c65476e5c6ee74bff146269460c962be5456c`.
  Actual `cu-verb` head metadata and typed source cells are retained as witnesses.
- Productive morphology is independently specified from University of Texas *Old
  Church Slavonic Online*: lesson 1 §4.2 (imperfect), lesson 2 §9 (imperative),
  lesson 3 §§14.1-14.3 (aorists), lesson 6 §26 and lesson 7 §§31.1-32
  (participles). Exact links live beside the rules in `docs/MORPHOLOGY_SPEC.md`.
- The extractor fixture covers every admitted derivation, reordered tags,
  contradiction rejection, target exclusion, normalized provenance, deterministic
  refresh, content-derived ID/alias rewriting, and atomic failure preservation.
- Extraction counts remain 3,081 lexemes and 137,406 accepted form rows. All 153,310
  declined-participle rows remain rejected because the snapshot does not preserve a
  complete independently verifiable tense/voice table-block identity.
- UD OCS PROIEL and Syntacticus/PROIEL/TOROT stay optional CC BY-NC-SA 4.0 local
  inputs. Only non-reconstructive aggregates are committed; packages contain none of
  those corpora or derived token rows.

## Deliberate behavior/API changes

- Registry schema is now 2 and the accuracy report schema is 4.
- `FormSet`, `FormSource`, rule IDs, metadata evidence, and past-active formation
  enums gained explicit analysis/provenance variants.
- An invalid generated code, incomplete group, duplicate/rank gap, empty/non-NFC or
  non-Cyrillic productive stem, orphan ID, or unapproved override now fails
  validation rather than reaching generation.
- `DictionaryTable`, `DictionaryPrincipalPart`, `CuratedGrammarOverride`, explicit
  caller metadata, corpus-only observation, and productive output are distinct.
- Missing source forms may now generate for an ordinary known lemma. Exact source
  cells are neither displaced nor relabeled.
- Contradictory normalized metadata, represented unsupported formations, and
  historically invalid cells now have distinct typed errors.

## Review findings and remaining boundary

The implementation review found and fixed material integration risks: override rows
initially displaced source rows instead of remaining separate; the corpus facade
initially called a table-only generic getter; held filtering initially relied on
same spelling rather than explicitly excluding equivalent 2sg/3sg cells; and
generated metadata needed closed-code, group, Unicode, reference, and provenance
validation at both extraction and runtime boundaries. Final passes also made the
registry check independently rederive every committed metadata row, closed the
override feature vocabulary and rejected source-cell shadowing, kept primary and
secondary sigmatic codes as distinct unsupported enum variants, and restricted
productive stems to Cyrillic so source-only Glagolitic cannot become mixed-script
output. The last API audit added an explicit grammar-sourced imperfect variant
policy and separated contradictory metadata, represented unsupported formations,
and historically invalid cells into distinct typed errors.

Confirmed no-implementation findings are retained deliberately:

- sigmatic aorist subtypes need independently sourced root-grade, singular-base,
  seam, and optional-ending policies;
- automatic `PalatalizedA`, yat-series imperative, and special past-active j/ov seam
  selection lack recoverable diagnostics in this snapshot;
- irregular verbs beyond the reviewed `бꙑти` cells remain table-backed;
- contracted imperfect distribution is not safe to generalize from aggregate
  corpus mismatches; and
- declined-participle extraction remains blocked on a pinned, atomic table-block
  identity parser.

Unsupported cells therefore still fail explicitly rather than guessing.

## Verification

The final workspace passed `cargo fmt --all -- --check`, workspace/all-targets/
all-features clippy with `-D warnings`, and the complete test suite (3 facade unit,
12 public API, 19 core, 23 extractor unit, 1 extractor refresh, and 10 xtask tests).
`cargo xtask check-all` verified the 3,081-lexeme/137,406-variant registry,
row-for-row metadata rederivation, current reports, examples, attribution, and the
runtime no-I/O boundary. `cargo xtask guard-witnesses` detected and reverted every
injected failure, including incomplete analyses, non-Cyrillic stems, invalid
override features, stale reports, availability regression, and conditional-accuracy
regression.

`cargo xtask accuracy` regenerated the dictionary aggregates. `cargo xtask
accuracy-corpus --ud <PINNED_UD> --syntacticus <PINNED_SYNTACTICUS> --write`
verified every configured commit/file hash and refreshed only aggregate reports.
Both `cargo package -p old-church-slavonic-core --allow-dirty` and `cargo package -p
old-church-slavonic --allow-dirty` built and verified: 16 files/132.5 KiB for core
and 20 files/23.6 MiB for the facade. Archive inspection found attribution and no
external corpus, token-detail, TSV, XML, CoNLL-U, or mismatch-detail payloads.
