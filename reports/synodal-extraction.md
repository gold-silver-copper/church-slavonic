# Synodal extraction report

Target recension: `synodal-russian`. The curated normalized layer contains 10177 rows across 23 tables; 0 rows are quarantined (ceiling 0).

| Table | Rows |
|---|---:|
| `abbreviation_evaluation.tsv` | 74 |
| `abbreviations.tsv` | 159 |
| `accent_paradigms.tsv` | 18 |
| `accents.tsv` | 45 |
| `alignments.tsv` | 6 |
| `conflicts.tsv` | 1 |
| `engine_capabilities.tsv` | 56 |
| `evaluation.tsv` | 2291 |
| `exact_forms.tsv` | 3041 |
| `examples.tsv` | 8 |
| `irregular_overrides.tsv` | 9 |
| `lexemes.tsv` | 89 |
| `lexical_reviews.tsv` | 847 |
| `linguistic_evaluation.tsv` | 11 |
| `noun_restrictions.tsv` | 1 |
| `phrase_evaluation.tsv` | 5 |
| `positional_rules.tsv` | 4 |
| `principal_parts.tsv` | 31 |
| `reviewed_evidence.tsv` | 3368 |
| `semantic_alignments.tsv` | 6 |
| `senses.tsv` | 83 |
| `training_passages.tsv` | 20 |
| `transformation_rules.tsv` | 4 |

Generated morphology SHA-256: `79fde867720d10634161aa18d3df0956ef88f68aaaad7c28c6c5e7f225cf1e7c`.

Generated dictionary SHA-256: `0913fc45b25c9860891e80b702e311759cce3afba06e0e6d60e368cd186f4d24`.

Large raw inputs are processed by streaming adapters; malformed rows are retained in JSONL quarantine output and the output replacement is atomic.
