# Synodal v0.2 implementation audit

This audit maps `SYNODAL_DATA_PIPELINE_AND_COVERAGE_PROMPT.md` to the state found
at commit `859c6b1` and is updated as the v0.2 implementation proceeds. A green
unit test is not treated as proof of a broader requirement unless the test covers
the complete boundary named here.

## Baseline data flow

The v0.1 raw cache was populated by `references/fetch-sources.sh`. That script
contained source URLs directly, accepted any nonempty existing file, and rewrote
`SHA256SUMS` after an ordinary fetch. The curated `data/synodal/*.tsv` tables were
then compiled into static Rust registries by the extractor. There was no
executable raw-to-curated pipeline, and runtime crates correctly performed no
I/O.

## Baseline gaps and implementation evidence

| Area | Baseline finding | Required completion evidence |
|---|---|---|
| Artifact locking | Source-level manifest plus 244 checksums; URLs and sizes were not locked per artifact | Individual URL/path/size/format/hash lock, immutable fetch/verify behavior, drift tests |
| Mutable sources | Kaikki, Wikisource, CCMH, Ponomar, CrossWire, Unicode index, and Alypy used moving endpoints | Exact revision or content lock for every byte; explicit reviewed refresh |
| Wikisource | Current-page `Special:Export` request | Exact revision manifest and revision-addressed retrieval |
| Fetch prerequisites | Bash utilities were assumed | Checked dependencies with actionable diagnostics |
| Fetch integrity | Nonempty files were skipped; checksums were regenerated | Existing and downloaded bytes must match the committed lock before admission |
| Atomicity/resume | Partial files were used, but corruption and unsafe resume were not tested | HTTP fixture tests for atomic fetch, range resume, drift, and malformed content |
| Data layers | Curated TSV mixed generated facts and review decisions | Enforced raw, candidate, quarantine, reviewed, generated, and evaluation layers |
| Bootstrap | No end-to-end command | `cargo xtask synodal-bootstrap` with offline/cache/source controls and deterministic output |
| Source adapters | Only Ponomar line and Kaikki JSONL library adapters existed | Executable adapters for every source named by the prompt, with fixtures and reports |
| Evaluation | 11 token cells and one phrase | Passage- and lemma-disjoint real-data evaluation with provenance-specific metrics |
| Morphology | Useful seed systems, but broad gaps documented in `SYNODAL_MORPHOLOGY.md` | Corpus-prioritized reviewed growth without weakening Strict policy |
| CI | No `.github` workflows existed | Bounded fixture CI plus an explicitly manual full-source audit |

## Work completed after the baseline

- `references/SOURCE_LOCK.tsv` now records every artifact independently.
- Ordinary fetch and verification are lock-preserving Rust operations exposed as
  `cargo xtask synodal-sources`.
- Existing files are checked by size, SHA-256, and file signature.
- Downloads use partial files, atomic rename, upstream-drift rejection, and
  archive validation.
- Fetch, resume, wrong-checksum, unsafe-path, and HTML-error fixtures exist.
- All 198 Alypy pages are explicit locked artifacts.
- Wikisource now has 78 exact revisions in `WIKISOURCE_REVISIONS.tsv`, and each
  revision's wikitext is an independent locked artifact.

The rows above describe the baseline and are not themselves completion evidence;
the implemented state and verification evidence follow.

## Implemented completion state

The executable pipeline now closes the baseline architecture gaps:

- thirteen source adapters cover Ponomar, Alypy, D’yachenko, exact-revision
  Wikisource, CrossWire, both Polivanova layers, UD, Syntacticus, CCMH, DIACU,
  Kaikki OCS, and the modern Ponomar frequency list;
- automatic candidates and quarantine records are atomically written to local,
  gitignored layers, while committed reviews stay read-only;
- every reviewed evidence row names a stable candidate and a full bootstrap
  fails on an orphan or changed candidate;
- D’yachenko's scan has no embedded OCR, so the fallback renders exactly 1,158
  pages and uses the checksum-pinned `tessdata_fast` 4.1.0 Russian model;
- the default-CI fixture adapter has a golden SHA-256 and is byte-identical across
  independent directories; `synodal-fixture-bootstrap` also reconstructs both
  runtime registries twice and matches the committed files;
- bounded default CI, manual full-source CI, and a lock-preserving source
  availability audit are present; and
- the reviewed registry has grown from 16 to 61 lexemes and from 138 to 260 exact
  forms. Evaluation has grown from 11 to 38 passage-held-out cells: all 38
  expected forms occur in the returned top-k set, while 37/38 are top-1 in both
  expanded and printed profiles. The single difference is the corpus variant
  `три` behind the grammar-table primary `трїе`, and is reported explicitly.

This does not claim language-wide coverage. Remaining linguistic gaps are listed
in `SYNODAL_MORPHOLOGY.md`; they fail explicitly and do not block reproducible
reconstruction of the reviewed registry.
