# Attested-corpus verb evaluation

This evaluator challenges the production APIs against words in canonical Old
Church Slavonic manuscripts. It is observational: an unattested generated cell is
never counted wrong, and corpus frequency is never treated as paradigm completeness.

## Reproducible external inputs

`data/evaluation-sources.json` is the machine-checked authority. It pins:

- UD Old Church Slavonic PROIEL release `r2.18`, commit
  `64eddf87abfaa51e7f5acf0bef1bebcdaca1559f`, including its train/dev/test
  CoNLL-U hashes; and
- Syntacticus treebank data release `20230428`, commit
  `525cee4fb40590d7d514376c11acaed1bdd91c15`, selecting Codex Marianus,
  Suprasliensis, Euchologium Sinaiticum, the Kiev Missal, Psalterium Sinaiticum,
  and Zographensis native XML with a hash for each file.

Both inputs are CC BY-NC-SA 4.0 and remain external. The committed report contains
only aggregates. Local token rows, predicted spellings, and mismatch excerpts are
gitignored and must not be committed or packaged.

CCMH is not used in the current score. Its Helsinki catalogue advertises OCS XML
under CC BY 4.0, but this project has not yet audited an exact downloaded artifact,
file-level license, schema, encoding, and hash. No CCMH text or claimed regression
fixture is included until that audit is complete.

## Feature-compatibility matrix

| Source bundle | Project cell | Status |
|---|---|---|
| UD `VerbForm=Fin,Tense=Pres,Mood=Ind,Voice=Act,Person,Number` | present finite | lossless; fused `Polarity=Neg` forms are rejected |
| UD `VerbForm=Fin,Tense=Pres,Mood=Imp,Voice=Act,Person,Number` | imperative | lossless only for a typed historical imperative cell |
| UD `VerbForm=Inf` | infinitive | lossless |
| UD `VerbForm=Sup` | supine | lossless |
| UD `VerbForm=PartRes,Tense=Past,Voice=Act,Variant=Short,Case=Nom,Gender,Number` | l-participle | lossless after all fixed resultative dimensions are validated |
| UD `VerbForm=Part,Tense,Voice,Case,Number,Gender,Variant` | declined participle | lossless only when every value maps explicitly; masculine accusative tries both explicit animacy cells |
| UD finite `Tense=Past` | aorist or imperfect | **never mapped**: counted as `incompatible-past-subtype` |
| UD `Aspect=Perf/Imp` | lexical aspect | never used as a tense subtype |
| native PROIEL person + number + tense `i` + mood `i` + voice `a` | imperfect finite | lossless |
| native PROIEL person + number + tense `a` + mood `i` + voice `a` | aorist finite | lossless |
| incomplete, uncertain, contradictory, or other native bundles | none | rejected with an exact reason |

The native ten-position mapper is fixture-tested. Hash verification plus required
schema markers makes an unreviewed upstream schema change fail closed.

## Four separate questions

1. **Exact dictionary-cell recall** verifies that every normalized source cell and
   ordered variant round-trips through the public table resolver.
2. **Core generalization with declared principal parts** uses native imperfect and
   aorist labels. One morphologically diagnostic token may supply an oracle stem and
   formation. Every token in that person-number source cell is excluded, and the
   production core generates the remaining observations. Results are labeled
   `oracle-metadata`, not end-to-end accuracy.
3. **True OOV view** partitions those oracle-metadata lexemes by the normalized
   lemma. It measures the same production rule on a lemma-disjoint final group and
   states that the metadata came from another native corpus token. It does not claim
   automatic principal-part discovery for a wholly unseen lexeme.
4. **Dictionary-metadata held-cell generation** removes the dictionary target,
   equivalent 2sg/3sg cells, and every same-spelling feature before rebuilding
   principal parts. It calls the public metadata resolver and reports metadata
   construction separately from generation. Exact-table hits cannot enter its
   numerator.

The UD **facade real-text recall** view is an additional observation over question 1
and question 4 paths: it calls the ordinary table-first public facade and slices
exact-table versus dictionary-metadata results. It uses only dictionary/curated
lexical evidence, never a corpus-derived principal part. Because dictionary metadata
is not rebuilt separately for each corpus token, this view is not labeled a held-cell
score.

No evaluator reimplements endings. Dictionary metadata is decoded through
`DictionaryVerbMetadata` and uses the public metadata resolver; facade evaluation
calls `form_by_id`; core/OOV evaluation constructs the typed `VerbLexeme` and calls
the production core.

## Leakage controls and partitions

- The normalized lemma key is hashed with 64-bit FNV-1a. Modulo-five residue `0` is
  the frozen final holdout; residues `1–4` are development.
- All homographs sharing that key remain in one lemma partition.
- The same function partitions complete manuscript/document labels for a separate
  document-held-out view.
- Official UD train/dev/test labels remain in the pinned file manifest but are not
  presented as lemma-disjoint OOV.
- Native metadata source priority is fixed: 1sg, 1du, 1pl, 3pl, 3du, 2pl, 2du,
  2sg, then 3sg. Editorially plain forms precede marked forms within the same cell.
- A diagnostic native source cell is excluded across every occurrence and document,
  not merely the individual token used.
- Dictionary held-cell evaluation applies exclusions before derivation or
  cross-checking. It rejects ambiguous lemma keys and never loads curated overrides;
  the current `бꙑти` override is development-only and has separate public-API
  provenance tests.
- The final holdout must not drive an override. A future override learned from it
  requires moving the lemma to development and transparently resetting the baseline.

## Scoring and denominators

Diplomatic exact is byte-for-byte equality. Project lookup exact uses only the
runtime's shared NFC plus Unicode-lowercase key. No morphology-normalized or lossy
orthographic score is enabled. Top-1 and any-returned-variant are separate; they are
identical for the current single-output productive core but not for dictionary cells.

Each section reports all tokens, verb/AUX tokens, compatible bundles, unambiguous or
valid lemmas, sufficient metadata, attempts, returned forms, and exact skip reasons.
It also records conditional correctness and coverage by category, complete cell,
document, lemma frequency, and declared native formation in JSON. The committed
human report keeps the main slices compact.

The corpus manifest freezes conservative non-regression floors in basis points.
Current floors are 65% facade attempt coverage, 20% facade lookup-any conditional
accuracy, 30% native oracle attempt coverage, and 47% native oracle lookup-any
conditional accuracy. The dictionary-metadata evaluator separately requires at
least 30% development and 35% final metadata availability among unambiguous targets,
plus 95% lookup-any conditional correctness in each partition. These are guardrails,
not desired linguistic accuracy claims.

On the current dictionary snapshot, development finds and returns metadata for
2,784/8,628 unambiguous compatible targets and matches 2,690; the frozen final
partition returns 997/2,564 and matches 964. These are held-dictionary-cell results,
not corpus scores. Citation participles are absent from this numerator because
removing their only safely typed citation also removes their formation selector.

## Running the evaluation

```bash
cargo xtask accuracy
cargo xtask accuracy-corpus \
  --ud /path/to/UD_Old_Church_Slavonic-PROIEL \
  --syntacticus /path/to/syntacticus-treebank-data
```

Add `--write` only to refresh aggregate committed reports; it requires both pinned
corpus paths so a partial diagnostic cannot replace the complete report. Add
`--details reports/corpus-details.tsv` for local debugging; the native detail file
gets a `-native.tsv` suffix. A missing file, hash mismatch, license mismatch, schema
mismatch, or non-regression failure aborts. Ordinary tests never access the network
or these local corpora.

## Clean baseline and current result

Before the productive verb expansion, the UD-only evaluator reported 29,036
compatible bundles, 19,432 attempts, 4,153 diplomatic-any matches, and 4,268
lookup-any matches. That first evaluator unioned ambiguous lemma candidates and did
not distinguish top-1 from any, so it is retained as an audit baseline rather than
compared as if its denominator were identical.

The audited schema-2 facade excludes ambiguous lemmas and feature bundles that lose
polarity, finite voice, fixed resultative dimensions, or the typed imperative
inventory. Before dictionary metadata was connected, 18,712 attempts returned only
8,715 exact-table results, with 3,811 diplomatic-any and 3,909 lookup-any matches.
The same attempts now return 11,063 forms: 4,711 diplomatic-any and 4,850 lookup-any.
The 2,348 additional returns are sliced by generation path in the committed report;
external token rows remain local. The native oracle evaluator sees
14,393 compatible imperfect/aorist tokens; safe oracle metadata permits 4,368
non-source-cell attempts, with 1,971 diplomatic and 2,058 lookup matches. The
lemma-disjoint final view is 324/623 diplomatic and 341/623 lookup. Full category,
formation, document, and skip counts are in `reports/corpus-accuracy.json` and
`reports/corpus-accuracy.md`.

Low native conditional accuracy is retained rather than normalized away. It exposes
manuscript spelling, contraction, suppletion, formation ambiguity, and the limits of
using one corpus principal part. Those are targets for later independently specified
metadata and variant work, not permission to guess.
