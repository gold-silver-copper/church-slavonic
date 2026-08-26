# Synodal extraction report

Target recension: `synodal-russian`. The curated normalized layer contains 14530 rows across 29 tables; 0 rows are quarantined (ceiling 0).

| Table | Rows |
|---|---:|
| `abbreviation_evaluation.tsv` | 81 |
| `abbreviation_families.tsv` | 62 |
| `abbreviation_inventory.tsv` | 48 |
| `abbreviations.tsv` | 220 |
| `accent_paradigms.tsv` | 1386 |
| `accents.tsv` | 180 |
| `alignments.tsv` | 6 |
| `conflicts.tsv` | 1 |
| `engine_capabilities.tsv` | 97 |
| `evaluation.tsv` | 2501 |
| `exact_forms.tsv` | 3246 |
| `examples.tsv` | 8 |
| `irregular_overrides.tsv` | 16 |
| `irregular_verb_inventory.tsv` | 98 |
| `lexemes.tsv` | 391 |
| `lexical_reviews.tsv` | 846 |
| `linguistic_evaluation.tsv` | 36 |
| `noun_restrictions.tsv` | 7 |
| `past_classification_reviews.tsv` | 73 |
| `phrase_evaluation.tsv` | 14 |
| `positional_rules.tsv` | 19 |
| `principal_parts.tsv` | 411 |
| `reviewed_evidence.tsv` | 4405 |
| `semantic_alignments.tsv` | 6 |
| `senses.tsv` | 338 |
| `training_passages.tsv` | 20 |
| `transformation_rules.tsv` | 4 |
| `v10_exact_cell_corrections.tsv` | 5 |
| `verb_defectiveness.tsv` | 5 |

Generated morphology SHA-256: `a57b65cc2b68788f55c88706b577881fabfd9a9541a6f82d392eb89b48784fb9`.

Generated dictionary SHA-256: `c942e2f052a6dc147c65b8b8928401a8f7ca058c4e71afa26e12db3cd1c6ccb9`.

Large raw inputs are processed by streaming adapters; malformed rows are retained in JSONL quarantine output and the output replacement is atomic.
