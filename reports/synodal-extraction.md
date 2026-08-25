# Synodal extraction report

Target recension: `synodal-russian`. The curated normalized layer contains 12879 rows across 29 tables; 0 rows are quarantined (ceiling 0).

| Table | Rows |
|---|---:|
| `abbreviation_evaluation.tsv` | 81 |
| `abbreviation_families.tsv` | 62 |
| `abbreviation_inventory.tsv` | 48 |
| `abbreviations.tsv` | 220 |
| `accent_paradigms.tsv` | 839 |
| `accents.tsv` | 109 |
| `alignments.tsv` | 6 |
| `conflicts.tsv` | 1 |
| `engine_capabilities.tsv` | 96 |
| `evaluation.tsv` | 2334 |
| `exact_forms.tsv` | 3241 |
| `examples.tsv` | 8 |
| `irregular_overrides.tsv` | 16 |
| `irregular_verb_inventory.tsv` | 98 |
| `lexemes.tsv` | 294 |
| `lexical_reviews.tsv` | 846 |
| `linguistic_evaluation.tsv` | 27 |
| `noun_restrictions.tsv` | 7 |
| `past_classification_reviews.tsv` | 73 |
| `phrase_evaluation.tsv` | 14 |
| `positional_rules.tsv` | 19 |
| `principal_parts.tsv` | 224 |
| `reviewed_evidence.tsv` | 3928 |
| `semantic_alignments.tsv` | 6 |
| `senses.tsv` | 249 |
| `training_passages.tsv` | 20 |
| `transformation_rules.tsv` | 4 |
| `v10_exact_cell_corrections.tsv` | 4 |
| `verb_defectiveness.tsv` | 5 |

Generated morphology SHA-256: `cc2f5dc42b5686dc1b0ca24abcacc50fe9673dc411deb48938ab25b9ac3dc0a8`.

Generated dictionary SHA-256: `ab52d23d97e4ee31a6bdcd4f85f8f673424cd0b9be248b2a739ba0962e815cdb`.

Large raw inputs are processed by streaming adapters; malformed rows are retained in JSONL quarantine output and the output replacement is atomic.
