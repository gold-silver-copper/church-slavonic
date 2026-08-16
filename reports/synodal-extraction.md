# Synodal extraction report

Target recension: `synodal-russian`. The curated normalized layer contains 10469 rows across 29 tables; 0 rows are quarantined (ceiling 0).

| Table | Rows |
|---|---:|
| `abbreviation_evaluation.tsv` | 74 |
| `abbreviation_families.tsv` | 61 |
| `abbreviation_inventory.tsv` | 48 |
| `abbreviations.tsv` | 191 |
| `accent_paradigms.tsv` | 21 |
| `accents.tsv` | 45 |
| `alignments.tsv` | 6 |
| `conflicts.tsv` | 1 |
| `engine_capabilities.tsv` | 87 |
| `evaluation.tsv` | 2139 |
| `exact_forms.tsv` | 2955 |
| `examples.tsv` | 8 |
| `irregular_overrides.tsv` | 16 |
| `irregular_verb_inventory.tsv` | 98 |
| `lexemes.tsv` | 229 |
| `lexical_reviews.tsv` | 831 |
| `linguistic_evaluation.tsv` | 12 |
| `noun_restrictions.tsv` | 2 |
| `past_classification_reviews.tsv` | 73 |
| `phrase_evaluation.tsv` | 14 |
| `positional_rules.tsv` | 4 |
| `principal_parts.tsv` | 79 |
| `reviewed_evidence.tsv` | 3247 |
| `semantic_alignments.tsv` | 6 |
| `senses.tsv` | 191 |
| `training_passages.tsv` | 20 |
| `transformation_rules.tsv` | 4 |
| `v10_exact_cell_corrections.tsv` | 3 |
| `verb_defectiveness.tsv` | 4 |

Generated morphology SHA-256: `9b828929cd756c479b2286350b5f6f41371fd49b93ab454f7563333ebf09b863`.

Generated dictionary SHA-256: `b2c1b080aafcd482fbe1b2ba457490dc88c20ee9069d7d67e206a6e9133ed4a4`.

Large raw inputs are processed by streaming adapters; malformed rows are retained in JSONL quarantine output and the output replacement is atomic.
