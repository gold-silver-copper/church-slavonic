# Synodal extraction report

Target recension: `synodal-russian`. The curated normalized layer contains 12714 rows across 29 tables; 0 rows are quarantined (ceiling 0).

| Table | Rows |
|---|---:|
| `abbreviation_evaluation.tsv` | 81 |
| `abbreviation_families.tsv` | 62 |
| `abbreviation_inventory.tsv` | 48 |
| `abbreviations.tsv` | 220 |
| `accent_paradigms.tsv` | 814 |
| `accents.tsv` | 83 |
| `alignments.tsv` | 6 |
| `conflicts.tsv` | 1 |
| `engine_capabilities.tsv` | 96 |
| `evaluation.tsv` | 2315 |
| `exact_forms.tsv` | 3240 |
| `examples.tsv` | 8 |
| `irregular_overrides.tsv` | 16 |
| `irregular_verb_inventory.tsv` | 98 |
| `lexemes.tsv` | 290 |
| `lexical_reviews.tsv` | 846 |
| `linguistic_evaluation.tsv` | 23 |
| `noun_restrictions.tsv` | 7 |
| `past_classification_reviews.tsv` | 73 |
| `phrase_evaluation.tsv` | 14 |
| `positional_rules.tsv` | 19 |
| `principal_parts.tsv` | 193 |
| `reviewed_evidence.tsv` | 3876 |
| `semantic_alignments.tsv` | 6 |
| `senses.tsv` | 246 |
| `training_passages.tsv` | 20 |
| `transformation_rules.tsv` | 4 |
| `v10_exact_cell_corrections.tsv` | 4 |
| `verb_defectiveness.tsv` | 5 |

Generated morphology SHA-256: `34250351bb4c8d70221753433ff79dd579431bd5b39680ad1c301610e138c0c5`.

Generated dictionary SHA-256: `246c5dc606fbffdee5bd70061e469ce7ecbc20f9b18a44c7748f7d53daadd4f6`.

Large raw inputs are processed by streaming adapters; malformed rows are retained in JSONL quarantine output and the output replacement is atomic.
