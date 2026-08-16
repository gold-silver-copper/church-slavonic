# Synodal extraction report

Target recension: `synodal-russian`. The curated normalized layer contains 10411 rows across 29 tables; 0 rows are quarantined (ceiling 0).

| Table | Rows |
|---|---:|
| `abbreviation_evaluation.tsv` | 74 |
| `abbreviation_families.tsv` | 50 |
| `abbreviation_inventory.tsv` | 48 |
| `abbreviations.tsv` | 180 |
| `accent_paradigms.tsv` | 21 |
| `accents.tsv` | 45 |
| `alignments.tsv` | 6 |
| `conflicts.tsv` | 1 |
| `engine_capabilities.tsv` | 83 |
| `evaluation.tsv` | 2139 |
| `exact_forms.tsv` | 2952 |
| `examples.tsv` | 8 |
| `irregular_overrides.tsv` | 16 |
| `irregular_verb_inventory.tsv` | 98 |
| `lexemes.tsv` | 220 |
| `lexical_reviews.tsv` | 831 |
| `linguistic_evaluation.tsv` | 12 |
| `noun_restrictions.tsv` | 2 |
| `past_classification_reviews.tsv` | 73 |
| `phrase_evaluation.tsv` | 14 |
| `positional_rules.tsv` | 4 |
| `principal_parts.tsv` | 73 |
| `reviewed_evidence.tsv` | 3241 |
| `semantic_alignments.tsv` | 6 |
| `senses.tsv` | 183 |
| `training_passages.tsv` | 20 |
| `transformation_rules.tsv` | 4 |
| `v10_exact_cell_corrections.tsv` | 3 |
| `verb_defectiveness.tsv` | 4 |

Generated morphology SHA-256: `636da2f35172749732bc8c63e66a591fb14e6076b61086845b371ba841730594`.

Generated dictionary SHA-256: `97b641fd4974a7973d8a87183fd47ad83fb95ceae5cfdb6613ff761cbac8be88`.

Large raw inputs are processed by streaming adapters; malformed rows are retained in JSONL quarantine output and the output replacement is atomic.
