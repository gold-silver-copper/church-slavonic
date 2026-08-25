# Synodal extraction report

Target recension: `synodal-russian`. The curated normalized layer contains 14402 rows across 29 tables; 0 rows are quarantined (ceiling 0).

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
| `exact_forms.tsv` | 3245 |
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

Generated morphology SHA-256: `6c7bdda19b66a07e908a9a457eac8d39139f01bebecb622675f23f72905e2698`.

Generated dictionary SHA-256: `c6e9a98d503f5bd790307bc65fb27193f03092dde5e6a861efbcde0db827435c`.

Large raw inputs are processed by streaming adapters; malformed rows are retained in JSONL quarantine output and the output replacement is atomic.
