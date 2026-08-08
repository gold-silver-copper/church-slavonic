# Semantic guards and failure witnesses

The guards protect semantic owners, not opaque file hashes. Run the normal checks
with `cargo xtask check-all`; run the injected, temporary failure cases with
`cargo xtask guard-witnesses`. The latter copies only the required artifacts to a
process-specific temporary directory, mutates the copy, requires the named guard to
fail, restores the copied artifact, and deletes the directory.

All witnesses below were executed successfully on 2026-08-08. No witness mutates the
committed workspace.

| Invariant | Guard owner | Minimal injected witness | Observed failure |
|---|---|---|---|
| facade root remains curated | public API structure scan | append `pub use old_church_slavonic_core::*` to copied facade source | blanket core re-export |
| ordinary calls take direct grammar dimensions | public API signature scan | replace copied `noun(lemma, case, number)` parameters with `NounCell` | ordinary root function requires a cell struct |
| successful form sets are structurally nonempty | private-field/constructor scan | make copied `FormSet::variants` public | nonempty construction invariant lost |
| convenience calls use the canonical resolver | root delegation scan | replace copied `resolver::noun` delegation with a direct core call | canonical delegation missing |
| paradigms retain failed cells | paradigm-builder scan | conditionally continue when copied `noun_by_id` returns an error | canonical `CellOutcome` missing / failure filter detected |
| every root function has a runnable rustdoc example | root rustdoc scan plus doctests | remove the copied noun example fences | root function lacks an example |
| generated registry is current | `check-registry` deterministic emitter comparison | append one comment to copied `generated/registry.rs` | stale generated registry |
| cell/rank keys are unique | registry semantic validator | duplicate one copied form row | duplicate form key |
| metadata codes are closed and typed | registry semantic validator | replace one formation with `unknown-formation` | unknown metadata formation |
| metadata analyses are complete | registry semantic validator | remove one imperfect `variant-policy` row | incomplete metadata analysis |
| metadata field groups are unique | registry semantic validator | duplicate one normalized metadata row | duplicate metadata field |
| metadata belongs to a verb lexeme | registry semantic validator | replace one metadata ID with an orphan | orphan metadata lexeme |
| metadata stems are present and NFC | registry semantic validator | empty a stem, then replace one with decomposed `И` + breve | empty/non-NFC metadata value |
| productive metadata stems are Cyrillic | registry semantic validator | replace one stem with Latin text | non-Cyrillic metadata stem |
| overrides name valid missing cells | override parser and registry validator | replace an override feature with unsupported `future` | invalid override feature |
| public forms are nonempty and not sentinels | registry semantic validator | replace one copied form with `—` | sentinel public form |
| public spellings contain no MediaWiki markup | registry semantic validator | replace one copied form with `сло{{{2}}}во` | markup public form |
| canonical noun citations remain reachable | citation validator plus sourced exemption registry | change `обѣдъ` nominative singular | missing canonical citation |
| source variant order survives end to end | public-facade accuracy sweep | swap form/romanization payloads at ranks 0 and 1 | variant-order mismatch |
| paradigms and cell getters agree | public-facade accuracy sweep | decrement the observed matching-paradigm count | paradigm/cell disagreement |
| extraction coverage cannot silently collapse | pinned semantic floor | evaluate one fewer than 3,000 accepted lexemes | coverage collapse |
| hostile inputs do not panic | exhaustive public API integration test | a panic in any exercised public operation makes `catch_unwind` fail | targeted test fails; baseline is also exercised by `guard-witnesses` |
| runtime crates perform no file/network/JSON/TSV/XML/Lua access | runtime source and dependency boundary scan | append `use std::fs;` to copied core source | runtime boundary violation |
| attribution and licenses ship | package attribution guard and `cargo package` | remove the pinned source SHA from copied package attribution | attribution failure |
| extraction report matches registry | `check-registry` report regeneration/denominator checks | set copied JSON accepted-form count to 1 | report mismatch |
| accuracy report matches fresh evaluation | non-writing full evaluator comparison | append text to copied accuracy Markdown | stale accuracy report |
| held metadata cannot win by abstaining | dictionary-metadata funnel guard | lower final metadata availability below 35% | metadata availability floor |
| held metadata output remains accurate | dictionary-metadata funnel guard | lower final lookup-any conditional correctness below 95% | metadata conditional-accuracy floor |

The API-shape, duplicate-rank, sentinel, markup, citation, coverage, Unicode/hostile-input,
strict verb-shape, metadata-code, metadata-rank/provenance, leakage-filter, and
frozen-partition witnesses also have direct unit or integration tests. This gives
fast local failures while the temporary injected suite demonstrates that the
complete maintenance guards are wired to the committed artifacts.
