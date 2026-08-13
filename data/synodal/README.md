# Reviewable Synodal seed registries

These UTF-8 TSV files are the human-review boundary for the generated runtime
registry. They are intentionally small: a row is admitted only when its target
recension and normative evidence have been reviewed. The offline extractor
validates and converts them to deterministic Rust; generated Rust must not be
edited directly.

`accent_paradigms.tsv` stores reusable, source-backed accent rules separately
from the exact per-cell strings in `accents.tsv`. Its scope, stem/ending
placement, mark, optional independently positioned breathing, evidence, and
precise citation are validated before code generation. One paradigm may have
several disjoint scope rows for documented mobility.

`engine_capabilities.tsv` is the concise v0.10 engine contract. It distinguishes
typed categories, productive rules, exact/irregular systems, reusable accent
paradigms, and unsupported systems. `cargo xtask synodal-engine-audit` renders
it into the human-readable v0.10 audit, while `--check` verifies byte currency.

`linguistic_evaluation.tsv` is a small, frequency-independent behavioral gate.
Each row names one stable identity and typed cell, its ordered variants or
stable error code, the expected provenance class, and its source citation. The
facade integration test executes every row directly; it is not weighted by
corpus frequency or token recovery.

The v0.9 productive lexical upgrades in `lexemes.tsv` are individually reviewed:
`мꙋжъ`, `имѧ`, `небо`, and `мати` carry explicit class, stem, gender, source,
and target-recension metadata. Their existing exact cells remain separate and
win first. No suffix guessing or corpus-frequency rule materializes additional
productive lexemes.

`noun_restrictions.tsv` carries independently evidenced number inventories for
registered productive nouns. These rows compile to the closed
`NounNumberInventory` enum; they are not inferred from a plural-looking lemma.

The v0.10 noun additions are `ѻтроча : ѻтрочат-`, `свекры : свекров-`, and
`камень : камен-`. Their stems are independent metadata. The ordinary
`камень` plural deliberately does not absorb the separate collective
`каменїе`; its cited `-їѧ` and `-ема` alternatives belong to a closed
lexeme-specific contract and do not leak into the general masculine `-ен-`
family. Further unmodeled lexeme-specific variants remain explicit evidence
work.

Active short-participle principal parts encode both adjective class and typed
citation-edge formation, for example
`hard:present-first-unpalatalized` or `hard:past-consonant`. Comparison stems use
closed formation codes such as `later-yat`; runtime code never dispatches on
unvalidated free-form class names.

`exact_forms.tsv` contains both normative grammar-table forms and exact
source-partition target attestations. `source_kind` distinguishes
`normative-table`, `normative-variant`, and `synodal-attestation`; every
attestation must cite a reviewed whole-token corpus candidate outside all
held-out passages.

`target_identity_ambiguities.tsv` is the explicit exception registry for a
single target token that contextually supports more than one runtime lexical
identity. Each row binds both the stable target evidence and candidate IDs,
the exact NFC surfaces, a sorted lexeme pair, and a review note. Generation
rejects unadjudicated cross-identity reuse even when separate evidence IDs
alias the same target candidate.

The v0.6 decision ledgers are `v06_exact_reviews.tsv`,
`v06_abbreviation_reviews.tsv`, and `v06_spelling_reviews.tsv`. They retain
predicted versus surface-realized gain, explicit deferrals and rejections, and
precise evidence blockers. `v06_verification.tsv` records only checks actually
run for the generated v0.6 completion audit.

The v0.7 evidence-acquisition decisions are retained in the ordered
`v07_exact_reviews*.tsv` waves, `v07_variant_reviews.tsv`, and
`v07_abbreviation_reviews.tsv`. `v07_identity_reviews.tsv` and
`v07_semantic_reviews.tsv` keep identity and sense alignment separate;
`v07_identity_corrections.tsv` and `v07_evidence_corrections.tsv` preserve
reviewed merges and passage-disjoint replacement witnesses. The generated
completion audit reads `v07_verification.tsv` and rejects a missing or failed
required gate.

The exact-review waves are cumulative: the latest packet decision is
authoritative. Materialization retracts exact, evaluation, and exclusively
owned evidence rows after a later rejection or deferral, while preserving the
rejected review decision itself for audit history.
`v07_packet_ownership.tsv` is the durable cumulative materialization ledger;
the companion `v07_packet_evidence_ownership.tsv` and
`v07_packet_lexical_ownership.tsv` retain its restorable provenance and lexical
dependencies. Together they retain every historically materialized packet's
exact tuple, source candidate, held-out evaluation witness, and owned review
rows, including inactive decisions. Ordinary `synodal-v07-apply` strips
packet-owned runtime rows and restores only current admissions.
`synodal-v07-apply --refresh-ownership` merges newly admitted current packet
facts into the ledgers without discarding historical tombstones. The complete
historical owner set is count- and digest-locked so a truncated or malformed
ledger fails closed.

The alignment registry never imports an OCS surface cell. It connects stable
lexeme identities and records morphology and semantics independently.

Dictionary glosses sourced from the mixed historical D'yachenko dictionary are
admitted only with `reviewed-with-synodal-corpus` status; that status records a
target-usage check and does not turn the mixed dictionary into a Synodal
orthographic authority. Examples used as source evidence are never part of a
held-out evaluation partition.
