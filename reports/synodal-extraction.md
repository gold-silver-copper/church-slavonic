# Synodal extraction report

Target recension: `synodal-russian`. The curated normalized layer contains 610 rows across 17 tables; 0 rows are quarantined (ceiling 0).

| Table | Rows |
|---|---:|
| `abbreviations.tsv` | 1 |
| `accents.tsv` | 25 |
| `alignments.tsv` | 6 |
| `conflicts.tsv` | 1 |
| `evaluation.tsv` | 38 |
| `exact_forms.tsv` | 260 |
| `examples.tsv` | 8 |
| `irregular_overrides.tsv` | 5 |
| `lexemes.tsv` | 61 |
| `phrase_evaluation.tsv` | 1 |
| `positional_rules.tsv` | 4 |
| `principal_parts.tsv` | 29 |
| `reviewed_evidence.tsv` | 80 |
| `semantic_alignments.tsv` | 6 |
| `senses.tsv` | 61 |
| `training_passages.tsv` | 20 |
| `transformation_rules.tsv` | 4 |

Generated morphology SHA-256: `228aed161d5463faf350444be71abb8391f9e2005f11e887c2b5be060bb2fddb`.

Generated dictionary SHA-256: `1bb5b35bdfe8d0b3a5dbe3793101450046a1bb22c9cc8ce7ca47f4f983f7cca6`.

Large raw inputs are processed by streaming adapters; malformed rows are retained in JSONL quarantine output and the output replacement is atomic.
