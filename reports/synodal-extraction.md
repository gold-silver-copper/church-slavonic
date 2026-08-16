# Synodal extraction report

Target recension: `synodal-russian`. The curated normalized layer contains 10329 rows across 29 tables; 0 rows are quarantined (ceiling 0).

| Table | Rows |
|---|---:|
| `abbreviation_evaluation.tsv` | 74 |
| `abbreviation_families.tsv` | 29 |
| `abbreviation_inventory.tsv` | 48 |
| `abbreviations.tsv` | 159 |
| `accent_paradigms.tsv` | 21 |
| `accents.tsv` | 45 |
| `alignments.tsv` | 6 |
| `conflicts.tsv` | 1 |
| `engine_capabilities.tsv` | 82 |
| `evaluation.tsv` | 2139 |
| `exact_forms.tsv` | 2952 |
| `examples.tsv` | 8 |
| `irregular_overrides.tsv` | 16 |
| `irregular_verb_inventory.tsv` | 98 |
| `lexemes.tsv` | 201 |
| `lexical_reviews.tsv` | 831 |
| `linguistic_evaluation.tsv` | 12 |
| `noun_restrictions.tsv` | 2 |
| `past_classification_reviews.tsv` | 73 |
| `phrase_evaluation.tsv` | 14 |
| `positional_rules.tsv` | 4 |
| `principal_parts.tsv` | 73 |
| `reviewed_evidence.tsv` | 3240 |
| `semantic_alignments.tsv` | 6 |
| `senses.tsv` | 164 |
| `training_passages.tsv` | 20 |
| `transformation_rules.tsv` | 4 |
| `v10_exact_cell_corrections.tsv` | 3 |
| `verb_defectiveness.tsv` | 4 |

Generated morphology SHA-256: `b6b22f7c4cd21c89bc6244ad936b60d24198e5f77068ec724d7418c4b104ed09`.

Generated dictionary SHA-256: `1b8c7d9dcfddf0ee4a45e1a8bba31f82820799a0b2005a8838f3d224d4155036`.

Large raw inputs are processed by streaming adapters; malformed rows are retained in JSONL quarantine output and the output replacement is atomic.
