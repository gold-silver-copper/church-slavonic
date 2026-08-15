# Synodal and inherited source inventory

The machine-readable download inventory is
[`references/SOURCES.toml`](../references/SOURCES.toml); cached bytes live under
the gitignored `references/downloads/`, and every local artifact is covered by
[`references/SHA256SUMS`](../references/SHA256SUMS). The active OCS extraction pin
remains in [`data/SOURCES.toml`](../data/SOURCES.toml). A source is one lineage
witness even when it has several download or presentation layers.

## Authority policy

Sources are classified independently by recension, authority role, epistemic
role, redistribution status, and lineage. "Machine readable" does not mean
"normative." Unknown and mixed-recension rows cannot enter the Synodal exact-form
registry. Noncommercial, evaluation-only, and unresolved-license bytes remain
local and are excluded from crates.io packages.

Direct target evidence outranks inherited OCS evidence. Corpus frequency never
overrides a normative grammatical source by itself. Conflicts are recorded rather
than averaged away.

## Downloaded sources

| ID | Recension and role | Pin/checksum | License and use |
|---|---|---|---|
| `unicode-tn41-revision-1` | mixed-recension encoding, typography, collation, numeral authority | version 1 (2015-11-04); aggregate `4514cbb8…` | Unicode Terms of Use; local normative research copy |
| `english-wiktionary-ocs-kaikki-2026-08-07` | OCS lexical/paradigm evidence | 2026-08-07; `fb20336e…` | CC BY-SA 4.0 distribution; inherited candidates only |
| `english-wiktionary-ocs-lineage-2026-08-07` | mixed raw dump/templates/modules lineage | enwiktionary 20260801, Wiktextract `d9fa233`; aggregate `601ccbc…` | CC BY-SA/GFDL; same witness as Kaikki |
| `polivanova-osd-source` | OCS grammar/root dictionary | 2020-01-10; `f412042a…` | exact spreadsheet notice unresolved; local research |
| `polivanova-fup-2023` | OCS scholarly grammar TEI XML and PDF | 2023 XML/PDF eISBNs; aggregate `8838c88d…` | CC BY 4.0; same scholarly lineage as source spreadsheet |
| `ud-ocs-proiel-r2.18` | OCS morphology/evaluation | commit `64eddf8…`; `579b20ed…` | CC BY-NC-SA 4.0; optional local evaluation |
| `syntacticus-20230428` | OCS native PROIEL/TOROT | commit `525cee4…`; `e328440…` | CC BY-NC-SA 4.0; same witness lineage as UD |
| `ccmh-2021-04-23` | OCS corpus/counterexamples | 2021-04-23; aggregate `67a4bcc…` | version terms unresolved; evaluation only |
| `diacu-1.0` | mixed diachronic contamination set | commit `d4b00ba…`; `43d50771…` | no clear data license; quarantine/evaluation only |
| `ponomar-elizabeth-bible-2026-08-09` | target Bible spelling/accent/evaluation | commit `0af645f…`; `86c5e584…` | repository GPL-3.0-or-later; raw text local pending file-level audit |
| `crosswire-csl-elizabeth-1.5.2` | target/modernized Elizabeth Bible contrast | 1.5.2; `96705c57…` | public domain module; modernized spelling is not exact print authority |
| `wikisource-church-slavonic-bible-2026-08-09` | target community Bible transcription | 78 exact page revisions; aggregate `ba522409…` | CC BY-SA 4.0; compare with named editions and lineage fingerprints |
| `ponomar-library-catalog-2026-08-09` | target liturgical-genre discovery | 2026-08-09; aggregate `900e368…` | catalog terms allow share-alike, but each edition needs a rights audit |
| `ponomar-modern-church-slavonic-corpus-2016` | mixed modern corpus/dictionary/frequency | component dates 2014–2016; aggregate `d2f78135…` | per-work lineage unresolved; local evaluation and discovery |
| `alypy-gamanovich-grammar-web-2023` | target grammar, accent, orthography, numerals | corrected through 2023-12-10; 198 HTML files; aggregate `f140bcc4…` | preserve Ponomar/edition notices; normative research anchor |
| `dyachenko-1900-scan` | mixed historical lexical/semantic dictionary | Wikimedia revision 2019-06-11 plus pinned tessdata_fast 4.1.0 Russian model; aggregate `64069228…` | public-domain scan; OCR candidates require image and target-corpus review |

Full hashes, URLs, formats, upstream lineages, and paths are in the machine
manifest; abbreviations above are only for readability.

## Metadata-only or pending sources

- Gorshkov's 2002 OCS grammar and Leuta–Havryliuk's 2018 university
  grammar were reviewed through stable full-text hosts for the compound-ordinal
  contract. Their copyrighted bytes are not committed; only bibliography,
  short rule references, conflicts, and source-linked tests are retained.
- GORAZD has no confirmed bulk export mechanism or database redistribution terms,
  so it is used only for manual review and discovery.
- The Russian National Corpus has no authorized bulk download in this workflow;
  it remains a query/discovery and held-out check source.
- Ponomar Library individual editions are not mirrored until work-level rights
  and transcription lineage are recorded.

## Normalization and partitions

Source-specific adapters must write raw, normalized, quarantine, and report
layers. The common contract is
[`data/normalization/README.md`](../data/normalization/README.md). Source and
evaluation partitions are passage-disjoint. Any passage used for a lexeme,
principal part, mapping, rule, or exception is excluded from held-out evaluation
for that decision.

The Elizabeth Bible hosts may share an electronic transcription. Verse-level
fingerprints and editorial-error comparison are required before their agreement
can count as independent evidence. Kaikki/raw Wiktextract/Wiktionary and
UD/Syntacticus likewise remain explicitly single lineages.

The dated `enwiktionary-20260801` XML, extraction revision, templates, modules,
and raw Wiktextract output are locked as the reproducible Wiktionary lineage.
The raw all-language JSONL does not expose the OCS records as top-level `cu`
rows until Kaikki's language post-processing stage, so the adapter consumes the
separately checksum-locked OCS JSONL from that same extraction. It never follows
an unlocked `latest` result: any changed bytes fail ordinary fetch and require an
explicit reviewed refresh.

## Executable adapter coverage

All locked, downloaded sources named below have command-line-accessible adapters
under `cargo xtask synodal-bootstrap`; a source-filtered offline run is useful for
diagnostics:

```sh
cargo xtask synodal-bootstrap --offline \
  --cache references/downloads \
  --source SOURCE_ID
```

| Source | Adapter output and evidential restriction |
|---|---|
| Ponomar Elizabeth Bible | all 78 `.text` files, ordered verse JSONL, markup-safe spelling, passage partitions, frequency table |
| Alypy/Gamanovich | all 198 locked pages, stable section records and every `DSText` witness; review candidates only |
| D’yachenko | 1,158-page embedded text or pinned-model OCR with page, bounding box, confidence, and uncorrected status; lexical candidates only |
| Wikisource Bible | exact-revision MediaWiki exports, reviewed 78-title canonical book map, Cyrillic chapter/verse parsing, deterministic wikitext stripping, and revision/template lineage |
| CrossWire CSlElizabeth | SWORD `mod2imp` verse export, canonical book/chapter/verse identity, version, and modernized-spelling label |
| Polivanova OSD/FUP | XLS and TEI records with their common scholarly lineage preserved |
| UD/Syntacticus | CoNLL-U and native CoNLL with shared PROIEL/TOROT lineage; inherited OCS/evaluation only |
| CCMH | supported text/XML historical witnesses; comparative evidence only |
| DIACU | JSON documents/control labels; period and contamination evaluation only |
| English Wiktionary/Kaikki | streaming OCS JSONL; inherited candidates only, never target surface rows |
| Ponomar modern corpus | locked frequency list for discovery and prioritization |

Automatic JSONL candidates, quarantine records, reviewed overlays, runtime facts,
and evaluation rows are separate layers. See `SYNODAL_DATA_PIPELINE.md` for the
complete lock/refresh and clean-reconstruction procedure.
