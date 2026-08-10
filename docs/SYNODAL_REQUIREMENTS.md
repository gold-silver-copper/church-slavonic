# Synodal implementation requirements matrix

This is the completion audit for the first reviewed release described by
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
| Semantic/reverse-analysis dictionary crate | complete for the reviewed registry | `synodal-church-slavonic-dictionary`; 61 reviewed senses with reverse lookup and explicit ambiguity |
| Streaming deterministic extractor | complete | atomic candidate/intermediate/quarantine/reviewed/generated layers plus adapters for Ponomar, Alypy, D’yachenko, exact-revision Wikisource, CrossWire, Polivanova, UD, Syntacticus, CCMH, DIACU, Kaikki, and the Ponomar frequency list |
| Validated text and hostile Unicode guards | complete | other-script/private-use/control/combining-order tests and injected guard witnesses |
| Provenance, evidence, traces, and policies | complete | per-variant evidence/source/recension/mapping/confidence/trace; three policies |
| Independent registries | complete | 61 lexemes, 260 exact forms, typed principal parts, reviewed accent witnesses, abbreviation, positional, override, sense/example, and reviewed-evidence tables |
| Reviewed OCS/Synodal alignment gold set | complete for current reuse | accepted and rejected mappings, false-friend controls, and separate semantic decisions/conflict registry; every runtime reuse fact names reviewed evidence |
| Noun/adjective productive morphology | broad but not language-wide | seven noun classes, short/long positive adjectives, and full long comparative/superlative declension from independently reviewed stems; irregular alternants remain explicit data |
| Pronoun/determiner/numeral morphology | broad closed-class coverage | full reviewed `той`, `азъ`, and `ты`; demonstrative `сей`, relative `иже`, interrogative `кто`/`что`, indefinite `нѣкто`; reviewed `два`, `три`, and `четыре`; productive regular non-velar determiners and ordinal adjectives; exceptional cells such as `всѧкъ` remain exact evidence |
| Independent verb systems | broad but not language-wide | regular principal-part bundles, exact irregular auxiliaries, reviewed `дати`, productive tense/voice-specific declined participles, and l-participles; supine and verbal-noun formation remain intentionally unsupported pending a Synodal rule |
| Cyrillic numerals, transliteration, collation | complete for documented range | numeral 1–9,999 plus 100,000/1,000,000; two loss-reporting transliterations; UTN #41 word collation |
| Abbreviation and printed orthography | partial | semantic `богъ` nomen sacrum, reviewed accents and explicit positional operation; no blind/general contraction |
| Analyzer, capabilities, phrases, manifest lint | complete for registered data | multi-analysis reverse lookup, missing metadata, five analytic constructions, and typed vocabulary lint without runtime data-format access |
| Deterministic reports and structural guards | complete | generated/extraction/evaluation freshness, recension, forbidden-authority, PUA, I/O, package and witness guards |
| Leakage-resistant real evaluation | complete for the current registry; still statistically limited | 38 passage-held-out token cells (38/38 top-k, 37/38 top-1) plus analytic, masked-cell, 260-row exact round-trip, inherited-cell, and partition-contamination checks; accent witnesses use disjoint source partitions |
| Native/no-default/WASM builds | complete | native all-feature tests, no-default checks, `wasm32-unknown-unknown` all-feature check |
| Package content and publish dry-run | guarded, publication externally sequenced | runtime package lists are checked for required generated registries and for excluded raw/intermediate/reference data; publication remains a separate explicitly authorized operation |

## Release interpretation

This matrix does not claim that the library inflects every Synodal form or knows
every word. It claims that a first version has a coherent target-recension
boundary, a complete uncertainty/provenance contract, useful reviewed systems,
real-data regression tests, and explicit failures everywhere else. The main next
work is larger independently reviewed lexical coverage, irregular and defective
paradigms, broader accent/positional-orthography evidence, and additional
passage-disjoint evaluation—not speculative completion of unsupported cells.
