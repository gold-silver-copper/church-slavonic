# Synodal extraction report

Target recension: `synodal-russian`. The curated normalized layer contains 5794 rows across 19 tables; 0 rows are quarantined (ceiling 0).

| Table | Rows |
|---|---:|
| `abbreviation_evaluation.tsv` | 64 |
| `abbreviations.tsv` | 149 |
| `accents.tsv` | 45 |
| `alignments.tsv` | 6 |
| `conflicts.tsv` | 1 |
| `evaluation.tsv` | 1187 |
| `exact_forms.tsv` | 1876 |
| `examples.tsv` | 8 |
| `irregular_overrides.tsv` | 9 |
| `lexemes.tsv` | 81 |
| `lexical_reviews.tsv` | 647 |
| `phrase_evaluation.tsv` | 5 |
| `positional_rules.tsv` | 4 |
| `principal_parts.tsv` | 29 |
| `reviewed_evidence.tsv` | 1572 |
| `semantic_alignments.tsv` | 6 |
| `senses.tsv` | 81 |
| `training_passages.tsv` | 20 |
| `transformation_rules.tsv` | 4 |

Generated morphology SHA-256: `39873dfb08c6b32e75220d4145c837083bb41db101dcfa8bc33875f0506cba10`.

Generated dictionary SHA-256: `99de7f994b4e443ed7116f00e1f5122dd88491288ba7bec52f82a72a0230d33a`.

Large raw inputs are processed by streaming adapters; malformed rows are retained in JSONL quarantine output and the output replacement is atomic.
