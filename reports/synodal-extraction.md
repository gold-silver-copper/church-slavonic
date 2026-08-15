# Synodal extraction report

Target recension: `synodal-russian`. The curated normalized layer contains 9871 rows across 24 tables; 0 rows are quarantined (ceiling 0).

| Table | Rows |
|---|---:|
| `abbreviation_evaluation.tsv` | 74 |
| `abbreviations.tsv` | 159 |
| `accent_paradigms.tsv` | 21 |
| `accents.tsv` | 45 |
| `alignments.tsv` | 6 |
| `conflicts.tsv` | 1 |
| `engine_capabilities.tsv` | 81 |
| `evaluation.tsv` | 2140 |
| `exact_forms.tsv` | 2838 |
| `examples.tsv` | 8 |
| `irregular_overrides.tsv` | 9 |
| `lexemes.tsv` | 195 |
| `lexical_reviews.tsv` | 831 |
| `linguistic_evaluation.tsv` | 11 |
| `noun_restrictions.tsv` | 2 |
| `phrase_evaluation.tsv` | 5 |
| `positional_rules.tsv` | 4 |
| `principal_parts.tsv` | 31 |
| `reviewed_evidence.tsv` | 3214 |
| `semantic_alignments.tsv` | 6 |
| `senses.tsv` | 163 |
| `training_passages.tsv` | 20 |
| `transformation_rules.tsv` | 4 |
| `v10_exact_cell_corrections.tsv` | 3 |

Generated morphology SHA-256: `1e98d869973682ce18a0f557ddeb8cf93ac16eee898bf5e795599e02cd01bd89`.

Generated dictionary SHA-256: `908a274ef9ce682b8519bf520b7308ccf0a2c4e161de268acc55e8f08a6b5bca`.

Large raw inputs are processed by streaming adapters; malformed rows are retained in JSONL quarantine output and the output replacement is atomic.
