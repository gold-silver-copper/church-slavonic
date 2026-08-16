# Synodal extraction report

Target recension: `synodal-russian`. The curated normalized layer contains 10535 rows across 29 tables; 0 rows are quarantined (ceiling 0).

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
| `engine_capabilities.tsv` | 89 |
| `evaluation.tsv` | 2139 |
| `exact_forms.tsv` | 2962 |
| `examples.tsv` | 8 |
| `irregular_overrides.tsv` | 16 |
| `irregular_verb_inventory.tsv` | 98 |
| `lexemes.tsv` | 229 |
| `lexical_reviews.tsv` | 831 |
| `linguistic_evaluation.tsv` | 12 |
| `noun_restrictions.tsv` | 3 |
| `past_classification_reviews.tsv` | 73 |
| `phrase_evaluation.tsv` | 14 |
| `positional_rules.tsv` | 19 |
| `principal_parts.tsv` | 109 |
| `reviewed_evidence.tsv` | 3257 |
| `semantic_alignments.tsv` | 6 |
| `senses.tsv` | 191 |
| `training_passages.tsv` | 20 |
| `transformation_rules.tsv` | 4 |
| `v10_exact_cell_corrections.tsv` | 3 |
| `verb_defectiveness.tsv` | 5 |

Generated morphology SHA-256: `716eb7c74b23249428109bb7711d40e30bf9584db08e631e4e31aaef7f63b21a`.

Generated dictionary SHA-256: `b2c1b080aafcd482fbe1b2ba457490dc88c20ee9069d7d67e206a6e9133ed4a4`.

Large raw inputs are processed by streaming adapters; malformed rows are retained in JSONL quarantine output and the output replacement is atomic.
