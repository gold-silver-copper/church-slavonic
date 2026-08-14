# Church Slavonic workspace refactor

This report records the workspace-wide consolidation performed from the
authoritative working tree on 2026-08-13. Linguistic coverage, reviewed data,
normalization, output order, provenance, and the exact/irregular/productive
precedence contracts were treated as behavior rather than implementation
detail.

## Workspace map

| Member | Layer and ownership | Internal direction |
|---|---|---|
| `old-church-slavonic-core` | Pure Old Church Slavonic grammar, orthography, and productive rules | Leaf |
| `old-church-slavonic` | Stable identities, generated registries, table-first resolution, handles, and paradigms | Depends on Old core |
| `old-church-slavonic-dictionary` | Senses, examples, and semantic lookup | Depends on the Old runtime layers |
| `old-church-slavonic-extractor` | Offline source ingestion, validation, registry emission, and dictionary emission | May use Old core; runtime never depends on it |
| `synodal-church-slavonic-core` | Typed Synodal grammar, evidence, orthography, results, and pure morphology | Leaf |
| `synodal-church-slavonic` | Synodal identities, reviewed registries, precedence, presentation, handles, and paradigms | Depends on Synodal core |
| `synodal-church-slavonic-dictionary` | Synodal semantic and reverse-analysis APIs | Depends on the Synodal runtime layers |
| `synodal-church-slavonic-extractor` | Offline Synodal row validation, adapters, and deterministic registry generation | Depends on Synodal core; runtime never depends on it |
| `xtask` | Repository checks, audits, generation, fixture evaluation, and source orchestration | Top-level tool over both stacks |

Generated Rust remains owned by the two extractor crates and their `xtask`
entry points. Runtime `build.rs` files only include or verify generated output;
they do not own source interpretation.

## LOC method and baseline

The measurement enumerates every workspace member from the root manifest and
then enumerates its Rust files with `rg --files <member> -g '*.rs'`. It counts
physical lines and assigns them as follows:

- `generated/*.rs` is generated code;
- integration tests, examples, benches, fixtures, and the terminal
  `#[cfg(test)]` portion of a source file are tests;
- extractor and `xtask` source before a terminal test module, plus `build.rs`,
  is tools/generation code;
- remaining source before a terminal test module is handwritten production
  code.

All source-file test modules were terminal at measurement time. Generated
changes are reported but never counted as a refactor win.

| Member | Baseline prod | Final prod | Baseline tests | Final tests | Baseline generated | Final generated | Baseline tools | Final tools |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `old-church-slavonic-core` | 2,965 | 3,051 | 926 | 926 | 0 | 0 | 0 | 0 |
| `old-church-slavonic` | 4,325 | 4,018 | 1,405 | 1,405 | 147,728 | 147,728 | 8 | 8 |
| `old-church-slavonic-dictionary` | 1,298 | 1,298 | 182 | 182 | 5,177 | 5,177 | 9 | 9 |
| `old-church-slavonic-extractor` | 0 | 0 | 744 | 744 | 0 | 0 | 3,942 | 3,949 |
| `synodal-church-slavonic-core` | 4,984 | 5,249 | 2,471 | 2,610 | 0 | 0 | 0 | 0 |
| `synodal-church-slavonic` | 5,464 | 5,167 | 2,137 | 2,240 | 4,988 | 8,657 | 0 | 0 |
| `synodal-church-slavonic-dictionary` | 4,100 | 4,109 | 972 | 1,019 | 882 | 858 | 0 | 0 |
| `synodal-church-slavonic-extractor` | 0 | 0 | 568 | 921 | 0 | 0 | 4,868 | 5,440 |
| `xtask` | 0 | 0 | 1,019 | 1,179 | 0 | 0 | 20,226 | 21,071 |
| **Total** | **23,136** | **22,892** | **10,424** | **11,226** | **158,775** | **162,420** | **29,053** | **30,477** |

Handwritten production—the primary metric—fell from 23,136 to 22,892 physical
lines: 244 fewer lines (1.05%). Production plus tools rose from 52,189 to
53,369 lines because the intended change set also adds independently reviewed
Synodal data pipelines and because the completion review added provenance,
identity-adjudication, ledger-retraction, and generated-artifact guards. That
1,424-line tool increase is reported explicitly and is not presented as a
refactor win. Generated changes likewise are not counted as a reduction.

## Ranked opportunities and decisions

1. **Canonical Synodal grammar-cell codec.** Accepted. Formatting and parsing
   had separate implementations in the core-facing resolver, abbreviation
   runtime, extractor validation, evaluation fixture, and `xtask`.
2. **Resolved handle construction and accessors.** Accepted within each
   facade. Repetition added no linguistic meaning and made inherent APIs drift.
3. **Canonical cell inventories and paradigm row assembly.** Accepted. Stable
   grammatical order belongs with typed cells; outcome classification belongs
   in one paradigm builder.
4. **Old resolver scaffolding.** Accepted. Query alias handling, identity
   lookup, table/metadata/override ordering, and explicit prediction wrapping
   are now shared without changing their order.
5. **Extractor output plumbing.** Accepted for the duplicated Old extractor
   batch writer. Its existing rollback-capable implementation is now used by
   both morphology and dictionary emission.
6. **A common Old/Synodal morphology abstraction.** Rejected. The two stacks
   intentionally differ in grammar inventory, normalization, evidence,
   provenance, and result semantics; a shared ending table or universal
   `FormSet` would hide recension boundaries.
7. **Merging dictionary analysis implementations.** Rejected. Public Synodal
   analysis and coverage analysis intentionally rank and collapse candidates
   differently, including accent-insensitive warnings and matched-text
   identity. Sharing them without a larger semantic redesign would be risky.

## Implemented consolidation

- `GrammarCell` owns its stable registry key and `FromStr` parser. Closed
  grammar enums own their stable codes. Abbreviations, extractor validation,
  runtime lookup, fixtures, and `xtask` use that codec.
- Exact-form and abbreviation cell strings now fail during extractor
  validation, before generated Rust is emitted. This strengthens failure timing
  without admitting new rows or changing generated order.
- Abbreviation rows preserve their reviewed wildcard registry keys while using
  typed cells for requests; wildcard agreement remains compatible with animate
  and inanimate lookup rather than being silently narrowed during serialization.
- Lexical-review sense provenance is derived from the registered source
  inventory. Old Church Slavonic, mixed, and Synodal source IDs can no longer be
  emitted with a contradictory recension or semantic status.
- Reviewed exact-form evidence now has a generated provenance registry derived
  from `reviewed_evidence.tsv`, lexical reviews, and `SOURCES.toml`. Runtime
  variants retain every comma-separated evidence ID separately, with its real
  source, source recension, citation, and epistemic role; direct form sources
  designate an actual target-recension attestation rather than a fused ID.
- Old grammar cells expose canonical, stably ordered inventories. Every Old
  paradigm builder consumes those inventories through one outcome builder.
- Old table-driven verb resolution still performs: part-of-speech validation,
  exact table lookup, metadata validation, reviewed override lookup, then
  productive generation. The shared helper encodes that exact order.
- Both facades generate their repeated inherent handle constructors and
  identity accessors locally. The public method names and signatures remain
  unchanged.
- Synodal registered, caller-generated, and explicit paradigms construct and
  classify rows through one implementation.
- Synodal capability reporting now covers exact future and undifferentiated
  past systems as well as productive adjective, determiner, and ordinal
  backgrounds. Dictionary family system summaries consume that inventory.
- Pronoun paradigms derive their gender/person profiles from each lexeme's
  validated reviewed table, so personal and gendered systems retain their
  distinct inventories without a global gender/person cross-product.
- Old morphology and dictionary extraction share one atomic batch writer with
  duplicate-target checks, synchronized temporary files, rollback, and cleanup.

## Verification

Targeted verification completed during implementation:

- `cargo check` for both changed runtime stacks with all targets and features;
- all Old core/facade tests and public API tests;
- Old extractor unit and atomicity tests;
- Synodal grammar-cell codec, abbreviation, linguistic-evaluation, and
  extractor tests;
- the baseline all-target/all-feature runtime suite, including the exhaustive
  Synodal dictionary and CLI tests.

The final completion-gate commands and CI/review state are recorded in the pull
request after the full verification and independent review passes.

## Independent review findings

The separate full-diff review confirmed and prompted fixes for:

- incorrect hard-coded Old Church Slavonic provenance on reviewed senses from
  mixed or Synodal source IDs;
- fused exact-form evidence IDs and lexeme-level source attribution that erased
  the distinction between OCS support, Synodal authority, and target corpus
  attestation;
- exact and abbreviation rows whose typed cells were not checked against the
  referenced lexeme's part of speech;
- a source-boundary guard that validated a metadata mirror instead of the
  provenance inventory consumed by generation;
- grammar-cell wildcard parsing that accepted the wrong dimension sentinels
  and narrowed abbreviation wildcard animacy;
- incomplete public capability and dictionary-family inventories for
  determiner, ordinal, future, and undifferentiated-past systems;
- pronoun paradigms that omitted every personal-pronoun gender/person profile;
- provider validation that admitted closed-class `Indeclinable` cells for
  caller-supplied noun, adjective, or verb specifications;
- a `show` CLI parser that silently ignored unknown or excess arguments;
- append-only v0.7 materialization that retained rows after a later rejection
  or deferral, could not restore deleted admissions, and emitted duplicate
  runtime variants with split provenance; cumulative durable packet ownership
  now supports retraction, restoration, and historical re-admission;
- target-token reuse that treated homographs as proof of incompatible lexical
  identities or created separate lexemes for inflected pronoun/noun forms;
- an evidence-ID aliasing path that could bypass the target-identity guard;
- stale live coverage, evaluation, queue, registry-count, and LOC reporting
  after the corrective retractions; and
- a historical v0.7 audit command that described its immutable snapshot as
  current.
- sparse exact verb cells that hid missing productive stem diagnostics, and a
  reverse-analysis inventory that omitted represented supine and verbal-noun
  cells.

All were validated against the current code and addressed with regression
coverage.

## 2026-08-14 follow-up consolidation

This follow-up started from merge commit `5ab1586` and re-audited all nine
workspace members using the same architectural boundaries and physical-LOC
method documented above. It did not change reviewed linguistic data, generated
registries, morphology rules, normalization, provenance, or corpus coverage.

### Follow-up LOC accounting

| Member | Baseline prod | Final prod | Baseline tests | Final tests | Baseline generated | Final generated | Baseline tools | Final tools |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `old-church-slavonic-core` | 3,051 | 3,051 | 926 | 926 | 0 | 0 | 0 | 0 |
| `old-church-slavonic` | 4,018 | 4,018 | 1,405 | 1,405 | 147,728 | 147,728 | 8 | 8 |
| `old-church-slavonic-dictionary` | 1,298 | 1,298 | 182 | 182 | 5,177 | 5,177 | 9 | 9 |
| `old-church-slavonic-extractor` | 0 | 0 | 744 | 744 | 0 | 0 | 3,949 | 3,949 |
| `synodal-church-slavonic-core` | 5,251 | 5,403 | 2,610 | 2,674 | 0 | 0 | 0 | 0 |
| `synodal-church-slavonic` | 5,167 | 5,117 | 2,240 | 2,275 | 8,657 | 8,657 | 0 | 0 |
| `synodal-church-slavonic-dictionary` | 4,493 | 4,317 | 1,322 | 1,341 | 858 | 858 | 0 | 0 |
| `synodal-church-slavonic-extractor` | 0 | 0 | 950 | 950 | 0 | 0 | 5,505 | 5,505 |
| `xtask` | 0 | 0 | 1,179 | 1,213 | 0 | 0 | 21,082 | 20,996 |
| **Total** | **23,278** | **23,204** | **11,558** | **11,710** | **162,420** | **162,420** | **30,553** | **30,467** |

The primary metric fell by 74 handwritten production lines. Tooling fell by 86
lines independently; production plus tools fell from 53,831 to 53,671 lines,
a net reduction of 160 handwritten lines. The 152 added test lines exercise the
new shared order, inventory bounds, wildcard lookup compatibility, and report
I/O failure contracts. No code moved into generated output, data, macros, or
tools to improve the production metric.

### Follow-up decisions and implementation

1. **Typed Synodal grammar inventories:** accepted. `NounCell`,
   `AdjectiveCell`, `FiniteVerbCell`, `ParticipleCell`, `PronounCell`, and
   `NumeralCell` now own ordered Cartesian inventory construction. The facade
   supplies its canonical paradigm policies, while the dictionary explicitly
   supplies exhaustive animacy, form, comparison, gender, and person ranges.
   This shares traversal mechanics without pretending those two inventories
   are linguistically identical.
2. **Exact-registry compatibility keys:** accepted. The dictionary now calls
   the facade's `grammar_cell_registry_keys` API instead of reproducing
   wildcard animacy/gender compatibility and key order. Exact lookup and
   reverse-analysis filtering therefore cannot drift independently.
3. **Part-of-speech codes:** accepted. `PartOfSpeech::{code, from_code}` is the
   single closed-code mapping used by registry parsing, CLI parsing, and CLI
   display. CLI aliases remain a presentation-layer concern.
4. **Report file plumbing:** accepted. Six Synodal report commands share
   stale-content checks, seven share conditional direct/atomic writes, and six
   report readers share typed JSON loading. Existing output paths, byte
   content, atomic/direct modes, and stale-report messages remain explicit at
   each call site.
5. **A generic dictionary CLI option parser:** rejected after implementation
   and measurement. It added production lines after formatting and obscured
   command-specific positional and stdin rules, so the experiment was removed.
6. **Standalone imperative and l-participle inventory helpers:** rejected after
   comparison. The dictionary deliberately interleaves these cells by number,
   so such helpers would have only one real caller and would not remove a
   duplicated implementation.
7. **A universal Old/Synodal inventory or resolver abstraction:** rejected.
   Old and Synodal cells, animacy behavior, exact-table semantics, provenance,
   and failure models remain separate recension-owned systems.

Public API additions are limited to typed inventory constructors,
`PartOfSpeech::{code, from_code}`, and `grammar_cell_registry_keys`. They expose
already-existing stable order and registry compatibility; no accepted form,
variant order, source, warning, trace, or failure category changes. No
generated artifact required regeneration.

### Follow-up regression coverage

- typed inventory sizes, boundary order, caller-supplied animacy policy, and
  grammar-profile cross products;
- canonical-to-wildcard exact-registry key order for pronouns and numerals;
- exhaustive dictionary candidate counts for every inflecting category and
  every represented verb system; and
- shared report direct/atomic writes, typed JSON loading, and both historical
  stale-report diagnostic forms.

Targeted all-feature tests passed for Synodal core, facade, dictionary, CLI,
and `xtask`, including the independent exhaustive analyzer oracle. The final
workspace commands, platform builds, documentation build, package checks, and
independent diff review are recorded in the pull request completion gate.
