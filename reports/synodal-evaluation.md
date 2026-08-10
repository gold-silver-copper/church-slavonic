# Synodal evaluation

Target recension: `synodal-russian`. Fixture: pinned passage-held-out Ponomar Elizabeth Bible rows across Matthew, Acts, Daniel, Apocalypse, Amos, and Deuteronomy (38 held-out token cells).

| Metric | Returned | Top-1 | Top-k | Abstained | Total |
|---|---:|---:|---:|---:|---:|
| Expanded | 38 | 37 | 38 | 0 | 38 |
| Printed | 38 | 37 | 38 | 0 | 38 |

Analytic phrases: expanded 1/1, printed 1/1 (1 held-out phrases).

Exact registry round trips (top-k, including reviewed variants): expanded 260/260, printed 260/260.

Masked cells: expanded 28/29, printed 28/29. Leave-one-Synodal-lexeme-out inherited cells: expanded 1/1, printed 1/1.

Accent agreement: 36/36 accent-bearing rows.

Inherited evidence contributed 1/1 returned held-out cells, with 1/1 exact expanded forms. The reviewed alignment registry has 5 accepted mappings, 5 aligned target lexemes, and 1 rejected negative controls.

## Expanded accuracy by generation policy

| Slice | Returned | Top-1 | Top-k | Abstained | Total |
|---|---:|---:|---:|---:|---:|
| `exploratory` | 38 | 37 | 38 | 0 | 38 |
| `productive` | 38 | 37 | 38 | 0 | 38 |
| `strict` | 37 | 36 | 37 | 1 | 38 |

## Expanded accuracy by attestation status

| Slice | Returned | Top-1 | Top-k | Abstained | Total |
|---|---:|---:|---:|---:|---:|
| `attested` | 0 | 0 | 0 | 0 | 0 |
| `expected-form-not-returned` | 0 | 0 | 0 | 0 | 0 |
| `predicted` | 38 | 37 | 38 | 0 | 38 |

## Expanded accuracy by morphological system

| Slice | Returned | Top-1 | Top-k | Abstained | Total |
|---|---:|---:|---:|---:|---:|
| `aorist` | 1 | 1 | 1 | 0 | 1 |
| `imperative` | 2 | 2 | 2 | 0 | 2 |
| `l-participle` | 1 | 1 | 1 | 0 | 1 |
| `noun` | 17 | 17 | 17 | 0 | 17 |
| `numeral` | 5 | 4 | 5 | 0 | 5 |
| `participle` | 1 | 1 | 1 | 0 | 1 |
| `present` | 5 | 5 | 5 | 0 | 5 |
| `pronoun` | 6 | 6 | 6 | 0 | 6 |

## Expanded accuracy by provenance path

| Slice | Returned | Top-1 | Top-k | Abstained | Total |
|---|---:|---:|---:|---:|---:|
| `inherited-ocs-prediction` | 1 | 1 | 1 | 0 | 1 |
| `synodal-normative-table` | 18 | 17 | 18 | 0 | 18 |
| `synodal-productive-rule` | 19 | 19 | 19 | 0 | 19 |

## Expanded accuracy by regularity

| Slice | Returned | Top-1 | Top-k | Abstained | Total |
|---|---:|---:|---:|---:|---:|
| `irregular` | 7 | 7 | 7 | 0 | 7 |
| `irregular-held-out` | 10 | 9 | 10 | 0 | 10 |
| `irregular-participle` | 1 | 1 | 1 | 0 | 1 |
| `regular` | 2 | 2 | 2 | 0 | 2 |
| `regular-held-out` | 17 | 17 | 17 | 0 | 17 |
| `regular-inherited` | 1 | 1 | 1 | 0 | 1 |

## Top-1 disagreements

- Expanded `eval:acts-9-9-tri` (`numeral:cardinal:nominative:plural:masculine:inanimate`): expected `три`, top-1 `трїе`.
- Printed `eval:acts-9-9-tri` (`numeral:cardinal:nominative:plural:masculine:inanimate`): expected `трѝ`, top-1 `трїѐ`.

## Inherited OCS evaluation

The accepted registry contains 2 explicit identity mappings and 3 transformed mappings. The structural Productive-policy admission check has 5 true-positive admissions, 0 false-positive admissions, and precision 10000/10,000 basis points on the reviewed gold registry. This is a policy guard, not an independent estimate of automatic alignment quality.


## Inherited cells by identity/transformed mapping

| Slice | Returned | Top-1 | Top-k | Abstained | Total |
|---|---:|---:|---:|---:|---:|
| `identity` | 1 | 1 | 1 | 0 | 1 |

## Inherited cells by morphological system

| Slice | Returned | Top-1 | Top-k | Abstained | Total |
|---|---:|---:|---:|---:|---:|
| `noun` | 1 | 1 | 1 | 0 | 1 |

## Inherited cells by confidence band

| Slice | Returned | Top-1 | Top-k | Abstained | Total |
|---|---:|---:|---:|---:|---:|
| `high-9500-10000` | 1 | 1 | 1 | 0 | 1 |

Returned inherited confidence: 9500 basis points; empirical exact expanded agreement: 10000 basis points; absolute descriptive calibration gap: 500 basis points.

## Abstention

No held-out row abstained in this reviewed fixture. Unsupported and missing-metadata behavior is exercised separately by paradigms and guard witnesses.

## Interpretation and limitations

- The corpus passages are evaluation-only; they are not generation inputs.
- The current real-text slice is intentionally small and reports counts, not statistical confidence.
- No legally cleared, machine-readable non-biblical Synodal liturgical corpus is currently pinned; catalog-only and unresolved-rights editions are intentionally excluded from held-out scoring.
- Productive liturgical rendering abstains when accent metadata is absent.
- One participle and one analytic perfect are covered by independent corpus witnesses; other analytic constructions remain typed unit fixtures until their lexical registries grow.
- Abbreviation, numeral, malformed-mark, and hostile-Unicode regressions are deterministic utility fixtures, not corpus-accuracy rows.
- Gold admission precision is a structural policy check over the reviewed registry, not an independently estimated automatic-alignment precision.
- The single inherited held-out cell is insufficient to assess confidence calibration; the reported gap is descriptive only.
