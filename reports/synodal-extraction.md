# Synodal extraction report

Target recension: `synodal-russian`. The curated normalized layer contains 9789 rows across 24 tables; 0 rows are quarantined (ceiling 0).

| Table | Rows |
|---|---:|
| `abbreviation_evaluation.tsv` | 74 |
| `abbreviations.tsv` | 159 |
| `accent_paradigms.tsv` | 21 |
| `accents.tsv` | 45 |
| `alignments.tsv` | 6 |
| `conflicts.tsv` | 1 |
| `engine_capabilities.tsv` | 81 |
| `evaluation.tsv` | 2140 |
| `exact_forms.tsv` | 2835 |
| `examples.tsv` | 8 |
| `irregular_overrides.tsv` | 9 |
| `lexemes.tsv` | 153 |
| `lexical_reviews.tsv` | 831 |
| `linguistic_evaluation.tsv` | 11 |
| `noun_restrictions.tsv` | 2 |
| `phrase_evaluation.tsv` | 5 |
| `positional_rules.tsv` | 4 |
| `principal_parts.tsv` | 31 |
| `reviewed_evidence.tsv` | 3211 |
| `semantic_alignments.tsv` | 6 |
| `senses.tsv` | 129 |
| `training_passages.tsv` | 20 |
| `transformation_rules.tsv` | 4 |
| `v10_exact_cell_corrections.tsv` | 3 |

Generated morphology SHA-256: `4d9c8296f306e0db436d9d288d08fd0407327503448c93bd8e0a763575579813`.

Generated dictionary SHA-256: `df3cf5400e40bf032bbdcbb88fc04ff70ef3f3c94a64c9a0bd825f5fcbf60b3e`.

Large raw inputs are processed by streaming adapters; malformed rows are retained in JSONL quarantine output and the output replacement is atomic.
