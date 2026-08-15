# Synodal implementation requirements matrix

This is the requirements matrix for the current reviewed release descended from
`SYNODAL_RUSSIAN_CHURCH_SLAVONIC_LIBRARY_PROMPT.md`. “Complete” means the
architecture and seed implementation exist and are guarded. “Partial” means the
typed contract exists but the explicitly listed linguistic/data surface is not
language-wide. Unsupported cells must remain typed failures; they are not hidden
to make a row look complete.

| Requirement | Status | Evidence and boundary |
|---|---|---|
| Exact recension and source precedence | complete | `SYNODAL_RECENSION.md`; target is always `SynodalRussian` |
| Pinned machine-readable source inventory | complete | `SOURCES.md`, `references/SOURCES.toml`, 321 individually locked local artifacts; access-blocked/rights-unresolved sources are explicit metadata-only rows |
| Normalization and partition contract | complete | `data/normalization/*`, `training_passages.tsv`, partition-disjointness guard |
| OCS reuse audit and mapping boundary | complete | `SYNODAL_REUSE_AUDIT.md`; no OCS crate dependency or target surface-row import |
| Orthography/Unicode specification | complete | `SYNODAL_ORTHOGRAPHY.md`, validated word/rendered-text types, UTN #41 collation |
| Morphology specification and category inventory | complete | `SYNODAL_MORPHOLOGY.md` identifies every supported rule and remaining category |
| Pure Synodal core crate | complete | `synodal-church-slavonic-core`; no registry or runtime I/O |
| Dictionary-backed facade and handles | complete | `synodal-church-slavonic`; seven handle types, canonical resolver, direct APIs |
| Semantic/reverse-analysis dictionary crate and CLI | complete for the reviewed registry | `synodal-church-slavonic-dictionary`; 846 reviewed senses, mark-sensitive multi-analysis lookup, reviewed/proposed family display, and `synodal-dict` search/show/families/show-family/analyze/lint/check-text/coverage commands |
| Streaming deterministic extractor | complete | atomic candidate/intermediate/quarantine/reviewed/generated layers plus adapters for Ponomar, Alypy, D’yachenko, exact-revision Wikisource, CrossWire, Polivanova, UD, Syntacticus, CCMH, DIACU, Kaikki, and the Ponomar frequency list |
| Validated text and hostile Unicode guards | complete | other-script/private-use/control/combining-order tests and injected guard witnesses |
| Provenance, evidence, traces, and policies | complete | per-variant evidence/source/recension/mapping/confidence/trace; three policies |
| Independent registries | complete | 846 lexemes, 3,550 generated exact forms, 846 senses, typed principal parts, reviewed accent witnesses, 159 typed abbreviation cells, family decisions, positional, override, example, rejected-candidate, and reviewed-evidence tables |
| Reviewed OCS/Synodal alignment gold set | complete for current reuse | accepted and rejected mappings, false-friend controls, and separate semantic decisions/conflict registry; every runtime reuse fact names reviewed evidence |
| Noun/adjective productive morphology | broad but not language-wide | thirty-six noun contracts exhaust Alypy §§33–44 regular and named irregular families; caller specifications can express arbitrary ordered irregular variants and typed defects; adjective coverage includes short/long positive, complete long and short comparative inventories, full long superlatives, and the exceptional nine-cell predicate short-superlative agreement contract from independently reviewed stems |
| Pronoun/determiner/numeral morphology | broad closed-class coverage | complete productive Alypy §§45–48 pronouns; complete determiner-owned `самъ`/`самый`, `весь`, `всѧкъ`/`всѧкїй`, and `всѧческїй` systems with exact-first variants and typed no-dual/form restrictions; reviewed `два`, `три`, and `четыре`; productive ordinal adjectives; remaining gap is the wider cardinal/collective/fractional/distributive numeral inventory |
| Independent verb systems | broad but not language-wide | regular principal-part bundles, exact irregular auxiliaries including the nine-cell simple future of `быти`, reviewed `дати`, productive tense/voice-specific declined participles, and l-participles; the distinct supine is source-closed as absent from the Russian/Synodal recension with a caller-exact interoperability seam, while productive verbal-noun formation remains under review |
| Cyrillic numerals, transliteration, collation | complete for documented range | numeral 1–9,999 plus 100,000/1,000,000; two loss-reporting transliterations; UTN #41 word collation |
| Abbreviation and printed orthography | partial | 159 individually reviewed contraction cells, reviewed accents and explicit positional operation; no blind/general contraction or inferred paradigm |
| Analyzer, capabilities, phrases, manifest lint | complete for registered data | multi-analysis reverse lookup, missing metadata, five analytic constructions, and typed vocabulary lint without runtime data-format access |
| Deterministic reports and structural guards | complete | generated/extraction/evaluation/coverage freshness, typed-gap precedence, corpus hashes, recension, forbidden-authority, PUA, I/O, review-queue, package, and witness guards |
| Leakage-resistant real evaluation | complete for the current registry; still statistically limited | 2,137 active passage-held-out morphology cells (2,137/2,137 Productive/Exploratory top-k) plus three historically preserved but correction-ledger-retracted `господи` misclassifications, five analytic constructions, 74 typed contraction cases, 502 active masked-cell controls, 2,828-row exact-registry round-trip, held-out exact-attestation rows, inherited-cell, strict abstention, hostile-Unicode, and partition-contamination checks; accent and abbreviation witnesses use disjoint source partitions |
| Native/no-default/WASM builds | complete | native all-feature tests, no-default checks, `wasm32-unknown-unknown` all-feature check |
| Package content and publish dry-run | guarded, publication externally sequenced | runtime package lists are checked for required generated registries and for excluded raw/intermediate/reference data; publication remains a separate explicitly authorized operation |

## Release interpretation

This matrix does not claim that the library inflects every Synodal form or knows
every word. It claims that a first version has a coherent target-recension
boundary, a complete uncertainty/provenance contract, useful reviewed systems,
real-data regression tests, and explicit failures everywhere else. The main next
work is broader adjective, closed-class, verb, analytic, accent, and
positional-orthography coverage, plus additional
passage-disjoint evaluation—not speculative completion of unsupported cells.
