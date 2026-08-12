# Synodal extraction report

Target recension: `synodal-russian`. The curated normalized layer contains 10079 rows across 19 tables; 0 rows are quarantined (ceiling 0).

| Table | Rows |
|---|---:|
| `abbreviation_evaluation.tsv` | 74 |
| `abbreviations.tsv` | 159 |
| `accents.tsv` | 45 |
| `alignments.tsv` | 6 |
| `conflicts.tsv` | 1 |
| `evaluation.tsv` | 2291 |
| `exact_forms.tsv` | 3041 |
| `examples.tsv` | 8 |
| `irregular_overrides.tsv` | 9 |
| `lexemes.tsv` | 81 |
| `lexical_reviews.tsv` | 847 |
| `phrase_evaluation.tsv` | 5 |
| `positional_rules.tsv` | 4 |
| `principal_parts.tsv` | 29 |
| `reviewed_evidence.tsv` | 3368 |
| `semantic_alignments.tsv` | 6 |
| `senses.tsv` | 81 |
| `training_passages.tsv` | 20 |
| `transformation_rules.tsv` | 4 |

Generated morphology SHA-256: `7c23cb60fb0dedb86532434244f4eb0a33a4e26cfc601cb91a3c37bf58098534`.

Generated dictionary SHA-256: `c864a1ae7a69c2981da93c7e2841e476b32d5de436655be260aa061df3f06775`.

Large raw inputs are processed by streaming adapters; malformed rows are retained in JSONL quarantine output and the output replacement is atomic.
