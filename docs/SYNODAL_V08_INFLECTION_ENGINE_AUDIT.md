# Synodal v0.8 inflection-engine audit

## Headline result

The engine now inflects caller-supplied typed noun, adjective, and verb specifications without dictionary registration. The substantive new productive morphology is the complete short comparison declension and the complete short present/past active-participle declension, including their special citation edges and historically invalid vocatives. A reviewed fixed-stem accent paradigm now realizes multiple generated cells through both explicit and registry-backed APIs.

Corpus coverage is not the optimization target. Commit `aa4e693136ef094aab0da6ab166e1f23f49f9792` remains the frozen v0.7 checkpoint at 919,752 of 1,313,344 top-k tokens (70.031%). This audit neither raises that target nor treats incidental coverage movement as evidence of engine quality.

## Public engine contract

`NounSpec`, `AdjectiveSpec`, and `VerbSpec` accept closed linguistic types, validated Unicode stems and principal parts, explicit provenance, optional irregular/defective cells, and an optional typed `AccentParadigm`. `Inflector::form_spec` and the specialized paradigm methods retain caller-specified predictions as predictions. Registry and explicit routes delegate to the same pure productive kernel after their respective identity/override layers.

Paradigms retain every canonical attempted cell. `ParadigmStatus` separately reports attestation, irregular override, sourced prediction, caller-specified prediction, inherited prediction, ambiguity, historical invalidity, incomplete evidence, missing metadata, missing orthographic metadata, and unsupported behavior.

## Capability summary

The matrix contains 40 reviewed system/subtype rows: 31 productive rows, 5 rows involving exact tables, 2 explicit irregular rows, and 3 unsupported rows. Counts describe engine contracts, not corpus forms or tokens.

The machine-readable source of truth is `data/synodal/engine_capabilities.tsv`. Every row records its target recension, valid and invalid inventory, required metadata, alternations, accent contract, source citation, golden/boundary example, implementation, test, and typed failure.

## Complete capability matrix

| Category | Subtype | Status | Stable rule | Valid inventory | Required metadata | Accent contract | Citation | Typed failure |
|---|---|---|---|---|---|---|---|---|
| noun | first-hard-masculine | productive | `SYN-NOUN-I-HARD-M-ALYPY-34` | 7 cases × singular/dual/plural | lemma; stem; masculine; class | reusable paradigm or exact accent required for liturgical | Alypy Gamanovich grammar §§34–38 | `ContradictoryMetadata` |
| noun | first-hard-neuter | productive | `SYN-NOUN-I-HARD-N-ALYPY-34` | 7 cases × singular/dual/plural | lemma; stem; neuter; class | reusable paradigm or exact accent required for liturgical | Alypy Gamanovich grammar §§34–38 | `ContradictoryMetadata` |
| noun | first-soft-masculine | productive | `SYN-NOUN-I-SOFT-M-ALYPY-34` | 7 cases × singular/dual/plural | lemma; stem; masculine; class | reusable paradigm or exact accent required for liturgical | Alypy Gamanovich grammar §§34–38 | `ContradictoryMetadata` |
| noun | first-soft-neuter | productive | `SYN-NOUN-I-SOFT-N-ALYPY-34` | 7 cases × singular/dual/plural | lemma; stem; neuter; class | reusable paradigm or exact accent required for liturgical | Alypy Gamanovich grammar §§34–38 | `ContradictoryMetadata` |
| noun | second-hard | productive | `SYN-NOUN-II-HARD-ALYPY-39` | 7 cases × singular/dual/plural | lemma; stem; feminine; class | reusable paradigm or exact accent required for liturgical | Alypy Gamanovich grammar §§39–40 | `ContradictoryMetadata` |
| noun | second-soft | productive | `SYN-NOUN-II-SOFT-ALYPY-39` | 7 cases × singular/dual/plural | lemma; stem; feminine; class | reusable paradigm or exact accent required for liturgical | Alypy Gamanovich grammar §§39–40 | `ContradictoryMetadata` |
| noun | third-feminine | productive | `SYN-NOUN-III-F-ALYPY-41` | 7 cases × singular/dual/plural | lemma; stem; feminine; class | reusable paradigm or exact accent required for liturgical | Alypy Gamanovich grammar §41 | `ContradictoryMetadata` |
| adjective | short-hard-positive | productive | `SYN-ADJ-SHORT-HARD-ALYPY-53` | 72 canonical cells: 3 genders × 3 numbers × 7 cases, plus alternate-animacy accusatives | lemma; stem; hard class | reusable paradigm or exact accent required for liturgical | Alypy Gamanovich grammar §53 | `MissingPrincipalPart` |
| adjective | short-soft-positive | productive | `SYN-ADJ-SHORT-SOFT-ALYPY-53` | 72 canonical cells: 3 genders × 3 numbers × 7 cases, plus alternate-animacy accusatives | lemma; stem; soft class | reusable paradigm or exact accent required for liturgical | Alypy Gamanovich grammar §53 | `MissingPrincipalPart` |
| adjective | long-hard-positive | productive | `SYN-ADJ-LONG-HARD-ALYPY-57` | 72 canonical cells: 3 genders × 3 numbers × 7 cases, plus alternate-animacy accusatives | lemma; stem; hard class | reusable paradigm or exact accent required for liturgical | Alypy Gamanovich grammar §57 | `OrthographicMetadataRequired` |
| adjective | long-soft-positive | productive | `SYN-ADJ-LONG-SOFT-ALYPY-57` | 72 canonical cells: 3 genders × 3 numbers × 7 cases, plus alternate-animacy accusatives | lemma; stem; soft class | reusable paradigm or exact accent required for liturgical | Alypy Gamanovich grammar §57 | `OrthographicMetadataRequired` |
| adjective | long-comparative | productive | `SYN-ADJ-COMPARATIVE-LONG-ALYPY-58` | 72 canonical cells: 3 genders × 3 numbers × 7 cases, plus alternate-animacy accusatives | independent comparison stem; typed formation | reusable paradigm or exact accent required for liturgical | Alypy Gamanovich grammar §58 | `MissingPrincipalPart` |
| adjective | short-comparative | productive | `SYN-ADJ-COMPARATIVE-SHORT-ALYPY-58-98` | 63 canonical cells; singular/dual/plural; six cases; accusative animacy | independent comparison stem; AncientHard/AncientSoft/LaterYat/LaterAi formation | reusable paradigm or exact accent required for liturgical | Alypy Gamanovich grammar §58 citation forms and §98 complete declension | `HistoricallyInvalidCell / ContradictoryMetadata` |
| adjective | long-superlative | productive | `SYN-ADJ-SUPERLATIVE-LONG-ALYPY-59` | 72 canonical cells: 3 genders × 3 numbers × 7 cases, plus alternate-animacy accusatives | independent reviewed superlative/comparison stem | reusable paradigm or exact accent required for liturgical | Alypy Gamanovich grammar §59 | `MissingPrincipalPart` |
| adjective | short-superlative | unsupported | `SYN-ADJ-SHORT-SUPERLATIVE-UNSUPPORTED` | none | reviewed independent formation | unknown | Alypy Gamanovich grammar §59 does not license the engine's guessed short formation | `UnsupportedFormation` |
| verb | present | productive | `SYN-VERB-PRESENT-ALYPY-80` | 3 persons × singular/dual/plural | conjugation; present medial stem; independent 1sg and 3pl edges | reusable paradigm or exact accent required for liturgical | Alypy Gamanovich grammar §§79–80 | `MissingPrincipalPart` |
| verb | aorist-vowel | productive | `SYN-VERB-AORIST-VOWEL-ALYPY-86` | 3 persons × singular/dual/plural | aorist stem; VowelStem formation | reusable paradigm or exact accent required for liturgical | Alypy Gamanovich grammar §86 | `UnsupportedFormation` |
| verb | aorist-consonant | productive | `SYN-VERB-AORIST-CONSONANT-ALYPY-86` | 3 persons × singular/dual/plural | aorist stem; ConsonantStem formation | reusable paradigm or exact accent required for liturgical | Alypy Gamanovich grammar §86 | `MissingPrincipalPart` |
| verb | imperfect-h | productive | `SYN-VERB-IMPERFECT-H-ALYPY-87` | 3 persons × singular/dual/plural | aspect; independent imperfect stem; H formation | reusable paradigm or exact accent required for liturgical | Alypy Gamanovich grammar §87 | `HistoricallyInvalidCell` |
| verb | imperfect-yah | productive | `SYN-VERB-IMPERFECT-YAH-ALYPY-87` | 3 persons × singular/dual/plural | aspect; independent imperfect stem; Yah formation | reusable paradigm or exact accent required for liturgical | Alypy Gamanovich grammar §87 | `MissingPrincipalPart` |
| verb | imperfect-ah | productive | `SYN-VERB-IMPERFECT-AH-ALYPY-87` | 3 persons × singular/dual/plural | aspect; independent imperfect stem; Ah formation | reusable paradigm or exact accent required for liturgical | Alypy Gamanovich grammar §87 | `MissingPrincipalPart` |
| verb | imperative | productive | `SYN-VERB-IMPERATIVE-ALYPY-93` | 6 cells: 2sg/3sg, 1du/2du, and 1pl/2pl | independent imperative stem; typed formation | reusable paradigm or exact accent required for liturgical | Alypy Gamanovich grammar §93 | `HistoricallyInvalidCell` |
| verb | infinitive | productive-lexical | `SYN-VERB-INFINITIVE-LEXICAL` | infinitive | validated lexical infinitive lemma | reusable paradigm or exact accent required for liturgical | lexical metadata validated Synodal lemma | `InvalidUnicode` |
| verb | l-participle | productive | `SYN-VERB-LPART-ALYPY-97` | gender × singular/dual/plural | independent l-participle stem | reusable paradigm or exact accent required for liturgical | Alypy Gamanovich grammar §97 | `MissingPrincipalPart` |
| participle | present-active-long | productive | `SYN-VERB-PARTICIPLE-PRESENT-ACTIVE-ALYPY-95` | 72 canonical long-adjective cells: 3 genders × 3 numbers × 7 cases, plus alternate-animacy accusatives | aspect; independent present-active long stem; class | reusable paradigm or exact accent required for liturgical | Alypy Gamanovich grammar §95 | `HistoricallyInvalidCell` |
| participle | past-active-long | productive | `SYN-VERB-PARTICIPLE-PAST-ACTIVE-ALYPY-96` | 72 canonical long-adjective cells: 3 genders × 3 numbers × 7 cases, plus alternate-animacy accusatives | independent past-active long stem; class | reusable paradigm or exact accent required for liturgical | Alypy Gamanovich grammar §96 | `MissingPrincipalPart` |
| participle | present-passive-short-and-long | productive | `SYN-VERB-PARTICIPLE-PRESENT-PASSIVE-ALYPY-99` | 144 cells: 72 short plus 72 long canonical adjective cells | independent short and/or long stem; class | reusable paradigm or exact accent required for liturgical | Alypy Gamanovich grammar §99 | `MissingPrincipalPart` |
| participle | past-passive-short-and-long | productive | `SYN-VERB-PARTICIPLE-PAST-PASSIVE-ALYPY-100` | 144 cells: 72 short plus 72 long canonical adjective cells | independent short and/or long stem; class | reusable paradigm or exact accent required for liturgical | Alypy Gamanovich grammar §100 | `MissingPrincipalPart` |
| participle | present-active-short | productive | `SYN-VERB-PARTICIPLE-PRESENT-ACTIVE-SHORT-ALYPY-95-98` | 63 canonical cells; singular/dual/plural; six cases; accusative animacy | independent short stem; PresentFirstUnpalatalized/PresentFirstPalatalized/PresentSecond/PresentAfterSibilant formation; aspect | reusable paradigm or exact accent required for liturgical | Alypy Gamanovich grammar §95 formation and §98 complete declension | `HistoricallyInvalidCell / ContradictoryMetadata` |
| participle | past-active-short | productive | `SYN-VERB-PARTICIPLE-PAST-ACTIVE-SHORT-ALYPY-96-98` | 63 canonical cells; singular/dual/plural; six cases; accusative animacy | independent short stem; PastConsonant/PastVowel/PastIotated formation | reusable paradigm or exact accent required for liturgical | Alypy Gamanovich grammar §96 formation and §98 complete declension | `HistoricallyInvalidCell / ContradictoryMetadata` |
| verb | simple-future | exact-only | `SYN-VERB-FUTURE-EXACT` | reviewed exact table cells | stable lexeme identity | exact printed table | Alypy Gamanovich grammar lexeme-specific sections | `UnsupportedCell` |
| verb | finite-past-underspecified | exact-only | `SYN-VERB-PAST-EXACT` | reviewed exact cells | stable lexeme identity | exact printed table | reviewed target evidence cell-specific citation | `UnsupportedCell` |
| verb | supine | unsupported | `SYN-VERB-SUPINE-UNSUPPORTED` | none | independent sourced formation not established | unknown | source review open no complete Synodal input/output contract | `UnsupportedCell` |
| verb | verbal-noun | unsupported | `SYN-VERB-VERBAL-NOUN-UNSUPPORTED` | none | stem tuple is represented but realization rule is not established | unknown | source review open no complete Synodal suffix/declension contract | `UnsupportedCell` |
| pronoun | closed-class | exact-only | `SYN-PRONOUN-EXACT` | reviewed complete/partial exact tables | stable lexeme identity | exact printed table | Alypy Gamanovich grammar §§60–78 and reviewed cells | `UnsupportedCell` |
| numeral | cardinal-and-collective | exact-only | `SYN-NUMERAL-EXACT` | reviewed exact table cells | stable lexeme identity | exact printed table | Alypy Gamanovich grammar §§45–52 | `UnsupportedCell` |
| numeral | ordinal | productive | `SYN-NUMERAL-ORDINAL-ADJECTIVAL` | 72 gendered long-adjective cells; a genderless request is metadata-deficient | ordinal adjective stem; class; grammatical gender | reusable paradigm or exact accent required for liturgical | Alypy Gamanovich grammar §52 | `MissingMetadata` |
| accent | mudr-fixed-stem | productive-reviewed | `synodal-accent:mudr-fixed-stem` | long positive singular adjective cells | stem vowel index 0; acute; scoped rule | one row applies across multiple cells | Alypy Gamanovich grammar §57 мꙋ́дръ paradigm | `OrthographicMetadataRequired` |
| irregular | byti-finite-systems | irregular-exact | `SYN-REGISTRY-IRREGULAR-OVERRIDE` | reviewed present/aorist/imperfect/imperative cells | stable identity; exact registry | exact printed table | Alypy Gamanovich grammar §81 | `UnsupportedCell` |
| irregular | syn-partial-noun | partial-irregular-with-regular-background | `SYN-REGISTRY-IRREGULAR-OVERRIDE` | dative singular and reviewed plural overrides; other cells use explicit first-hard-m class | stable identity; exact overrides; explicitly classed regular background | exact override before reusable/productive accent | Alypy Gamanovich grammar §37 | `UnsupportedCell only without exact or licensed background` |

## New source-backed morphology

- `SYN-ADJ-COMPARATIVE-SHORT-ALYPY-58-98`: Alypy §58 supplies ancient and later comparison-stem formations and special nominative citation edges; §98 supplies the complete short-comparison declension. The API requires an independent comparison stem plus `ComparisonFormation`.
- `SYN-VERB-PARTICIPLE-PRESENT-ACTIVE-SHORT-ALYPY-95-98`: Alypy §95 supplies present-active stems/citation edges and the imperfective restriction; §98 supplies the complete declension.
- `SYN-VERB-PARTICIPLE-PAST-ACTIVE-SHORT-ALYPY-96-98`: Alypy §96 supplies consonant, vowel, and iotated past-active formations/citation edges; §98 supplies the complete declension.

All three rules cover 63 canonical valid cells: singular, dual, and plural; six licensed cases; all genders; and the additional animate accusative cells. Vocatives are retained as `HistoricallyInvalidCell`. Complete typed goldens exercise 189 successful cells plus 27 invalid vocatives.

## Reusable accent realization

`synodal-accent:mudr-fixed-stem` is a reviewed fixed-first-stem-vowel acute paradigm for long positive singular forms of `мꙋдръ`, cited to Alypy §57. It is one scoped rule that generates multiple cells, not a renamed list of accented strings. The exact nominative accent row still wins first; other licensed singular cells use the reusable paradigm. Missing scope remains `OrthographicMetadataRequired { field: AccentParadigm }`. The model separately represents stem versus ending placement, cell/number scopes, acute/grave/kamora, and an independently positioned psili breathing.

## Irregular and defective behavior

Exact attested rows remain attestations. Normative cells in a declared irregular system are tagged `SynodalIrregularOverride`. `сынъ` demonstrates a partial irregular system: reviewed dative-singular/plural overrides precede generation, while cells outside that declared override fall back only because the lexeme has an explicit first-hard masculine background. Explicit specs can likewise attach caller-specified overrides and can retain either historically absent or evidence-incomplete cells as distinct outcomes.

## Behavioral verification

The engine tests cover unregistered noun/adjective/verb specifications, independent present edges and non-present stems, complete short-comparison and active-participle inventories, ordered variants, vocative invalidity, perfective restrictions, missing and contradictory metadata, explicit/registry parity, exact/irregular/productive precedence, partial irregular fallback, evidence-incomplete cells, reusable accents through both routes, exact accent precedence, combining-mark order, and hostile Unicode.

At the v0.8 completion gate, the Synodal core passes 42 unit tests and 1 doctest, the facade passes 37 unit tests and 6 doctests, and the dictionary passes 27 unit tests, 5 CLI integration tests, and 1 doctest. The complete all-target workspace suite, workspace doctests, native/no-default-feature checks, `wasm32-unknown-unknown` checks, generated-registry checks, audit byte-current check, and package dry-runs also pass.

The completion gate is:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test -p synodal-church-slavonic-core --all-features
cargo test -p synodal-church-slavonic --all-features
cargo test -p synodal-church-slavonic-dictionary --all-features
cargo test --workspace --all-targets --all-features
cargo test --workspace --doc
cargo xtask synodal-engine-audit --check
cargo xtask synodal-check
cargo xtask check-all
```

## Remaining source blockers

- **adjective / short-superlative:** all cells Failure: `UnsupportedFormation`.
- **verb / supine:** all cells Failure: `UnsupportedCell`.
- **verb / verbal-noun:** all cells Failure: `UnsupportedCell`.

Simple future, underspecified finite past, pronouns, and cardinal/collective numerals remain exact-table systems. The engine does not claim complete Church Slavonic support.

## Incidental corpus regression signal

The frozen v0.7 checkpoint remains 919,752 top-k and 601,108 top-1 tokens. Against the same 1,313,344-token denominator, the live v0.8 regression run reports 919,786 top-k (+34), 601,081 top-1 (-27), 17,149 ambiguous (unchanged), and 392,520 unresolved (-34), or 70.033898% top-k. The shape is consistent with exposing additional ordered productive candidates: some formerly unresolved surfaces become analyzable, while some formerly unique surfaces gain another compatible analysis. This is a secondary regression observation, not an optimization result, and it did not drive rule selection.
