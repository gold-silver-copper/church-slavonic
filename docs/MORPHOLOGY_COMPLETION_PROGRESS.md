# Church Slavonic morphology completion progress

This report is generated from `data/morphology/completion.toml` by `cargo xtask morphology-completeness`. It is a progress inventory, not a claim of completion.

Inventory date: `2026-08-14`. Source-frontier date: `2026-08-14`.

## Headline

The matrix contains **53** required system contracts: **32** have final states and **21** remain non-final. Source discovery has 1 recorded pass and has not converged.

## State totals

| State | Systems |
|---|---:|
| `implementation-missing` | 16 |
| `not-inflectional` | 2 |
| `productive-complete` | 30 |
| `source-review-open` | 5 |

## Recension totals

| Recension | Final | Total |
|---|---:|---:|
| cross-recension | 3 | 4 |
| old-church-slavonic | 19 | 26 |
| synodal-russian | 10 | 23 |

## Category totals

| Recension | Category | Final | Total |
|---|---|---:|---:|
| cross-recension | api | 1 | 1 |
| cross-recension | evaluation | 1 | 1 |
| cross-recension | provenance | 1 | 1 |
| cross-recension | research | 0 | 1 |
| old-church-slavonic | accent | 0 | 1 |
| old-church-slavonic | adjective | 1 | 1 |
| old-church-slavonic | analytic-form | 1 | 1 |
| old-church-slavonic | comparison | 2 | 2 |
| old-church-slavonic | determiner | 0 | 1 |
| old-church-slavonic | finite-verb | 6 | 6 |
| old-church-slavonic | imperative | 2 | 2 |
| old-church-slavonic | invariant | 1 | 1 |
| old-church-slavonic | irregular-verb | 0 | 1 |
| old-church-slavonic | lexicon | 0 | 1 |
| old-church-slavonic | nonfinite-verb | 1 | 1 |
| old-church-slavonic | noun | 1 | 1 |
| old-church-slavonic | numeral | 0 | 1 |
| old-church-slavonic | orthography | 1 | 2 |
| old-church-slavonic | participle | 1 | 1 |
| old-church-slavonic | pronoun | 2 | 2 |
| old-church-slavonic | verbal-noun | 0 | 1 |
| synodal-russian | adjective | 1 | 1 |
| synodal-russian | analytic-form | 1 | 2 |
| synodal-russian | comparison | 0 | 1 |
| synodal-russian | determiner | 0 | 1 |
| synodal-russian | finite-verb | 3 | 5 |
| synodal-russian | invariant | 1 | 1 |
| synodal-russian | irregular-verb | 0 | 1 |
| synodal-russian | lexicon | 0 | 1 |
| synodal-russian | nonfinite-and-mood | 1 | 1 |
| synodal-russian | nonfinite-verb | 0 | 1 |
| synodal-russian | noun | 1 | 2 |
| synodal-russian | numeral | 0 | 1 |
| synodal-russian | orthography | 1 | 2 |
| synodal-russian | participle | 1 | 1 |
| synodal-russian | pronoun | 0 | 1 |
| synodal-russian | verbal-noun | 0 | 1 |

## Non-final systems

- `ocs.determiner` — **implementation-missing**: Complete the lexical inventory and productive adjectival relation without treating exact tables as class rules.
- `ocs.numeral` — **implementation-missing**: Implement the source-backed numeral subtype inventory, agreement/government, compounds, and irregulars.
- `ocs.verb.irregular-root` — **implementation-missing**: Lock the irregular/root lexeme union, import independent LOVe principal parts, and complete or explicitly defect every paradigm.
- `ocs.verb.verbal-noun` — **source-review-open**: Resolve the inflection-versus-derivation boundary and implement only complete source-backed contracts.
- `ocs.orthography.accent` — **implementation-missing**: Design reusable OCS accent metadata, crosswalk lexical paradigms, and preserve exact/source/predictive provenance.
- `ocs.orthography.glagolitic` — **implementation-missing**: Establish a source-backed Glagolitic realization/transliteration contract for every productive rule.
- `ocs.lexicon.classification` — **implementation-missing**: Define the source-union merge, import lawful lexical metadata, and eliminate silent rejects/unclassified lexemes.
- `syn.noun.irregular` — **implementation-missing**: Complete irregular noun families and unify любовь identities without generating unattested alternant combinations.
- `syn.adjective.short-superlative` — **source-review-open**: Complete source discovery and either implement the category or finalize every cell as absent-from-recension with citations.
- `syn.pronoun` — **implementation-missing**: Complete all pronoun subtypes, productive relationships, clitics, lexical restrictions, and source-union identities.
- `syn.determiner` — **implementation-missing**: Finish velar and exceptional determiner families, then separate final productive and irregular matrix entries.
- `syn.numeral` — **implementation-missing**: Implement complete cardinal, collective, fractional, distributive, compound, government, and irregular contracts.
- `syn.verb.future` — **implementation-missing**: Determine the source-defined productive/simple-future classes and complete all irregular future inventories.
- `syn.verb.past-underspecified` — **source-review-open**: Audit every such row; reclassify it to aorist/imperfect/analytic where evidence permits or finalize a closed exact evidential category.
- `syn.verb.supine` — **source-review-open**: Finish source discovery and encode either productive-complete or absent-from-recension with citations.
- `syn.verb.verbal-noun` — **implementation-missing**: Review the target grammar's suffix/class contracts and implement the closed formations without expanding into unrestricted derivation.
- `syn.verb.irregular` — **implementation-missing**: Complete the irregular source-union inventory and explicitly classify every missing/defective system.
- `syn.analytic.wider` — **implementation-missing**: Crosswalk Alypy and target texts, then implement every licensed wider construction as a typed phrase.
- `syn.orthography.liturgical-accent-abbreviation` — **implementation-missing**: Classify every lexical accent/printed pattern in the source union and complete positional and abbreviation families without guessing.
- `syn.lexicon.classification` — **implementation-missing**: Define and exhaustively classify the locked source union; no candidate or rejected source row may remain silent.
- `cross.source-frontier` — **source-review-open**: Complete source crosswalks and at least two consecutive post-seed passes with no contract-changing eligible source.

## Complete matrix

| ID | Recension | Category | State | Rule IDs | Evidence |
|---|---|---|---|---|---|
| `ocs.noun.declension` | old-church-slavonic | noun | `productive-complete` | `N-O-M-HARD-01`<br>`N-O-N-HARD-01`<br>`N-JO-M-SOFT-01`<br>`N-JO-N-SOFT-01`<br>`N-A-HARD-01`<br>`N-JA-SOFT-01`<br>`N-I-F-01`<br>`N-I-M-01`<br>`N-U-M-01`<br>`N-N-M-01`<br>`N-N-N-01`<br>`N-NT-N-01`<br>`N-R-01`<br>`N-S-N-01`<br>`N-V-F-01`<br>`N-INDECL-01` | source-backed productive prediction; exact dictionary cells retain table provenance |
| `ocs.adjective.positive` | old-church-slavonic | adjective | `productive-complete` | `ADJ-HARD-SHORT-01`<br>`ADJ-HARD-LONG-01`<br>`ADJ-SOFT-SHORT-01`<br>`ADJ-SOFT-LONG-01` | source-backed productive prediction |
| `ocs.adjective.comparison` | old-church-slavonic | comparison | `productive-complete` | `ADJ-COMP-NEW-01`<br>`ADJ-COMP-OLD-01` | independent grammar paradigms plus strict explicit-principal-part prediction; dictionary citations remain exact |
| `ocs.adjective.superlative` | old-church-slavonic | comparison | `productive-complete` | `PHRASE-SUP-REL-GEN-01`<br>`PHRASE-SUP-ZELO-01`<br>`ADJ-SUP-PRE-01`<br>`ADJ-COMP-OLD-01` | independent grammar inventory with structured component provenance and productive derived realization |
| `ocs.pronoun.personal-reflexive` | old-church-slavonic | pronoun | `productive-complete` | `PRON-PERS-1-01`<br>`PRON-PERS-2-01`<br>`PRON-REFL-01`<br>`PRON-PERS-CLITIC-01`<br>`PRON-ANAPH-3-01`<br>`PRON-ANAPH-PREP-N-01` | independently reviewed closed grammar inventory, conditioned allomorphy, explicit source dispute, and exhaustive source-union routing |
| `ocs.pronoun.other` | old-church-slavonic | pronoun | `productive-complete` | `PRON-2P-HARD-01`<br>`PRON-2P-SOFT-01`<br>`PRON-2P-J-01`<br>`PRON-2P-VELAR-01`<br>`PRON-REL-IZHE-01`<br>`PRON-REL-PREP-N-01`<br>`PRON-2PSTAR-VES-01`<br>`PRON-2PSTAR-SIC-01`<br>`PRON-UNIQUE-SI-01`<br>`PRON-UNIQUE-KTO-01`<br>`PRON-UNIQUE-CHTO-01`<br>`PRON-DERIVED-FAMILY-01`<br>`DET-UNIQUE-KYI-01` | all 34 class 2/p identities allocated with exhaustive typed regular or exceptional inventories, source-union aliases, and structured derived-particle composition |
| `ocs.determiner` | old-church-slavonic | determiner | `implementation-missing` | `OCS-DETERMINER-PENDING` | partial exact dictionary evidence |
| `ocs.numeral` | old-church-slavonic | numeral | `implementation-missing` | `OCS-NUMERAL-PENDING` | partial exact dictionary evidence |
| `ocs.verb.present` | old-church-slavonic | finite-verb | `productive-complete` | `V-IA1-01`<br>`V-IA2-01`<br>`V-II1-01`<br>`V-II2-01`<br>`V-II3-01` | source-backed productive prediction |
| `ocs.verb.imperfect.uncontracted` | old-church-slavonic | finite-verb | `productive-complete` | `V-IMPF-A-01`<br>`V-IMPF-YAT-A-01`<br>`V-IMPF-PAL-A-01`<br>`V-IMPF-PRESENT-01` | source-backed productive uncontracted prediction |
| `ocs.verb.imperfect.contracted` | old-church-slavonic | finite-verb | `productive-complete` | `V-IMPF-CONTRACTED-A-01`<br>`V-IMPF-CONTRACTED-YAT-A-01`<br>`V-IMPF-CONTRACTED-PAL-A-01`<br>`V-IMPF-PRESENT-CONTRACTED-01` | complete grammar terminal table, multi-manuscript examples, and explicit source-order metadata |
| `ocs.verb.imperfect.iotated` | old-church-slavonic | finite-verb | `productive-complete` | `V-IMPF-IOTATED-01` | complete grammar terminal table and independently cited examples |
| `ocs.verb.aorist.asigmatic-new` | old-church-slavonic | finite-verb | `productive-complete` | `V-AOR-ASIG-01`<br>`V-AOR-NEW-01` | source-backed productive prediction |
| `ocs.verb.aorist.sigmatic` | old-church-slavonic | finite-verb | `productive-complete` | `V-AOR-SIG-PRIMARY-01`<br>`V-AOR-SIG-SECONDARY-01`<br>`V-AOR-SIG-VOWEL-01` | independent grammar paradigms plus source-backed principal-part contract |
| `ocs.verb.imperative.synthetic` | old-church-slavonic | imperative | `productive-complete` | `V-IMP-01` | source-backed productive prediction |
| `ocs.verb.imperative.analytic` | old-church-slavonic | imperative | `productive-complete` | `PHRASE-IMPV-DA-01` | independent grammar rule with structured dictionary-resolved present component provenance |
| `ocs.verb.infinitive-supine-lparticiple` | old-church-slavonic | nonfinite-verb | `productive-complete` | `V-INF-01`<br>`V-SUP-01`<br>`V-LPART-01` | source-backed productive prediction |
| `ocs.verb.participle` | old-church-slavonic | participle | `productive-complete` | `V-PTCP-PRES-ACT-01`<br>`V-PTCP-PRES-PASS-01`<br>`V-PTCP-PAST-ACT-01`<br>`V-PTCP-PAST-PASS-01` | source-backed productive prediction; unsafe extracted rows excluded |
| `ocs.verb.irregular-root` | old-church-slavonic | irregular-verb | `implementation-missing` | `OCS-VERB-IRREGULAR-INVENTORY-PENDING` | partial exact tables and one curated imperfect family |
| `ocs.verb.analytic` | old-church-slavonic | analytic-form | `productive-complete` | `V-COP-ES-PRES-01`<br>`V-COP-BUD-PRES-01`<br>`V-COP-BE-IMPF-01`<br>`V-COP-BE-AOR-01`<br>`V-COP-BI-COND-01`<br>`V-COP-BY-COND-AOR-01`<br>`PHRASE-PERFECT-01`<br>`PHRASE-PLUPERFECT-01`<br>`PHRASE-FUT-INF-01`<br>`PHRASE-FUT-PTCP-01`<br>`PHRASE-FUT-PERFECT-01`<br>`PHRASE-COND-OPT-01`<br>`PHRASE-COND-OPT-DA-01`<br>`PHRASE-COND-OPT-ELLIP-01`<br>`PHRASE-COND-OPT-PASS-01`<br>`PHRASE-PASSIVE-01` | independently crosschecked grammar inventory with reviewed tables, explicit reconstructions, curated irregular auxiliaries, and structured word-level provenance |
| `ocs.verb.verbal-noun` | old-church-slavonic | verbal-noun | `source-review-open` | `OCS-VERBAL-NOUN-PENDING` | sparse exact dictionary cells |
| `ocs.orthography.canonical` | old-church-slavonic | orthography | `productive-complete` | `OCS-ORTHOGRAPHY-CANONICAL` | technical and source contract |
| `ocs.orthography.accent` | old-church-slavonic | accent | `implementation-missing` | `OCS-ACCENT-PENDING` | exact source spellings only |
| `ocs.orthography.glagolitic` | old-church-slavonic | orthography | `implementation-missing` | `OCS-GLAGOLITIC-PENDING` | partial exact dictionary evidence |
| `ocs.lexicon.classification` | old-church-slavonic | lexicon | `implementation-missing` | `OCS-LEXICON-UNION-PENDING` | single locked dictionary lineage plus partial external evaluation |
| `ocs.invariant-parts-of-speech` | old-church-slavonic | invariant | `not-inflectional` | `OCS-NOT-INFLECTIONAL` | lexical exact forms |
| `syn.noun.declension` | synodal-russian | noun | `productive-complete` | `SYN-NOUN-I-HARD-M-ALYPY-34`<br>`SYN-NOUN-I-HARD-VELAR-M-ALYPY-34`<br>`SYN-NOUN-I-MIXED-M-ALYPY-33-34`<br>`SYN-NOUN-I-HARD-N-ALYPY-34`<br>`SYN-NOUN-I-SOFT-M-ALYPY-34`<br>`SYN-NOUN-I-SOFT-N-ALYPY-34`<br>`SYN-NOUN-II-HARD-ALYPY-39`<br>`SYN-NOUN-II-SOFT-ALYPY-39`<br>`SYN-NOUN-III-F-ALYPY-41`<br>`SYN-NOUN-III-M-ALYPY-41`<br>`SYN-NOUN-IV-N-EN-ALYPY-42-43`<br>`SYN-NOUN-IV-N-ES-ALYPY-42-43`<br>`SYN-NOUN-IV-N-AT-ALYPY-42-43`<br>`SYN-NOUN-IV-F-ER-ALYPY-42-43`<br>`SYN-NOUN-IV-F-OV-ALYPY-42-44`<br>`SYN-NOUN-IV-M-EN-ALYPY-42-44`<br>`SYN-NOUN-IV-M-EN-KAMEN-ALYPY-43` | normative productive prediction |
| `syn.noun.irregular` | synodal-russian | noun | `implementation-missing` | `SYN-NOUN-IRREGULAR-INVENTORY-PENDING` | partial reviewed exact/override data |
| `syn.adjective.positive-comparison` | synodal-russian | adjective | `productive-complete` | `SYN-ADJ-SHORT-HARD-ALYPY-53`<br>`SYN-ADJ-SHORT-SOFT-ALYPY-53`<br>`SYN-ADJ-LONG-HARD-ALYPY-57`<br>`SYN-ADJ-LONG-SOFT-ALYPY-57`<br>`SYN-ADJ-COMPARATIVE-LONG-ALYPY-58`<br>`SYN-ADJ-SUPERLATIVE-LONG-ALYPY-59`<br>`SYN-ADJ-COMPARATIVE-SHORT-ALYPY-58-98` | normative productive prediction |
| `syn.adjective.short-superlative` | synodal-russian | comparison | `source-review-open` | `SYN-ADJ-SHORT-SUPERLATIVE-REVIEW` | negative grammar evidence under review |
| `syn.pronoun` | synodal-russian | pronoun | `implementation-missing` | `SYN-PRONOUN-COMPLETE-PENDING` | broad but incomplete exact closed-class coverage |
| `syn.determiner` | synodal-russian | determiner | `implementation-missing` | `SYN-DETERMINER-ADJECTIVAL`<br>`SYN-DETERMINER-IRREGULAR-PENDING` | productive regular background plus partial exact irregulars |
| `syn.numeral` | synodal-russian | numeral | `implementation-missing` | `SYN-NUMERAL-ORDINAL-ADJECTIVAL`<br>`SYN-NUMERAL-COMPLETE-PENDING` | productive ordinal plus partial exact cardinal/collective data |
| `syn.verb.present` | synodal-russian | finite-verb | `productive-complete` | `SYN-VERB-PRESENT-ALYPY-80` | normative productive prediction |
| `syn.verb.aorist` | synodal-russian | finite-verb | `productive-complete` | `SYN-VERB-AORIST-VOWEL-ALYPY-86`<br>`SYN-VERB-AORIST-CONSONANT-ALYPY-86` | normative productive prediction |
| `syn.verb.imperfect` | synodal-russian | finite-verb | `productive-complete` | `SYN-VERB-IMPERFECT-H-ALYPY-87`<br>`SYN-VERB-IMPERFECT-YAH-ALYPY-87`<br>`SYN-VERB-IMPERFECT-AH-ALYPY-87` | normative productive prediction |
| `syn.verb.future` | synodal-russian | finite-verb | `implementation-missing` | `SYN-VERB-FUTURE-EXACT`<br>`SYN-VERB-FUTURE-PRODUCTIVE-PENDING` | one complete exact irregular table and sparse exact cells |
| `syn.verb.past-underspecified` | synodal-russian | finite-verb | `source-review-open` | `SYN-VERB-PAST-EXACT` | underspecified exact source evidence |
| `syn.verb.imperative-infinitive-lparticiple` | synodal-russian | nonfinite-and-mood | `productive-complete` | `SYN-VERB-IMPERATIVE-ALYPY-93`<br>`SYN-VERB-INFINITIVE-LEXICAL`<br>`SYN-VERB-LPART-ALYPY-97` | normative productive prediction |
| `syn.verb.supine` | synodal-russian | nonfinite-verb | `source-review-open` | `SYN-VERB-SUPINE-REVIEW` | open normative source review |
| `syn.verb.participle` | synodal-russian | participle | `productive-complete` | `SYN-VERB-PARTICIPLE-PRESENT-ACTIVE-ALYPY-95`<br>`SYN-VERB-PARTICIPLE-PAST-ACTIVE-ALYPY-96`<br>`SYN-VERB-PARTICIPLE-PRESENT-PASSIVE-ALYPY-99`<br>`SYN-VERB-PARTICIPLE-PAST-PASSIVE-ALYPY-100`<br>`SYN-VERB-PARTICIPLE-PRESENT-ACTIVE-SHORT-ALYPY-95-98`<br>`SYN-VERB-PARTICIPLE-PAST-ACTIVE-SHORT-ALYPY-96-98` | normative productive prediction |
| `syn.verb.verbal-noun` | synodal-russian | verbal-noun | `implementation-missing` | `SYN-VERB-VERBAL-NOUN-PENDING` | typed metadata seam without productive realization |
| `syn.verb.irregular` | synodal-russian | irregular-verb | `implementation-missing` | `SYN-REGISTRY-IRREGULAR-OVERRIDE`<br>`SYN-VERB-IRREGULAR-COMPLETE-PENDING` | partial exact irregular and productive principal-part data |
| `syn.analytic.current` | synodal-russian | analytic-form | `productive-complete` | `SYN-PHRASE-FUTURE-ALYPY-85`<br>`SYN-PHRASE-PERFECT-ALYPY-88`<br>`SYN-PHRASE-PLUPERFECT-ALYPY-89`<br>`SYN-PHRASE-CONDITIONAL-ALYPY-91`<br>`SYN-PHRASE-PASSIVE-ALYPY-101-102` | normative structured prediction |
| `syn.analytic.wider` | synodal-russian | analytic-form | `implementation-missing` | `SYN-PHRASE-WIDER-PENDING` | generic typed token support but no complete construction inventory |
| `syn.orthography.canonical` | synodal-russian | orthography | `productive-complete` | `SYN-ORTHOGRAPHY-EXPANDED`<br>`SYN-COLLATION-UTN41-R1`<br>`SYN-NUMERAL-NOTATION-ALYPY-5` | normative technical and grammatical contract |
| `syn.orthography.liturgical-accent-abbreviation` | synodal-russian | orthography | `implementation-missing` | `synodal-accent:mudr-fixed-stem`<br>`synodal-accent:mati-fixed-stem`<br>`synodal-accent:imya-mobile`<br>`synodal-accent:nebo-mobile`<br>`SYN-LITURGICAL-COMPLETE-PENDING` | small reviewed productive/exact subset |
| `syn.lexicon.classification` | synodal-russian | lexicon | `implementation-missing` | `SYN-LEXICON-UNION-PENDING` | 833 reviewed identities plus large unreviewed candidate queues |
| `syn.invariant-parts-of-speech` | synodal-russian | invariant | `not-inflectional` | `SYN-NOT-INFLECTIONAL` | reviewed lexical exact forms |
| `cross.open-lexicon-provider` | cross-recension | api | `productive-complete` | `OCS-ADVANCED-RULES`<br>`SYN-LEXICON-PROVIDER-V10`<br>`SYN-BATCH-V10` | architecture/API contract |
| `cross.provenance-and-prediction` | cross-recension | provenance | `productive-complete` | `CROSS-PROVENANCE-CONTRACT` | architecture invariant |
| `cross.source-frontier` | cross-recension | research | `source-review-open` | `SOURCE-FRONTIER-CONVERGENCE` | seed plus multilingual discovery pass 1 |
| `cross.corpus-and-heldout-evaluation` | cross-recension | evaluation | `productive-complete` | `CROSS-EVALUATION-CONTRACT` | method complete for current rules; validation set must grow with each new rule |

## Source frontier

The frontier contains **15** source/lineage records. Authority policy: target-recension normative grammars and critical editions; manuscript-grounded dictionaries; independent target texts; manually annotated corpora; explicitly labeled inherited/comparative evidence; automatic, crowd-edited, OCR, and generated evidence only as candidates or evaluation

| ID | Tier | Type | Recension | Access | Impact |
|---|---:|---|---|---|---|
| `ut-ocs-online` | 1 | teaching grammar with primary-text lessons | old-church-slavonic | accessible-metadata | Defines the current OCS grammatical inventory and productive generalizations; its pronoun tables independently enumerate first and second person, the numberless reflexive, marked clitics, the defective gendered third-person anaphoric with prepositional n- forms, and relative иже formed with же. It also crosschecks old/new comparator formation, identifies analytic rather than universally synthetic superlative strategies, licenses да plus present commands, and supplies the complete analytic tense, mood, future, and passive construction inventory. |
| `polivanova-fup-2023` | 1 | scholarly grammar and grammatical dictionaries | old-church-slavonic | pinned-local | Adds a completeness-oriented independent grammar and grammatical dictionaries; §§287–320 and 375–380 define the regular pronominal terminal system, twofold/velar behavior, derived families, and exceptional closed paradigms. §§318 and 381–382 define the four intrinsic personal/reflexive/anaphoric identities, their defective dimensions, grammar-table clitics, and conditioned prepositional allomorphs. Its comparative chapters establish the two principal parts, three syncopated cell families, four alien endings, productive new formation, and closed old inventory. Its aorist inventory established the independent main/singular subbundles and the distinct standard vowel, old s-, and old x-sigmatic contracts. Its imperfect terminal table and manuscript discussion establish explicit uncontracted, contracted, iotated, and present-platform analyses without treating source frequency as a global default. Its unique-verb profiles independently establish the suppletive copular series and the complete имѣти/хотѣти auxiliary paradigms used by analytic constructions. |
| `lunt-ocs-grammar-2001` | 1 | scholarly reference grammar | old-church-slavonic | accessible-metadata | Contents independently locates twofold and compound comparative declension and the -ьj/-ěj formation split; full-text comparison remains open and no rule claim relies on inaccessible content. |
| `love-lmu-ocs-verbs` | 2 | scholarly lexical database | old-church-slavonic | accessible-metadata | Newly admitted in discovery pass 1; supplies independent lexeme-level allomorph evidence for sigmatic aorists and irregular/root verbs. |
| `punco-lmu-reference-grammar` | 2 | scholarly digital reference grammar and annotated-corpus documentation | old-church-slavonic, mixed | accessible-metadata | Independently crosschecks the regular hard and soft pronominal class, the personal/reflexive inventory, and corpus-annotation distinctions, with mixed-period forms admitted only when separately licensed by OCS authorities. It also confirms да plus present forms for commands in all persons and crosschecks the OCS-to-Church-Slavonic analytic inventory: perfect/pluperfect, infinitival future and future-in-the-past, future perfect, conditional/aorist oscillation, optative particles, auxiliary omission, and passive participial constructions. |
| `english-wiktionary-ocs-kaikki-2026-08-07` | 6 | crowd-edited machine-readable dictionary | old-church-slavonic | pinned-local | Current OCS exact registry and extraction target. |
| `ud-ocs-proiel-r2.18` | 4 | manually based annotated corpus | old-church-slavonic | pinned-local | Independent real-text regression evidence. |
| `syntacticus-20230428` | 4 | native annotated corpus | old-church-slavonic | pinned-local | Independent evaluator input sharing one lineage with UD, not a second confirmation. |
| `ccmh-2021-04-23` | 3 | primary-text corpus | old-church-slavonic | pinned-local | Potential independent OCS held-out evidence. |
| `alypy-gamanovich-grammar-web-2023` | 1 | normative grammar | synodal-russian | pinned-local | Defines all currently admitted Synodal productive rules and the open supine, verbal-noun, comparison, closed-class, accent, and analytic reviews. |
| `unicode-tn41-revision-1` | 1 | technical standard | mixed | pinned-local | Defines safe stored text, validation, mark ordering, numeral notation, and collation. |
| `russian-national-corpus-church-slavonic` | 4 | large annotated corpus | synodal-russian, mixed | accessible-metadata | 5,364,905-word, 1,447-text held-out discovery/evaluation frontier; never a sole grammar oracle. |
| `gorazd` | 2 | scholarly digital dictionary/card index | old-church-slavonic | accessible-metadata | Manual lexeme and variant discovery frontier. |
| `dyachenko-1900-scan` | 6 | historical dictionary scan | mixed | pinned-local | Candidate source for irregular lexemes and meanings, not direct productive authority. |
| `synodal-target-text-union` | 3 | primary target-text union | synodal-russian | pinned-local | Locked exact-form and held-out evaluation evidence. |

## Discovery passes

- `seed-and-multilingual-pass-1` (2026-08-14) — new sources: lunt-ocs-grammar-2001, love-lmu-ocs-verbs; changed inventory/contracts/conflicts/validation: `true/true/true/true`. The pass added Lunt as an independent comprehensive grammar and LOVe as a 970-lemma principal-part/aspect source. Both require complete crosswalks, so convergence is not established.

## Next checkpoint

`ocs.determiner`: Complete the lexical inventory and productive adjectival relation without treating exact tables as class rules.
