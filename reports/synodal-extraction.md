# Synodal extraction report

Target recension: `synodal-russian`. The curated normalized layer contains 12390 rows across 29 tables; 0 rows are quarantined (ceiling 0).

| Table | Rows |
|---|---:|
| `abbreviation_evaluation.tsv` | 81 |
| `abbreviation_families.tsv` | 62 |
| `abbreviation_inventory.tsv` | 48 |
| `abbreviations.tsv` | 220 |
| `accent_paradigms.tsv` | 764 |
| `accents.tsv` | 45 |
| `alignments.tsv` | 6 |
| `conflicts.tsv` | 1 |
| `engine_capabilities.tsv` | 94 |
| `evaluation.tsv` | 2270 |
| `exact_forms.tsv` | 3238 |
| `examples.tsv` | 8 |
| `irregular_overrides.tsv` | 16 |
| `irregular_verb_inventory.tsv` | 98 |
| `lexemes.tsv` | 283 |
| `lexical_reviews.tsv` | 846 |
| `linguistic_evaluation.tsv` | 12 |
| `noun_restrictions.tsv` | 7 |
| `past_classification_reviews.tsv` | 73 |
| `phrase_evaluation.tsv` | 14 |
| `positional_rules.tsv` | 19 |
| `principal_parts.tsv` | 141 |
| `reviewed_evidence.tsv` | 3767 |
| `semantic_alignments.tsv` | 6 |
| `senses.tsv` | 239 |
| `training_passages.tsv` | 20 |
| `transformation_rules.tsv` | 4 |
| `v10_exact_cell_corrections.tsv` | 3 |
| `verb_defectiveness.tsv` | 5 |

Generated morphology SHA-256: `dd0daa81bee070e25d511a5090edbb82973f5c6fd57cdb2f5adfb9317a4f2c10`.

Generated dictionary SHA-256: `b85718567bcbf414b9071e1735a5ee4b4224110bd2e1ab1f41549188442202f9`.

Large raw inputs are processed by streaming adapters; malformed rows are retained in JSONL quarantine output and the output replacement is atomic.
