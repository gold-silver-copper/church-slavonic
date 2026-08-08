# Verb source-shape audit

Audit date: 2026-08-07. Input: the exact 46,091,411-byte Kaikki OCS JSONL pinned in
`data/SOURCES.toml`, SHA-256
`5bd61e747aa7aeb677af92b4e32c65476e5c6ee74bff146269460c962be5456c`.
The downloaded audit copy matched the committed hash.

## Stratified source review

The raw sample included one page for every extracted verb class plus an
unclassified page containing all four declined participle blocks:

| Shape | Page | Page revision | Governing template revision |
|---|---|---:|---:|
| IA1 | `пасти` | 89809244 | `Template:cu-conj-IA1` 89441241 |
| IA2 | `зъвати` | 89753320 | `Template:cu-conj-IA2` 89324151 |
| II1 | `таити` | 88093766 | `Template:cu-conj-II1` 89715230 |
| II2 | `врьтѣти` | 89259113 | `Template:cu-conj-II2` 89715250 |
| II3 | `боꙗти` | 88093695 | `Template:cu-conj-II3` 89324162 |
| four declined participle blocks | `ковати` | 89390101 | `Template:cu-conj` 90264442; `Template:l-self` 84474956 |

The revision timestamps precede the pinned Wiktionary dump. For each page, the OCS
page wikitext, conjugation invocation, rendered/expanded JSON rows, ordered sentinel
rows, and extractor feature proposal were compared.

Raw class populations and hazards were also counted across all verb entries:

| Raw class marker | Verb entries | Entries with declined cells | Declined rows | Entries with `error-unrecognized-form` | Error rows |
|---|---:|---:|---:|---:|---:|
| IA1 | 105 | 0 | 0 | 85 | 1,356 |
| IA2 | 3 | 0 | 0 | 3 | 30 |
| II1 | 48 | 0 | 0 | 48 | 480 |
| II2 | 28 | 0 | 0 | 28 | 280 |
| II3 | 4 | 0 | 0 | 4 | 40 |
| `o-stem` | 4 | 4 | 84 | 0 | 0 |
| unclassified | 1,092 | 568 | 158,985 | 520 | 17,390 |

Raw totals are 159,069 case-tagged verb rows and 19,576
`error-unrecognized-form` rows. They are larger than the extractor's final drop
counts because earlier atomic guards remove form-of entries, invalid words,
sentinels, and duplicate records before verb-shape accounting.

## Findings

- Finite present, imperfect, and imperative rows frequently also carry the unrelated
  `l-participle` tag. Complete tense/person/number remains sufficient to identify a
  finite cell; the spurious tag is ignored only in that complete context.
- The following aorist-looking block loses its tense heading and is emitted only as
  `error-unrecognized-form` plus person/number and the spurious l-participle tag.
  Rejecting it is correct; removing the error guard would turn a broken heading into
  an invented analysis.
- On large pages such as `ковати`, four declined participle paradigms are emitted in
  a stable order, separated by generic `table-tags`/`l-self` rows. Individual case
  rows retain case, number, gender, and short/long form but do not retain the
  present/past and active/passive block identity. Some sentinels say `present`, while
  other block sentinels say only `no-table-tags`; row-local tags are therefore
  insufficient.
- Citation participles precede those blocks and retain tense/voice. They remain safe
  citation cells. The extractor previously checked the `present` tag as if it were
  finite before checking `active/passive`, so 325 complete present-active/passive
  citations were wrongly rejected for lacking person. Voice-first citation mapping
  now admits those rows and fixture-tests reordered, duplicated, contradictory, and
  declined signatures. Using citation sequence to label hundreds of later rows
  would still require a separately versioned positional block parser with atomic
  shape validation; no such parser is introduced here.
- Reordered, duplicated, missing-heading, and `error-unrecognized-form` synthetic
  fixtures continue to fail closed. Existing finite, sentinel, and malformed-row
  fixtures passed unchanged.

## Before/after extraction accounting

Only the independently complete present participle citations were re-admitted. No
declined row or `error-unrecognized-form` row was admitted and no guard was weakened.

| Category | Before | After | Change |
|---|---:|---:|---:|
| all accepted source variants | 137,081 | 137,406 | +325 |
| accepted finite verb cells | 3,945 | 3,945 | 0 |
| accepted l-participle cells | 6,408 | 6,408 | 0 |
| accepted other verb cells | 2,760 | 2,760 | 0 |
| accepted participle citations | 369 | 694 | +325 |
| rejected declined participle rows | 153,310 | 153,310 | 0 |
| rejected `error-unrecognized-form` rows | 17,912 | 17,912 | 0 |

Productive participle and past-tense capability is supplied by explicit core
metadata, not by relabeling ambiguous source rows. A future source-coverage change
must first pin and fixture-test every admitted positional block shape, reject an
unknown block atomically, and regenerate this accounting with reviewed witnesses.

The registry's established IDs are content-derived. Adding citation features changed
the IDs of 191 affected verb records (191 removed IDs and 191 replacements) without
adding or removing lexemes. Alias targets and generated tables were regenerated
atomically, and complete by-ID/public round-trip checks cover the replacement set.
