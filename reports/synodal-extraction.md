# Synodal extraction report

Target recension: `synodal-russian`. The curated normalized layer contains 9638 rows across 23 tables; 0 rows are quarantined (ceiling 0).

| Table | Rows |
|---|---:|
| `abbreviation_evaluation.tsv` | 74 |
| `abbreviations.tsv` | 159 |
| `accent_paradigms.tsv` | 18 |
| `accents.tsv` | 45 |
| `alignments.tsv` | 6 |
| `conflicts.tsv` | 1 |
| `engine_capabilities.tsv` | 56 |
| `evaluation.tsv` | 2140 |
| `exact_forms.tsv` | 2830 |
| `examples.tsv` | 8 |
| `irregular_overrides.tsv` | 9 |
| `lexemes.tsv` | 89 |
| `lexical_reviews.tsv` | 832 |
| `linguistic_evaluation.tsv` | 11 |
| `noun_restrictions.tsv` | 1 |
| `phrase_evaluation.tsv` | 5 |
| `positional_rules.tsv` | 4 |
| `principal_parts.tsv` | 31 |
| `reviewed_evidence.tsv` | 3206 |
| `semantic_alignments.tsv` | 6 |
| `senses.tsv` | 83 |
| `training_passages.tsv` | 20 |
| `transformation_rules.tsv` | 4 |

Generated morphology SHA-256: `69087aad3f0a2c30505e5a4c4e7d42805452aeb650980d7a4176ca2c08e49949`.

Generated dictionary SHA-256: `1eec6c6355c3a1395374e51b817abbd0a97d5cf67b1e5c1a4202316b71ede13c`.

Large raw inputs are processed by streaming adapters; malformed rows are retained in JSONL quarantine output and the output replacement is atomic.
