# Synodal extraction report

Target recension: `synodal-russian`. The curated normalized layer contains 9680 rows across 23 tables; 0 rows are quarantined (ceiling 0).

| Table | Rows |
|---|---:|
| `abbreviation_evaluation.tsv` | 74 |
| `abbreviations.tsv` | 159 |
| `accent_paradigms.tsv` | 18 |
| `accents.tsv` | 45 |
| `alignments.tsv` | 6 |
| `conflicts.tsv` | 1 |
| `engine_capabilities.tsv` | 56 |
| `evaluation.tsv` | 2140 |
| `exact_forms.tsv` | 2828 |
| `examples.tsv` | 8 |
| `irregular_overrides.tsv` | 9 |
| `lexemes.tsv` | 118 |
| `lexical_reviews.tsv` | 831 |
| `linguistic_evaluation.tsv` | 11 |
| `noun_restrictions.tsv` | 2 |
| `phrase_evaluation.tsv` | 5 |
| `positional_rules.tsv` | 4 |
| `principal_parts.tsv` | 31 |
| `reviewed_evidence.tsv` | 3207 |
| `semantic_alignments.tsv` | 6 |
| `senses.tsv` | 97 |
| `training_passages.tsv` | 20 |
| `transformation_rules.tsv` | 4 |

Generated morphology SHA-256: `b483efe9217fdfa30c5d5e18b2e3ccf54e3774db297ce9785521e37a7c41c733`.

Generated dictionary SHA-256: `923dfba6ee811d1b0ffc68b1f8ba143726dabe0ddad9ddbb8f3bb4b159b0917e`.

Large raw inputs are processed by streaming adapters; malformed rows are retained in JSONL quarantine output and the output replacement is atomic.
