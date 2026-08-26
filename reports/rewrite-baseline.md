# Rewrite baseline (Phase 0)

Recorded 2026-08-25 at tag `v0.14-pre-rewrite` (commit 16e4ff6, wide-е fold
included). Every later rewrite phase must meet or beat these numbers.

## OCS accuracy (`cargo xtask accuracy`)

- Primary-correct cells: **134,761 / 134,761 (100%)** against
  `data/extracted/forms.tsv`.
- Dictionary lookup: diplomatic and project-lookup, top-1 and any:
  2,690 / 964 (see full table via re-run).
- Source dictionary verb lexemes: 711.
- Skipped OOV cells requiring unavailable lexical metadata: 5,007.

## Synodal evaluation (`cargo xtask synodal-evaluate`)

- Expanded **2,431 / 2,499**, printed **2,341 / 2,499** evaluation rows.

## Synodal coverage fixture (`cargo xtask synodal-coverage --offline --fixture --check`)

- 10 passages, 155 tokens, 113 types, 126 top-k, **29 unresolved**.

## Public API snapshot

- `reports/rewrite-api-snapshot.txt` (1,153 lines of `pub` items across the
  six published crates) — input to the Phase 4 deprecation map.

## Reproduction

```
cargo run -p xtask -- accuracy
cargo run -p xtask -- synodal-evaluate
cargo run -p xtask -- synodal-coverage --offline --fixture --check
cargo run -p xtask -- synodal-check
```
