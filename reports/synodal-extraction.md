# Synodal extraction report

Target recension: `synodal-russian`. The curated normalized layer contains 9777 rows across 24 tables; 0 rows are quarantined (ceiling 0).

| Table | Rows |
|---|---:|
| `abbreviation_evaluation.tsv` | 74 |
| `abbreviations.tsv` | 159 |
| `accent_paradigms.tsv` | 18 |
| `accents.tsv` | 45 |
| `alignments.tsv` | 6 |
| `conflicts.tsv` | 1 |
| `engine_capabilities.tsv` | 75 |
| `evaluation.tsv` | 2140 |
| `exact_forms.tsv` | 2835 |
| `examples.tsv` | 8 |
| `irregular_overrides.tsv` | 9 |
| `lexemes.tsv` | 152 |
| `lexical_reviews.tsv` | 831 |
| `linguistic_evaluation.tsv` | 11 |
| `noun_restrictions.tsv` | 2 |
| `phrase_evaluation.tsv` | 5 |
| `positional_rules.tsv` | 4 |
| `principal_parts.tsv` | 31 |
| `reviewed_evidence.tsv` | 3210 |
| `semantic_alignments.tsv` | 6 |
| `senses.tsv` | 128 |
| `training_passages.tsv` | 20 |
| `transformation_rules.tsv` | 4 |
| `v10_exact_cell_corrections.tsv` | 3 |

Generated morphology SHA-256: `892041bfbee770aafc3ca837bc2b0bd1465fdb6da4e6f85f436bedd76d48c567`.

Generated dictionary SHA-256: `8e7760432e1f8330288b36e482763003fb15209d9aaf345f2fd93cbcbea47939`.

Large raw inputs are processed by streaming adapters; malformed rows are retained in JSONL quarantine output and the output replacement is atomic.
