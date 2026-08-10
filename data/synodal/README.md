# Reviewable Synodal seed registries

These UTF-8 TSV files are the human-review boundary for the generated runtime
registry. They are intentionally small: a row is admitted only when its target
recension and normative evidence have been reviewed. The offline extractor
validates and converts them to deterministic Rust; generated Rust must not be
edited directly.

`exact_forms.tsv` currently contains normative grammar-table forms, not corpus
attestation. Its `source_kind` therefore remains `normative-table`, and runtime
provenance labels those forms as normative generations. Future passage-backed
rows use a distinct `synodal-attestation` kind and an edition/passage evidence ID.

The alignment registry never imports an OCS surface cell. It connects stable
lexeme identities and records morphology and semantics independently.

Dictionary glosses sourced from the mixed historical D'yachenko dictionary are
admitted only with `reviewed-with-synodal-corpus` status; that status records a
target-usage check and does not turn the mixed dictionary into a Synodal
orthographic authority. Examples used as source evidence are never part of a
held-out evaluation partition.
