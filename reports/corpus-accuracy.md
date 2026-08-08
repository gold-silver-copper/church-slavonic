# Attested verb corpus accuracy

All listed inputs are external and every pinned file hash was verified before evaluation:

- UD Old Church Slavonic PROIEL `r2.18` at `64eddf87abfaa51e7f5acf0bef1bebcdaca1559f` (CC BY-NC-SA 4.0).
- Syntacticus native PROIEL/TOROT Old Church Slavonic selection `20230428` at `525cee4fb40590d7d514376c11acaed1bdd91c15` (CC BY-NC-SA 4.0).

UD finite `Tense=Past` is deliberately excluded because it does not distinguish aorist from imperfect. `Aspect` is never used as a substitute.

## 1. Facade attested-token recall (UD)

This asks whether an unambiguous known dictionary lexeme/cell exposes the attested variant through the public table-first facade. It is not productive-rule accuracy.

| Stage | Tokens |
|---|---:|
| all corpus tokens | 198843 |
| verb/AUX tokens | 47241 |
| losslessly compatible bundles | 28773 |
| unambiguous/valid lemma matches | 18712 |
| sufficient lexical metadata | 18712 |
| generation attempts | 18712 |
| attempts returning forms | 8715 |
| diplomatic top-1 / any | 3810 / 3811 |
| project-lookup top-1 / any | 3908 / 3909 |

### Fixed holdouts

| Partition | Eligible | Attempted | Returned | Raw top-1 | Raw any | Lookup top-1 | Lookup any |
|---|---:|---:|---:|---:|---:|---:|---:|
| lemma development | 23754 | 15287 | 6952 | 2798/15287 (18.30%) | 2798/15287 (18.30%) | 2848/15287 (18.63%) | 2848/15287 (18.63%) |
| lemma final holdout | 5019 | 3425 | 1763 | 1012/3425 (29.55%) | 1013/3425 (29.58%) | 1060/3425 (30.95%) | 1061/3425 (30.98%) |
| document development | 22679 | 14748 | 6841 | 3014/14748 (20.44%) | 3014/14748 (20.44%) | 3079/14748 (20.88%) | 3079/14748 (20.88%) |
| document holdout | 6094 | 3964 | 1874 | 796/3964 (20.08%) | 797/3964 (20.11%) | 829/3964 (20.91%) | 830/3964 (20.94%) |

### By verb category

| Category | Eligible | Attempted | Returned | Raw top-1 | Raw any | Lookup top-1 | Lookup any |
|---|---:|---:|---:|---:|---:|---:|---:|
| imperative | 3245 | 2014 | 1157 | 553/2014 (27.46%) | 553/2014 (27.46%) | 610/2014 (30.29%) | 610/2014 (30.29%) |
| infinitive | 3366 | 2071 | 2071 | 1233/2071 (59.54%) | 1233/2071 (59.54%) | 1247/2071 (60.21%) | 1247/2071 (60.21%) |
| l-participle | 841 | 461 | 461 | 224/461 (48.59%) | 224/461 (48.59%) | 232/461 (50.33%) | 232/461 (50.33%) |
| past-active-participle | 3743 | 2466 | 0 | 0/2466 (0.00%) | 0/2466 (0.00%) | 0/2466 (0.00%) | 0/2466 (0.00%) |
| past-passive-participle | 1195 | 489 | 0 | 0/489 (0.00%) | 0/489 (0.00%) | 0/489 (0.00%) | 0/489 (0.00%) |
| present | 12277 | 8364 | 4948 | 1747/8364 (20.89%) | 1748/8364 (20.90%) | 1765/8364 (21.10%) | 1766/8364 (21.11%) |
| present-active-participle | 3615 | 2587 | 0 | 0/2587 (0.00%) | 0/2587 (0.00%) | 0/2587 (0.00%) | 0/2587 (0.00%) |
| present-passive-participle | 356 | 182 | 0 | 0/182 (0.00%) | 0/182 (0.00%) | 0/182 (0.00%) | 0/182 (0.00%) |
| supine | 135 | 78 | 78 | 53/78 (67.95%) | 53/78 (67.95%) | 54/78 (69.23%) | 54/78 (69.23%) |

### By public generation path

| Path | Eligible | Attempted | Returned | Raw top-1 | Raw any | Lookup top-1 | Lookup any |
|---|---:|---:|---:|---:|---:|---:|---:|
| dictionary-table | 8715 | 8715 | 8715 | 3810/8715 (43.72%) | 3811/8715 (43.73%) | 3908/8715 (44.84%) | 3909/8715 (44.85%) |

### Facade skip and incompatibility reasons

- `ambiguous-lemma`: 461
- `incompatible-finite-mood`: 138
- `incompatible-finite-tense`: 550
- `incompatible-finite-voice`: 1
- `incompatible-negative-form`: 252
- `incompatible-participle-gender`: 100
- `incompatible-participle-kind`: 14
- `incompatible-past-subtype`: 14308
- `missing-participle-gender`: 4
- `missing-participle-variant`: 3036
- `missing-resultative-variant`: 1
- `missing-verb-form`: 55
- `no-public-form`: 9997
- `unknown-lemma`: 9600
- `unsupported-imperative-cell`: 9

## 2. Core generalization with declared principal parts (native PROIEL/TOROT)

oracle principal part derived from one morphologically diagnostic token of the same lemma; every token in the source person-number cell is excluded; no target surface is consulted during generation. This is explicitly an oracle-metadata result, not end-to-end lemmatization or class induction.

| Stage | Tokens |
|---|---:|
| all corpus tokens | 200761 |
| verb/AUX tokens | 47581 |
| losslessly compatible bundles | 14393 |
| unambiguous/valid lemma matches | 14393 |
| sufficient lexical metadata | 5678 |
| generation attempts | 4368 |
| attempts returning forms | 4368 |
| diplomatic top-1 / any | 1971 / 1971 |
| project-lookup top-1 / any | 2058 / 2058 |

### Aggregate and category results

| Slice | Eligible | Attempted | Returned | Raw top-1 | Raw any | Lookup top-1 | Lookup any |
|---|---:|---:|---:|---:|---:|---:|---:|
| all native oracle cells | 14393 | 4368 | 4368 | 1971/4368 (45.12%) | 1971/4368 (45.12%) | 2058/4368 (47.12%) | 2058/4368 (47.12%) |
| aorist-new | 11440 | 2643 | 2643 | 1607/2643 (60.80%) | 1607/2643 (60.80%) | 1682/2643 (63.64%) | 1682/2643 (63.64%) |
| imperfect | 2953 | 1725 | 1725 | 364/1725 (21.10%) | 364/1725 (21.10%) | 376/1725 (21.80%) | 376/1725 (21.80%) |

### By independently declared formation

| Formation | Eligible | Attempted | Returned | Raw top-1 | Raw any | Lookup top-1 | Lookup any |
|---|---:|---:|---:|---:|---:|---:|---:|
| aorist-new-ox | 2947 | 2643 | 2643 | 1607/2643 (60.80%) | 1607/2643 (60.80%) | 1682/2643 (63.64%) | 1682/2643 (63.64%) |
| imperfect-a-explicit-base | 1217 | 656 | 656 | 190/656 (28.96%) | 190/656 (28.96%) | 197/656 (30.03%) | 197/656 (30.03%) |
| imperfect-yat-a | 1514 | 1069 | 1069 | 174/1069 (16.28%) | 174/1069 (16.28%) | 179/1069 (16.74%) | 179/1069 (16.74%) |
| unknown | 8715 | 0 | 0 | - | - | - | - |

### Native skip and incompatibility reasons

- `native-incomplete-person`: 16588
- `native-metadata-source-cell-excluded`: 1310
- `native-missing-safe-oracle-metadata`: 8715
- `native-not-imperfect-or-aorist`: 16599
- `native-not-indicative`: 1

## 3. Lemma-disjoint OOV view (native oracle metadata)

same native-corpus oracle principal-part policy as core generalization; lemmas are assigned wholly to FNV-1a development or final holdout. The final partition was frozen by the shared hash rule before rule tuning.

| Partition | Eligible | Attempted | Returned | Raw top-1 | Raw any | Lookup top-1 | Lookup any |
|---|---:|---:|---:|---:|---:|---:|---:|
| lemma development | 12033 | 3745 | 3745 | 1647/3745 (43.98%) | 1647/3745 (43.98%) | 1717/3745 (45.85%) | 1717/3745 (45.85%) |
| lemma final holdout | 2360 | 623 | 623 | 324/623 (52.01%) | 324/623 (52.01%) | 341/623 (54.74%) | 341/623 (54.74%) |
| document development | 13423 | 4024 | 4024 | 1813/4024 (45.05%) | 1813/4024 (45.05%) | 1892/4024 (47.02%) | 1892/4024 (47.02%) |
| document holdout | 970 | 344 | 344 | 158/344 (45.93%) | 158/344 (45.93%) | 166/344 (48.26%) | 166/344 (48.26%) |

The morphology-normalized diagnostic is disabled: no independently audited lossless fold exists. Detailed token mismatches are emitted only with `--details PATH` (the native file receives a `-native` suffix) and must not be committed for these CC BY-NC-SA sources.
