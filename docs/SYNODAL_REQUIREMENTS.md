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
| Pinned machine-readable source inventory | complete | `SOURCES.md`, `references/SOURCES.toml`, 244 checksum-verified local artifacts; access-blocked/rights-unresolved sources are explicit metadata-only rows |
| Normalization and partition contract | complete | `data/normalization/*`, `training_passages.tsv`, partition-disjointness guard |
| OCS reuse audit and mapping boundary | complete | `SYNODAL_REUSE_AUDIT.md`; no OCS crate dependency or target surface-row import |
| Orthography/Unicode specification | complete | `SYNODAL_ORTHOGRAPHY.md`, validated word/rendered-text types, UTN #41 collation |
| Morphology specification and category inventory | complete | `SYNODAL_MORPHOLOGY.md` identifies every supported rule and remaining category |
| Pure Synodal core crate | complete | `synodal-church-slavonic-core`; no registry or runtime I/O |
| Dictionary-backed facade and handles | complete | `synodal-church-slavonic`; seven handle types, canonical resolver, direct APIs |
| Semantic/reverse-analysis dictionary crate | complete | `synodal-church-slavonic-dictionary`; 16 reviewed seed entries |
| Streaming deterministic extractor | complete for seed; adapters partial | atomic TSV-to-Rust generation plus streaming Ponomar and Kaikki adapters; other downloaded formats remain backlog |
| Validated text and hostile Unicode guards | complete | other-script/private-use/control/combining-order tests and injected guard witnesses |
| Provenance, evidence, traces, and policies | complete | per-variant evidence/source/recension/mapping/confidence/trace; three policies |
| Independent registries | complete | lexemes, 138 exact forms, 18 principal parts, 8 accents, abbreviation, positional, override, sense/example tables |
| Reviewed OCS/Synodal alignment gold set | complete for seed | five accepted mappings, one rejected false-friend control, separate semantic decisions/conflict registry |
| Noun/adjective productive morphology | partial | five noun classes and four positive adjective systems; missing classes/alternants/comparison are explicit |
| Pronoun/determiner/numeral morphology | partial | full reviewed `той`, reviewed `два`, one `всѧкъ` cell; no arbitrary closed-class generator |
| Independent verb systems | partial | three regular principal-part bundles plus exact `быти`/`имати`; productive declined participles, supine, verbal nouns remain unsupported |
| Cyrillic numerals, transliteration, collation | complete for documented range | numeral 1–9,999 plus 100,000/1,000,000; two loss-reporting transliterations; UTN #41 word collation |
| Abbreviation and printed orthography | partial | semantic `богъ` nomen sacrum, reviewed accents and explicit positional operation; no blind/general contraction |
| Analyzer, capabilities, phrases, manifest lint | complete for registered data | multi-analysis reverse lookup, missing metadata, five analytic constructions, serde/JSON vocabulary lint |
| Deterministic reports and structural guards | complete | generated/extraction/evaluation freshness, recension, forbidden-authority, PUA, I/O, package and witness guards |
| Leakage-resistant real evaluation | complete for seed; statistically limited | 11 held-out token cells, one held-out analytic phrase, five masked cells, one leave-one-target-lexeme-out inherited cell |
| Native/no-default/WASM builds | complete | native all-feature tests, no-default checks, `wasm32-unknown-unknown` all-feature check |
| Package content and publish dry-run | partial, externally sequenced | all three runtime package lists contain READMEs, attribution, both license texts, and only runtime/generated seed facts; core dry-run passes, facade/dictionary verification requires publishing the new core/facade versions first, which this task explicitly forbids |

## Release interpretation

This matrix does not claim that the library inflects every Synodal form or knows
every word. It claims that a first version has a coherent target-recension
boundary, a complete uncertainty/provenance contract, useful reviewed systems,
real-data regression tests, and explicit failures everywhere else. The main next
work is corpus-backed registry growth and independently specified productive
participle, comparison, irregular, accent, and positional-orthography systems.
