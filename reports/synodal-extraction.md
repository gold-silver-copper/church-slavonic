# Synodal extraction report

Target recension: `synodal-russian`. The curated normalized layer contains 14581 rows across 29 tables; 0 rows are quarantined (ceiling 0).

| Table | Rows |
|---|---:|
| `abbreviation_evaluation.tsv` | 81 |
| `abbreviation_families.tsv` | 62 |
| `abbreviation_inventory.tsv` | 48 |
| `abbreviations.tsv` | 220 |
| `accent_paradigms.tsv` | 1408 |
| `accents.tsv` | 180 |
| `alignments.tsv` | 6 |
| `conflicts.tsv` | 1 |
| `engine_capabilities.tsv` | 97 |
| `evaluation.tsv` | 2504 |
| `exact_forms.tsv` | 3245 |
| `examples.tsv` | 8 |
| `irregular_overrides.tsv` | 16 |
| `irregular_verb_inventory.tsv` | 98 |
| `lexemes.tsv` | 395 |
| `lexical_reviews.tsv` | 846 |
| `linguistic_evaluation.tsv` | 36 |
| `noun_restrictions.tsv` | 7 |
| `past_classification_reviews.tsv` | 73 |
| `phrase_evaluation.tsv` | 14 |
| `positional_rules.tsv` | 19 |
| `principal_parts.tsv` | 411 |
| `reviewed_evidence.tsv` | 4421 |
| `semantic_alignments.tsv` | 6 |
| `senses.tsv` | 342 |
| `training_passages.tsv` | 20 |
| `transformation_rules.tsv` | 4 |
| `v10_exact_cell_corrections.tsv` | 8 |
| `verb_defectiveness.tsv` | 5 |

Generated morphology SHA-256: `68f5c08ef48316c4d16a2ac7f3473e04c417f69674871b7b2d720b52e884ef52`.

Generated dictionary SHA-256: `59b5d0190fe5da2d334a545d4a07ed524782626051c6f6178e1a87d7725c3871`.

Large raw inputs are processed by streaming adapters; malformed rows are retained in JSONL quarantine output and the output replacement is atomic.
