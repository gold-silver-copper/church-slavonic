# Synodal v0.9 inflection-engine audit

## Headline result

The engine now productively covers thirteen reviewed noun classes. The v0.9 slice adds first-declension velar and mixed masculines, third-declension masculines, and three fourth-declension stem-extending families. It also represents lexical number inventories and complete `имѧ`, `небо`, `мати`, `мꙋжъ`, and `пꙋть` paradigms without replacing productive rules with mass exact tables. Three complete noun accent paradigms add fixed-stem and genuinely cell-conditioned mobile behavior.

Corpus coverage is not the optimization target. The frozen v0.7 checkpoint remains a regression baseline only; no v0.9 rule, lexical upgrade, or accent pattern was selected from frequency or coverage movement.

## Public engine contract

`NounSpec`, `AdjectiveSpec`, and `VerbSpec` accept closed linguistic types, validated Unicode stems and principal parts, explicit provenance, optional irregular/defective cells, and an optional typed `AccentParadigm`. Fourth-declension nouns require the independently supplied extended stem; `NounNumberInventory` makes absent numbers explicit. `PresentPrincipalParts` and `VerbSpecBuilder::present_series` install the three independent present inputs atomically. Registry and explicit routes delegate to the same pure productive kernel after identity and override layers.

`VerbSystem` selects every represented finite, imperative, infinitive, l-participle, participial, supine, and verbal-noun inventory through one paradigm API. Paradigms retain every attempted cell. `ParadigmStatus`, `ParadigmRow::error_code`, and `ErrorCode` give stable access to successes, ambiguity, historical invalidity, incomplete evidence, missing metadata, missing orthographic metadata, and unsupported behavior without parsing diagnostic prose.

## Capability summary

The matrix contains 49 reviewed system/subtype rows: 40 productive rows, 5 rows involving exact tables, 2 explicit irregular rows, and 3 unsupported rows. Counts describe engine contracts, not corpus forms or tokens.

The machine-readable source of truth is `data/synodal/engine_capabilities.tsv`. Every row records its target recension, valid and invalid inventory, required metadata, alternations, accent contract, source citation, golden/boundary example, implementation, test, and typed failure.

## Complete capability matrix

| Category | Subtype | Status | Stable rule | Valid inventory | Required metadata | Accent contract | Citation | Typed failure |
|---|---|---|---|---|---|---|---|---|
| noun | first-hard-masculine | productive | `SYN-NOUN-I-HARD-M-ALYPY-34` | 7 cases × singular/dual/plural | lemma; stem; masculine; class | reusable paradigm or exact accent required for liturgical | Alypy Gamanovich grammar §§34–38 | `ContradictoryMetadata` |
| noun | first-hard-velar-masculine | productive | `SYN-NOUN-I-HARD-VELAR-M-ALYPY-34` | 7 cases × singular/dual/plural | lemma; final-velar stem; masculine; class | reusable paradigm or exact accent required for liturgical | Alypy Gamanovich grammar §§34–35 | `ContradictoryMetadata` |
| noun | first-mixed-masculine | productive | `SYN-NOUN-I-MIXED-M-ALYPY-33-34` | 7 cases × singular/dual/plural | lemma; sibilant stem; masculine; class | reusable paradigm or exact accent required for liturgical | Alypy Gamanovich grammar §§33–35 | `ContradictoryMetadata` |
| noun | first-hard-neuter | productive | `SYN-NOUN-I-HARD-N-ALYPY-34` | 7 cases × singular/dual/plural | lemma; stem; neuter; class | reusable paradigm or exact accent required for liturgical | Alypy Gamanovich grammar §§34–38 | `ContradictoryMetadata` |
| noun | first-soft-masculine | productive | `SYN-NOUN-I-SOFT-M-ALYPY-34` | 7 cases × singular/dual/plural | lemma; stem; masculine; class | reusable paradigm or exact accent required for liturgical | Alypy Gamanovich grammar §§34–38 | `ContradictoryMetadata` |
| noun | first-soft-neuter | productive | `SYN-NOUN-I-SOFT-N-ALYPY-34` | 7 cases × singular/dual/plural | lemma; stem; neuter; class | reusable paradigm or exact accent required for liturgical | Alypy Gamanovich grammar §§34–38 | `ContradictoryMetadata` |
| noun | second-hard | productive | `SYN-NOUN-II-HARD-ALYPY-39` | 7 cases × singular/dual/plural | lemma; stem; feminine; class | reusable paradigm or exact accent required for liturgical | Alypy Gamanovich grammar §§39–40 | `ContradictoryMetadata` |
| noun | second-soft | productive | `SYN-NOUN-II-SOFT-ALYPY-39` | 7 cases × singular/dual/plural | lemma; stem; feminine; class | reusable paradigm or exact accent required for liturgical | Alypy Gamanovich grammar §§39–40 | `ContradictoryMetadata` |
| noun | third-feminine | productive | `SYN-NOUN-III-F-ALYPY-41` | 7 cases × singular/dual/plural | lemma; stem; feminine; class | reusable paradigm or exact accent required for liturgical | Alypy Gamanovich grammar §41 | `ContradictoryMetadata` |
| noun | third-masculine | productive | `SYN-NOUN-III-M-ALYPY-41` | 7 cases × licensed singular/dual/plural inventory | lemma; consonantal stem; masculine; class; number inventory | reusable paradigm or exact accent required for liturgical | Alypy Gamanovich grammar §41 | `HistoricallyInvalidCell / ContradictoryMetadata` |
| noun | fourth-neuter-en | productive | `SYN-NOUN-IV-N-EN-ALYPY-42-43` | 7 cases × singular/dual/plural | citation lemma; independent extended -ен- stem; neuter; class | imya-mobile reusable cell-scoped accent paradigm or exact accent required for liturgical | Alypy Gamanovich grammar §§42–43 | `ContradictoryMetadata / OrthographicMetadataRequired` |
| noun | fourth-neuter-es | productive | `SYN-NOUN-IV-N-ES-ALYPY-42-43` | 7 cases × singular/dual/plural | citation lemma; independent extended -ес- stem; neuter; class | nebo-mobile reusable cell-scoped accent paradigm or exact accent required for liturgical | Alypy Gamanovich grammar §§42–43 | `ContradictoryMetadata / OrthographicMetadataRequired` |
| noun | fourth-feminine-er | productive | `SYN-NOUN-IV-F-ER-ALYPY-42-43` | 7 cases × singular/dual/plural | citation lemma; independent extended -ер- stem; feminine; class | mati-fixed-stem reusable accent paradigm or exact accent required for liturgical | Alypy Gamanovich grammar §§42–43 | `ContradictoryMetadata / OrthographicMetadataRequired` |
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
| accent | mati-fixed-stem | productive-reviewed | `synodal-accent:mati-fixed-stem` | all noun cases × singular/dual/plural | stable lexeme identity; stem vowel index 0; acute | exact cell precedes reusable paradigm | Alypy Gamanovich grammar §43 complete ма́ти paradigm | `ContradictoryMetadata / OrthographicMetadataRequired` |
| accent | imya-mobile | productive-reviewed | `synodal-accent:imya-mobile` | all noun cases × singular/dual/plural | stable lexeme identity; case-and-number scopes; acute/grave; initial psili | exact cell precedes reusable paradigm | Alypy Gamanovich grammar §43 complete и҆́мѧ paradigm | `ContradictoryMetadata / OrthographicMetadataRequired` |
| accent | nebo-mobile | productive-reviewed | `synodal-accent:nebo-mobile` | all noun cases × singular/dual/plural | stable lexeme identity; case-and-number scopes; acute/grave | exact cell precedes reusable paradigm | Alypy Gamanovich grammar §43 complete не́бо paradigm | `ContradictoryMetadata / OrthographicMetadataRequired` |
| irregular | byti-finite-systems | irregular-exact | `SYN-REGISTRY-IRREGULAR-OVERRIDE` | reviewed present/aorist/imperfect/imperative cells | stable identity; exact registry | exact printed table | Alypy Gamanovich grammar §81 | `UnsupportedCell` |
| irregular | syn-partial-noun | partial-irregular-with-regular-background | `SYN-REGISTRY-IRREGULAR-OVERRIDE` | dative singular and reviewed plural overrides; other cells use explicit first-hard-m class | stable identity; exact overrides; explicitly classed regular background | exact override before reusable/productive accent | Alypy Gamanovich grammar §37 | `UnsupportedCell only without exact or licensed background` |

## New source-backed morphology

- `SYN-NOUN-I-HARD-VELAR-M-ALYPY-34` implements the reviewed г/к/х alternations at the exact §34 seams, with separate first and second palatalization behavior and boundary tests for all three velars.
- `SYN-NOUN-I-MIXED-M-ALYPY-33-34` implements the complete `мꙋжъ` mixed paradigm and its ordered nominative variants. Alypy §35 says `-(ь)ми` is lexical and unavailable to some nouns, so the class deliberately does not invent it.
- `SYN-NOUN-III-M-ALYPY-41` implements the complete `пꙋть` consonantal paradigm, including ordered vocative and genitive-plural variants. `NounNumberInventory` separately represents plural-only nouns such as `людїе`.
- `SYN-NOUN-IV-N-EN-ALYPY-42-43`, `SYN-NOUN-IV-N-ES-ALYPY-42-43`, and `SYN-NOUN-IV-F-ER-ALYPY-42-43` require explicit extended stems and implement the complete `имѧ : имен-`, `небо : небес-`, and `мати : матер-` tables, including reviewed wide-letter alternations and ordered ending variants.

## Reusable accent realization

`synodal-accent:mati-fixed-stem`, `synodal-accent:imya-mobile`, and `synodal-accent:nebo-mobile` encode the complete Alypy §43 tables as reusable rules. `мати` uses fixed first-stem-vowel stress. `имѧ` and `небо` use disjoint number-and-case scopes with stem/ending placement and acute/grave selection; `имѧ` also preserves initial psili before the accent mark. The implementation rejects missing and overlapping scopes, preserves exact-cell precedence, and retains `OrthographicMetadataRequired` when no rule applies.

## Irregular and defective behavior

Exact attested rows remain attestations. Normative cells in a declared irregular system are tagged `SynodalIrregularOverride`. In addition to `сынъ`, the upgraded `мꙋжъ` identity now demonstrates exact-first resolution with a productive mixed-declension background for uncovered cells. The reviewed `имѧ`, `небо`, and `мати` identities retain their exact normative tables while carrying productive classes and reusable accent metadata. Plural-only `людїе` adds a separately evidenced `NounNumberInventory` restriction so singular and dual requests remain typed invalid cells. Explicit specs can attach caller-specified overrides and retain historically absent or evidence-incomplete cells as distinct outcomes.

## Behavioral verification

The engine tests cover complete new noun paradigms, all velar boundaries, number restrictions, ordered variants, independent present edges and non-present stems, unified verb-system inventories, stable error codes, missing and contradictory metadata, explicit/registry parity, exact/irregular/productive precedence, reusable accents through both routes, exact accent precedence, missing/overlapping accent scopes, combining-mark order, and hostile Unicode.

The completion gate includes the package-specific and complete workspace suites, doctests, clippy with warnings denied, native no-default-feature builds, `wasm32-unknown-unknown` builds, byte-current generated registries and audit, package dry-runs, and a separate full-diff review.

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

## Corpus regression policy

Corpus evaluation remains available only as a regression signal. The v0.9 implementation was selected and validated from complete target-recension grammatical tables and independently reviewed lexical metadata; no frequency-ranked exact forms were added to simulate morphology.
