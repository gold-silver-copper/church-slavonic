# Synodal extraction report

Target recension: `synodal-russian`. The curated normalized layer contains 14400 rows across 29 tables; 0 rows are quarantined (ceiling 0).

| Table | Rows |
|---|---:|
| `abbreviation_evaluation.tsv` | 81 |
| `abbreviation_families.tsv` | 62 |
| `abbreviation_inventory.tsv` | 48 |
| `abbreviations.tsv` | 220 |
| `accent_paradigms.tsv` | 1350 |
| `accents.tsv` | 180 |
| `alignments.tsv` | 6 |
| `conflicts.tsv` | 1 |
| `engine_capabilities.tsv` | 97 |
| `evaluation.tsv` | 2492 |
| `exact_forms.tsv` | 3243 |
| `examples.tsv` | 8 |
| `irregular_overrides.tsv` | 16 |
| `irregular_verb_inventory.tsv` | 98 |
| `lexemes.tsv` | 382 |
| `lexical_reviews.tsv` | 846 |
| `linguistic_evaluation.tsv` | 36 |
| `noun_restrictions.tsv` | 7 |
| `past_classification_reviews.tsv` | 73 |
| `phrase_evaluation.tsv` | 14 |
| `positional_rules.tsv` | 19 |
| `principal_parts.tsv` | 390 |
| `reviewed_evidence.tsv` | 4362 |
| `semantic_alignments.tsv` | 6 |
| `senses.tsv` | 329 |
| `training_passages.tsv` | 20 |
| `transformation_rules.tsv` | 4 |
| `v10_exact_cell_corrections.tsv` | 5 |
| `verb_defectiveness.tsv` | 5 |

Generated morphology SHA-256: `3cc8668878c5d19a01d387a88c06f08b8406f0d9601ddd013c552955b2b5c877`.

Generated dictionary SHA-256: `ee960cd230ec746e8bdf7dcdc854cdadf4e3b8376b3a4c788d4c18d31170fc12`.

Large raw inputs are processed by streaming adapters; malformed rows are retained in JSONL quarantine output and the output replacement is atomic.
