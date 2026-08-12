# Reviewable Synodal seed registries

These UTF-8 TSV files are the human-review boundary for the generated runtime
registry. They are intentionally small: a row is admitted only when its target
recension and normative evidence have been reviewed. The offline extractor
validates and converts them to deterministic Rust; generated Rust must not be
edited directly.

`exact_forms.tsv` contains both normative grammar-table forms and exact
source-partition target attestations. `source_kind` distinguishes
`normative-table`, `normative-variant`, and `synodal-attestation`; every
attestation must cite a reviewed whole-token corpus candidate outside all
held-out passages.

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

The alignment registry never imports an OCS surface cell. It connects stable
lexeme identities and records morphology and semantics independently.

Dictionary glosses sourced from the mixed historical D'yachenko dictionary are
admitted only with `reviewed-with-synodal-corpus` status; that status records a
target-usage check and does not turn the mixed dictionary into a Synodal
orthographic authority. Examples used as source evidence are never part of a
held-out evaluation partition.
