# Synodal extraction report

Target recension: `synodal-russian`. The curated normalized layer contains 14051 rows across 29 tables; 0 rows are quarantined (ceiling 0).

| Table | Rows |
|---|---:|
| `abbreviation_evaluation.tsv` | 81 |
| `abbreviation_families.tsv` | 62 |
| `abbreviation_inventory.tsv` | 48 |
| `abbreviations.tsv` | 220 |
| `accent_paradigms.tsv` | 1195 |
| `accents.tsv` | 179 |
| `alignments.tsv` | 6 |
| `conflicts.tsv` | 1 |
| `engine_capabilities.tsv` | 97 |
| `evaluation.tsv` | 2468 |
| `exact_forms.tsv` | 3243 |
| `examples.tsv` | 8 |
| `irregular_overrides.tsv` | 16 |
| `irregular_verb_inventory.tsv` | 98 |
| `lexemes.tsv` | 357 |
| `lexical_reviews.tsv` | 846 |
| `linguistic_evaluation.tsv` | 36 |
| `noun_restrictions.tsv` | 7 |
| `past_classification_reviews.tsv` | 73 |
| `phrase_evaluation.tsv` | 14 |
| `positional_rules.tsv` | 19 |
| `principal_parts.tsv` | 366 |
| `reviewed_evidence.tsv` | 4267 |
| `semantic_alignments.tsv` | 6 |
| `senses.tsv` | 304 |
| `training_passages.tsv` | 20 |
| `transformation_rules.tsv` | 4 |
| `v10_exact_cell_corrections.tsv` | 5 |
| `verb_defectiveness.tsv` | 5 |

Generated morphology SHA-256: `ed52b02d9f2742de2d97390bc565e9b7a074e06135fc80f1cdecf23ad35d8afc`.

Generated dictionary SHA-256: `3deb85a00b926686b43daeae98891095c799bf8ecc68d499714e7586768e1ff8`.

Large raw inputs are processed by streaming adapters; malformed rows are retained in JSONL quarantine output and the output replacement is atomic.
