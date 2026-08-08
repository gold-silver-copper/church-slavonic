# Semantic guards and failure witnesses

The guards protect semantic owners, not opaque file hashes. Run the normal checks
with `cargo xtask check-all`; run the injected, temporary failure cases with
`cargo xtask guard-witnesses`. The latter copies only the required artifacts to a
process-specific temporary directory, mutates the copy, requires the named guard to
fail, restores the copied artifact, and deletes the directory.

All witnesses below were executed successfully on 2026-08-07. No witness mutates the
committed workspace.

| Invariant | Guard owner | Minimal injected witness | Observed failure |
|---|---|---|---|
| generated registry is current | `check-registry` deterministic emitter comparison | append one comment to copied `generated/registry.rs` | stale generated registry |
| cell/rank keys are unique | registry semantic validator | duplicate one copied form row | duplicate form key |
| public forms are nonempty and not sentinels | registry semantic validator | replace one copied form with `—` | sentinel public form |
| public spellings contain no MediaWiki markup | registry semantic validator | replace one copied form with `сло{{{2}}}во` | markup public form |
| canonical noun citations remain reachable | citation validator plus sourced exemption registry | change `обѣдъ` nominative singular | missing canonical citation |
| source variant order survives end to end | public-facade accuracy sweep | swap form/romanization payloads at ranks 0 and 1 | variant-order mismatch |
| paradigms and cell getters agree | public-facade accuracy sweep | decrement the observed matching-paradigm count | paradigm/cell disagreement |
| extraction coverage cannot silently collapse | pinned semantic floor | evaluate one fewer than 3,000 accepted lexemes | coverage collapse |
| hostile inputs do not panic | exhaustive public API integration test | a panic in any exercised public operation makes `catch_unwind` fail | targeted test fails; baseline is also exercised by `guard-witnesses` |
| runtime crates perform no I/O/network access | runtime source-boundary scan | append `use std::fs;` to copied core source | runtime boundary violation |
| attribution and licenses ship | package attribution guard and `cargo package` | remove the pinned source SHA from copied package attribution | attribution failure |
| extraction report matches registry | `check-registry` report regeneration/denominator checks | set copied JSON accepted-form count to 1 | report mismatch |
| accuracy report matches fresh evaluation | non-writing full evaluator comparison | append text to copied accuracy Markdown | stale accuracy report |

The duplicate-rank, sentinel, markup, citation, coverage, Unicode/hostile-input, and
strict verb-shape witnesses also have direct unit or integration tests. This gives fast
local failures while the temporary injected suite demonstrates that the complete
maintenance guards are wired to the committed artifacts.
