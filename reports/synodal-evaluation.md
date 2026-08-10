# Synodal evaluation

Target recension: `synodal-russian`. Fixture: pinned Ponomar Elizabeth Bible, Matthew 1–5 and Acts 1:18 (11 held-out token cells).

| Metric | Returned | Top-1 | Top-k | Abstained | Total |
|---|---:|---:|---:|---:|---:|
| Expanded | 11 | 11 | 11 | 0 | 11 |
| Printed | 11 | 11 | 11 | 0 | 11 |

Analytic phrases: expanded 1/1, printed 1/1 (1 held-out phrases).

Masked cells: expanded 5/5, printed 5/5. Leave-one-Synodal-lexeme-out inherited cells: expanded 1/1, printed 1/1.

Accent agreement: 11/11 accent-bearing rows.

Inherited evidence contributed 1/1 returned held-out cells, with 1/1 exact expanded forms. The reviewed alignment registry has 5 accepted mappings, 5 aligned target lexemes, and 1 rejected negative controls.

## Expanded accuracy by morphological system

| Slice | Returned | Top-1 | Top-k | Abstained | Total |
|---|---:|---:|---:|---:|---:|
| `imperative` | 1 | 1 | 1 | 0 | 1 |
| `noun` | 3 | 3 | 3 | 0 | 3 |
| `numeral` | 1 | 1 | 1 | 0 | 1 |
| `participle` | 1 | 1 | 1 | 0 | 1 |
| `present` | 4 | 4 | 4 | 0 | 4 |
| `pronoun` | 1 | 1 | 1 | 0 | 1 |

## Expanded accuracy by provenance path

| Slice | Returned | Top-1 | Top-k | Abstained | Total |
|---|---:|---:|---:|---:|---:|
| `inherited-ocs-prediction` | 1 | 1 | 1 | 0 | 1 |
| `synodal-normative-table` | 8 | 8 | 8 | 0 | 8 |
| `synodal-productive-rule` | 2 | 2 | 2 | 0 | 2 |

## Expanded accuracy by regularity

| Slice | Returned | Top-1 | Top-k | Abstained | Total |
|---|---:|---:|---:|---:|---:|
| `irregular` | 7 | 7 | 7 | 0 | 7 |
| `irregular-participle` | 1 | 1 | 1 | 0 | 1 |
| `regular` | 2 | 2 | 2 | 0 | 2 |
| `regular-inherited` | 1 | 1 | 1 | 0 | 1 |

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

No held-out row abstained in this seed fixture. Unsupported and missing-metadata behavior is exercised separately by paradigms and guard witnesses.

## Interpretation and limitations

- The corpus passages are evaluation-only; they are not generation inputs.
- The current real-text slice is intentionally small and reports counts, not statistical confidence.
- Productive liturgical rendering abstains when accent metadata is absent.
- One participle and one analytic perfect are covered by independent corpus witnesses; other analytic constructions remain typed unit fixtures until their lexical registries grow.
- Gold admission precision is a structural policy check over the reviewed registry, not an independently estimated automatic-alignment precision.
- The single inherited held-out cell is insufficient to assess confidence calibration; the reported gap is descriptive only.
