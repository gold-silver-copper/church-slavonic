# Source normalization contract

Every source adapter is deterministic and records:

- source ID, immutable revision, input SHA-256, adapter version, and invocation;
- source recension, work, edition, passage, and upstream transcription lineage;
- Unicode changes as an ordered list rather than overwriting the raw spelling;
- markup removals and token-boundary decisions;
- lexical, grammatical, orthographic, accentual, and epistemic authority roles;
- accepted, quarantined, and rejected row counts with reason codes;
- source/training/evaluation partition assignment; and
- SHA-256 values for every normalized output.

The raw layer is immutable and gitignored. Reviewable normalized TSV/JSON is
written before generated Rust. An adapter first writes a temporary directory,
validates all files and failure ceilings, then atomically replaces the output.

Required normalized fields include stable source record ID, raw spelling,
expanded spelling when known, normalized lookup key, source and target recension,
work/edition/passage, part of speech, typed grammatical cell, evidence kind,
authority roles, source license, lineage IDs, and parse status.

Rows with unknown recension, malformed Unicode, missing passage identity, or
incompatible licensing never enter runtime target registries. They remain in a
counted quarantine report. OCS rows remain in an OCS registry; only explicit
mapping records connect them to Synodal lexemes.

The initial adapter manifests are:

- `wiktionary-ocs.toml`: inherited dictionary candidates;
- `synodal-corpora.toml`: Ponomar, CrossWire, and Wikisource target text;
- `ocs-corpora.toml`: Syntacticus/UD/CCMH historical evidence; and
- `lexical-references.toml`: Polivanova and D'yachenko candidate extraction.
