# Synodal extraction report

Target recension: `synodal-russian`. The curated normalized layer contains 246 rows across 16 tables; 0 rows are quarantined (ceiling 0).

| Table | Rows |
|---|---:|
| `abbreviations.tsv` | 1 |
| `accents.tsv` | 8 |
| `alignments.tsv` | 6 |
| `conflicts.tsv` | 1 |
| `evaluation.tsv` | 11 |
| `exact_forms.tsv` | 138 |
| `examples.tsv` | 8 |
| `irregular_overrides.tsv` | 5 |
| `lexemes.tsv` | 16 |
| `phrase_evaluation.tsv` | 1 |
| `positional_rules.tsv` | 4 |
| `principal_parts.tsv` | 18 |
| `semantic_alignments.tsv` | 6 |
| `senses.tsv` | 16 |
| `training_passages.tsv` | 3 |
| `transformation_rules.tsv` | 4 |

Generated morphology SHA-256: `f19868fb319c4b7266e8f6b09dca232c1b840df5cfc7c0005762d3291b0fef31`.

Generated dictionary SHA-256: `bc0097c9237c4c944fa9c1a172cb410f01a17c06a0b3d2ac57177523e5b79633`.

Large raw inputs are processed by streaming adapters; malformed rows are retained in JSONL quarantine output and the output replacement is atomic.
