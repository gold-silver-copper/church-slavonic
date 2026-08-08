# V0.1 implementation report

## Result

The workspace follows one data path: a pinned local Kaikki JSONL snapshot is streamed
into a reviewable TSV registry, the registry plus approved overrides deterministically
emits static Rust, and both single-cell and paradigm APIs resolve through the same
public facade. Runtime crates perform no I/O and never parse the source data.

The extractor deliberately excludes all-form-of pages as duplicate lexemes, separates
the repeated personal/reflexive source table, retains number-invariant reflexive
cells, and rejects unsafe verb shapes rather than inferring missing dimensions. The
normalized lexeme records retain raw page words, raw class markers, serialized head
templates, source tags, ordered ranks, romanization, and explicit alias relationships.
Every form row also retains the exact raw source spelling, including when a safe
comma-delimited source list is normalized into separate ordered public variants.

## Review and witnesses

The revision-pinned curated fixture is
`crates/old-church-slavonic/tests/fixtures/goldens.tsv`. It covers all productive noun
classes, all case/number combinations, both adjective forms and stem types, all five
productive present classes, a root verb, `бꙑти`, safe nonfinite categories,
personal/reflexive pronouns, a numeral, alternatives, combining marks, and a
source-backed Glagolitic paradigm. Core noun tests additionally pin all 21 outputs for
every supported class, including the explicit indeclinable class.

On 2026-08-07, `cargo xtask guard-witnesses` injected every temporary artifact
mutation documented in [GUARDS.md](GUARDS.md), observed each guard fail for the stated
reason, restored the copy, and completed successfully. The final verification and
separate full-tree review are recorded in the final project handoff rather than
hard-coded here, because performance and package size are machine-dependent.

The separate full-tree/data review found and fixed three material issues before the
final gate: rollback could remove an untouched artifact after a partial batch-backup
failure; four source-template failures exposed literal `{{{2}}}` placeholders as
noun paradigms; and six phrase-valued rows were exposed through the word-level API.
Batch rollback now tracks installed/backed-up state, MediaWiki markup and multiword
forms fail closed, and the two genuine comma-delimited pronoun lists become structured
variants with their raw source strings preserved. Direct tests and injected guard
witnesses cover these boundaries.

## Known boundary

Dictionary lookup exposes only source cells that passed the schema audit. The core
predicts nominal paradigms and five present-conjugation classes with explicit lexical
metadata; it does not predict imperfects, aorists, imperatives, or participial stems.
The largest next linguistic gain is a separately specified, independently validated
verb stem/class system that can replace today’s intentionally excluded malformed
verb blocks without guessing from table order.
