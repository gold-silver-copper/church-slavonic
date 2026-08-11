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

The alignment registry never imports an OCS surface cell. It connects stable
lexeme identities and records morphology and semantics independently.

Dictionary glosses sourced from the mixed historical D'yachenko dictionary are
admitted only with `reviewed-with-synodal-corpus` status; that status records a
target-usage check and does not turn the mixed dictionary into a Synodal
orthographic authority. Examples used as source evidence are never part of a
held-out evaluation partition.
