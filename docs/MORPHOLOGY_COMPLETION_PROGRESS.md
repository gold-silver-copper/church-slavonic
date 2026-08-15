# Church Slavonic morphology completion progress

This report is generated from `data/morphology/completion.toml` by `cargo xtask morphology-completeness`. It is a progress inventory, not a claim of completion.

Inventory date: `2026-08-15`. Source-frontier date: `2026-08-15`.

## Headline

The matrix contains **53** required system contracts: **34** have final states and **19** remain non-final. Source discovery has 3 recorded passes and has not converged.

## State totals

| State | Systems |
|---|---:|
| `implementation-missing` | 13 |
| `not-inflectional` | 2 |
| `partial` | 1 |
| `productive-complete` | 32 |
| `source-review-open` | 5 |

## Recension totals

| Recension | Final | Total |
|---|---:|---:|
| cross-recension | 3 | 4 |
| old-church-slavonic | 21 | 26 |
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
| old-church-slavonic | determiner | 1 | 1 |
| old-church-slavonic | finite-verb | 6 | 6 |
| old-church-slavonic | imperative | 2 | 2 |
| old-church-slavonic | invariant | 1 | 1 |
| old-church-slavonic | irregular-verb | 0 | 1 |
| old-church-slavonic | lexicon | 0 | 1 |
| old-church-slavonic | nonfinite-verb | 1 | 1 |
| old-church-slavonic | noun | 1 | 1 |
| old-church-slavonic | numeral | 1 | 1 |
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

- `ocs.verb.irregular-root` — **partial**: Expand the seventy irregular-workstem anchors over their member and prefix allomorphs, crosswalk impersonal verbs, and attach per-cell facade provenance without collapsing disputed analyses or independent ѥсмь and бꙑти profiles.
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
| `ocs.adjective.positive` | old-church-slavonic | adjective | `productive-complete` | `ADJ-HARD-SHORT-01`<br>`ADJ-HARD-LONG-01`<br>`ADJ-SOFT-SHORT-01`<br>`ADJ-SOFT-LONG-01` | source-backed productive prediction with exhaustive reviewed lexical defectivity |
| `ocs.adjective.comparison` | old-church-slavonic | comparison | `productive-complete` | `ADJ-COMP-NEW-01`<br>`ADJ-COMP-OLD-01` | independent grammar paradigms plus strict explicit-principal-part prediction; dictionary citations remain exact |
| `ocs.adjective.superlative` | old-church-slavonic | comparison | `productive-complete` | `PHRASE-SUP-REL-GEN-01`<br>`PHRASE-SUP-ZELO-01`<br>`ADJ-SUP-PRE-01`<br>`ADJ-COMP-OLD-01` | independent grammar inventory with structured component provenance and productive derived realization |
| `ocs.pronoun.personal-reflexive` | old-church-slavonic | pronoun | `productive-complete` | `PRON-PERS-1-01`<br>`PRON-PERS-2-01`<br>`PRON-REFL-01`<br>`PRON-PERS-CLITIC-01`<br>`PRON-ANAPH-3-01`<br>`PRON-ANAPH-PREP-N-01` | independently reviewed closed grammar inventory, conditioned allomorphy, explicit source dispute, and exhaustive source-union routing |
| `ocs.pronoun.other` | old-church-slavonic | pronoun | `productive-complete` | `PRON-2P-HARD-01`<br>`PRON-2P-SOFT-01`<br>`PRON-2P-J-01`<br>`PRON-2P-VELAR-01`<br>`PRON-REL-IZHE-01`<br>`PRON-REL-PREP-N-01`<br>`PRON-2PSTAR-VES-01`<br>`PRON-2PSTAR-SIC-01`<br>`PRON-UNIQUE-SI-01`<br>`PRON-UNIQUE-KTO-01`<br>`PRON-UNIQUE-CHTO-01`<br>`PRON-DERIVED-FAMILY-01`<br>`DET-UNIQUE-KYI-01` | all 34 class 2/p identities allocated with exhaustive typed regular or exceptional inventories, source-union aliases, and structured derived-particle composition |
| `ocs.determiner` | old-church-slavonic | determiner | `productive-complete` | `PRON-2P-HARD-01`<br>`PRON-2P-SOFT-01`<br>`PRON-2P-J-01`<br>`PRON-2P-VELAR-01`<br>`DET-UNIQUE-KYI-01`<br>`ADJ-HARD-SHORT-01`<br>`ADJ-HARD-LONG-01` | source-exhaustive reviewed lexical allocation over shared productive rules and one closed irregular paradigm |
| `ocs.numeral` | old-church-slavonic | numeral | `productive-complete` | `PRON-2P-HARD-01`<br>`PRON-2P-SOFT-01`<br>`PRON-2P-J-01`<br>`N-I-F-01`<br>`N-O-N-HARD-01`<br>`N-JA-SOFT-01`<br>`N-A-HARD-01`<br>`N-U-M-01`<br>`NUM-CARD-THREE-01`<br>`NUM-CARD-FOUR-01`<br>`NUM-CARD-TEN-01`<br>`NUM-CARD-TEEN-01`<br>`NUM-CARD-TENS-01`<br>`NUM-CARD-HUNDRED-01`<br>`NUM-CARD-THOUSAND-01`<br>`NUM-CARD-MYRIAD-01`<br>`NUM-CARD-ADDITIVE-01`<br>`NUM-CARD-DISTRIBUTIVE-01`<br>`NUM-ORD-HARD-01`<br>`NUM-ORD-J-01`<br>`NUM-ORD-TEEN-01`<br>`NUM-ORD-DECADE-01`<br>`NUM-ORD-HUNDRED-01`<br>`NUM-ORD-THOUSAND-01`<br>`NUM-ORD-ADDITIVE-01`<br>`NUM-ORD-CIRCUMLOCUTIVE-01`<br>`NUM-COLL-PRON-01`<br>`NUM-COLL-ADJ-01`<br>`NUM-FRAC-NOUN-01`<br>`NUM-INDEF-NOUN-01`<br>`NUM-SCOPE-BOUNDARY-01` | source-reviewed complete OCS numeral morphology: cardinals and distributives through the source-defined exact 10,000 inventory, simple and compound ordinals through the independently specified thousandth boundary, both disputed asyndetic accounts and both rare 21–29 patterns, the inherited collective series, the four-identity fractional-noun inventory, and the indefinite-quantity noun несъвѣда, with typed agreement/government, complete licensed cell products, correlated multiword analyses, direct/reconstructed/disputed/corpus/primary-text variants, explicit attested-versus-productive evidence, and derived items routed to their owning parts of speech |
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
| `ocs.verb.irregular-root` | old-church-slavonic | irregular-verb | `partial` | `V-IRREG-EXACT-01`<br>`V-IMPF-PRESENT-01`<br>`V-PTCP-PRES-ACT-01`<br>`V-PTCP-PRES-PASS-01` | closed nineteen-profile and 106-member unique-family union plus thirteen complete reusable group representatives and seventy source anchors; group members, impersonals, and facade partial |
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

The frontier contains **24** source/lineage records. Authority policy: target-recension normative grammars and critical editions; manuscript-grounded dictionaries; independent target texts; manually annotated corpora; explicitly labeled inherited/comparative evidence; automatic, crowd-edited, OCR, and generated evidence only as candidates or evaluation

| ID | Tier | Type | Recension | Access | Impact |
|---|---:|---|---|---|---|
| `ut-ocs-online` | 1 | teaching grammar with primary-text lessons | old-church-slavonic | accessible-metadata | Defines the current OCS grammatical inventory and productive generalizations; its pronoun tables independently enumerate first and second person, the numberless reflexive, marked clitics, the defective gendered third-person anaphoric with prepositional n- forms, and relative иже formed with же. Its pronominal and interrogative-adjective chapters independently crosscheck the shared hard/soft/j-stem terminals and the exceptional кꙑи/чии profiles used by the determiner facade. Its numeral lesson defines singular agreement for one, dual agreement for two/both, plural agreement for three/four, genitive-plural government for five and higher simple cardinals, the component-inflection patterns of teens and multiplicative tens/hundreds/thousands, the hundred and thousand magnitude heads, the two 10,000 realizations, and additive и between nonzero decimal chunks. It also crosschecks old/new comparator formation, identifies analytic rather than universally synthetic superlative strategies, licenses да plus present commands, and supplies the complete analytic tense, mood, future, and passive construction inventory. |
| `polivanova-fup-2023` | 1 | scholarly grammar and grammatical dictionaries | old-church-slavonic | pinned-local | Adds a completeness-oriented independent grammar and grammatical dictionaries; §§285 and 303–305 define the exhaustive long-only adjective inventory, while §§287–320, 375–380, and Paradigmatic Dictionary entry 343 define the regular, exceptional, and adjectival profiles allocated to the determiner and pronoun facades. §§70, 72, 285, 299, and 303–306 plus the OSD spreadsheet define the exact ten-member simple-ordinal lexicon, its nine hard profiles, the трет.ьj boundary, and productive short/long agreement. The OSD also directly allocates дъвои, обои, and трои to j-pronominal class 2/p and четворъ, седморъ, and десѧторъ to hard adjectival class 2/a, establishing both inflectional halves of the collective system, and independently lists the fractional noun lexemes полъ and десѧтина. §§321–322, 345–351, 373–374, and 383–384 independently define the unique three/four tables, both thousand identities over the productive feminine ja-stem profile, the feminine i-stem background of five through nine, and the attested mixed paradigm and variants of ten. §§318 and 381–382 define the four intrinsic personal/reflexive/anaphoric identities, their defective dimensions, grammar-table clitics, and conditioned prepositional allomorphs. Its comparative chapters establish the two principal parts, three syncopated cell families, four alien endings, productive new formation, and closed old inventory. Its aorist inventory established the independent main/singular subbundles and the distinct standard vowel, old s-, and old x-sigmatic contracts. Its imperfect terminal table and manuscript discussion establish explicit uncontracted, contracted, iotated, and present-platform analyses without treating source frequency as a global default. Its unique-verb profiles independently establish the suppletive copular series and the complete имѣти/хотѣти auxiliary paradigms used by analytic constructions. |
| `gorshkov-ocs-2002` | 2 | teaching grammar | old-church-slavonic | accessible-full-text | Independently lists полъ ‘half’ among the six secure masculine u-stem nouns and supplies that class's full singular, dual, and plural declension. It also states that hundredth and thousandth take -ьнъ, ordinals admit short and long adjective forms, teens use an inflected simple ordinal before invariant на десѧте, decades may use десѧтъ or десѧтьнъ, and compound ordinals follow cardinal composition principles. Its enumerated ordinal inventory ends at thousandth and supplies no stem or component-inflection rule for a higher magnitude. Section 115 calls both тъма and несъвѣда 10,000 but declines only тъма; the lexical monograph and manuscript studies below control the resulting semantic conflict. |
| `leuta-havryliuk-ocs-2018` | 2 | university teaching grammar and primary-text exercise collection | old-church-slavonic | accessible-full-text | Supplies the reviewed fused teen, decade, hundred, and thousand stem inventory, documents thematic -о- variants, licenses additive и, ти, and zero-connector constructions, names manuscript examples for analytic/fused teens and 20th, 70th, 100th, 500th, and 1000th, and identifies two rare alternative 21–29 constructions. Its ordinal inventory terminates at the thousandth stem and gives no higher-magnitude formation. Its four semantic numeral classes provide no independent synthetic distributive series, while по дьвѣма ли тремъ мѣрамъ supplies the analytic по + dative pattern and the temporal по дъвою дьнию locative supplies its case/meaning boundary. It closes the OCS fractional inventory as the noun-based полъ, половина, четврьть, and десѧтина system, assigns them respectively to u-, a-, i-, and a-stem declension, and places третина and compound fractions such as полътора in later Church Slavonic. |
| `elkina-ocs-1960` | 2 | historical-language textbook | old-church-slavonic | accessible-full-text | Independently confirms the noun-like magnitude heads, cardinal composition, short and long ordinal agreement, additive ordinal connectors, and collective paradigms. Its OCS ordinal account and examples end with the thousandth head and do not specify how a multiplier combines morphologically with that head, so it corroborates a source-bounded stop at 1,000 rather than a later Russian extrapolation. |
| `suprun-slavic-numerals-1969` | 2 | scholarly historical monograph | old-church-slavonic, inherited-comparative | accessible-full-text | Confirms that ordinal words inflect as adjectives, fractional quarter/tenth words as nouns, and apparent additional numeral series such as двоякий, дважды, and сам-третей types belong morphologically to adjectives or adverbs. This closes the supposed residual derived-numeral inventory by routing each formation to its owning part of speech rather than inventing another numeral paradigm. |
| `lvov-ocs-lexicon-1966` | 1 | scholarly manuscript-grounded lexical monograph | old-church-slavonic | accessible-full-text | Distinguishes exact тъма ‘ten thousand’ or ‘thousands’ from несъвѣда ‘incalculable quantity’, and directly cites несъвѣдами in both the Suprasliensis and John the Exarch. This licenses the complete inherited hard a-stem noun paradigm while keeping the lexeme outside exact integer composition. |
| `simonov-nesveda-2006` | 2 | peer-reviewed historical-semantic article | old-russian-comparative | accessible-full-text | Shows that later exact-value readings do not fit Кирик's notation: the word marks million and ten-million ranks beyond the contemporary abacus rather than a stable power of ten. It corroborates exclusion from exact OCS cardinal composition without supplying OCS paradigm cells. |
| `pronin-large-numerals-2024` | 2 | peer-reviewed historical-semantic synthesis | old-church-slavonic, old-russian-comparative, later-russian-comparative | accessible-full-text | Confirms exact 10,000 for тьма in the small/church number system while documenting other chronological values; confirms that Кирик's несъведа replaces a stable magnitude only for million-scale out-of-range ranks. This prevents later numeral systems from contaminating the OCS integer API. |
| `lunt-ocs-grammar-2001` | 1 | scholarly reference grammar | old-church-slavonic | accessible-metadata | Contents independently locates twofold and compound comparative declension and the -ьj/-ěj formation split, and locates the book's numeral declension, syntax, and historical sections for the open crosswalk. The available USC endpoint exposes only the front matter and contents, so no numeral rule claim relies on inaccessible body text. |
| `krysko-collective-2020` | 1 | peer-reviewed manuscript-grounded historical grammar article | old-church-slavonic, mixed | accessible-full-text | Independently confirms that the low collective numerical pronouns are not singularia or pluralia tantum: дъвои and обои have singular, dual, and plural uses, including attributive and substantivized functions. |
| `essja-collective-series` | 2 | academy historical-etymological dictionary | old-church-slavonic, inherited-comparative | accessible-full-text | Establishes the inherited adjective series from four through ten, its parallel -er-/-or- stems, its upper bound at ten, and which spellings have direct Old Church Slavonic citation rather than reconstruction alone. |
| `love-lmu-ocs-verbs` | 2 | scholarly lexical database | old-church-slavonic | accessible-full-text | Supplies independent lemma-level present, aorist, imperative, aspect, and prefix-family principal parts. The official export crosschecks the reconstructable workstems of the closed unique-verb profiles, including comparative plěv-/plě-/plěvi for sparse plěti; it does not by itself turn predicted cells into attestations. |
| `punco-lmu-reference-grammar` | 2 | scholarly digital reference grammar and annotated-corpus documentation | old-church-slavonic, mixed | accessible-metadata | Independently crosschecks the regular hard and soft pronominal class, the personal/reflexive inventory, and corpus-annotation distinctions, with mixed-period forms admitted only when separately licensed by OCS authorities. Its numeral overview independently confirms the substantival treatment and genitive-plural government of five through ten and the corresponding higher constructions. It also confirms да plus present forms for commands in all persons and crosschecks the OCS-to-Church-Slavonic analytic inventory: perfect/pluperfect, infinitival future and future-in-the-past, future perfect, conditional/aorist oscillation, optative particles, auxiliary omission, and passive participial constructions. |
| `english-wiktionary-ocs-kaikki-2026-08-07` | 6 | crowd-edited machine-readable dictionary | old-church-slavonic | pinned-local | Current OCS exact registry and extraction target. |
| `ud-ocs-proiel-r2.18` | 4 | manually based annotated corpus | old-church-slavonic | pinned-local | Independent real-text regression evidence; the simple-ordinal audit confirms short forms throughout the ten-member lexicon and preserves cell-specific третии spellings as corpus evidence without promoting corpus sparsity into a productive rule. Compound-ordinal examples directly exercise analytic teens, fused 19th/20th/70th/100th/1000th heads, conjunction и in 28th and 79th, and zero-connector agreement in 104th. The collective audit independently exercises singular and plural low collectives, short adjectival четворъ/десѧторъ cells, and derivative adverbs; the spelling десꙙторо remains a cell-specific corpus observation. Fractional полъ and tithe десѧтинѫ provide exact accusative-singular semantic crosschecks without licensing sibling cells. Repeated по ѥдиномоу, по дьвѣма, по пѧти десѧтъ, по сътоу, and composed-tens witnesses directly establish distributive по selecting dative cardinal components; they license the construction while larger unattested values remain productive predictions. |
| `syntacticus-20230428` | 4 | native annotated corpus | old-church-slavonic | pinned-local | Lossless morphology and token-order crosscheck for compound ordinals, the fractional полъ and десѧтинѫ cells, and distributive по immediately governing dative cardinal components; it shares one PROIEL/TOROT witness lineage with UD and therefore is not counted as a second confirmation. |
| `ccmh-2021-04-23` | 3 | primary-text corpus | old-church-slavonic | pinned-local | Potential independent OCS held-out evidence; the current literal spelling scan found no usable compound-ordinal match and corpus absence was not treated as a grammatical counterexample. |
| `alypy-gamanovich-grammar-web-2023` | 1 | normative grammar | synodal-russian | pinned-local | Defines all currently admitted Synodal productive rules and the open supine, verbal-noun, comparison, closed-class, accent, and analytic reviews. |
| `unicode-tn41-revision-1` | 1 | technical standard | mixed | pinned-local | Defines safe stored text, validation, mark ordering, numeral notation, and collation. |
| `russian-national-corpus-church-slavonic` | 4 | large annotated corpus | synodal-russian, mixed | accessible-metadata | 5,364,905-word, 1,447-text held-out discovery/evaluation frontier; never a sole grammar oracle. |
| `gorazd` | 2 | scholarly digital dictionary/card index | old-church-slavonic | accessible-metadata | Manual lexeme and variant discovery frontier. |
| `dyachenko-1900-scan` | 6 | historical dictionary scan | mixed | pinned-local | Candidate source for irregular lexemes and meanings, not direct productive authority. |
| `synodal-target-text-union` | 3 | primary target-text union | synodal-russian | pinned-local | Locked exact-form and held-out evaluation evidence. |

## Discovery passes

- `seed-and-multilingual-pass-1` (2026-08-14) — new sources: lunt-ocs-grammar-2001, love-lmu-ocs-verbs; changed inventory/contracts/conflicts/validation: `true/true/true/true`. The pass added Lunt as an independent comprehensive grammar and LOVe as a 970-lemma principal-part/aspect source. Both require complete crosswalks, so convergence is not established.
- `ocs-numeral-frontier-pass-2` (2026-08-15) — new sources: elkina-ocs-1960, suprun-slavic-numerals-1969; changed inventory/contracts/conflicts/validation: `true/true/true/true`. The pass added an independent OCS textbook and Suprun's historical synthesis. Together with Gorshkov, Polivanova, Leuta–Havryliuk, and the pinned corpora, they close the ordinal profile at the independently specified thousandth head and classify the apparent residual derived series under adjectives, nouns, or invariant adverbs. No deterministic OCS formation above 1,000 was found; later Russian forms were excluded as recension contamination. Because this pass changes the numeral contract, it does not count as a no-change convergence pass.
- `ocs-large-number-frontier-pass-3` (2026-08-15) — new sources: lvov-ocs-lexicon-1966, simonov-nesveda-2006, pronin-large-numerals-2024; changed inventory/contracts/conflicts/validation: `true/true/true/true`. The pass followed Suprun's bibliography to Lvov's primary-text lexical study and then checked Simonov and Pronin's complete-context semantic analyses. It adds the hard a-stem indefinite-quantity noun несъвѣда, preserves its directly cited instrumental plural, rejects a context-free exact magnitude value, and documents the chronological boundary around later large-number systems.

## Next checkpoint

`ocs.verb.irregular-root`: Expand the seventy irregular-workstem anchors over their member and prefix allomorphs, crosswalk impersonal verbs, and attach per-cell facade provenance without collapsing disputed analyses or independent ѥсмь and бꙑти profiles.
