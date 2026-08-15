# Synodal extraction report

Target recension: `synodal-russian`. The curated normalized layer contains 10224 rows across 27 tables; 0 rows are quarantined (ceiling 0).

| Table | Rows |
|---|---:|
| `abbreviation_evaluation.tsv` | 74 |
| `abbreviations.tsv` | 159 |
| `accent_paradigms.tsv` | 21 |
| `accents.tsv` | 45 |
| `alignments.tsv` | 6 |
| `conflicts.tsv` | 1 |
| `engine_capabilities.tsv` | 82 |
| `evaluation.tsv` | 2139 |
| `exact_forms.tsv` | 2946 |
| `examples.tsv` | 8 |
| `irregular_overrides.tsv` | 16 |
| `irregular_verb_inventory.tsv` | 98 |
| `lexemes.tsv` | 198 |
| `lexical_reviews.tsv` | 831 |
| `linguistic_evaluation.tsv` | 12 |
| `noun_restrictions.tsv` | 2 |
| `past_classification_reviews.tsv` | 73 |
| `phrase_evaluation.tsv` | 5 |
| `positional_rules.tsv` | 4 |
| `principal_parts.tsv` | 69 |
| `reviewed_evidence.tsv` | 3235 |
| `semantic_alignments.tsv` | 6 |
| `senses.tsv` | 163 |
| `training_passages.tsv` | 20 |
| `transformation_rules.tsv` | 4 |
| `v10_exact_cell_corrections.tsv` | 3 |
| `verb_defectiveness.tsv` | 4 |

Generated morphology SHA-256: `8d0ad6c26c11696f5c7f78bf3e101609db61cfc4dc12d574e0d686ddb1a4e3d2`.

Generated dictionary SHA-256: `908a274ef9ce682b8519bf520b7308ccf0a2c4e161de268acc55e8f08a6b5bca`.

Large raw inputs are processed by streaming adapters; malformed rows are retained in JSONL quarantine output and the output replacement is atomic.
