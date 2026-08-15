# Accuracy

Dictionary round-trip and OOV prediction are separate measurements.

The OOV split is lemma-level: 64-bit FNV-1a of the shared normalized lemma key, modulo 5. Residue 0 is the fixed held-out final-evaluation partition; residues 1-4 are development. Homographs and parts of speech sharing a lemma key therefore cannot cross partitions. The held-out partition is deterministic, not cryptographically sealed, and must not be used for rule tuning.

## Dictionary registry round-trip

| Metric | Value |
|---|---:|
| lexemes | 3081 |
| cells | 134761 |
| variants | 137406 |
| reachable variants | 137406 / 137406 |
| exact variant-order cells | 134761 / 134761 |
| primary-correct cells | 134761 / 134761 |
| ambiguous bare lemma/POS pairs | 112 |
| complete dictionary paradigm key sets | 3081 / 3081 |

Cells by public provenance:

- `dictionary-table`: 134761

## Leakage-controlled dictionary-metadata generation

This primary fallback score removes the target feature, an equivalent 2sg/3sg finite or imperative feature, and every same-spelling dictionary feature before rebuilding metadata. It then calls the public dictionary-metadata resolver; exact table lookup is unavailable to this path. Development and final lemmas use the same frozen modulo-five partition as OOV.

Source dictionary verb lexemes: 711.

### Metadata coverage by field

| Field or declared value | Lexemes |
|---|---:|
| `aorist/formation` | 0 |
| `aorist/formation=asigmatic` | 0 |
| `aorist/formation=new` | 0 |
| `aorist/formation=sigmatic-primary` | 0 |
| `aorist/formation=sigmatic-secondary` | 0 |
| `aorist/formation=sigmatic-vowel` | 0 |
| `aorist/second-third-singular` | 0 |
| `aorist/stem` | 0 |
| `aspect/aspect` | 647 |
| `aspect/aspect=biaspectual` | 9 |
| `aspect/aspect=imperfective` | 374 |
| `aspect/aspect=perfective` | 264 |
| `imperative/formation` | 73 |
| `imperative/formation=i-series` | 73 |
| `imperative/formation=yat-series` | 0 |
| `imperative/stem` | 73 |
| `imperfect/formation` | 150 |
| `imperfect/formation=a` | 44 |
| `imperfect/formation=palatalized-a` | 0 |
| `imperfect/formation=present-a` | 0 |
| `imperfect/formation=present-yat-a` | 0 |
| `imperfect/formation=yat-a` | 106 |
| `imperfect/stem` | 150 |
| `imperfect/variant-policy` | 150 |
| `imperfect/variant-policy=contracted-only` | 0 |
| `imperfect/variant-policy=iotated-only` | 0 |
| `imperfect/variant-policy=uncontracted-only` | 150 |
| `l-participle/stem` | 185 |
| `past-active-participle/formation` | 185 |
| `past-active-participle/formation=ish` | 44 |
| `past-active-participle/formation=ush` | 96 |
| `past-active-participle/formation=vush` | 89 |
| `past-active-participle/formation=vush-after-j-deletion` | 0 |
| `past-active-participle/formation=vush-after-ov-to-u` | 0 |
| `past-active-participle/stem` | 185 |
| `past-passive-participle/formation` | 134 |
| `past-passive-participle/formation=en` | 67 |
| `past-passive-participle/formation=n` | 51 |
| `past-passive-participle/formation=t` | 16 |
| `past-passive-participle/stem` | 134 |
| `present-active-participle/formation` | 186 |
| `present-active-participle/formation=yesht-soft` | 78 |
| `present-active-participle/formation=yusht-hard` | 108 |
| `present-active-participle/formation=yusht-soft` | 0 |
| `present-active-participle/stem` | 186 |
| `present-passive-participle/formation` | 134 |
| `present-passive-participle/formation=em` | 0 |
| `present-passive-participle/formation=im` | 62 |
| `present-passive-participle/formation=iotated-em` | 0 |
| `present-passive-participle/formation=om` | 72 |
| `present-passive-participle/stem` | 134 |
| `present/class` | 121 |
| `present/class=IA1` | 79 |
| `present/class=IA2` | 2 |
| `present/class=II1` | 26 |
| `present/class=II2` | 11 |
| `present/class=II3` | 3 |
| `present/first-singular-stem` | 121 |
| `present/stem` | 121 |

### Held-cell stage funnel

| Stage | Development | Final holdout |
|---|---:|---:|
| compatible requested cells | 8895 | 2748 |
| unambiguous lexeme cells | 8628 | 2564 |
| metadata records found | 2784 | 997 |
| metadata records validated | 2784 | 997 |
| generation attempts | 2784 | 997 |
| returned forms | 2784 | 997 |
| diplomatic top-1 correct | 2690 | 964 |
| diplomatic any correct | 2690 | 964 |
| project-lookup top-1 correct | 2690 | 964 |
| project-lookup any correct | 2690 | 964 |

The slice tables below report diplomatic-any in `Exact` and shared NFC/lowercase-any in `NFC/lowercase`; top-1 remains separate in the funnel.

#### Development by system

| Slice | Exact | NFC/lowercase | Returned |
|---|---:|---:|---:|
| `imperative` | 220 | 220 | 293 |
| `imperfect` | 824 | 824 | 824 |
| `l-participle` | 1048 | 1048 | 1048 |
| `present` | 598 | 598 | 619 |

#### Final holdout by system

| Slice | Exact | NFC/lowercase | Returned |
|---|---:|---:|---:|
| `imperative` | 64 | 64 | 91 |
| `imperfect` | 296 | 296 | 296 |
| `l-participle` | 352 | 352 | 352 |
| `present` | 252 | 252 | 258 |

#### Development by complete cell

| Slice | Exact | NFC/lowercase | Returned |
|---|---:|---:|---:|
| `verb:finite:imperfect:1:du` | 103 | 103 | 103 |
| `verb:finite:imperfect:1:pl` | 103 | 103 | 103 |
| `verb:finite:imperfect:2:du` | 103 | 103 | 103 |
| `verb:finite:imperfect:2:pl` | 103 | 103 | 103 |
| `verb:finite:imperfect:2:sg` | 103 | 103 | 103 |
| `verb:finite:imperfect:3:du` | 103 | 103 | 103 |
| `verb:finite:imperfect:3:pl` | 103 | 103 | 103 |
| `verb:finite:imperfect:3:sg` | 103 | 103 | 103 |
| `verb:finite:present:1:du` | 78 | 78 | 78 |
| `verb:finite:present:1:pl` | 78 | 78 | 78 |
| `verb:finite:present:1:sg` | 52 | 52 | 52 |
| `verb:finite:present:2:du` | 78 | 78 | 78 |
| `verb:finite:present:2:pl` | 78 | 78 | 78 |
| `verb:finite:present:3:du` | 78 | 78 | 78 |
| `verb:finite:present:3:pl` | 78 | 78 | 99 |
| `verb:finite:present:3:sg` | 78 | 78 | 78 |
| `verb:imperative:1:du` | 55 | 55 | 128 |
| `verb:imperative:1:pl` | 55 | 55 | 55 |
| `verb:imperative:2:du` | 55 | 55 | 55 |
| `verb:imperative:2:pl` | 55 | 55 | 55 |
| `verb:l-participle:f:du` | 131 | 131 | 131 |
| `verb:l-participle:f:pl` | 131 | 131 | 131 |
| `verb:l-participle:f:sg` | 131 | 131 | 131 |
| `verb:l-participle:m:du` | 131 | 131 | 131 |
| `verb:l-participle:m:pl` | 131 | 131 | 131 |
| `verb:l-participle:n:du` | 131 | 131 | 131 |
| `verb:l-participle:n:pl` | 131 | 131 | 131 |
| `verb:l-participle:n:sg` | 131 | 131 | 131 |

#### Final holdout by complete cell

| Slice | Exact | NFC/lowercase | Returned |
|---|---:|---:|---:|
| `verb:finite:imperfect:1:du` | 37 | 37 | 37 |
| `verb:finite:imperfect:1:pl` | 37 | 37 | 37 |
| `verb:finite:imperfect:2:du` | 37 | 37 | 37 |
| `verb:finite:imperfect:2:pl` | 37 | 37 | 37 |
| `verb:finite:imperfect:2:sg` | 37 | 37 | 37 |
| `verb:finite:imperfect:3:du` | 37 | 37 | 37 |
| `verb:finite:imperfect:3:pl` | 37 | 37 | 37 |
| `verb:finite:imperfect:3:sg` | 37 | 37 | 37 |
| `verb:finite:present:1:du` | 33 | 33 | 33 |
| `verb:finite:present:1:pl` | 33 | 33 | 33 |
| `verb:finite:present:1:sg` | 21 | 21 | 21 |
| `verb:finite:present:2:du` | 33 | 33 | 33 |
| `verb:finite:present:2:pl` | 33 | 33 | 33 |
| `verb:finite:present:3:du` | 33 | 33 | 33 |
| `verb:finite:present:3:pl` | 33 | 33 | 39 |
| `verb:finite:present:3:sg` | 33 | 33 | 33 |
| `verb:imperative:1:du` | 16 | 16 | 43 |
| `verb:imperative:1:pl` | 16 | 16 | 16 |
| `verb:imperative:2:du` | 16 | 16 | 16 |
| `verb:imperative:2:pl` | 16 | 16 | 16 |
| `verb:l-participle:f:du` | 44 | 44 | 44 |
| `verb:l-participle:f:pl` | 44 | 44 | 44 |
| `verb:l-participle:f:sg` | 44 | 44 | 44 |
| `verb:l-participle:m:du` | 44 | 44 | 44 |
| `verb:l-participle:m:pl` | 44 | 44 | 44 |
| `verb:l-participle:n:du` | 44 | 44 | 44 |
| `verb:l-participle:n:pl` | 44 | 44 | 44 |
| `verb:l-participle:n:sg` | 44 | 44 | 44 |

#### Development by generation path

| Slice | Exact | NFC/lowercase | Returned |
|---|---:|---:|---:|
| `dictionary-metadata-rule:V-IA1-01` | 400 | 400 | 421 |
| `dictionary-metadata-rule:V-IA2-01` | 16 | 16 | 16 |
| `dictionary-metadata-rule:V-II1-01` | 126 | 126 | 126 |
| `dictionary-metadata-rule:V-II2-01` | 42 | 42 | 42 |
| `dictionary-metadata-rule:V-II3-01` | 14 | 14 | 14 |
| `dictionary-metadata-rule:V-IMP-01` | 220 | 220 | 293 |
| `dictionary-metadata-rule:V-IMPF-A-01` | 264 | 264 | 264 |
| `dictionary-metadata-rule:V-IMPF-YAT-A-01` | 560 | 560 | 560 |
| `dictionary-metadata-rule:V-LPART-01` | 1048 | 1048 | 1048 |

#### Final holdout by generation path

| Slice | Exact | NFC/lowercase | Returned |
|---|---:|---:|---:|
| `dictionary-metadata-rule:V-IA1-01` | 168 | 168 | 173 |
| `dictionary-metadata-rule:V-IA2-01` | 0 | 0 | 1 |
| `dictionary-metadata-rule:V-II1-01` | 49 | 49 | 49 |
| `dictionary-metadata-rule:V-II2-01` | 28 | 28 | 28 |
| `dictionary-metadata-rule:V-II3-01` | 7 | 7 | 7 |
| `dictionary-metadata-rule:V-IMP-01` | 64 | 64 | 91 |
| `dictionary-metadata-rule:V-IMPF-A-01` | 80 | 80 | 80 |
| `dictionary-metadata-rule:V-IMPF-YAT-A-01` | 216 | 216 | 216 |
| `dictionary-metadata-rule:V-LPART-01` | 352 | 352 | 352 |

#### Development by present class

| Slice | Exact | NFC/lowercase | Returned |
|---|---:|---:|---:|
| `IA1` | 1360 | 1360 | 1452 |
| `IA2` | 48 | 48 | 50 |
| `II1` | 790 | 790 | 790 |
| `II2` | 438 | 438 | 438 |
| `II3` | 54 | 54 | 54 |

#### Final holdout by present class

| Slice | Exact | NFC/lowercase | Returned |
|---|---:|---:|---:|
| `IA1` | 544 | 544 | 575 |
| `IA2` | 16 | 16 | 18 |
| `II1` | 221 | 221 | 221 |
| `II2` | 144 | 144 | 144 |
| `II3` | 39 | 39 | 39 |

#### Development by formation

| Slice | Exact | NFC/lowercase | Returned |
|---|---:|---:|---:|
| `imperative:i-series` | 220 | 220 | 220 |
| `imperative:yat-series` | 0 | 0 | 73 |
| `imperfect:a` | 264 | 264 | 264 |
| `imperfect:yat-a` | 560 | 560 | 560 |
| `l-participle:no-formation` | 1048 | 1048 | 1048 |
| `present:IA1` | 400 | 400 | 421 |
| `present:IA2` | 16 | 16 | 16 |
| `present:II1` | 126 | 126 | 126 |
| `present:II2` | 42 | 42 | 42 |
| `present:II3` | 14 | 14 | 14 |

#### Final holdout by formation

| Slice | Exact | NFC/lowercase | Returned |
|---|---:|---:|---:|
| `imperative:i-series` | 64 | 64 | 64 |
| `imperative:yat-series` | 0 | 0 | 27 |
| `imperfect:a` | 80 | 80 | 80 |
| `imperfect:yat-a` | 216 | 216 | 216 |
| `l-participle:no-formation` | 352 | 352 | 352 |
| `present:IA1` | 168 | 168 | 173 |
| `present:IA2` | 0 | 0 | 1 |
| `present:II1` | 49 | 49 | 49 |
| `present:II2` | 28 | 28 | 28 |
| `present:II3` | 7 | 7 | 7 |

#### Development by metadata source-cell policy

| Slice | Exact | NFC/lowercase | Returned |
|---|---:|---:|---:|
| `headword:class + verb:finite:present:1:sg + verb:finite:present:2:sg` | 546 | 546 | 567 |
| `headword:class + verb:finite:present:2:sg` | 52 | 52 | 52 |
| `verb:finite:imperfect:1:sg` | 824 | 824 | 824 |
| `verb:imperative:1:du + verb:imperative:2:sg` | 165 | 165 | 165 |
| `verb:imperative:2:du + verb:imperative:2:sg` | 55 | 55 | 128 |
| `verb:l-participle:m:sg` | 1048 | 1048 | 1048 |

#### Final holdout by metadata source-cell policy

| Slice | Exact | NFC/lowercase | Returned |
|---|---:|---:|---:|
| `headword:class + verb:finite:present:1:sg + verb:finite:present:2:sg` | 231 | 231 | 237 |
| `headword:class + verb:finite:present:2:sg` | 21 | 21 | 21 |
| `verb:finite:imperfect:1:sg` | 296 | 296 | 296 |
| `verb:imperative:1:du + verb:imperative:2:sg` | 48 | 48 | 48 |
| `verb:imperative:2:du + verb:imperative:2:sg` | 16 | 16 | 43 |
| `verb:l-participle:m:sg` | 352 | 352 | 352 |

#### Development by regular/analysis kind

| Slice | Exact | NFC/lowercase | Returned |
|---|---:|---:|---:|
| `regular-single-analysis` | 2690 | 2690 | 2784 |

#### Final holdout by regular/analysis kind

| Slice | Exact | NFC/lowercase | Returned |
|---|---:|---:|---:|
| `regular-single-analysis` | 964 | 964 | 997 |

#### Development by lemma dictionary frequency

| Slice | Exact | NFC/lowercase | Returned |
|---|---:|---:|---:|
| `11-50` | 2690 | 2690 | 2784 |

#### Final holdout by lemma dictionary frequency

| Slice | Exact | NFC/lowercase | Returned |
|---|---:|---:|---:|
| `11-50` | 964 | 964 | 997 |

Skip and failure reasons:

- `ambiguous-lemma`: 451
- `missing-principal-part-after-exclusion`: 7411

The legacy oracle/core OOV diagnostic below may use the 2nd-singular present, masculine-singular l-participle, 1st-singular imperfect/new aorist, or 2nd-singular imperative. Every metadata source cell and equivalent duplicate target is excluded. Participle citation targets use only those independently held principal parts plus declared class/formation policies; they are never used to derive themselves.

## Development OOV

| Rule slice | Exact | NFC/lowercase | Total | Exact recall | Normalized recall |
|---|---:|---:|---:|---:|---:|
| `a-hard` | 4853 | 4853 | 5285 | 91.83% | 91.83% |
| `adj-hard-long` | 27900 | 27900 | 28476 | 97.98% | 97.98% |
| `adj-hard-short` | 27000 | 27000 | 28476 | 94.82% | 94.82% |
| `adj-soft-long` | 2356 | 2356 | 2556 | 92.18% | 92.18% |
| `adj-soft-short` | 1983 | 1983 | 2556 | 77.58% | 77.58% |
| `i-f` | 2887 | 2887 | 3031 | 95.25% | 95.25% |
| `i-m` | 448 | 448 | 450 | 99.56% | 99.56% |
| `ja-soft` | 1021 | 1021 | 1456 | 70.12% | 70.12% |
| `jo-m-soft` | 2544 | 2544 | 3048 | 83.46% | 83.46% |
| `jo-n-soft` | 2703 | 2703 | 3017 | 89.59% | 89.59% |
| `n-m` | 144 | 144 | 144 | 100.00% | 100.00% |
| `n-n` | 147 | 147 | 147 | 100.00% | 100.00% |
| `nt-n` | 126 | 126 | 126 | 100.00% | 100.00% |
| `o-m-hard` | 8724 | 8724 | 8940 | 97.58% | 97.58% |
| `o-n-hard` | 2667 | 2667 | 2688 | 99.22% | 99.22% |
| `r-n` | 42 | 42 | 42 | 100.00% | 100.00% |
| `s-n` | 216 | 216 | 252 | 85.71% | 85.71% |
| `u-m` | 378 | 378 | 378 | 100.00% | 100.00% |
| `v-f` | 210 | 210 | 210 | 100.00% | 100.00% |
| `verb-IA1-present` | 558 | 558 | 600 | 93.00% | 93.00% |
| `verb-IA2-present` | 16 | 16 | 16 | 100.00% | 100.00% |
| `verb-II1-present` | 259 | 259 | 259 | 100.00% | 100.00% |
| `verb-II2-present` | 140 | 140 | 140 | 100.00% | 100.00% |
| `verb-II3-present` | 14 | 14 | 14 | 100.00% | 100.00% |
| `verb-imperative` | 459 | 459 | 536 | 85.63% | 85.63% |
| `verb-imperfect` | 1104 | 1104 | 1104 | 100.00% | 100.00% |
| `verb-infinitive` | 559 | 559 | 559 | 100.00% | 100.00% |
| `verb-l-participle` | 4432 | 4432 | 4432 | 100.00% | 100.00% |
| `verb-past-active-participle-ush` | 29 | 29 | 69 | 42.03% | 42.03% |
| `verb-past-active-participle-vush` | 65 | 65 | 71 | 91.55% | 91.55% |
| `verb-past-passive-participle-en` | 7 | 7 | 49 | 14.29% | 14.29% |
| `verb-present-active-participle-yesht-soft` | 59 | 59 | 59 | 100.00% | 100.00% |
| `verb-present-active-participle-yusht-hard` | 56 | 56 | 77 | 72.73% | 72.73% |
| `verb-present-passive-participle-im` | 45 | 45 | 45 | 100.00% | 100.00% |
| `verb-present-passive-participle-om` | 33 | 33 | 49 | 67.35% | 67.35% |
| `verb-supine` | 544 | 544 | 559 | 97.32% | 97.32% |

Macro average across reported rule slices: 89.97% exact, 89.97% normalized.

### POS, class, and cell detail

| Cell slice | Exact | NFC/lowercase | Total | Exact recall | Normalized recall |
|---|---:|---:|---:|---:|---:|
| `adj/adj-hard-long/adj:long:acc:du:f:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:acc:du:f:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:acc:du:m:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:acc:du:m:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:acc:du:n:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:acc:du:n:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:acc:pl:f:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:acc:pl:f:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:acc:pl:m:an` | 0 | 0 | 226 | 0.00% | 0.00% |
| `adj/adj-hard-long/adj:long:acc:pl:m:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:acc:pl:n:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:acc:pl:n:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:acc:sg:f:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:acc:sg:f:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:acc:sg:m:an` | 0 | 0 | 226 | 0.00% | 0.00% |
| `adj/adj-hard-long/adj:long:acc:sg:m:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:acc:sg:n:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:acc:sg:n:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:dat:du:f:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:dat:du:f:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:dat:du:m:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:dat:du:m:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:dat:du:n:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:dat:du:n:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:dat:pl:f:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:dat:pl:f:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:dat:pl:m:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:dat:pl:m:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:dat:pl:n:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:dat:pl:n:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:dat:sg:f:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:dat:sg:f:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:dat:sg:m:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:dat:sg:m:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:dat:sg:n:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:dat:sg:n:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:gen:du:f:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:gen:du:f:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:gen:du:m:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:gen:du:m:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:gen:du:n:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:gen:du:n:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:gen:pl:f:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:gen:pl:f:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:gen:pl:m:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:gen:pl:m:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:gen:pl:n:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:gen:pl:n:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:gen:sg:f:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:gen:sg:f:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:gen:sg:m:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:gen:sg:m:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:gen:sg:n:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:gen:sg:n:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:ins:du:f:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:ins:du:f:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:ins:du:m:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:ins:du:m:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:ins:du:n:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:ins:du:n:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:ins:pl:f:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:ins:pl:f:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:ins:pl:m:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:ins:pl:m:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:ins:pl:n:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:ins:pl:n:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:ins:sg:f:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:ins:sg:f:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:ins:sg:m:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:ins:sg:m:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:ins:sg:n:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:ins:sg:n:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:loc:du:f:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:loc:du:f:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:loc:du:m:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:loc:du:m:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:loc:du:n:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:loc:du:n:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:loc:pl:f:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:loc:pl:f:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:loc:pl:m:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:loc:pl:m:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:loc:pl:n:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:loc:pl:n:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:loc:sg:f:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:loc:sg:f:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:loc:sg:m:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:loc:sg:m:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:loc:sg:n:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:loc:sg:n:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:nom:du:f:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:nom:du:f:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:nom:du:m:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:nom:du:m:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:nom:du:n:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:nom:du:n:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:nom:pl:f:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:nom:pl:f:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:nom:pl:m:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:nom:pl:m:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:nom:pl:n:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:nom:pl:n:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:nom:sg:f:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:nom:sg:f:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:nom:sg:m:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:nom:sg:m:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:nom:sg:n:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:nom:sg:n:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:voc:du:f:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:voc:du:f:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:voc:du:m:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:voc:du:m:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:voc:du:n:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:voc:du:n:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:voc:pl:f:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:voc:pl:f:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:voc:pl:m:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:voc:pl:m:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:voc:pl:n:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:voc:pl:n:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:voc:sg:f:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:voc:sg:f:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:voc:sg:m:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:voc:sg:m:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:voc:sg:n:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-long/adj:long:voc:sg:n:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:acc:du:f:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:acc:du:f:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:acc:du:m:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:acc:du:m:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:acc:du:n:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:acc:du:n:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:acc:pl:f:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:acc:pl:f:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:acc:pl:m:an` | 0 | 0 | 226 | 0.00% | 0.00% |
| `adj/adj-hard-short/adj:short:acc:pl:m:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:acc:pl:n:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:acc:pl:n:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:acc:sg:f:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:acc:sg:f:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:acc:sg:m:an` | 0 | 0 | 226 | 0.00% | 0.00% |
| `adj/adj-hard-short/adj:short:acc:sg:m:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:acc:sg:n:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:acc:sg:n:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:dat:du:f:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:dat:du:f:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:dat:du:m:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:dat:du:m:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:dat:du:n:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:dat:du:n:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:dat:pl:f:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:dat:pl:f:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:dat:pl:m:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:dat:pl:m:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:dat:pl:n:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:dat:pl:n:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:dat:sg:f:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:dat:sg:f:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:dat:sg:m:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:dat:sg:m:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:dat:sg:n:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:dat:sg:n:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:gen:du:f:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:gen:du:f:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:gen:du:m:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:gen:du:m:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:gen:du:n:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:gen:du:n:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:gen:pl:f:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:gen:pl:f:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:gen:pl:m:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:gen:pl:m:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:gen:pl:n:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:gen:pl:n:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:gen:sg:f:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:gen:sg:f:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:gen:sg:m:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:gen:sg:m:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:gen:sg:n:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:gen:sg:n:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:ins:du:f:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:ins:du:f:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:ins:du:m:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:ins:du:m:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:ins:du:n:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:ins:du:n:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:ins:pl:f:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:ins:pl:f:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:ins:pl:m:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:ins:pl:m:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:ins:pl:n:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:ins:pl:n:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:ins:sg:f:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:ins:sg:f:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:ins:sg:m:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:ins:sg:m:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:ins:sg:n:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:ins:sg:n:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:loc:du:f:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:loc:du:f:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:loc:du:m:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:loc:du:m:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:loc:du:n:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:loc:du:n:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:loc:pl:f:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:loc:pl:f:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:loc:pl:m:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:loc:pl:m:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:loc:pl:n:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:loc:pl:n:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:loc:sg:f:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:loc:sg:f:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:loc:sg:m:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:loc:sg:m:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:loc:sg:n:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:loc:sg:n:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:nom:du:f:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:nom:du:f:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:nom:du:m:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:nom:du:m:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:nom:du:n:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:nom:du:n:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:nom:pl:f:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:nom:pl:f:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:nom:pl:m:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:nom:pl:m:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:nom:pl:n:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:nom:pl:n:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:nom:sg:f:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:nom:sg:f:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:nom:sg:m:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:nom:sg:m:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:nom:sg:n:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:nom:sg:n:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:voc:du:f:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:voc:du:f:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:voc:du:m:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:voc:du:m:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:voc:du:n:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:voc:du:n:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:voc:pl:f:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:voc:pl:f:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:voc:pl:m:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:voc:pl:m:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:voc:pl:n:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:voc:pl:n:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:voc:sg:f:an` | 0 | 0 | 226 | 0.00% | 0.00% |
| `adj/adj-hard-short/adj:short:voc:sg:f:in` | 0 | 0 | 226 | 0.00% | 0.00% |
| `adj/adj-hard-short/adj:short:voc:sg:m:an` | 0 | 0 | 226 | 0.00% | 0.00% |
| `adj/adj-hard-short/adj:short:voc:sg:m:in` | 0 | 0 | 226 | 0.00% | 0.00% |
| `adj/adj-hard-short/adj:short:voc:sg:n:an` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-hard-short/adj:short:voc:sg:n:in` | 225 | 225 | 226 | 99.56% | 99.56% |
| `adj/adj-soft-long/adj:long:acc:du:f:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:acc:du:f:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:acc:du:m:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:acc:du:m:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:acc:du:n:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:acc:du:n:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:acc:pl:f:an` | 19 | 19 | 21 | 90.48% | 90.48% |
| `adj/adj-soft-long/adj:long:acc:pl:f:in` | 19 | 19 | 21 | 90.48% | 90.48% |
| `adj/adj-soft-long/adj:long:acc:pl:m:an` | 0 | 0 | 21 | 0.00% | 0.00% |
| `adj/adj-soft-long/adj:long:acc:pl:m:in` | 19 | 19 | 21 | 90.48% | 90.48% |
| `adj/adj-soft-long/adj:long:acc:pl:n:an` | 19 | 19 | 21 | 90.48% | 90.48% |
| `adj/adj-soft-long/adj:long:acc:pl:n:in` | 19 | 19 | 21 | 90.48% | 90.48% |
| `adj/adj-soft-long/adj:long:acc:sg:f:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:acc:sg:f:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:acc:sg:m:an` | 0 | 0 | 20 | 0.00% | 0.00% |
| `adj/adj-soft-long/adj:long:acc:sg:m:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:acc:sg:n:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:acc:sg:n:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:dat:du:f:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:dat:du:f:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:dat:du:m:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:dat:du:m:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:dat:du:n:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:dat:du:n:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:dat:pl:f:an` | 19 | 19 | 21 | 90.48% | 90.48% |
| `adj/adj-soft-long/adj:long:dat:pl:f:in` | 19 | 19 | 21 | 90.48% | 90.48% |
| `adj/adj-soft-long/adj:long:dat:pl:m:an` | 19 | 19 | 21 | 90.48% | 90.48% |
| `adj/adj-soft-long/adj:long:dat:pl:m:in` | 19 | 19 | 21 | 90.48% | 90.48% |
| `adj/adj-soft-long/adj:long:dat:pl:n:an` | 19 | 19 | 21 | 90.48% | 90.48% |
| `adj/adj-soft-long/adj:long:dat:pl:n:in` | 19 | 19 | 21 | 90.48% | 90.48% |
| `adj/adj-soft-long/adj:long:dat:sg:f:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:dat:sg:f:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:dat:sg:m:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:dat:sg:m:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:dat:sg:n:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:dat:sg:n:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:gen:du:f:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:gen:du:f:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:gen:du:m:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:gen:du:m:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:gen:du:n:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:gen:du:n:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:gen:pl:f:an` | 19 | 19 | 21 | 90.48% | 90.48% |
| `adj/adj-soft-long/adj:long:gen:pl:f:in` | 19 | 19 | 21 | 90.48% | 90.48% |
| `adj/adj-soft-long/adj:long:gen:pl:m:an` | 19 | 19 | 21 | 90.48% | 90.48% |
| `adj/adj-soft-long/adj:long:gen:pl:m:in` | 19 | 19 | 21 | 90.48% | 90.48% |
| `adj/adj-soft-long/adj:long:gen:pl:n:an` | 19 | 19 | 21 | 90.48% | 90.48% |
| `adj/adj-soft-long/adj:long:gen:pl:n:in` | 19 | 19 | 21 | 90.48% | 90.48% |
| `adj/adj-soft-long/adj:long:gen:sg:f:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:gen:sg:f:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:gen:sg:m:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:gen:sg:m:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:gen:sg:n:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:gen:sg:n:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:ins:du:f:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:ins:du:f:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:ins:du:m:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:ins:du:m:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:ins:du:n:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:ins:du:n:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:ins:pl:f:an` | 19 | 19 | 21 | 90.48% | 90.48% |
| `adj/adj-soft-long/adj:long:ins:pl:f:in` | 19 | 19 | 21 | 90.48% | 90.48% |
| `adj/adj-soft-long/adj:long:ins:pl:m:an` | 19 | 19 | 21 | 90.48% | 90.48% |
| `adj/adj-soft-long/adj:long:ins:pl:m:in` | 19 | 19 | 21 | 90.48% | 90.48% |
| `adj/adj-soft-long/adj:long:ins:pl:n:an` | 19 | 19 | 21 | 90.48% | 90.48% |
| `adj/adj-soft-long/adj:long:ins:pl:n:in` | 19 | 19 | 21 | 90.48% | 90.48% |
| `adj/adj-soft-long/adj:long:ins:sg:f:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:ins:sg:f:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:ins:sg:m:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:ins:sg:m:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:ins:sg:n:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:ins:sg:n:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:loc:du:f:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:loc:du:f:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:loc:du:m:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:loc:du:m:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:loc:du:n:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:loc:du:n:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:loc:pl:f:an` | 19 | 19 | 21 | 90.48% | 90.48% |
| `adj/adj-soft-long/adj:long:loc:pl:f:in` | 19 | 19 | 21 | 90.48% | 90.48% |
| `adj/adj-soft-long/adj:long:loc:pl:m:an` | 19 | 19 | 21 | 90.48% | 90.48% |
| `adj/adj-soft-long/adj:long:loc:pl:m:in` | 19 | 19 | 21 | 90.48% | 90.48% |
| `adj/adj-soft-long/adj:long:loc:pl:n:an` | 19 | 19 | 21 | 90.48% | 90.48% |
| `adj/adj-soft-long/adj:long:loc:pl:n:in` | 19 | 19 | 21 | 90.48% | 90.48% |
| `adj/adj-soft-long/adj:long:loc:sg:f:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:loc:sg:f:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:loc:sg:m:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:loc:sg:m:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:loc:sg:n:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:loc:sg:n:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:nom:du:f:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:nom:du:f:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:nom:du:m:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:nom:du:m:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:nom:du:n:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:nom:du:n:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:nom:pl:f:an` | 19 | 19 | 21 | 90.48% | 90.48% |
| `adj/adj-soft-long/adj:long:nom:pl:f:in` | 19 | 19 | 21 | 90.48% | 90.48% |
| `adj/adj-soft-long/adj:long:nom:pl:m:an` | 19 | 19 | 21 | 90.48% | 90.48% |
| `adj/adj-soft-long/adj:long:nom:pl:m:in` | 19 | 19 | 21 | 90.48% | 90.48% |
| `adj/adj-soft-long/adj:long:nom:pl:n:an` | 19 | 19 | 21 | 90.48% | 90.48% |
| `adj/adj-soft-long/adj:long:nom:pl:n:in` | 19 | 19 | 21 | 90.48% | 90.48% |
| `adj/adj-soft-long/adj:long:nom:sg:f:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:nom:sg:f:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:nom:sg:m:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:nom:sg:m:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:nom:sg:n:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:nom:sg:n:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:voc:du:f:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:voc:du:f:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:voc:du:m:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:voc:du:m:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:voc:du:n:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:voc:du:n:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:voc:pl:f:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:voc:pl:f:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:voc:pl:m:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:voc:pl:m:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:voc:pl:n:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:voc:pl:n:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:voc:sg:f:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:voc:sg:f:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:voc:sg:m:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:voc:sg:m:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:voc:sg:n:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-long/adj:long:voc:sg:n:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-short/adj:short:acc:du:f:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-short/adj:short:acc:du:f:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-short/adj:short:acc:du:m:an` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:acc:du:m:in` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:acc:du:n:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-short/adj:short:acc:du:n:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-short/adj:short:acc:pl:f:an` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:acc:pl:f:in` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:acc:pl:m:an` | 0 | 0 | 20 | 0.00% | 0.00% |
| `adj/adj-soft-short/adj:short:acc:pl:m:in` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:acc:pl:n:an` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:acc:pl:n:in` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:acc:sg:f:an` | 15 | 15 | 21 | 71.43% | 71.43% |
| `adj/adj-soft-short/adj:short:acc:sg:f:in` | 15 | 15 | 21 | 71.43% | 71.43% |
| `adj/adj-soft-short/adj:short:acc:sg:m:an` | 0 | 0 | 21 | 0.00% | 0.00% |
| `adj/adj-soft-short/adj:short:acc:sg:m:in` | 18 | 18 | 21 | 85.71% | 85.71% |
| `adj/adj-soft-short/adj:short:acc:sg:n:an` | 16 | 16 | 21 | 76.19% | 76.19% |
| `adj/adj-soft-short/adj:short:acc:sg:n:in` | 16 | 16 | 21 | 76.19% | 76.19% |
| `adj/adj-soft-short/adj:short:dat:du:f:an` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:dat:du:f:in` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:dat:du:m:an` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:dat:du:m:in` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:dat:du:n:an` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:dat:du:n:in` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:dat:pl:f:an` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:dat:pl:f:in` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:dat:pl:m:an` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:dat:pl:m:in` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:dat:pl:n:an` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:dat:pl:n:in` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:dat:sg:f:an` | 19 | 19 | 21 | 90.48% | 90.48% |
| `adj/adj-soft-short/adj:short:dat:sg:f:in` | 19 | 19 | 21 | 90.48% | 90.48% |
| `adj/adj-soft-short/adj:short:dat:sg:m:an` | 15 | 15 | 21 | 71.43% | 71.43% |
| `adj/adj-soft-short/adj:short:dat:sg:m:in` | 15 | 15 | 21 | 71.43% | 71.43% |
| `adj/adj-soft-short/adj:short:dat:sg:n:an` | 15 | 15 | 21 | 71.43% | 71.43% |
| `adj/adj-soft-short/adj:short:dat:sg:n:in` | 15 | 15 | 21 | 71.43% | 71.43% |
| `adj/adj-soft-short/adj:short:gen:du:f:an` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:gen:du:f:in` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:gen:du:m:an` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:gen:du:m:in` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:gen:du:n:an` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:gen:du:n:in` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:gen:pl:f:an` | 17 | 17 | 20 | 85.00% | 85.00% |
| `adj/adj-soft-short/adj:short:gen:pl:f:in` | 17 | 17 | 20 | 85.00% | 85.00% |
| `adj/adj-soft-short/adj:short:gen:pl:m:an` | 17 | 17 | 20 | 85.00% | 85.00% |
| `adj/adj-soft-short/adj:short:gen:pl:m:in` | 17 | 17 | 20 | 85.00% | 85.00% |
| `adj/adj-soft-short/adj:short:gen:pl:n:an` | 17 | 17 | 20 | 85.00% | 85.00% |
| `adj/adj-soft-short/adj:short:gen:pl:n:in` | 17 | 17 | 20 | 85.00% | 85.00% |
| `adj/adj-soft-short/adj:short:gen:sg:f:an` | 15 | 15 | 21 | 71.43% | 71.43% |
| `adj/adj-soft-short/adj:short:gen:sg:f:in` | 15 | 15 | 21 | 71.43% | 71.43% |
| `adj/adj-soft-short/adj:short:gen:sg:m:an` | 15 | 15 | 21 | 71.43% | 71.43% |
| `adj/adj-soft-short/adj:short:gen:sg:m:in` | 15 | 15 | 21 | 71.43% | 71.43% |
| `adj/adj-soft-short/adj:short:gen:sg:n:an` | 15 | 15 | 21 | 71.43% | 71.43% |
| `adj/adj-soft-short/adj:short:gen:sg:n:in` | 15 | 15 | 21 | 71.43% | 71.43% |
| `adj/adj-soft-short/adj:short:ins:du:f:an` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:ins:du:f:in` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:ins:du:m:an` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:ins:du:m:in` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:ins:du:n:an` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:ins:du:n:in` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:ins:pl:f:an` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:ins:pl:f:in` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:ins:pl:m:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-short/adj:short:ins:pl:m:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-short/adj:short:ins:pl:n:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-short/adj:short:ins:pl:n:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-short/adj:short:ins:sg:f:an` | 15 | 15 | 21 | 71.43% | 71.43% |
| `adj/adj-soft-short/adj:short:ins:sg:f:in` | 15 | 15 | 21 | 71.43% | 71.43% |
| `adj/adj-soft-short/adj:short:ins:sg:m:an` | 15 | 15 | 21 | 71.43% | 71.43% |
| `adj/adj-soft-short/adj:short:ins:sg:m:in` | 15 | 15 | 21 | 71.43% | 71.43% |
| `adj/adj-soft-short/adj:short:ins:sg:n:an` | 15 | 15 | 21 | 71.43% | 71.43% |
| `adj/adj-soft-short/adj:short:ins:sg:n:in` | 15 | 15 | 21 | 71.43% | 71.43% |
| `adj/adj-soft-short/adj:short:loc:du:f:an` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:loc:du:f:in` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:loc:du:m:an` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:loc:du:m:in` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:loc:du:n:an` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:loc:du:n:in` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:loc:pl:f:an` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:loc:pl:f:in` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:loc:pl:m:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-short/adj:short:loc:pl:m:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-short/adj:short:loc:pl:n:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-short/adj:short:loc:pl:n:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-short/adj:short:loc:sg:f:an` | 19 | 19 | 21 | 90.48% | 90.48% |
| `adj/adj-soft-short/adj:short:loc:sg:f:in` | 19 | 19 | 21 | 90.48% | 90.48% |
| `adj/adj-soft-short/adj:short:loc:sg:m:an` | 19 | 19 | 21 | 90.48% | 90.48% |
| `adj/adj-soft-short/adj:short:loc:sg:m:in` | 19 | 19 | 21 | 90.48% | 90.48% |
| `adj/adj-soft-short/adj:short:loc:sg:n:an` | 19 | 19 | 21 | 90.48% | 90.48% |
| `adj/adj-soft-short/adj:short:loc:sg:n:in` | 19 | 19 | 21 | 90.48% | 90.48% |
| `adj/adj-soft-short/adj:short:nom:du:f:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-short/adj:short:nom:du:f:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-short/adj:short:nom:du:m:an` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:nom:du:m:in` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:nom:du:n:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-short/adj:short:nom:du:n:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-short/adj:short:nom:pl:f:an` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:nom:pl:f:in` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:nom:pl:m:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-short/adj:short:nom:pl:m:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-short/adj:short:nom:pl:n:an` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:nom:pl:n:in` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:nom:sg:f:an` | 15 | 15 | 21 | 71.43% | 71.43% |
| `adj/adj-soft-short/adj:short:nom:sg:f:in` | 15 | 15 | 21 | 71.43% | 71.43% |
| `adj/adj-soft-short/adj:short:nom:sg:m:an` | 18 | 18 | 21 | 85.71% | 85.71% |
| `adj/adj-soft-short/adj:short:nom:sg:m:in` | 18 | 18 | 21 | 85.71% | 85.71% |
| `adj/adj-soft-short/adj:short:nom:sg:n:an` | 16 | 16 | 21 | 76.19% | 76.19% |
| `adj/adj-soft-short/adj:short:nom:sg:n:in` | 16 | 16 | 21 | 76.19% | 76.19% |
| `adj/adj-soft-short/adj:short:voc:du:f:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-short/adj:short:voc:du:f:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-short/adj:short:voc:du:m:an` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:voc:du:m:in` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:voc:du:n:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-short/adj:short:voc:du:n:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-short/adj:short:voc:pl:f:an` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:voc:pl:f:in` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:voc:pl:m:an` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-short/adj:short:voc:pl:m:in` | 19 | 19 | 20 | 95.00% | 95.00% |
| `adj/adj-soft-short/adj:short:voc:pl:n:an` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:voc:pl:n:in` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:voc:sg:f:an` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:voc:sg:f:in` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:voc:sg:m:an` | 0 | 0 | 20 | 0.00% | 0.00% |
| `adj/adj-soft-short/adj:short:voc:sg:m:in` | 0 | 0 | 20 | 0.00% | 0.00% |
| `adj/adj-soft-short/adj:short:voc:sg:n:an` | 15 | 15 | 20 | 75.00% | 75.00% |
| `adj/adj-soft-short/adj:short:voc:sg:n:in` | 15 | 15 | 20 | 75.00% | 75.00% |
| `noun/a-hard/noun:acc:du` | 212 | 212 | 250 | 84.80% | 84.80% |
| `noun/a-hard/noun:acc:pl` | 218 | 218 | 250 | 87.20% | 87.20% |
| `noun/a-hard/noun:acc:sg` | 253 | 253 | 255 | 99.22% | 99.22% |
| `noun/a-hard/noun:dat:du` | 248 | 248 | 250 | 99.20% | 99.20% |
| `noun/a-hard/noun:dat:pl` | 248 | 248 | 250 | 99.20% | 99.20% |
| `noun/a-hard/noun:dat:sg` | 217 | 217 | 255 | 85.10% | 85.10% |
| `noun/a-hard/noun:gen:du` | 248 | 248 | 250 | 99.20% | 99.20% |
| `noun/a-hard/noun:gen:pl` | 218 | 218 | 250 | 87.20% | 87.20% |
| `noun/a-hard/noun:gen:sg` | 223 | 223 | 255 | 87.45% | 87.45% |
| `noun/a-hard/noun:ins:du` | 248 | 248 | 250 | 99.20% | 99.20% |
| `noun/a-hard/noun:ins:pl` | 248 | 248 | 250 | 99.20% | 99.20% |
| `noun/a-hard/noun:ins:sg` | 223 | 223 | 255 | 87.45% | 87.45% |
| `noun/a-hard/noun:loc:du` | 248 | 248 | 250 | 99.20% | 99.20% |
| `noun/a-hard/noun:loc:pl` | 248 | 248 | 250 | 99.20% | 99.20% |
| `noun/a-hard/noun:loc:sg` | 217 | 217 | 255 | 85.10% | 85.10% |
| `noun/a-hard/noun:nom:du` | 212 | 212 | 250 | 84.80% | 84.80% |
| `noun/a-hard/noun:nom:pl` | 218 | 218 | 250 | 87.20% | 87.20% |
| `noun/a-hard/noun:nom:sg` | 253 | 253 | 255 | 99.22% | 99.22% |
| `noun/a-hard/noun:voc:du` | 212 | 212 | 250 | 84.80% | 84.80% |
| `noun/a-hard/noun:voc:pl` | 218 | 218 | 250 | 87.20% | 87.20% |
| `noun/a-hard/noun:voc:sg` | 223 | 223 | 255 | 87.45% | 87.45% |
| `noun/i-f/noun:acc:du` | 144 | 144 | 144 | 100.00% | 100.00% |
| `noun/i-f/noun:acc:pl` | 144 | 144 | 144 | 100.00% | 100.00% |
| `noun/i-f/noun:acc:sg` | 145 | 145 | 145 | 100.00% | 100.00% |
| `noun/i-f/noun:dat:du` | 144 | 144 | 144 | 100.00% | 100.00% |
| `noun/i-f/noun:dat:pl` | 144 | 144 | 144 | 100.00% | 100.00% |
| `noun/i-f/noun:dat:sg` | 145 | 145 | 145 | 100.00% | 100.00% |
| `noun/i-f/noun:gen:du` | 144 | 144 | 144 | 100.00% | 100.00% |
| `noun/i-f/noun:gen:pl` | 144 | 144 | 144 | 100.00% | 100.00% |
| `noun/i-f/noun:gen:sg` | 145 | 145 | 145 | 100.00% | 100.00% |
| `noun/i-f/noun:ins:du` | 144 | 144 | 144 | 100.00% | 100.00% |
| `noun/i-f/noun:ins:pl` | 144 | 144 | 144 | 100.00% | 100.00% |
| `noun/i-f/noun:ins:sg` | 145 | 145 | 145 | 100.00% | 100.00% |
| `noun/i-f/noun:loc:du` | 144 | 144 | 144 | 100.00% | 100.00% |
| `noun/i-f/noun:loc:pl` | 144 | 144 | 144 | 100.00% | 100.00% |
| `noun/i-f/noun:loc:sg` | 145 | 145 | 145 | 100.00% | 100.00% |
| `noun/i-f/noun:nom:du` | 144 | 144 | 144 | 100.00% | 100.00% |
| `noun/i-f/noun:nom:pl` | 144 | 144 | 144 | 100.00% | 100.00% |
| `noun/i-f/noun:nom:sg` | 145 | 145 | 145 | 100.00% | 100.00% |
| `noun/i-f/noun:voc:du` | 144 | 144 | 144 | 100.00% | 100.00% |
| `noun/i-f/noun:voc:pl` | 0 | 0 | 144 | 0.00% | 0.00% |
| `noun/i-f/noun:voc:sg` | 145 | 145 | 145 | 100.00% | 100.00% |
| `noun/i-m/noun:dat:du` | 25 | 25 | 25 | 100.00% | 100.00% |
| `noun/i-m/noun:dat:pl` | 25 | 25 | 25 | 100.00% | 100.00% |
| `noun/i-m/noun:dat:sg` | 25 | 25 | 25 | 100.00% | 100.00% |
| `noun/i-m/noun:gen:du` | 25 | 25 | 25 | 100.00% | 100.00% |
| `noun/i-m/noun:gen:pl` | 25 | 25 | 25 | 100.00% | 100.00% |
| `noun/i-m/noun:gen:sg` | 25 | 25 | 25 | 100.00% | 100.00% |
| `noun/i-m/noun:ins:du` | 25 | 25 | 25 | 100.00% | 100.00% |
| `noun/i-m/noun:ins:pl` | 25 | 25 | 25 | 100.00% | 100.00% |
| `noun/i-m/noun:ins:sg` | 24 | 24 | 25 | 96.00% | 96.00% |
| `noun/i-m/noun:loc:du` | 25 | 25 | 25 | 100.00% | 100.00% |
| `noun/i-m/noun:loc:pl` | 25 | 25 | 25 | 100.00% | 100.00% |
| `noun/i-m/noun:loc:sg` | 25 | 25 | 25 | 100.00% | 100.00% |
| `noun/i-m/noun:nom:du` | 25 | 25 | 25 | 100.00% | 100.00% |
| `noun/i-m/noun:nom:pl` | 24 | 24 | 25 | 96.00% | 96.00% |
| `noun/i-m/noun:nom:sg` | 25 | 25 | 25 | 100.00% | 100.00% |
| `noun/i-m/noun:voc:du` | 25 | 25 | 25 | 100.00% | 100.00% |
| `noun/i-m/noun:voc:pl` | 25 | 25 | 25 | 100.00% | 100.00% |
| `noun/i-m/noun:voc:sg` | 25 | 25 | 25 | 100.00% | 100.00% |
| `noun/ja-soft/noun:acc:du` | 52 | 52 | 67 | 77.61% | 77.61% |
| `noun/ja-soft/noun:acc:pl` | 48 | 48 | 68 | 70.59% | 70.59% |
| `noun/ja-soft/noun:acc:sg` | 53 | 53 | 73 | 72.60% | 72.60% |
| `noun/ja-soft/noun:dat:du` | 47 | 47 | 67 | 70.15% | 70.15% |
| `noun/ja-soft/noun:dat:pl` | 47 | 47 | 68 | 69.12% | 69.12% |
| `noun/ja-soft/noun:dat:sg` | 57 | 57 | 73 | 78.08% | 78.08% |
| `noun/ja-soft/noun:gen:du` | 47 | 47 | 67 | 70.15% | 70.15% |
| `noun/ja-soft/noun:gen:pl` | 44 | 44 | 68 | 64.71% | 64.71% |
| `noun/ja-soft/noun:gen:sg` | 53 | 53 | 73 | 72.60% | 72.60% |
| `noun/ja-soft/noun:ins:du` | 47 | 47 | 67 | 70.15% | 70.15% |
| `noun/ja-soft/noun:ins:pl` | 47 | 47 | 68 | 69.12% | 69.12% |
| `noun/ja-soft/noun:ins:sg` | 44 | 44 | 73 | 60.27% | 60.27% |
| `noun/ja-soft/noun:loc:du` | 47 | 47 | 67 | 70.15% | 70.15% |
| `noun/ja-soft/noun:loc:pl` | 47 | 47 | 68 | 69.12% | 69.12% |
| `noun/ja-soft/noun:loc:sg` | 57 | 57 | 73 | 78.08% | 78.08% |
| `noun/ja-soft/noun:nom:du` | 52 | 52 | 67 | 77.61% | 77.61% |
| `noun/ja-soft/noun:nom:pl` | 48 | 48 | 68 | 70.59% | 70.59% |
| `noun/ja-soft/noun:nom:sg` | 40 | 40 | 73 | 54.79% | 54.79% |
| `noun/ja-soft/noun:voc:du` | 52 | 52 | 67 | 77.61% | 77.61% |
| `noun/ja-soft/noun:voc:pl` | 48 | 48 | 68 | 70.59% | 70.59% |
| `noun/ja-soft/noun:voc:sg` | 44 | 44 | 73 | 60.27% | 60.27% |
| `noun/jo-m-soft/noun:dat:du` | 147 | 147 | 168 | 87.50% | 87.50% |
| `noun/jo-m-soft/noun:dat:pl` | 147 | 147 | 168 | 87.50% | 87.50% |
| `noun/jo-m-soft/noun:dat:sg` | 133 | 133 | 172 | 77.33% | 77.33% |
| `noun/jo-m-soft/noun:gen:du` | 131 | 131 | 168 | 77.98% | 77.98% |
| `noun/jo-m-soft/noun:gen:pl` | 147 | 147 | 168 | 87.50% | 87.50% |
| `noun/jo-m-soft/noun:gen:sg` | 133 | 133 | 172 | 77.33% | 77.33% |
| `noun/jo-m-soft/noun:ins:du` | 147 | 147 | 168 | 87.50% | 87.50% |
| `noun/jo-m-soft/noun:ins:pl` | 147 | 147 | 168 | 87.50% | 87.50% |
| `noun/jo-m-soft/noun:ins:sg` | 149 | 149 | 172 | 86.63% | 86.63% |
| `noun/jo-m-soft/noun:loc:du` | 131 | 131 | 168 | 77.98% | 77.98% |
| `noun/jo-m-soft/noun:loc:pl` | 147 | 147 | 168 | 87.50% | 87.50% |
| `noun/jo-m-soft/noun:loc:sg` | 149 | 149 | 172 | 86.63% | 86.63% |
| `noun/jo-m-soft/noun:nom:du` | 131 | 131 | 168 | 77.98% | 77.98% |
| `noun/jo-m-soft/noun:nom:pl` | 147 | 147 | 168 | 87.50% | 87.50% |
| `noun/jo-m-soft/noun:nom:sg` | 149 | 149 | 172 | 86.63% | 86.63% |
| `noun/jo-m-soft/noun:voc:du` | 131 | 131 | 168 | 77.98% | 77.98% |
| `noun/jo-m-soft/noun:voc:pl` | 147 | 147 | 168 | 87.50% | 87.50% |
| `noun/jo-m-soft/noun:voc:sg` | 131 | 131 | 172 | 76.16% | 76.16% |
| `noun/jo-n-soft/noun:acc:du` | 143 | 143 | 143 | 100.00% | 100.00% |
| `noun/jo-n-soft/noun:acc:pl` | 143 | 143 | 143 | 100.00% | 100.00% |
| `noun/jo-n-soft/noun:acc:sg` | 145 | 145 | 145 | 100.00% | 100.00% |
| `noun/jo-n-soft/noun:dat:du` | 75 | 75 | 143 | 52.45% | 52.45% |
| `noun/jo-n-soft/noun:dat:pl` | 75 | 75 | 143 | 52.45% | 52.45% |
| `noun/jo-n-soft/noun:dat:sg` | 145 | 145 | 145 | 100.00% | 100.00% |
| `noun/jo-n-soft/noun:gen:du` | 143 | 143 | 143 | 100.00% | 100.00% |
| `noun/jo-n-soft/noun:gen:pl` | 102 | 102 | 143 | 71.33% | 71.33% |
| `noun/jo-n-soft/noun:gen:sg` | 145 | 145 | 145 | 100.00% | 100.00% |
| `noun/jo-n-soft/noun:ins:du` | 75 | 75 | 143 | 52.45% | 52.45% |
| `noun/jo-n-soft/noun:ins:pl` | 143 | 143 | 143 | 100.00% | 100.00% |
| `noun/jo-n-soft/noun:ins:sg` | 76 | 76 | 145 | 52.41% | 52.41% |
| `noun/jo-n-soft/noun:loc:du` | 143 | 143 | 143 | 100.00% | 100.00% |
| `noun/jo-n-soft/noun:loc:pl` | 143 | 143 | 143 | 100.00% | 100.00% |
| `noun/jo-n-soft/noun:loc:sg` | 145 | 145 | 145 | 100.00% | 100.00% |
| `noun/jo-n-soft/noun:nom:du` | 143 | 143 | 143 | 100.00% | 100.00% |
| `noun/jo-n-soft/noun:nom:pl` | 143 | 143 | 143 | 100.00% | 100.00% |
| `noun/jo-n-soft/noun:nom:sg` | 145 | 145 | 145 | 100.00% | 100.00% |
| `noun/jo-n-soft/noun:voc:du` | 143 | 143 | 143 | 100.00% | 100.00% |
| `noun/jo-n-soft/noun:voc:pl` | 143 | 143 | 143 | 100.00% | 100.00% |
| `noun/jo-n-soft/noun:voc:sg` | 145 | 145 | 145 | 100.00% | 100.00% |
| `noun/n-m/noun:dat:du` | 8 | 8 | 8 | 100.00% | 100.00% |
| `noun/n-m/noun:dat:pl` | 8 | 8 | 8 | 100.00% | 100.00% |
| `noun/n-m/noun:dat:sg` | 8 | 8 | 8 | 100.00% | 100.00% |
| `noun/n-m/noun:gen:du` | 8 | 8 | 8 | 100.00% | 100.00% |
| `noun/n-m/noun:gen:pl` | 8 | 8 | 8 | 100.00% | 100.00% |
| `noun/n-m/noun:gen:sg` | 8 | 8 | 8 | 100.00% | 100.00% |
| `noun/n-m/noun:ins:du` | 8 | 8 | 8 | 100.00% | 100.00% |
| `noun/n-m/noun:ins:pl` | 8 | 8 | 8 | 100.00% | 100.00% |
| `noun/n-m/noun:ins:sg` | 8 | 8 | 8 | 100.00% | 100.00% |
| `noun/n-m/noun:loc:du` | 8 | 8 | 8 | 100.00% | 100.00% |
| `noun/n-m/noun:loc:pl` | 8 | 8 | 8 | 100.00% | 100.00% |
| `noun/n-m/noun:loc:sg` | 8 | 8 | 8 | 100.00% | 100.00% |
| `noun/n-m/noun:nom:du` | 8 | 8 | 8 | 100.00% | 100.00% |
| `noun/n-m/noun:nom:pl` | 8 | 8 | 8 | 100.00% | 100.00% |
| `noun/n-m/noun:nom:sg` | 8 | 8 | 8 | 100.00% | 100.00% |
| `noun/n-m/noun:voc:du` | 8 | 8 | 8 | 100.00% | 100.00% |
| `noun/n-m/noun:voc:pl` | 8 | 8 | 8 | 100.00% | 100.00% |
| `noun/n-m/noun:voc:sg` | 8 | 8 | 8 | 100.00% | 100.00% |
| `noun/n-n/noun:acc:du` | 7 | 7 | 7 | 100.00% | 100.00% |
| `noun/n-n/noun:acc:pl` | 7 | 7 | 7 | 100.00% | 100.00% |
| `noun/n-n/noun:acc:sg` | 7 | 7 | 7 | 100.00% | 100.00% |
| `noun/n-n/noun:dat:du` | 7 | 7 | 7 | 100.00% | 100.00% |
| `noun/n-n/noun:dat:pl` | 7 | 7 | 7 | 100.00% | 100.00% |
| `noun/n-n/noun:dat:sg` | 7 | 7 | 7 | 100.00% | 100.00% |
| `noun/n-n/noun:gen:du` | 7 | 7 | 7 | 100.00% | 100.00% |
| `noun/n-n/noun:gen:pl` | 7 | 7 | 7 | 100.00% | 100.00% |
| `noun/n-n/noun:gen:sg` | 7 | 7 | 7 | 100.00% | 100.00% |
| `noun/n-n/noun:ins:du` | 7 | 7 | 7 | 100.00% | 100.00% |
| `noun/n-n/noun:ins:pl` | 7 | 7 | 7 | 100.00% | 100.00% |
| `noun/n-n/noun:ins:sg` | 7 | 7 | 7 | 100.00% | 100.00% |
| `noun/n-n/noun:loc:du` | 7 | 7 | 7 | 100.00% | 100.00% |
| `noun/n-n/noun:loc:pl` | 7 | 7 | 7 | 100.00% | 100.00% |
| `noun/n-n/noun:loc:sg` | 7 | 7 | 7 | 100.00% | 100.00% |
| `noun/n-n/noun:nom:du` | 7 | 7 | 7 | 100.00% | 100.00% |
| `noun/n-n/noun:nom:pl` | 7 | 7 | 7 | 100.00% | 100.00% |
| `noun/n-n/noun:nom:sg` | 7 | 7 | 7 | 100.00% | 100.00% |
| `noun/n-n/noun:voc:du` | 7 | 7 | 7 | 100.00% | 100.00% |
| `noun/n-n/noun:voc:pl` | 7 | 7 | 7 | 100.00% | 100.00% |
| `noun/n-n/noun:voc:sg` | 7 | 7 | 7 | 100.00% | 100.00% |
| `noun/nt-n/noun:acc:du` | 6 | 6 | 6 | 100.00% | 100.00% |
| `noun/nt-n/noun:acc:pl` | 6 | 6 | 6 | 100.00% | 100.00% |
| `noun/nt-n/noun:acc:sg` | 6 | 6 | 6 | 100.00% | 100.00% |
| `noun/nt-n/noun:dat:du` | 6 | 6 | 6 | 100.00% | 100.00% |
| `noun/nt-n/noun:dat:pl` | 6 | 6 | 6 | 100.00% | 100.00% |
| `noun/nt-n/noun:dat:sg` | 6 | 6 | 6 | 100.00% | 100.00% |
| `noun/nt-n/noun:gen:du` | 6 | 6 | 6 | 100.00% | 100.00% |
| `noun/nt-n/noun:gen:pl` | 6 | 6 | 6 | 100.00% | 100.00% |
| `noun/nt-n/noun:gen:sg` | 6 | 6 | 6 | 100.00% | 100.00% |
| `noun/nt-n/noun:ins:du` | 6 | 6 | 6 | 100.00% | 100.00% |
| `noun/nt-n/noun:ins:pl` | 6 | 6 | 6 | 100.00% | 100.00% |
| `noun/nt-n/noun:ins:sg` | 6 | 6 | 6 | 100.00% | 100.00% |
| `noun/nt-n/noun:loc:du` | 6 | 6 | 6 | 100.00% | 100.00% |
| `noun/nt-n/noun:loc:pl` | 6 | 6 | 6 | 100.00% | 100.00% |
| `noun/nt-n/noun:loc:sg` | 6 | 6 | 6 | 100.00% | 100.00% |
| `noun/nt-n/noun:nom:du` | 6 | 6 | 6 | 100.00% | 100.00% |
| `noun/nt-n/noun:nom:pl` | 6 | 6 | 6 | 100.00% | 100.00% |
| `noun/nt-n/noun:nom:sg` | 6 | 6 | 6 | 100.00% | 100.00% |
| `noun/nt-n/noun:voc:du` | 6 | 6 | 6 | 100.00% | 100.00% |
| `noun/nt-n/noun:voc:pl` | 6 | 6 | 6 | 100.00% | 100.00% |
| `noun/nt-n/noun:voc:sg` | 6 | 6 | 6 | 100.00% | 100.00% |
| `noun/o-m-hard/noun:dat:du` | 485 | 485 | 492 | 98.58% | 98.58% |
| `noun/o-m-hard/noun:dat:pl` | 475 | 475 | 492 | 96.54% | 96.54% |
| `noun/o-m-hard/noun:dat:sg` | 499 | 499 | 506 | 98.62% | 98.62% |
| `noun/o-m-hard/noun:gen:du` | 485 | 485 | 492 | 98.58% | 98.58% |
| `noun/o-m-hard/noun:gen:pl` | 475 | 475 | 492 | 96.54% | 96.54% |
| `noun/o-m-hard/noun:gen:sg` | 499 | 499 | 506 | 98.62% | 98.62% |
| `noun/o-m-hard/noun:ins:du` | 485 | 485 | 492 | 98.58% | 98.58% |
| `noun/o-m-hard/noun:ins:pl` | 475 | 475 | 492 | 96.54% | 96.54% |
| `noun/o-m-hard/noun:ins:sg` | 489 | 489 | 506 | 96.64% | 96.64% |
| `noun/o-m-hard/noun:loc:du` | 485 | 485 | 492 | 98.58% | 98.58% |
| `noun/o-m-hard/noun:loc:pl` | 471 | 471 | 492 | 95.73% | 95.73% |
| `noun/o-m-hard/noun:loc:sg` | 495 | 495 | 506 | 97.83% | 97.83% |
| `noun/o-m-hard/noun:nom:du` | 485 | 485 | 492 | 98.58% | 98.58% |
| `noun/o-m-hard/noun:nom:pl` | 471 | 471 | 492 | 95.73% | 95.73% |
| `noun/o-m-hard/noun:nom:sg` | 499 | 499 | 506 | 98.62% | 98.62% |
| `noun/o-m-hard/noun:voc:du` | 485 | 485 | 492 | 98.58% | 98.58% |
| `noun/o-m-hard/noun:voc:pl` | 471 | 471 | 492 | 95.73% | 95.73% |
| `noun/o-m-hard/noun:voc:sg` | 495 | 495 | 506 | 97.83% | 97.83% |
| `noun/o-n-hard/noun:acc:du` | 127 | 127 | 128 | 99.22% | 99.22% |
| `noun/o-n-hard/noun:acc:pl` | 127 | 127 | 128 | 99.22% | 99.22% |
| `noun/o-n-hard/noun:acc:sg` | 127 | 127 | 128 | 99.22% | 99.22% |
| `noun/o-n-hard/noun:dat:du` | 127 | 127 | 128 | 99.22% | 99.22% |
| `noun/o-n-hard/noun:dat:pl` | 127 | 127 | 128 | 99.22% | 99.22% |
| `noun/o-n-hard/noun:dat:sg` | 127 | 127 | 128 | 99.22% | 99.22% |
| `noun/o-n-hard/noun:gen:du` | 127 | 127 | 128 | 99.22% | 99.22% |
| `noun/o-n-hard/noun:gen:pl` | 127 | 127 | 128 | 99.22% | 99.22% |
| `noun/o-n-hard/noun:gen:sg` | 127 | 127 | 128 | 99.22% | 99.22% |
| `noun/o-n-hard/noun:ins:du` | 127 | 127 | 128 | 99.22% | 99.22% |
| `noun/o-n-hard/noun:ins:pl` | 127 | 127 | 128 | 99.22% | 99.22% |
| `noun/o-n-hard/noun:ins:sg` | 127 | 127 | 128 | 99.22% | 99.22% |
| `noun/o-n-hard/noun:loc:du` | 127 | 127 | 128 | 99.22% | 99.22% |
| `noun/o-n-hard/noun:loc:pl` | 127 | 127 | 128 | 99.22% | 99.22% |
| `noun/o-n-hard/noun:loc:sg` | 127 | 127 | 128 | 99.22% | 99.22% |
| `noun/o-n-hard/noun:nom:du` | 127 | 127 | 128 | 99.22% | 99.22% |
| `noun/o-n-hard/noun:nom:pl` | 127 | 127 | 128 | 99.22% | 99.22% |
| `noun/o-n-hard/noun:nom:sg` | 127 | 127 | 128 | 99.22% | 99.22% |
| `noun/o-n-hard/noun:voc:du` | 127 | 127 | 128 | 99.22% | 99.22% |
| `noun/o-n-hard/noun:voc:pl` | 127 | 127 | 128 | 99.22% | 99.22% |
| `noun/o-n-hard/noun:voc:sg` | 127 | 127 | 128 | 99.22% | 99.22% |
| `noun/r-n/noun:acc:du` | 2 | 2 | 2 | 100.00% | 100.00% |
| `noun/r-n/noun:acc:pl` | 2 | 2 | 2 | 100.00% | 100.00% |
| `noun/r-n/noun:acc:sg` | 2 | 2 | 2 | 100.00% | 100.00% |
| `noun/r-n/noun:dat:du` | 2 | 2 | 2 | 100.00% | 100.00% |
| `noun/r-n/noun:dat:pl` | 2 | 2 | 2 | 100.00% | 100.00% |
| `noun/r-n/noun:dat:sg` | 2 | 2 | 2 | 100.00% | 100.00% |
| `noun/r-n/noun:gen:du` | 2 | 2 | 2 | 100.00% | 100.00% |
| `noun/r-n/noun:gen:pl` | 2 | 2 | 2 | 100.00% | 100.00% |
| `noun/r-n/noun:gen:sg` | 2 | 2 | 2 | 100.00% | 100.00% |
| `noun/r-n/noun:ins:du` | 2 | 2 | 2 | 100.00% | 100.00% |
| `noun/r-n/noun:ins:pl` | 2 | 2 | 2 | 100.00% | 100.00% |
| `noun/r-n/noun:ins:sg` | 2 | 2 | 2 | 100.00% | 100.00% |
| `noun/r-n/noun:loc:du` | 2 | 2 | 2 | 100.00% | 100.00% |
| `noun/r-n/noun:loc:pl` | 2 | 2 | 2 | 100.00% | 100.00% |
| `noun/r-n/noun:loc:sg` | 2 | 2 | 2 | 100.00% | 100.00% |
| `noun/r-n/noun:nom:du` | 2 | 2 | 2 | 100.00% | 100.00% |
| `noun/r-n/noun:nom:pl` | 2 | 2 | 2 | 100.00% | 100.00% |
| `noun/r-n/noun:nom:sg` | 2 | 2 | 2 | 100.00% | 100.00% |
| `noun/r-n/noun:voc:du` | 2 | 2 | 2 | 100.00% | 100.00% |
| `noun/r-n/noun:voc:pl` | 2 | 2 | 2 | 100.00% | 100.00% |
| `noun/r-n/noun:voc:sg` | 2 | 2 | 2 | 100.00% | 100.00% |
| `noun/s-n/noun:acc:du` | 10 | 10 | 12 | 83.33% | 83.33% |
| `noun/s-n/noun:acc:pl` | 10 | 10 | 12 | 83.33% | 83.33% |
| `noun/s-n/noun:acc:sg` | 12 | 12 | 12 | 100.00% | 100.00% |
| `noun/s-n/noun:dat:du` | 10 | 10 | 12 | 83.33% | 83.33% |
| `noun/s-n/noun:dat:pl` | 10 | 10 | 12 | 83.33% | 83.33% |
| `noun/s-n/noun:dat:sg` | 10 | 10 | 12 | 83.33% | 83.33% |
| `noun/s-n/noun:gen:du` | 10 | 10 | 12 | 83.33% | 83.33% |
| `noun/s-n/noun:gen:pl` | 10 | 10 | 12 | 83.33% | 83.33% |
| `noun/s-n/noun:gen:sg` | 10 | 10 | 12 | 83.33% | 83.33% |
| `noun/s-n/noun:ins:du` | 10 | 10 | 12 | 83.33% | 83.33% |
| `noun/s-n/noun:ins:pl` | 10 | 10 | 12 | 83.33% | 83.33% |
| `noun/s-n/noun:ins:sg` | 10 | 10 | 12 | 83.33% | 83.33% |
| `noun/s-n/noun:loc:du` | 10 | 10 | 12 | 83.33% | 83.33% |
| `noun/s-n/noun:loc:pl` | 10 | 10 | 12 | 83.33% | 83.33% |
| `noun/s-n/noun:loc:sg` | 10 | 10 | 12 | 83.33% | 83.33% |
| `noun/s-n/noun:nom:du` | 10 | 10 | 12 | 83.33% | 83.33% |
| `noun/s-n/noun:nom:pl` | 10 | 10 | 12 | 83.33% | 83.33% |
| `noun/s-n/noun:nom:sg` | 12 | 12 | 12 | 100.00% | 100.00% |
| `noun/s-n/noun:voc:du` | 10 | 10 | 12 | 83.33% | 83.33% |
| `noun/s-n/noun:voc:pl` | 10 | 10 | 12 | 83.33% | 83.33% |
| `noun/s-n/noun:voc:sg` | 12 | 12 | 12 | 100.00% | 100.00% |
| `noun/u-m/noun:dat:du` | 21 | 21 | 21 | 100.00% | 100.00% |
| `noun/u-m/noun:dat:pl` | 21 | 21 | 21 | 100.00% | 100.00% |
| `noun/u-m/noun:dat:sg` | 21 | 21 | 21 | 100.00% | 100.00% |
| `noun/u-m/noun:gen:du` | 21 | 21 | 21 | 100.00% | 100.00% |
| `noun/u-m/noun:gen:pl` | 21 | 21 | 21 | 100.00% | 100.00% |
| `noun/u-m/noun:gen:sg` | 21 | 21 | 21 | 100.00% | 100.00% |
| `noun/u-m/noun:ins:du` | 21 | 21 | 21 | 100.00% | 100.00% |
| `noun/u-m/noun:ins:pl` | 21 | 21 | 21 | 100.00% | 100.00% |
| `noun/u-m/noun:ins:sg` | 21 | 21 | 21 | 100.00% | 100.00% |
| `noun/u-m/noun:loc:du` | 21 | 21 | 21 | 100.00% | 100.00% |
| `noun/u-m/noun:loc:pl` | 21 | 21 | 21 | 100.00% | 100.00% |
| `noun/u-m/noun:loc:sg` | 21 | 21 | 21 | 100.00% | 100.00% |
| `noun/u-m/noun:nom:du` | 21 | 21 | 21 | 100.00% | 100.00% |
| `noun/u-m/noun:nom:pl` | 21 | 21 | 21 | 100.00% | 100.00% |
| `noun/u-m/noun:nom:sg` | 21 | 21 | 21 | 100.00% | 100.00% |
| `noun/u-m/noun:voc:du` | 21 | 21 | 21 | 100.00% | 100.00% |
| `noun/u-m/noun:voc:pl` | 21 | 21 | 21 | 100.00% | 100.00% |
| `noun/u-m/noun:voc:sg` | 21 | 21 | 21 | 100.00% | 100.00% |
| `noun/v-f/noun:acc:du` | 10 | 10 | 10 | 100.00% | 100.00% |
| `noun/v-f/noun:acc:pl` | 10 | 10 | 10 | 100.00% | 100.00% |
| `noun/v-f/noun:acc:sg` | 10 | 10 | 10 | 100.00% | 100.00% |
| `noun/v-f/noun:dat:du` | 10 | 10 | 10 | 100.00% | 100.00% |
| `noun/v-f/noun:dat:pl` | 10 | 10 | 10 | 100.00% | 100.00% |
| `noun/v-f/noun:dat:sg` | 10 | 10 | 10 | 100.00% | 100.00% |
| `noun/v-f/noun:gen:du` | 10 | 10 | 10 | 100.00% | 100.00% |
| `noun/v-f/noun:gen:pl` | 10 | 10 | 10 | 100.00% | 100.00% |
| `noun/v-f/noun:gen:sg` | 10 | 10 | 10 | 100.00% | 100.00% |
| `noun/v-f/noun:ins:du` | 10 | 10 | 10 | 100.00% | 100.00% |
| `noun/v-f/noun:ins:pl` | 10 | 10 | 10 | 100.00% | 100.00% |
| `noun/v-f/noun:ins:sg` | 10 | 10 | 10 | 100.00% | 100.00% |
| `noun/v-f/noun:loc:du` | 10 | 10 | 10 | 100.00% | 100.00% |
| `noun/v-f/noun:loc:pl` | 10 | 10 | 10 | 100.00% | 100.00% |
| `noun/v-f/noun:loc:sg` | 10 | 10 | 10 | 100.00% | 100.00% |
| `noun/v-f/noun:nom:du` | 10 | 10 | 10 | 100.00% | 100.00% |
| `noun/v-f/noun:nom:pl` | 10 | 10 | 10 | 100.00% | 100.00% |
| `noun/v-f/noun:nom:sg` | 10 | 10 | 10 | 100.00% | 100.00% |
| `noun/v-f/noun:voc:du` | 10 | 10 | 10 | 100.00% | 100.00% |
| `noun/v-f/noun:voc:pl` | 10 | 10 | 10 | 100.00% | 100.00% |
| `noun/v-f/noun:voc:sg` | 10 | 10 | 10 | 100.00% | 100.00% |
| `verb/verb-IA1-present/verb:finite:present:1:du` | 75 | 75 | 75 | 100.00% | 100.00% |
| `verb/verb-IA1-present/verb:finite:present:1:pl` | 75 | 75 | 75 | 100.00% | 100.00% |
| `verb/verb-IA1-present/verb:finite:present:1:sg` | 54 | 54 | 75 | 72.00% | 72.00% |
| `verb/verb-IA1-present/verb:finite:present:2:du` | 75 | 75 | 75 | 100.00% | 100.00% |
| `verb/verb-IA1-present/verb:finite:present:2:pl` | 75 | 75 | 75 | 100.00% | 100.00% |
| `verb/verb-IA1-present/verb:finite:present:3:du` | 75 | 75 | 75 | 100.00% | 100.00% |
| `verb/verb-IA1-present/verb:finite:present:3:pl` | 54 | 54 | 75 | 72.00% | 72.00% |
| `verb/verb-IA1-present/verb:finite:present:3:sg` | 75 | 75 | 75 | 100.00% | 100.00% |
| `verb/verb-IA2-present/verb:finite:present:1:du` | 2 | 2 | 2 | 100.00% | 100.00% |
| `verb/verb-IA2-present/verb:finite:present:1:pl` | 2 | 2 | 2 | 100.00% | 100.00% |
| `verb/verb-IA2-present/verb:finite:present:1:sg` | 2 | 2 | 2 | 100.00% | 100.00% |
| `verb/verb-IA2-present/verb:finite:present:2:du` | 2 | 2 | 2 | 100.00% | 100.00% |
| `verb/verb-IA2-present/verb:finite:present:2:pl` | 2 | 2 | 2 | 100.00% | 100.00% |
| `verb/verb-IA2-present/verb:finite:present:3:du` | 2 | 2 | 2 | 100.00% | 100.00% |
| `verb/verb-IA2-present/verb:finite:present:3:pl` | 2 | 2 | 2 | 100.00% | 100.00% |
| `verb/verb-IA2-present/verb:finite:present:3:sg` | 2 | 2 | 2 | 100.00% | 100.00% |
| `verb/verb-II1-present/verb:finite:present:1:du` | 37 | 37 | 37 | 100.00% | 100.00% |
| `verb/verb-II1-present/verb:finite:present:1:pl` | 37 | 37 | 37 | 100.00% | 100.00% |
| `verb/verb-II1-present/verb:finite:present:2:du` | 37 | 37 | 37 | 100.00% | 100.00% |
| `verb/verb-II1-present/verb:finite:present:2:pl` | 37 | 37 | 37 | 100.00% | 100.00% |
| `verb/verb-II1-present/verb:finite:present:3:du` | 37 | 37 | 37 | 100.00% | 100.00% |
| `verb/verb-II1-present/verb:finite:present:3:pl` | 37 | 37 | 37 | 100.00% | 100.00% |
| `verb/verb-II1-present/verb:finite:present:3:sg` | 37 | 37 | 37 | 100.00% | 100.00% |
| `verb/verb-II2-present/verb:finite:present:1:du` | 20 | 20 | 20 | 100.00% | 100.00% |
| `verb/verb-II2-present/verb:finite:present:1:pl` | 20 | 20 | 20 | 100.00% | 100.00% |
| `verb/verb-II2-present/verb:finite:present:2:du` | 20 | 20 | 20 | 100.00% | 100.00% |
| `verb/verb-II2-present/verb:finite:present:2:pl` | 20 | 20 | 20 | 100.00% | 100.00% |
| `verb/verb-II2-present/verb:finite:present:3:du` | 20 | 20 | 20 | 100.00% | 100.00% |
| `verb/verb-II2-present/verb:finite:present:3:pl` | 20 | 20 | 20 | 100.00% | 100.00% |
| `verb/verb-II2-present/verb:finite:present:3:sg` | 20 | 20 | 20 | 100.00% | 100.00% |
| `verb/verb-II3-present/verb:finite:present:1:du` | 2 | 2 | 2 | 100.00% | 100.00% |
| `verb/verb-II3-present/verb:finite:present:1:pl` | 2 | 2 | 2 | 100.00% | 100.00% |
| `verb/verb-II3-present/verb:finite:present:2:du` | 2 | 2 | 2 | 100.00% | 100.00% |
| `verb/verb-II3-present/verb:finite:present:2:pl` | 2 | 2 | 2 | 100.00% | 100.00% |
| `verb/verb-II3-present/verb:finite:present:3:du` | 2 | 2 | 2 | 100.00% | 100.00% |
| `verb/verb-II3-present/verb:finite:present:3:pl` | 2 | 2 | 2 | 100.00% | 100.00% |
| `verb/verb-II3-present/verb:finite:present:3:sg` | 2 | 2 | 2 | 100.00% | 100.00% |
| `verb/verb-imperative/verb:imperative:1:du` | 57 | 57 | 134 | 42.54% | 42.54% |
| `verb/verb-imperative/verb:imperative:1:pl` | 134 | 134 | 134 | 100.00% | 100.00% |
| `verb/verb-imperative/verb:imperative:2:du` | 134 | 134 | 134 | 100.00% | 100.00% |
| `verb/verb-imperative/verb:imperative:2:pl` | 134 | 134 | 134 | 100.00% | 100.00% |
| `verb/verb-imperfect/verb:finite:imperfect:1:du` | 138 | 138 | 138 | 100.00% | 100.00% |
| `verb/verb-imperfect/verb:finite:imperfect:1:pl` | 138 | 138 | 138 | 100.00% | 100.00% |
| `verb/verb-imperfect/verb:finite:imperfect:2:du` | 138 | 138 | 138 | 100.00% | 100.00% |
| `verb/verb-imperfect/verb:finite:imperfect:2:pl` | 138 | 138 | 138 | 100.00% | 100.00% |
| `verb/verb-imperfect/verb:finite:imperfect:2:sg` | 138 | 138 | 138 | 100.00% | 100.00% |
| `verb/verb-imperfect/verb:finite:imperfect:3:du` | 138 | 138 | 138 | 100.00% | 100.00% |
| `verb/verb-imperfect/verb:finite:imperfect:3:pl` | 138 | 138 | 138 | 100.00% | 100.00% |
| `verb/verb-imperfect/verb:finite:imperfect:3:sg` | 138 | 138 | 138 | 100.00% | 100.00% |
| `verb/verb-infinitive/verb:infinitive` | 559 | 559 | 559 | 100.00% | 100.00% |
| `verb/verb-l-participle/verb:l-participle:f:du` | 554 | 554 | 554 | 100.00% | 100.00% |
| `verb/verb-l-participle/verb:l-participle:f:pl` | 554 | 554 | 554 | 100.00% | 100.00% |
| `verb/verb-l-participle/verb:l-participle:f:sg` | 554 | 554 | 554 | 100.00% | 100.00% |
| `verb/verb-l-participle/verb:l-participle:m:du` | 554 | 554 | 554 | 100.00% | 100.00% |
| `verb/verb-l-participle/verb:l-participle:m:pl` | 554 | 554 | 554 | 100.00% | 100.00% |
| `verb/verb-l-participle/verb:l-participle:n:du` | 554 | 554 | 554 | 100.00% | 100.00% |
| `verb/verb-l-participle/verb:l-participle:n:pl` | 554 | 554 | 554 | 100.00% | 100.00% |
| `verb/verb-l-participle/verb:l-participle:n:sg` | 554 | 554 | 554 | 100.00% | 100.00% |
| `verb/verb-past-active-participle-ush/verb:participle:past-active:citation` | 29 | 29 | 69 | 42.03% | 42.03% |
| `verb/verb-past-active-participle-vush/verb:participle:past-active:citation` | 65 | 65 | 71 | 91.55% | 91.55% |
| `verb/verb-past-passive-participle-en/verb:participle:past-passive:citation` | 7 | 7 | 49 | 14.29% | 14.29% |
| `verb/verb-present-active-participle-yesht-soft/verb:participle:present-active:citation` | 59 | 59 | 59 | 100.00% | 100.00% |
| `verb/verb-present-active-participle-yusht-hard/verb:participle:present-active:citation` | 56 | 56 | 77 | 72.73% | 72.73% |
| `verb/verb-present-passive-participle-im/verb:participle:present-passive:citation` | 45 | 45 | 45 | 100.00% | 100.00% |
| `verb/verb-present-passive-participle-om/verb:participle:present-passive:citation` | 33 | 33 | 49 | 67.35% | 67.35% |
| `verb/verb-supine/verb:supine` | 544 | 544 | 559 | 97.32% | 97.32% |

## Held-out OOV

| Rule slice | Exact | NFC/lowercase | Total | Exact recall | Normalized recall |
|---|---:|---:|---:|---:|---:|
| `a-hard` | 1237 | 1237 | 1449 | 85.37% | 85.37% |
| `adj-hard-long` | 7542 | 7542 | 7794 | 96.77% | 96.77% |
| `adj-hard-short` | 7294 | 7294 | 7794 | 93.58% | 93.58% |
| `adj-soft-long` | 372 | 372 | 378 | 98.41% | 98.41% |
| `adj-soft-short` | 186 | 186 | 378 | 49.21% | 49.21% |
| `i-f` | 780 | 780 | 819 | 95.24% | 95.24% |
| `i-m` | 158 | 158 | 162 | 97.53% | 97.53% |
| `ja-soft` | 276 | 276 | 364 | 75.82% | 75.82% |
| `jo-m-soft` | 704 | 704 | 954 | 73.79% | 73.79% |
| `jo-n-soft` | 676 | 676 | 763 | 88.60% | 88.60% |
| `n-m` | 18 | 18 | 18 | 100.00% | 100.00% |
| `n-n` | 21 | 21 | 21 | 100.00% | 100.00% |
| `nt-n` | 84 | 84 | 84 | 100.00% | 100.00% |
| `o-m-hard` | 2027 | 2027 | 2058 | 98.49% | 98.49% |
| `o-n-hard` | 819 | 819 | 819 | 100.00% | 100.00% |
| `s-n` | 21 | 21 | 21 | 100.00% | 100.00% |
| `u-m` | 72 | 72 | 72 | 100.00% | 100.00% |
| `v-f` | 84 | 84 | 84 | 100.00% | 100.00% |
| `verb-IA1-present` | 230 | 230 | 240 | 95.83% | 95.83% |
| `verb-IA2-present` | 6 | 6 | 8 | 75.00% | 75.00% |
| `verb-II1-present` | 63 | 63 | 63 | 100.00% | 100.00% |
| `verb-II2-present` | 42 | 42 | 42 | 100.00% | 100.00% |
| `verb-II3-present` | 14 | 14 | 14 | 100.00% | 100.00% |
| `verb-imperative` | 157 | 157 | 188 | 83.51% | 83.51% |
| `verb-imperfect` | 384 | 384 | 384 | 100.00% | 100.00% |
| `verb-infinitive` | 152 | 152 | 152 | 100.00% | 100.00% |
| `verb-l-participle` | 1216 | 1216 | 1216 | 100.00% | 100.00% |
| `verb-past-active-participle-ush` | 14 | 14 | 25 | 56.00% | 56.00% |
| `verb-past-active-participle-vush` | 20 | 20 | 25 | 80.00% | 80.00% |
| `verb-past-passive-participle-en` | 8 | 8 | 23 | 34.78% | 34.78% |
| `verb-present-active-participle-yesht-soft` | 17 | 17 | 17 | 100.00% | 100.00% |
| `verb-present-active-participle-yusht-hard` | 25 | 25 | 31 | 80.65% | 80.65% |
| `verb-present-passive-participle-im` | 15 | 15 | 15 | 100.00% | 100.00% |
| `verb-present-passive-participle-om` | 19 | 19 | 23 | 82.61% | 82.61% |
| `verb-supine` | 145 | 145 | 151 | 96.03% | 96.03% |

Macro average across reported rule slices: 89.64% exact, 89.64% normalized.

### POS, class, and cell detail

| Cell slice | Exact | NFC/lowercase | Total | Exact recall | Normalized recall |
|---|---:|---:|---:|---:|---:|
| `adj/adj-hard-long/adj:long:acc:du:f:an` | 60 | 60 | 62 | 96.77% | 96.77% |
| `adj/adj-hard-long/adj:long:acc:du:f:in` | 60 | 60 | 62 | 96.77% | 96.77% |
| `adj/adj-hard-long/adj:long:acc:du:m:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:acc:du:m:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:acc:du:n:an` | 60 | 60 | 62 | 96.77% | 96.77% |
| `adj/adj-hard-long/adj:long:acc:du:n:in` | 60 | 60 | 62 | 96.77% | 96.77% |
| `adj/adj-hard-long/adj:long:acc:pl:f:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:acc:pl:f:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:acc:pl:m:an` | 0 | 0 | 62 | 0.00% | 0.00% |
| `adj/adj-hard-long/adj:long:acc:pl:m:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:acc:pl:n:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:acc:pl:n:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:acc:sg:f:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:acc:sg:f:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:acc:sg:m:an` | 0 | 0 | 62 | 0.00% | 0.00% |
| `adj/adj-hard-long/adj:long:acc:sg:m:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:acc:sg:n:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:acc:sg:n:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:dat:du:f:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:dat:du:f:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:dat:du:m:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:dat:du:m:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:dat:du:n:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:dat:du:n:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:dat:pl:f:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:dat:pl:f:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:dat:pl:m:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:dat:pl:m:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:dat:pl:n:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:dat:pl:n:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:dat:sg:f:an` | 60 | 60 | 62 | 96.77% | 96.77% |
| `adj/adj-hard-long/adj:long:dat:sg:f:in` | 60 | 60 | 62 | 96.77% | 96.77% |
| `adj/adj-hard-long/adj:long:dat:sg:m:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:dat:sg:m:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:dat:sg:n:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:dat:sg:n:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:gen:du:f:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:gen:du:f:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:gen:du:m:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:gen:du:m:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:gen:du:n:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:gen:du:n:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:gen:pl:f:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:gen:pl:f:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:gen:pl:m:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:gen:pl:m:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:gen:pl:n:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:gen:pl:n:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:gen:sg:f:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:gen:sg:f:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:gen:sg:m:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:gen:sg:m:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:gen:sg:n:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:gen:sg:n:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:ins:du:f:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:ins:du:f:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:ins:du:m:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:ins:du:m:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:ins:du:n:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:ins:du:n:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:ins:pl:f:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:ins:pl:f:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:ins:pl:m:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:ins:pl:m:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:ins:pl:n:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:ins:pl:n:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:ins:sg:f:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:ins:sg:f:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:ins:sg:m:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:ins:sg:m:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:ins:sg:n:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:ins:sg:n:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:loc:du:f:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:loc:du:f:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:loc:du:m:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:loc:du:m:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:loc:du:n:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:loc:du:n:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:loc:pl:f:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:loc:pl:f:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:loc:pl:m:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:loc:pl:m:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:loc:pl:n:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:loc:pl:n:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:loc:sg:f:an` | 60 | 60 | 62 | 96.77% | 96.77% |
| `adj/adj-hard-long/adj:long:loc:sg:f:in` | 60 | 60 | 62 | 96.77% | 96.77% |
| `adj/adj-hard-long/adj:long:loc:sg:m:an` | 60 | 60 | 62 | 96.77% | 96.77% |
| `adj/adj-hard-long/adj:long:loc:sg:m:in` | 60 | 60 | 62 | 96.77% | 96.77% |
| `adj/adj-hard-long/adj:long:loc:sg:n:an` | 60 | 60 | 62 | 96.77% | 96.77% |
| `adj/adj-hard-long/adj:long:loc:sg:n:in` | 60 | 60 | 62 | 96.77% | 96.77% |
| `adj/adj-hard-long/adj:long:nom:du:f:an` | 60 | 60 | 62 | 96.77% | 96.77% |
| `adj/adj-hard-long/adj:long:nom:du:f:in` | 60 | 60 | 62 | 96.77% | 96.77% |
| `adj/adj-hard-long/adj:long:nom:du:m:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:nom:du:m:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:nom:du:n:an` | 60 | 60 | 62 | 96.77% | 96.77% |
| `adj/adj-hard-long/adj:long:nom:du:n:in` | 60 | 60 | 62 | 96.77% | 96.77% |
| `adj/adj-hard-long/adj:long:nom:pl:f:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:nom:pl:f:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:nom:pl:m:an` | 60 | 60 | 62 | 96.77% | 96.77% |
| `adj/adj-hard-long/adj:long:nom:pl:m:in` | 60 | 60 | 62 | 96.77% | 96.77% |
| `adj/adj-hard-long/adj:long:nom:pl:n:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:nom:pl:n:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:nom:sg:f:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:nom:sg:f:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:nom:sg:m:an` | 62 | 62 | 62 | 100.00% | 100.00% |
| `adj/adj-hard-long/adj:long:nom:sg:m:in` | 62 | 62 | 62 | 100.00% | 100.00% |
| `adj/adj-hard-long/adj:long:nom:sg:n:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:nom:sg:n:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-long/adj:long:voc:du:f:an` | 60 | 60 | 61 | 98.36% | 98.36% |
| `adj/adj-hard-long/adj:long:voc:du:f:in` | 60 | 60 | 61 | 98.36% | 98.36% |
| `adj/adj-hard-long/adj:long:voc:du:m:an` | 61 | 61 | 61 | 100.00% | 100.00% |
| `adj/adj-hard-long/adj:long:voc:du:m:in` | 61 | 61 | 61 | 100.00% | 100.00% |
| `adj/adj-hard-long/adj:long:voc:du:n:an` | 60 | 60 | 61 | 98.36% | 98.36% |
| `adj/adj-hard-long/adj:long:voc:du:n:in` | 60 | 60 | 61 | 98.36% | 98.36% |
| `adj/adj-hard-long/adj:long:voc:pl:f:an` | 61 | 61 | 61 | 100.00% | 100.00% |
| `adj/adj-hard-long/adj:long:voc:pl:f:in` | 61 | 61 | 61 | 100.00% | 100.00% |
| `adj/adj-hard-long/adj:long:voc:pl:m:an` | 60 | 60 | 61 | 98.36% | 98.36% |
| `adj/adj-hard-long/adj:long:voc:pl:m:in` | 60 | 60 | 61 | 98.36% | 98.36% |
| `adj/adj-hard-long/adj:long:voc:pl:n:an` | 61 | 61 | 61 | 100.00% | 100.00% |
| `adj/adj-hard-long/adj:long:voc:pl:n:in` | 61 | 61 | 61 | 100.00% | 100.00% |
| `adj/adj-hard-long/adj:long:voc:sg:f:an` | 61 | 61 | 61 | 100.00% | 100.00% |
| `adj/adj-hard-long/adj:long:voc:sg:f:in` | 61 | 61 | 61 | 100.00% | 100.00% |
| `adj/adj-hard-long/adj:long:voc:sg:m:an` | 61 | 61 | 61 | 100.00% | 100.00% |
| `adj/adj-hard-long/adj:long:voc:sg:m:in` | 61 | 61 | 61 | 100.00% | 100.00% |
| `adj/adj-hard-long/adj:long:voc:sg:n:an` | 61 | 61 | 61 | 100.00% | 100.00% |
| `adj/adj-hard-long/adj:long:voc:sg:n:in` | 61 | 61 | 61 | 100.00% | 100.00% |
| `adj/adj-hard-short/adj:short:acc:du:f:an` | 60 | 60 | 62 | 96.77% | 96.77% |
| `adj/adj-hard-short/adj:short:acc:du:f:in` | 60 | 60 | 62 | 96.77% | 96.77% |
| `adj/adj-hard-short/adj:short:acc:du:m:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:acc:du:m:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:acc:du:n:an` | 60 | 60 | 62 | 96.77% | 96.77% |
| `adj/adj-hard-short/adj:short:acc:du:n:in` | 60 | 60 | 62 | 96.77% | 96.77% |
| `adj/adj-hard-short/adj:short:acc:pl:f:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:acc:pl:f:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:acc:pl:m:an` | 0 | 0 | 62 | 0.00% | 0.00% |
| `adj/adj-hard-short/adj:short:acc:pl:m:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:acc:pl:n:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:acc:pl:n:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:acc:sg:f:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:acc:sg:f:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:acc:sg:m:an` | 0 | 0 | 62 | 0.00% | 0.00% |
| `adj/adj-hard-short/adj:short:acc:sg:m:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:acc:sg:n:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:acc:sg:n:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:dat:du:f:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:dat:du:f:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:dat:du:m:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:dat:du:m:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:dat:du:n:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:dat:du:n:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:dat:pl:f:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:dat:pl:f:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:dat:pl:m:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:dat:pl:m:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:dat:pl:n:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:dat:pl:n:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:dat:sg:f:an` | 60 | 60 | 62 | 96.77% | 96.77% |
| `adj/adj-hard-short/adj:short:dat:sg:f:in` | 60 | 60 | 62 | 96.77% | 96.77% |
| `adj/adj-hard-short/adj:short:dat:sg:m:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:dat:sg:m:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:dat:sg:n:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:dat:sg:n:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:gen:du:f:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:gen:du:f:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:gen:du:m:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:gen:du:m:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:gen:du:n:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:gen:du:n:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:gen:pl:f:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:gen:pl:f:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:gen:pl:m:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:gen:pl:m:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:gen:pl:n:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:gen:pl:n:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:gen:sg:f:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:gen:sg:f:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:gen:sg:m:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:gen:sg:m:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:gen:sg:n:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:gen:sg:n:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:ins:du:f:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:ins:du:f:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:ins:du:m:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:ins:du:m:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:ins:du:n:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:ins:du:n:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:ins:pl:f:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:ins:pl:f:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:ins:pl:m:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:ins:pl:m:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:ins:pl:n:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:ins:pl:n:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:ins:sg:f:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:ins:sg:f:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:ins:sg:m:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:ins:sg:m:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:ins:sg:n:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:ins:sg:n:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:loc:du:f:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:loc:du:f:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:loc:du:m:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:loc:du:m:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:loc:du:n:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:loc:du:n:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:loc:pl:f:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:loc:pl:f:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:loc:pl:m:an` | 60 | 60 | 62 | 96.77% | 96.77% |
| `adj/adj-hard-short/adj:short:loc:pl:m:in` | 60 | 60 | 62 | 96.77% | 96.77% |
| `adj/adj-hard-short/adj:short:loc:pl:n:an` | 60 | 60 | 62 | 96.77% | 96.77% |
| `adj/adj-hard-short/adj:short:loc:pl:n:in` | 60 | 60 | 62 | 96.77% | 96.77% |
| `adj/adj-hard-short/adj:short:loc:sg:f:an` | 60 | 60 | 62 | 96.77% | 96.77% |
| `adj/adj-hard-short/adj:short:loc:sg:f:in` | 60 | 60 | 62 | 96.77% | 96.77% |
| `adj/adj-hard-short/adj:short:loc:sg:m:an` | 60 | 60 | 62 | 96.77% | 96.77% |
| `adj/adj-hard-short/adj:short:loc:sg:m:in` | 60 | 60 | 62 | 96.77% | 96.77% |
| `adj/adj-hard-short/adj:short:loc:sg:n:an` | 60 | 60 | 62 | 96.77% | 96.77% |
| `adj/adj-hard-short/adj:short:loc:sg:n:in` | 60 | 60 | 62 | 96.77% | 96.77% |
| `adj/adj-hard-short/adj:short:nom:du:f:an` | 60 | 60 | 62 | 96.77% | 96.77% |
| `adj/adj-hard-short/adj:short:nom:du:f:in` | 60 | 60 | 62 | 96.77% | 96.77% |
| `adj/adj-hard-short/adj:short:nom:du:m:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:nom:du:m:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:nom:du:n:an` | 60 | 60 | 62 | 96.77% | 96.77% |
| `adj/adj-hard-short/adj:short:nom:du:n:in` | 60 | 60 | 62 | 96.77% | 96.77% |
| `adj/adj-hard-short/adj:short:nom:pl:f:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:nom:pl:f:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:nom:pl:m:an` | 60 | 60 | 62 | 96.77% | 96.77% |
| `adj/adj-hard-short/adj:short:nom:pl:m:in` | 60 | 60 | 62 | 96.77% | 96.77% |
| `adj/adj-hard-short/adj:short:nom:pl:n:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:nom:pl:n:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:nom:sg:f:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:nom:sg:f:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:nom:sg:m:an` | 62 | 62 | 62 | 100.00% | 100.00% |
| `adj/adj-hard-short/adj:short:nom:sg:m:in` | 62 | 62 | 62 | 100.00% | 100.00% |
| `adj/adj-hard-short/adj:short:nom:sg:n:an` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:nom:sg:n:in` | 61 | 61 | 62 | 98.39% | 98.39% |
| `adj/adj-hard-short/adj:short:voc:du:f:an` | 60 | 60 | 61 | 98.36% | 98.36% |
| `adj/adj-hard-short/adj:short:voc:du:f:in` | 60 | 60 | 61 | 98.36% | 98.36% |
| `adj/adj-hard-short/adj:short:voc:du:m:an` | 61 | 61 | 61 | 100.00% | 100.00% |
| `adj/adj-hard-short/adj:short:voc:du:m:in` | 61 | 61 | 61 | 100.00% | 100.00% |
| `adj/adj-hard-short/adj:short:voc:du:n:an` | 60 | 60 | 61 | 98.36% | 98.36% |
| `adj/adj-hard-short/adj:short:voc:du:n:in` | 60 | 60 | 61 | 98.36% | 98.36% |
| `adj/adj-hard-short/adj:short:voc:pl:f:an` | 61 | 61 | 61 | 100.00% | 100.00% |
| `adj/adj-hard-short/adj:short:voc:pl:f:in` | 61 | 61 | 61 | 100.00% | 100.00% |
| `adj/adj-hard-short/adj:short:voc:pl:m:an` | 60 | 60 | 61 | 98.36% | 98.36% |
| `adj/adj-hard-short/adj:short:voc:pl:m:in` | 60 | 60 | 61 | 98.36% | 98.36% |
| `adj/adj-hard-short/adj:short:voc:pl:n:an` | 61 | 61 | 61 | 100.00% | 100.00% |
| `adj/adj-hard-short/adj:short:voc:pl:n:in` | 61 | 61 | 61 | 100.00% | 100.00% |
| `adj/adj-hard-short/adj:short:voc:sg:f:an` | 0 | 0 | 61 | 0.00% | 0.00% |
| `adj/adj-hard-short/adj:short:voc:sg:f:in` | 0 | 0 | 61 | 0.00% | 0.00% |
| `adj/adj-hard-short/adj:short:voc:sg:m:an` | 0 | 0 | 61 | 0.00% | 0.00% |
| `adj/adj-hard-short/adj:short:voc:sg:m:in` | 0 | 0 | 61 | 0.00% | 0.00% |
| `adj/adj-hard-short/adj:short:voc:sg:n:an` | 61 | 61 | 61 | 100.00% | 100.00% |
| `adj/adj-hard-short/adj:short:voc:sg:n:in` | 61 | 61 | 61 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:acc:du:f:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:acc:du:f:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:acc:du:m:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:acc:du:m:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:acc:du:n:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:acc:du:n:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:acc:pl:f:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:acc:pl:f:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:acc:pl:m:an` | 0 | 0 | 3 | 0.00% | 0.00% |
| `adj/adj-soft-long/adj:long:acc:pl:m:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:acc:pl:n:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:acc:pl:n:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:acc:sg:f:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:acc:sg:f:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:acc:sg:m:an` | 0 | 0 | 3 | 0.00% | 0.00% |
| `adj/adj-soft-long/adj:long:acc:sg:m:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:acc:sg:n:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:acc:sg:n:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:dat:du:f:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:dat:du:f:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:dat:du:m:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:dat:du:m:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:dat:du:n:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:dat:du:n:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:dat:pl:f:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:dat:pl:f:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:dat:pl:m:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:dat:pl:m:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:dat:pl:n:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:dat:pl:n:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:dat:sg:f:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:dat:sg:f:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:dat:sg:m:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:dat:sg:m:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:dat:sg:n:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:dat:sg:n:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:gen:du:f:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:gen:du:f:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:gen:du:m:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:gen:du:m:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:gen:du:n:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:gen:du:n:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:gen:pl:f:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:gen:pl:f:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:gen:pl:m:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:gen:pl:m:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:gen:pl:n:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:gen:pl:n:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:gen:sg:f:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:gen:sg:f:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:gen:sg:m:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:gen:sg:m:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:gen:sg:n:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:gen:sg:n:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:ins:du:f:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:ins:du:f:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:ins:du:m:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:ins:du:m:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:ins:du:n:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:ins:du:n:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:ins:pl:f:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:ins:pl:f:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:ins:pl:m:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:ins:pl:m:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:ins:pl:n:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:ins:pl:n:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:ins:sg:f:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:ins:sg:f:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:ins:sg:m:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:ins:sg:m:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:ins:sg:n:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:ins:sg:n:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:loc:du:f:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:loc:du:f:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:loc:du:m:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:loc:du:m:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:loc:du:n:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:loc:du:n:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:loc:pl:f:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:loc:pl:f:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:loc:pl:m:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:loc:pl:m:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:loc:pl:n:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:loc:pl:n:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:loc:sg:f:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:loc:sg:f:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:loc:sg:m:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:loc:sg:m:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:loc:sg:n:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:loc:sg:n:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:nom:du:f:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:nom:du:f:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:nom:du:m:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:nom:du:m:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:nom:du:n:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:nom:du:n:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:nom:pl:f:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:nom:pl:f:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:nom:pl:m:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:nom:pl:m:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:nom:pl:n:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:nom:pl:n:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:nom:sg:f:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:nom:sg:f:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:nom:sg:m:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:nom:sg:m:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:nom:sg:n:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:nom:sg:n:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:voc:du:f:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:voc:du:f:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:voc:du:m:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:voc:du:m:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:voc:du:n:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:voc:du:n:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:voc:pl:f:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:voc:pl:f:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:voc:pl:m:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:voc:pl:m:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:voc:pl:n:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:voc:pl:n:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:voc:sg:f:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:voc:sg:f:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:voc:sg:m:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:voc:sg:m:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:voc:sg:n:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-long/adj:long:voc:sg:n:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-short/adj:short:acc:du:f:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-short/adj:short:acc:du:f:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-short/adj:short:acc:du:m:an` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:acc:du:m:in` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:acc:du:n:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-short/adj:short:acc:du:n:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-short/adj:short:acc:pl:f:an` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:acc:pl:f:in` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:acc:pl:m:an` | 0 | 0 | 3 | 0.00% | 0.00% |
| `adj/adj-soft-short/adj:short:acc:pl:m:in` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:acc:pl:n:an` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:acc:pl:n:in` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:acc:sg:f:an` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:acc:sg:f:in` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:acc:sg:m:an` | 0 | 0 | 3 | 0.00% | 0.00% |
| `adj/adj-soft-short/adj:short:acc:sg:m:in` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:acc:sg:n:an` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:acc:sg:n:in` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:dat:du:f:an` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:dat:du:f:in` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:dat:du:m:an` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:dat:du:m:in` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:dat:du:n:an` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:dat:du:n:in` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:dat:pl:f:an` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:dat:pl:f:in` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:dat:pl:m:an` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:dat:pl:m:in` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:dat:pl:n:an` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:dat:pl:n:in` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:dat:sg:f:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-short/adj:short:dat:sg:f:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-short/adj:short:dat:sg:m:an` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:dat:sg:m:in` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:dat:sg:n:an` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:dat:sg:n:in` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:gen:du:f:an` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:gen:du:f:in` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:gen:du:m:an` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:gen:du:m:in` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:gen:du:n:an` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:gen:du:n:in` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:gen:pl:f:an` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:gen:pl:f:in` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:gen:pl:m:an` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:gen:pl:m:in` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:gen:pl:n:an` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:gen:pl:n:in` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:gen:sg:f:an` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:gen:sg:f:in` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:gen:sg:m:an` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:gen:sg:m:in` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:gen:sg:n:an` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:gen:sg:n:in` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:ins:du:f:an` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:ins:du:f:in` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:ins:du:m:an` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:ins:du:m:in` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:ins:du:n:an` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:ins:du:n:in` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:ins:pl:f:an` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:ins:pl:f:in` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:ins:pl:m:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-short/adj:short:ins:pl:m:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-short/adj:short:ins:pl:n:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-short/adj:short:ins:pl:n:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-short/adj:short:ins:sg:f:an` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:ins:sg:f:in` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:ins:sg:m:an` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:ins:sg:m:in` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:ins:sg:n:an` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:ins:sg:n:in` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:loc:du:f:an` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:loc:du:f:in` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:loc:du:m:an` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:loc:du:m:in` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:loc:du:n:an` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:loc:du:n:in` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:loc:pl:f:an` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:loc:pl:f:in` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:loc:pl:m:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-short/adj:short:loc:pl:m:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-short/adj:short:loc:pl:n:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-short/adj:short:loc:pl:n:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-short/adj:short:loc:sg:f:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-short/adj:short:loc:sg:f:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-short/adj:short:loc:sg:m:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-short/adj:short:loc:sg:m:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-short/adj:short:loc:sg:n:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-short/adj:short:loc:sg:n:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-short/adj:short:nom:du:f:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-short/adj:short:nom:du:f:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-short/adj:short:nom:du:m:an` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:nom:du:m:in` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:nom:du:n:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-short/adj:short:nom:du:n:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-short/adj:short:nom:pl:f:an` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:nom:pl:f:in` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:nom:pl:m:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-short/adj:short:nom:pl:m:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-short/adj:short:nom:pl:n:an` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:nom:pl:n:in` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:nom:sg:f:an` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:nom:sg:f:in` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:nom:sg:m:an` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:nom:sg:m:in` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:nom:sg:n:an` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:nom:sg:n:in` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:voc:du:f:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-short/adj:short:voc:du:f:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-short/adj:short:voc:du:m:an` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:voc:du:m:in` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:voc:du:n:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-short/adj:short:voc:du:n:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-short/adj:short:voc:pl:f:an` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:voc:pl:f:in` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:voc:pl:m:an` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-short/adj:short:voc:pl:m:in` | 3 | 3 | 3 | 100.00% | 100.00% |
| `adj/adj-soft-short/adj:short:voc:pl:n:an` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:voc:pl:n:in` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:voc:sg:f:an` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:voc:sg:f:in` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:voc:sg:m:an` | 0 | 0 | 3 | 0.00% | 0.00% |
| `adj/adj-soft-short/adj:short:voc:sg:m:in` | 0 | 0 | 3 | 0.00% | 0.00% |
| `adj/adj-soft-short/adj:short:voc:sg:n:an` | 1 | 1 | 3 | 33.33% | 33.33% |
| `adj/adj-soft-short/adj:short:voc:sg:n:in` | 1 | 1 | 3 | 33.33% | 33.33% |
| `noun/a-hard/noun:acc:du` | 49 | 49 | 69 | 71.01% | 71.01% |
| `noun/a-hard/noun:acc:pl` | 53 | 53 | 69 | 76.81% | 76.81% |
| `noun/a-hard/noun:acc:sg` | 69 | 69 | 69 | 100.00% | 100.00% |
| `noun/a-hard/noun:dat:du` | 69 | 69 | 69 | 100.00% | 100.00% |
| `noun/a-hard/noun:dat:pl` | 69 | 69 | 69 | 100.00% | 100.00% |
| `noun/a-hard/noun:dat:sg` | 49 | 49 | 69 | 71.01% | 71.01% |
| `noun/a-hard/noun:gen:du` | 69 | 69 | 69 | 100.00% | 100.00% |
| `noun/a-hard/noun:gen:pl` | 53 | 53 | 69 | 76.81% | 76.81% |
| `noun/a-hard/noun:gen:sg` | 53 | 53 | 69 | 76.81% | 76.81% |
| `noun/a-hard/noun:ins:du` | 69 | 69 | 69 | 100.00% | 100.00% |
| `noun/a-hard/noun:ins:pl` | 69 | 69 | 69 | 100.00% | 100.00% |
| `noun/a-hard/noun:ins:sg` | 53 | 53 | 69 | 76.81% | 76.81% |
| `noun/a-hard/noun:loc:du` | 69 | 69 | 69 | 100.00% | 100.00% |
| `noun/a-hard/noun:loc:pl` | 69 | 69 | 69 | 100.00% | 100.00% |
| `noun/a-hard/noun:loc:sg` | 49 | 49 | 69 | 71.01% | 71.01% |
| `noun/a-hard/noun:nom:du` | 49 | 49 | 69 | 71.01% | 71.01% |
| `noun/a-hard/noun:nom:pl` | 53 | 53 | 69 | 76.81% | 76.81% |
| `noun/a-hard/noun:nom:sg` | 69 | 69 | 69 | 100.00% | 100.00% |
| `noun/a-hard/noun:voc:du` | 49 | 49 | 69 | 71.01% | 71.01% |
| `noun/a-hard/noun:voc:pl` | 53 | 53 | 69 | 76.81% | 76.81% |
| `noun/a-hard/noun:voc:sg` | 53 | 53 | 69 | 76.81% | 76.81% |
| `noun/i-f/noun:acc:du` | 39 | 39 | 39 | 100.00% | 100.00% |
| `noun/i-f/noun:acc:pl` | 39 | 39 | 39 | 100.00% | 100.00% |
| `noun/i-f/noun:acc:sg` | 39 | 39 | 39 | 100.00% | 100.00% |
| `noun/i-f/noun:dat:du` | 39 | 39 | 39 | 100.00% | 100.00% |
| `noun/i-f/noun:dat:pl` | 39 | 39 | 39 | 100.00% | 100.00% |
| `noun/i-f/noun:dat:sg` | 39 | 39 | 39 | 100.00% | 100.00% |
| `noun/i-f/noun:gen:du` | 39 | 39 | 39 | 100.00% | 100.00% |
| `noun/i-f/noun:gen:pl` | 39 | 39 | 39 | 100.00% | 100.00% |
| `noun/i-f/noun:gen:sg` | 39 | 39 | 39 | 100.00% | 100.00% |
| `noun/i-f/noun:ins:du` | 39 | 39 | 39 | 100.00% | 100.00% |
| `noun/i-f/noun:ins:pl` | 39 | 39 | 39 | 100.00% | 100.00% |
| `noun/i-f/noun:ins:sg` | 39 | 39 | 39 | 100.00% | 100.00% |
| `noun/i-f/noun:loc:du` | 39 | 39 | 39 | 100.00% | 100.00% |
| `noun/i-f/noun:loc:pl` | 39 | 39 | 39 | 100.00% | 100.00% |
| `noun/i-f/noun:loc:sg` | 39 | 39 | 39 | 100.00% | 100.00% |
| `noun/i-f/noun:nom:du` | 39 | 39 | 39 | 100.00% | 100.00% |
| `noun/i-f/noun:nom:pl` | 39 | 39 | 39 | 100.00% | 100.00% |
| `noun/i-f/noun:nom:sg` | 39 | 39 | 39 | 100.00% | 100.00% |
| `noun/i-f/noun:voc:du` | 39 | 39 | 39 | 100.00% | 100.00% |
| `noun/i-f/noun:voc:pl` | 0 | 0 | 39 | 0.00% | 0.00% |
| `noun/i-f/noun:voc:sg` | 39 | 39 | 39 | 100.00% | 100.00% |
| `noun/i-m/noun:dat:du` | 9 | 9 | 9 | 100.00% | 100.00% |
| `noun/i-m/noun:dat:pl` | 9 | 9 | 9 | 100.00% | 100.00% |
| `noun/i-m/noun:dat:sg` | 9 | 9 | 9 | 100.00% | 100.00% |
| `noun/i-m/noun:gen:du` | 9 | 9 | 9 | 100.00% | 100.00% |
| `noun/i-m/noun:gen:pl` | 9 | 9 | 9 | 100.00% | 100.00% |
| `noun/i-m/noun:gen:sg` | 9 | 9 | 9 | 100.00% | 100.00% |
| `noun/i-m/noun:ins:du` | 9 | 9 | 9 | 100.00% | 100.00% |
| `noun/i-m/noun:ins:pl` | 9 | 9 | 9 | 100.00% | 100.00% |
| `noun/i-m/noun:ins:sg` | 7 | 7 | 9 | 77.78% | 77.78% |
| `noun/i-m/noun:loc:du` | 9 | 9 | 9 | 100.00% | 100.00% |
| `noun/i-m/noun:loc:pl` | 9 | 9 | 9 | 100.00% | 100.00% |
| `noun/i-m/noun:loc:sg` | 9 | 9 | 9 | 100.00% | 100.00% |
| `noun/i-m/noun:nom:du` | 9 | 9 | 9 | 100.00% | 100.00% |
| `noun/i-m/noun:nom:pl` | 7 | 7 | 9 | 77.78% | 77.78% |
| `noun/i-m/noun:nom:sg` | 9 | 9 | 9 | 100.00% | 100.00% |
| `noun/i-m/noun:voc:du` | 9 | 9 | 9 | 100.00% | 100.00% |
| `noun/i-m/noun:voc:pl` | 9 | 9 | 9 | 100.00% | 100.00% |
| `noun/i-m/noun:voc:sg` | 9 | 9 | 9 | 100.00% | 100.00% |
| `noun/ja-soft/noun:acc:du` | 15 | 15 | 17 | 88.24% | 88.24% |
| `noun/ja-soft/noun:acc:pl` | 13 | 13 | 17 | 76.47% | 76.47% |
| `noun/ja-soft/noun:acc:sg` | 14 | 14 | 18 | 77.78% | 77.78% |
| `noun/ja-soft/noun:dat:du` | 13 | 13 | 17 | 76.47% | 76.47% |
| `noun/ja-soft/noun:dat:pl` | 13 | 13 | 17 | 76.47% | 76.47% |
| `noun/ja-soft/noun:dat:sg` | 16 | 16 | 18 | 88.89% | 88.89% |
| `noun/ja-soft/noun:gen:du` | 13 | 13 | 17 | 76.47% | 76.47% |
| `noun/ja-soft/noun:gen:pl` | 13 | 13 | 17 | 76.47% | 76.47% |
| `noun/ja-soft/noun:gen:sg` | 14 | 14 | 18 | 77.78% | 77.78% |
| `noun/ja-soft/noun:ins:du` | 13 | 13 | 17 | 76.47% | 76.47% |
| `noun/ja-soft/noun:ins:pl` | 13 | 13 | 17 | 76.47% | 76.47% |
| `noun/ja-soft/noun:ins:sg` | 10 | 10 | 18 | 55.56% | 55.56% |
| `noun/ja-soft/noun:loc:du` | 13 | 13 | 17 | 76.47% | 76.47% |
| `noun/ja-soft/noun:loc:pl` | 13 | 13 | 17 | 76.47% | 76.47% |
| `noun/ja-soft/noun:loc:sg` | 16 | 16 | 18 | 88.89% | 88.89% |
| `noun/ja-soft/noun:nom:du` | 15 | 15 | 17 | 88.24% | 88.24% |
| `noun/ja-soft/noun:nom:pl` | 13 | 13 | 17 | 76.47% | 76.47% |
| `noun/ja-soft/noun:nom:sg` | 8 | 8 | 18 | 44.44% | 44.44% |
| `noun/ja-soft/noun:voc:du` | 15 | 15 | 17 | 88.24% | 88.24% |
| `noun/ja-soft/noun:voc:pl` | 13 | 13 | 17 | 76.47% | 76.47% |
| `noun/ja-soft/noun:voc:sg` | 10 | 10 | 18 | 55.56% | 55.56% |
| `noun/jo-m-soft/noun:dat:du` | 40 | 40 | 53 | 75.47% | 75.47% |
| `noun/jo-m-soft/noun:dat:pl` | 40 | 40 | 53 | 75.47% | 75.47% |
| `noun/jo-m-soft/noun:dat:sg` | 38 | 38 | 53 | 71.70% | 71.70% |
| `noun/jo-m-soft/noun:gen:du` | 38 | 38 | 53 | 71.70% | 71.70% |
| `noun/jo-m-soft/noun:gen:pl` | 40 | 40 | 53 | 75.47% | 75.47% |
| `noun/jo-m-soft/noun:gen:sg` | 38 | 38 | 53 | 71.70% | 71.70% |
| `noun/jo-m-soft/noun:ins:du` | 40 | 40 | 53 | 75.47% | 75.47% |
| `noun/jo-m-soft/noun:ins:pl` | 40 | 40 | 53 | 75.47% | 75.47% |
| `noun/jo-m-soft/noun:ins:sg` | 40 | 40 | 53 | 75.47% | 75.47% |
| `noun/jo-m-soft/noun:loc:du` | 38 | 38 | 53 | 71.70% | 71.70% |
| `noun/jo-m-soft/noun:loc:pl` | 40 | 40 | 53 | 75.47% | 75.47% |
| `noun/jo-m-soft/noun:loc:sg` | 40 | 40 | 53 | 75.47% | 75.47% |
| `noun/jo-m-soft/noun:nom:du` | 38 | 38 | 53 | 71.70% | 71.70% |
| `noun/jo-m-soft/noun:nom:pl` | 40 | 40 | 53 | 75.47% | 75.47% |
| `noun/jo-m-soft/noun:nom:sg` | 40 | 40 | 53 | 75.47% | 75.47% |
| `noun/jo-m-soft/noun:voc:du` | 38 | 38 | 53 | 71.70% | 71.70% |
| `noun/jo-m-soft/noun:voc:pl` | 40 | 40 | 53 | 75.47% | 75.47% |
| `noun/jo-m-soft/noun:voc:sg` | 36 | 36 | 53 | 67.92% | 67.92% |
| `noun/jo-n-soft/noun:acc:du` | 36 | 36 | 36 | 100.00% | 100.00% |
| `noun/jo-n-soft/noun:acc:pl` | 36 | 36 | 36 | 100.00% | 100.00% |
| `noun/jo-n-soft/noun:acc:sg` | 37 | 37 | 37 | 100.00% | 100.00% |
| `noun/jo-n-soft/noun:dat:du` | 16 | 16 | 36 | 44.44% | 44.44% |
| `noun/jo-n-soft/noun:dat:pl` | 16 | 16 | 36 | 44.44% | 44.44% |
| `noun/jo-n-soft/noun:dat:sg` | 37 | 37 | 37 | 100.00% | 100.00% |
| `noun/jo-n-soft/noun:gen:du` | 36 | 36 | 36 | 100.00% | 100.00% |
| `noun/jo-n-soft/noun:gen:pl` | 30 | 30 | 36 | 83.33% | 83.33% |
| `noun/jo-n-soft/noun:gen:sg` | 37 | 37 | 37 | 100.00% | 100.00% |
| `noun/jo-n-soft/noun:ins:du` | 16 | 16 | 36 | 44.44% | 44.44% |
| `noun/jo-n-soft/noun:ins:pl` | 36 | 36 | 36 | 100.00% | 100.00% |
| `noun/jo-n-soft/noun:ins:sg` | 16 | 16 | 37 | 43.24% | 43.24% |
| `noun/jo-n-soft/noun:loc:du` | 36 | 36 | 36 | 100.00% | 100.00% |
| `noun/jo-n-soft/noun:loc:pl` | 36 | 36 | 36 | 100.00% | 100.00% |
| `noun/jo-n-soft/noun:loc:sg` | 37 | 37 | 37 | 100.00% | 100.00% |
| `noun/jo-n-soft/noun:nom:du` | 36 | 36 | 36 | 100.00% | 100.00% |
| `noun/jo-n-soft/noun:nom:pl` | 36 | 36 | 36 | 100.00% | 100.00% |
| `noun/jo-n-soft/noun:nom:sg` | 37 | 37 | 37 | 100.00% | 100.00% |
| `noun/jo-n-soft/noun:voc:du` | 36 | 36 | 36 | 100.00% | 100.00% |
| `noun/jo-n-soft/noun:voc:pl` | 36 | 36 | 36 | 100.00% | 100.00% |
| `noun/jo-n-soft/noun:voc:sg` | 37 | 37 | 37 | 100.00% | 100.00% |
| `noun/n-m/noun:dat:du` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/n-m/noun:dat:pl` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/n-m/noun:dat:sg` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/n-m/noun:gen:du` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/n-m/noun:gen:pl` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/n-m/noun:gen:sg` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/n-m/noun:ins:du` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/n-m/noun:ins:pl` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/n-m/noun:ins:sg` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/n-m/noun:loc:du` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/n-m/noun:loc:pl` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/n-m/noun:loc:sg` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/n-m/noun:nom:du` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/n-m/noun:nom:pl` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/n-m/noun:nom:sg` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/n-m/noun:voc:du` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/n-m/noun:voc:pl` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/n-m/noun:voc:sg` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/n-n/noun:acc:du` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/n-n/noun:acc:pl` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/n-n/noun:acc:sg` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/n-n/noun:dat:du` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/n-n/noun:dat:pl` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/n-n/noun:dat:sg` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/n-n/noun:gen:du` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/n-n/noun:gen:pl` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/n-n/noun:gen:sg` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/n-n/noun:ins:du` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/n-n/noun:ins:pl` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/n-n/noun:ins:sg` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/n-n/noun:loc:du` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/n-n/noun:loc:pl` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/n-n/noun:loc:sg` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/n-n/noun:nom:du` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/n-n/noun:nom:pl` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/n-n/noun:nom:sg` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/n-n/noun:voc:du` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/n-n/noun:voc:pl` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/n-n/noun:voc:sg` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/nt-n/noun:acc:du` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/nt-n/noun:acc:pl` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/nt-n/noun:acc:sg` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/nt-n/noun:dat:du` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/nt-n/noun:dat:pl` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/nt-n/noun:dat:sg` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/nt-n/noun:gen:du` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/nt-n/noun:gen:pl` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/nt-n/noun:gen:sg` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/nt-n/noun:ins:du` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/nt-n/noun:ins:pl` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/nt-n/noun:ins:sg` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/nt-n/noun:loc:du` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/nt-n/noun:loc:pl` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/nt-n/noun:loc:sg` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/nt-n/noun:nom:du` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/nt-n/noun:nom:pl` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/nt-n/noun:nom:sg` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/nt-n/noun:voc:du` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/nt-n/noun:voc:pl` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/nt-n/noun:voc:sg` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/o-m-hard/noun:dat:du` | 114 | 114 | 114 | 100.00% | 100.00% |
| `noun/o-m-hard/noun:dat:pl` | 111 | 111 | 114 | 97.37% | 97.37% |
| `noun/o-m-hard/noun:dat:sg` | 115 | 115 | 115 | 100.00% | 100.00% |
| `noun/o-m-hard/noun:gen:du` | 114 | 114 | 114 | 100.00% | 100.00% |
| `noun/o-m-hard/noun:gen:pl` | 111 | 111 | 114 | 97.37% | 97.37% |
| `noun/o-m-hard/noun:gen:sg` | 115 | 115 | 115 | 100.00% | 100.00% |
| `noun/o-m-hard/noun:ins:du` | 114 | 114 | 114 | 100.00% | 100.00% |
| `noun/o-m-hard/noun:ins:pl` | 111 | 111 | 114 | 97.37% | 97.37% |
| `noun/o-m-hard/noun:ins:sg` | 112 | 112 | 115 | 97.39% | 97.39% |
| `noun/o-m-hard/noun:loc:du` | 114 | 114 | 114 | 100.00% | 100.00% |
| `noun/o-m-hard/noun:loc:pl` | 109 | 109 | 114 | 95.61% | 95.61% |
| `noun/o-m-hard/noun:loc:sg` | 113 | 113 | 115 | 98.26% | 98.26% |
| `noun/o-m-hard/noun:nom:du` | 114 | 114 | 114 | 100.00% | 100.00% |
| `noun/o-m-hard/noun:nom:pl` | 109 | 109 | 114 | 95.61% | 95.61% |
| `noun/o-m-hard/noun:nom:sg` | 115 | 115 | 115 | 100.00% | 100.00% |
| `noun/o-m-hard/noun:voc:du` | 114 | 114 | 114 | 100.00% | 100.00% |
| `noun/o-m-hard/noun:voc:pl` | 109 | 109 | 114 | 95.61% | 95.61% |
| `noun/o-m-hard/noun:voc:sg` | 113 | 113 | 115 | 98.26% | 98.26% |
| `noun/o-n-hard/noun:acc:du` | 39 | 39 | 39 | 100.00% | 100.00% |
| `noun/o-n-hard/noun:acc:pl` | 39 | 39 | 39 | 100.00% | 100.00% |
| `noun/o-n-hard/noun:acc:sg` | 39 | 39 | 39 | 100.00% | 100.00% |
| `noun/o-n-hard/noun:dat:du` | 39 | 39 | 39 | 100.00% | 100.00% |
| `noun/o-n-hard/noun:dat:pl` | 39 | 39 | 39 | 100.00% | 100.00% |
| `noun/o-n-hard/noun:dat:sg` | 39 | 39 | 39 | 100.00% | 100.00% |
| `noun/o-n-hard/noun:gen:du` | 39 | 39 | 39 | 100.00% | 100.00% |
| `noun/o-n-hard/noun:gen:pl` | 39 | 39 | 39 | 100.00% | 100.00% |
| `noun/o-n-hard/noun:gen:sg` | 39 | 39 | 39 | 100.00% | 100.00% |
| `noun/o-n-hard/noun:ins:du` | 39 | 39 | 39 | 100.00% | 100.00% |
| `noun/o-n-hard/noun:ins:pl` | 39 | 39 | 39 | 100.00% | 100.00% |
| `noun/o-n-hard/noun:ins:sg` | 39 | 39 | 39 | 100.00% | 100.00% |
| `noun/o-n-hard/noun:loc:du` | 39 | 39 | 39 | 100.00% | 100.00% |
| `noun/o-n-hard/noun:loc:pl` | 39 | 39 | 39 | 100.00% | 100.00% |
| `noun/o-n-hard/noun:loc:sg` | 39 | 39 | 39 | 100.00% | 100.00% |
| `noun/o-n-hard/noun:nom:du` | 39 | 39 | 39 | 100.00% | 100.00% |
| `noun/o-n-hard/noun:nom:pl` | 39 | 39 | 39 | 100.00% | 100.00% |
| `noun/o-n-hard/noun:nom:sg` | 39 | 39 | 39 | 100.00% | 100.00% |
| `noun/o-n-hard/noun:voc:du` | 39 | 39 | 39 | 100.00% | 100.00% |
| `noun/o-n-hard/noun:voc:pl` | 39 | 39 | 39 | 100.00% | 100.00% |
| `noun/o-n-hard/noun:voc:sg` | 39 | 39 | 39 | 100.00% | 100.00% |
| `noun/s-n/noun:acc:du` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/s-n/noun:acc:pl` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/s-n/noun:acc:sg` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/s-n/noun:dat:du` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/s-n/noun:dat:pl` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/s-n/noun:dat:sg` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/s-n/noun:gen:du` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/s-n/noun:gen:pl` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/s-n/noun:gen:sg` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/s-n/noun:ins:du` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/s-n/noun:ins:pl` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/s-n/noun:ins:sg` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/s-n/noun:loc:du` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/s-n/noun:loc:pl` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/s-n/noun:loc:sg` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/s-n/noun:nom:du` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/s-n/noun:nom:pl` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/s-n/noun:nom:sg` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/s-n/noun:voc:du` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/s-n/noun:voc:pl` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/s-n/noun:voc:sg` | 1 | 1 | 1 | 100.00% | 100.00% |
| `noun/u-m/noun:dat:du` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/u-m/noun:dat:pl` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/u-m/noun:dat:sg` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/u-m/noun:gen:du` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/u-m/noun:gen:pl` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/u-m/noun:gen:sg` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/u-m/noun:ins:du` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/u-m/noun:ins:pl` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/u-m/noun:ins:sg` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/u-m/noun:loc:du` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/u-m/noun:loc:pl` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/u-m/noun:loc:sg` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/u-m/noun:nom:du` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/u-m/noun:nom:pl` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/u-m/noun:nom:sg` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/u-m/noun:voc:du` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/u-m/noun:voc:pl` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/u-m/noun:voc:sg` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/v-f/noun:acc:du` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/v-f/noun:acc:pl` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/v-f/noun:acc:sg` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/v-f/noun:dat:du` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/v-f/noun:dat:pl` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/v-f/noun:dat:sg` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/v-f/noun:gen:du` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/v-f/noun:gen:pl` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/v-f/noun:gen:sg` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/v-f/noun:ins:du` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/v-f/noun:ins:pl` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/v-f/noun:ins:sg` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/v-f/noun:loc:du` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/v-f/noun:loc:pl` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/v-f/noun:loc:sg` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/v-f/noun:nom:du` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/v-f/noun:nom:pl` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/v-f/noun:nom:sg` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/v-f/noun:voc:du` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/v-f/noun:voc:pl` | 4 | 4 | 4 | 100.00% | 100.00% |
| `noun/v-f/noun:voc:sg` | 4 | 4 | 4 | 100.00% | 100.00% |
| `verb/verb-IA1-present/verb:finite:present:1:du` | 30 | 30 | 30 | 100.00% | 100.00% |
| `verb/verb-IA1-present/verb:finite:present:1:pl` | 30 | 30 | 30 | 100.00% | 100.00% |
| `verb/verb-IA1-present/verb:finite:present:1:sg` | 25 | 25 | 30 | 83.33% | 83.33% |
| `verb/verb-IA1-present/verb:finite:present:2:du` | 30 | 30 | 30 | 100.00% | 100.00% |
| `verb/verb-IA1-present/verb:finite:present:2:pl` | 30 | 30 | 30 | 100.00% | 100.00% |
| `verb/verb-IA1-present/verb:finite:present:3:du` | 30 | 30 | 30 | 100.00% | 100.00% |
| `verb/verb-IA1-present/verb:finite:present:3:pl` | 25 | 25 | 30 | 83.33% | 83.33% |
| `verb/verb-IA1-present/verb:finite:present:3:sg` | 30 | 30 | 30 | 100.00% | 100.00% |
| `verb/verb-IA2-present/verb:finite:present:1:du` | 1 | 1 | 1 | 100.00% | 100.00% |
| `verb/verb-IA2-present/verb:finite:present:1:pl` | 1 | 1 | 1 | 100.00% | 100.00% |
| `verb/verb-IA2-present/verb:finite:present:1:sg` | 0 | 0 | 1 | 0.00% | 0.00% |
| `verb/verb-IA2-present/verb:finite:present:2:du` | 1 | 1 | 1 | 100.00% | 100.00% |
| `verb/verb-IA2-present/verb:finite:present:2:pl` | 1 | 1 | 1 | 100.00% | 100.00% |
| `verb/verb-IA2-present/verb:finite:present:3:du` | 1 | 1 | 1 | 100.00% | 100.00% |
| `verb/verb-IA2-present/verb:finite:present:3:pl` | 0 | 0 | 1 | 0.00% | 0.00% |
| `verb/verb-IA2-present/verb:finite:present:3:sg` | 1 | 1 | 1 | 100.00% | 100.00% |
| `verb/verb-II1-present/verb:finite:present:1:du` | 9 | 9 | 9 | 100.00% | 100.00% |
| `verb/verb-II1-present/verb:finite:present:1:pl` | 9 | 9 | 9 | 100.00% | 100.00% |
| `verb/verb-II1-present/verb:finite:present:2:du` | 9 | 9 | 9 | 100.00% | 100.00% |
| `verb/verb-II1-present/verb:finite:present:2:pl` | 9 | 9 | 9 | 100.00% | 100.00% |
| `verb/verb-II1-present/verb:finite:present:3:du` | 9 | 9 | 9 | 100.00% | 100.00% |
| `verb/verb-II1-present/verb:finite:present:3:pl` | 9 | 9 | 9 | 100.00% | 100.00% |
| `verb/verb-II1-present/verb:finite:present:3:sg` | 9 | 9 | 9 | 100.00% | 100.00% |
| `verb/verb-II2-present/verb:finite:present:1:du` | 6 | 6 | 6 | 100.00% | 100.00% |
| `verb/verb-II2-present/verb:finite:present:1:pl` | 6 | 6 | 6 | 100.00% | 100.00% |
| `verb/verb-II2-present/verb:finite:present:2:du` | 6 | 6 | 6 | 100.00% | 100.00% |
| `verb/verb-II2-present/verb:finite:present:2:pl` | 6 | 6 | 6 | 100.00% | 100.00% |
| `verb/verb-II2-present/verb:finite:present:3:du` | 6 | 6 | 6 | 100.00% | 100.00% |
| `verb/verb-II2-present/verb:finite:present:3:pl` | 6 | 6 | 6 | 100.00% | 100.00% |
| `verb/verb-II2-present/verb:finite:present:3:sg` | 6 | 6 | 6 | 100.00% | 100.00% |
| `verb/verb-II3-present/verb:finite:present:1:du` | 2 | 2 | 2 | 100.00% | 100.00% |
| `verb/verb-II3-present/verb:finite:present:1:pl` | 2 | 2 | 2 | 100.00% | 100.00% |
| `verb/verb-II3-present/verb:finite:present:2:du` | 2 | 2 | 2 | 100.00% | 100.00% |
| `verb/verb-II3-present/verb:finite:present:2:pl` | 2 | 2 | 2 | 100.00% | 100.00% |
| `verb/verb-II3-present/verb:finite:present:3:du` | 2 | 2 | 2 | 100.00% | 100.00% |
| `verb/verb-II3-present/verb:finite:present:3:pl` | 2 | 2 | 2 | 100.00% | 100.00% |
| `verb/verb-II3-present/verb:finite:present:3:sg` | 2 | 2 | 2 | 100.00% | 100.00% |
| `verb/verb-imperative/verb:imperative:1:du` | 16 | 16 | 47 | 34.04% | 34.04% |
| `verb/verb-imperative/verb:imperative:1:pl` | 47 | 47 | 47 | 100.00% | 100.00% |
| `verb/verb-imperative/verb:imperative:2:du` | 47 | 47 | 47 | 100.00% | 100.00% |
| `verb/verb-imperative/verb:imperative:2:pl` | 47 | 47 | 47 | 100.00% | 100.00% |
| `verb/verb-imperfect/verb:finite:imperfect:1:du` | 48 | 48 | 48 | 100.00% | 100.00% |
| `verb/verb-imperfect/verb:finite:imperfect:1:pl` | 48 | 48 | 48 | 100.00% | 100.00% |
| `verb/verb-imperfect/verb:finite:imperfect:2:du` | 48 | 48 | 48 | 100.00% | 100.00% |
| `verb/verb-imperfect/verb:finite:imperfect:2:pl` | 48 | 48 | 48 | 100.00% | 100.00% |
| `verb/verb-imperfect/verb:finite:imperfect:2:sg` | 48 | 48 | 48 | 100.00% | 100.00% |
| `verb/verb-imperfect/verb:finite:imperfect:3:du` | 48 | 48 | 48 | 100.00% | 100.00% |
| `verb/verb-imperfect/verb:finite:imperfect:3:pl` | 48 | 48 | 48 | 100.00% | 100.00% |
| `verb/verb-imperfect/verb:finite:imperfect:3:sg` | 48 | 48 | 48 | 100.00% | 100.00% |
| `verb/verb-infinitive/verb:infinitive` | 152 | 152 | 152 | 100.00% | 100.00% |
| `verb/verb-l-participle/verb:l-participle:f:du` | 152 | 152 | 152 | 100.00% | 100.00% |
| `verb/verb-l-participle/verb:l-participle:f:pl` | 152 | 152 | 152 | 100.00% | 100.00% |
| `verb/verb-l-participle/verb:l-participle:f:sg` | 152 | 152 | 152 | 100.00% | 100.00% |
| `verb/verb-l-participle/verb:l-participle:m:du` | 152 | 152 | 152 | 100.00% | 100.00% |
| `verb/verb-l-participle/verb:l-participle:m:pl` | 152 | 152 | 152 | 100.00% | 100.00% |
| `verb/verb-l-participle/verb:l-participle:n:du` | 152 | 152 | 152 | 100.00% | 100.00% |
| `verb/verb-l-participle/verb:l-participle:n:pl` | 152 | 152 | 152 | 100.00% | 100.00% |
| `verb/verb-l-participle/verb:l-participle:n:sg` | 152 | 152 | 152 | 100.00% | 100.00% |
| `verb/verb-past-active-participle-ush/verb:participle:past-active:citation` | 14 | 14 | 25 | 56.00% | 56.00% |
| `verb/verb-past-active-participle-vush/verb:participle:past-active:citation` | 20 | 20 | 25 | 80.00% | 80.00% |
| `verb/verb-past-passive-participle-en/verb:participle:past-passive:citation` | 8 | 8 | 23 | 34.78% | 34.78% |
| `verb/verb-present-active-participle-yesht-soft/verb:participle:present-active:citation` | 17 | 17 | 17 | 100.00% | 100.00% |
| `verb/verb-present-active-participle-yusht-hard/verb:participle:present-active:citation` | 25 | 25 | 31 | 80.65% | 80.65% |
| `verb/verb-present-passive-participle-im/verb:participle:present-passive:citation` | 15 | 15 | 15 | 100.00% | 100.00% |
| `verb/verb-present-passive-participle-om/verb:participle:present-passive:citation` | 19 | 19 | 23 | 82.61% | 82.61% |
| `verb/verb-supine/verb:supine` | 145 | 145 | 151 | 96.03% | 96.03% |

Skipped OOV cells requiring unavailable lexical metadata: 5007.

## Extraction exclusions

- `adjective-missing-gender`: 21
- `closed-class-missing-or-ambiguous-case`: 100
- `closed-class-missing-or-ambiguous-number`: 7
- `contextual-or-unsplit-source-form`: 1
- `declined-participle-not-safely-attributed`: 153310
- `duplicate-identical-lexeme`: 8
- `entry-without-safe-cells`: 733
- `form-of-entry-not-lexeme`: 469
- `invalid-word-level-source-form`: 130
- `l-participle-missing-gender`: 90
- `non-ocs-script-form`: 1962
- `noun-missing-or-ambiguous-case`: 60
- `participle-missing-tense`: 5
- `personal-reflexive-table-other-lexeme`: 139
- `table-metadata-or-sentinel`: 15217
- `template-markup-form`: 214
- `unsafe-or-unknown-verb-shape`: 619
- `unsafe-page-word`: 28
- `unsafe-verb-error-unrecognized-form`: 17912
- `unsupported-pos`: 296
