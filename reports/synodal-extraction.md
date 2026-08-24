# Synodal extraction report

Target recension: `synodal-russian`. The curated normalized layer contains 12571 rows across 29 tables; 0 rows are quarantined (ceiling 0).

| Table | Rows |
|---|---:|
| `abbreviation_evaluation.tsv` | 81 |
| `abbreviation_families.tsv` | 62 |
| `abbreviation_inventory.tsv` | 48 |
| `abbreviations.tsv` | 220 |
| `accent_paradigms.tsv` | 795 |
| `accents.tsv` | 54 |
| `alignments.tsv` | 6 |
| `conflicts.tsv` | 1 |
| `engine_capabilities.tsv` | 96 |
| `evaluation.tsv` | 2298 |
| `exact_forms.tsv` | 3238 |
| `examples.tsv` | 8 |
| `irregular_overrides.tsv` | 16 |
| `irregular_verb_inventory.tsv` | 98 |
| `lexemes.tsv` | 287 |
| `lexical_reviews.tsv` | 846 |
| `linguistic_evaluation.tsv` | 19 |
| `noun_restrictions.tsv` | 7 |
| `past_classification_reviews.tsv` | 73 |
| `phrase_evaluation.tsv` | 14 |
| `positional_rules.tsv` | 19 |
| `principal_parts.tsv` | 170 |
| `reviewed_evidence.tsv` | 3834 |
| `semantic_alignments.tsv` | 6 |
| `senses.tsv` | 243 |
| `training_passages.tsv` | 20 |
| `transformation_rules.tsv` | 4 |
| `v10_exact_cell_corrections.tsv` | 3 |
| `verb_defectiveness.tsv` | 5 |

Generated morphology SHA-256: `d4671a7d7d82734d04e5df29d74994797665641cd17a57c1fbd87d7dc24b0a90`.

Generated dictionary SHA-256: `76d1c1acc118b7f6abb5dfeec87556796ce98e0b58faabefa56761866fc854fc`.

Large raw inputs are processed by streaming adapters; malformed rows are retained in JSONL quarantine output and the output replacement is atomic.
