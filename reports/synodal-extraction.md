# Synodal extraction report

Target recension: `synodal-russian`. The curated normalized layer contains 12966 rows across 29 tables; 0 rows are quarantined (ceiling 0).

| Table | Rows |
|---|---:|
| `abbreviation_evaluation.tsv` | 81 |
| `abbreviation_families.tsv` | 62 |
| `abbreviation_inventory.tsv` | 48 |
| `abbreviations.tsv` | 220 |
| `accent_paradigms.tsv` | 854 |
| `accents.tsv` | 111 |
| `alignments.tsv` | 6 |
| `conflicts.tsv` | 1 |
| `engine_capabilities.tsv` | 97 |
| `evaluation.tsv` | 2346 |
| `exact_forms.tsv` | 3240 |
| `examples.tsv` | 8 |
| `irregular_overrides.tsv` | 16 |
| `irregular_verb_inventory.tsv` | 98 |
| `lexemes.tsv` | 297 |
| `lexical_reviews.tsv` | 846 |
| `linguistic_evaluation.tsv` | 30 |
| `noun_restrictions.tsv` | 7 |
| `past_classification_reviews.tsv` | 73 |
| `phrase_evaluation.tsv` | 14 |
| `positional_rules.tsv` | 19 |
| `principal_parts.tsv` | 244 |
| `reviewed_evidence.tsv` | 3956 |
| `semantic_alignments.tsv` | 6 |
| `senses.tsv` | 252 |
| `training_passages.tsv` | 20 |
| `transformation_rules.tsv` | 4 |
| `v10_exact_cell_corrections.tsv` | 5 |
| `verb_defectiveness.tsv` | 5 |

Generated morphology SHA-256: `2bba1b2570b57b4199e1d6a30242a897327623cb638ab2d29a5827da8c958e80`.

Generated dictionary SHA-256: `12338d6eee99329d6ad301bce9ae3da3a315b5503ef5711cd0df7b0f62653283`.

Large raw inputs are processed by streaming adapters; malformed rows are retained in JSONL quarantine output and the output replacement is atomic.
