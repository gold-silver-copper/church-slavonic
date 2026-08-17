# Synodal extraction report

Target recension: `synodal-russian`. The curated normalized layer contains 12425 rows across 29 tables; 0 rows are quarantined (ceiling 0).

| Table | Rows |
|---|---:|
| `abbreviation_evaluation.tsv` | 81 |
| `abbreviation_families.tsv` | 62 |
| `abbreviation_inventory.tsv` | 48 |
| `abbreviations.tsv` | 220 |
| `accent_paradigms.tsv` | 770 |
| `accents.tsv` | 45 |
| `alignments.tsv` | 6 |
| `conflicts.tsv` | 1 |
| `engine_capabilities.tsv` | 94 |
| `evaluation.tsv` | 2270 |
| `exact_forms.tsv` | 3238 |
| `examples.tsv` | 8 |
| `irregular_overrides.tsv` | 16 |
| `irregular_verb_inventory.tsv` | 98 |
| `lexemes.tsv` | 284 |
| `lexical_reviews.tsv` | 846 |
| `linguistic_evaluation.tsv` | 12 |
| `noun_restrictions.tsv` | 7 |
| `past_classification_reviews.tsv` | 73 |
| `phrase_evaluation.tsv` | 14 |
| `positional_rules.tsv` | 19 |
| `principal_parts.tsv` | 149 |
| `reviewed_evidence.tsv` | 3786 |
| `semantic_alignments.tsv` | 6 |
| `senses.tsv` | 240 |
| `training_passages.tsv` | 20 |
| `transformation_rules.tsv` | 4 |
| `v10_exact_cell_corrections.tsv` | 3 |
| `verb_defectiveness.tsv` | 5 |

Generated morphology SHA-256: `b01f3f14f1a0f9af2ea019cb82bf07c76e8b1f52709e4dbda192c3c6d676b8ff`.

Generated dictionary SHA-256: `e24b7ad15821a6b38e29e88cb9d41f6a995f6c8b1ea47dd50c4663fdf8566ab8`.

Large raw inputs are processed by streaming adapters; malformed rows are retained in JSONL quarantine output and the output replacement is atomic.
