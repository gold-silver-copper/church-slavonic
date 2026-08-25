# Synodal extraction report

Target recension: `synodal-russian`. The curated normalized layer contains 13099 rows across 29 tables; 0 rows are quarantined (ceiling 0).

| Table | Rows |
|---|---:|
| `abbreviation_evaluation.tsv` | 81 |
| `abbreviation_families.tsv` | 62 |
| `abbreviation_inventory.tsv` | 48 |
| `abbreviations.tsv` | 220 |
| `accent_paradigms.tsv` | 871 |
| `accents.tsv` | 123 |
| `alignments.tsv` | 6 |
| `conflicts.tsv` | 1 |
| `engine_capabilities.tsv` | 97 |
| `evaluation.tsv` | 2365 |
| `exact_forms.tsv` | 3241 |
| `examples.tsv` | 8 |
| `irregular_overrides.tsv` | 16 |
| `irregular_verb_inventory.tsv` | 98 |
| `lexemes.tsv` | 302 |
| `lexical_reviews.tsv` | 846 |
| `linguistic_evaluation.tsv` | 33 |
| `noun_restrictions.tsv` | 7 |
| `past_classification_reviews.tsv` | 73 |
| `phrase_evaluation.tsv` | 14 |
| `positional_rules.tsv` | 19 |
| `principal_parts.tsv` | 273 |
| `reviewed_evidence.tsv` | 3998 |
| `semantic_alignments.tsv` | 6 |
| `senses.tsv` | 257 |
| `training_passages.tsv` | 20 |
| `transformation_rules.tsv` | 4 |
| `v10_exact_cell_corrections.tsv` | 5 |
| `verb_defectiveness.tsv` | 5 |

Generated morphology SHA-256: `29d86e119fe7a47e3584ac34699b407b9e7158c7d3b291a3cfb39500c2f2eb13`.

Generated dictionary SHA-256: `0307b4226bea546e8ebc8f95e7a47fb74b4708c26ec9081f560f259504a76f85`.

Large raw inputs are processed by streaming adapters; malformed rows are retained in JSONL quarantine output and the output replacement is atomic.
