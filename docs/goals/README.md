# Goal history

Every completed or superseded goal prompt, in program order. The one active
prompt stays in the repository root
(`SYNODAL_V12_GENERALISATION_AND_PREDICTIVE_LEXICON_PROMPT.md`); its running
audit is `docs/SYNODAL_V12_GENERALISATION_AUDIT.md`.

| Prompt | Status |
|---|---|
| `OLD_CHURCH_SLAVONIC_INFLECTOR_PROMPT.md` | complete — the OCS inflector core |
| `OLD_CHURCH_SLAVONIC_API_IMPROVEMENT_PROMPT.md` | complete — OCS facade API |
| `CHURCH_SLAVONIC_VERB_METADATA_PROMPT.md` | complete — OCS verb metadata |
| `CHURCH_SLAVONIC_VERB_EXPANSION_PROMPT.md` | complete — OCS verb expansion |
| `CHURCH_SLAVONIC_GRAMMAR_COMPLETION_GOAL_PROMPT.md` | complete — OCS grammar completion |
| `OCS_INFLECTION_COVERAGE_AND_IMPLEMENTATION_GAPS.txt` | reference — the OCS gaps survey that first proposed the predictive third layer (realised in v0.12 phase 3) |
| `CHURCH_SLAVONIC_RUTHENIAN_API_PROMPT.md` | superseded — the recension boundary became the Synodal library |
| `SYNODAL_RUSSIAN_CHURCH_SLAVONIC_LIBRARY_PROMPT.md` | complete — the Synodal library exists |
| `SYNODAL_DATA_PIPELINE_AND_COVERAGE_PROMPT.md` | complete — locked sources, pipeline, coverage reports |
| `SYNODAL_V03_CORPUS_DRIVEN_COVERAGE_PROMPT.md` | complete — corpus-driven coverage |
| `SYNODAL_V04_MORPHOLOGICAL_FAMILY_COVERAGE_PROMPT.md` | complete — sealed v0.4 checkpoint |
| `SYNODAL_V05_TOP_K_COVERAGE_PROMPT.md` | complete — sealed v0.5 checkpoint |
| `SYNODAL_V06_65_PERCENT_TOP_K_COVERAGE_PROMPT.md` | complete — immutable 65% checkpoint |
| `SYNODAL_V07_70_PERCENT_TOP_K_COVERAGE_PROMPT.md` | complete — immutable 70% checkpoint |
| `SYNODAL_DICTIONARY_TEST_PERFORMANCE_PROMPT.md` | complete — test performance |
| `SYNODAL_V08_INFLECTION_ENGINE_IMPROVEMENT_PROMPT.md` | complete — v0.8 engine |
| `SYNODAL_V09_INFLECTION_ENGINE_IMPROVEMENT_PROMPT.md` | complete — v0.9 engine |
| `CHURCH_SLAVONIC_REFACTOR_PROMPT.md` | complete — workspace refactor |
| `SYNODAL_V10_PRODUCTIVE_MORPHOLOGY_AND_LEXICON_PROMPT.md` | complete — v0.10 productive morphology |
| `SYNODAL_V11_COVERAGE_INTEGRITY_AND_VERB_MORPHOLOGY_PROMPT.md` | complete (phases 1–3; phase 4 carried into v0.12) — coverage integrity, holdout, floors |

The immutable v0.4–v0.7 audit artifacts are verified in CI by one checksum
manifest (`cargo xtask synodal-archive --check`); the original audit commands
remain available for on-demand re-derivation.
