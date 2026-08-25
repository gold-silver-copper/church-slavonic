# Synodal prediction precision

The gate for the exploratory segmentation tier (`SYN-PREDICT-VERB-SEGMENTATION-V1`). Every reviewed verb is masked in turn: its generated surfaces are re-derived by the corpus-free predictor and the predicted cell is scored against the engine's own cell (present and simple future count as one pair). Only confidence buckets at or above the floor emit review candidates.

- Masked verb lexemes: 152
- Precision floor: 6000 bp (top prediction)
- Model: `SYN-PREDICT-VERB-SEGMENTATION-V1`

## Precision by confidence bucket

| Bucket (bp) | Surfaces | Top-1 precision | Any-prediction precision | Emits candidates |
|---|---:|---:|---:|---|
| 0-2399 | 169 | 4970 bp | 4970 bp | no |
| 2400-2999 | 330 | 6363 bp | 6636 bp | yes |
| 3000-3399 | 436 | 9151 bp | 9197 bp | yes |
| 3400+ | 309 | 8543 bp | 8543 bp | yes |

## Precision by system

| System | Surfaces | Top-1 precision | Any-prediction precision |
|---|---:|---:|---:|
| aorist | 420 | 6190 bp | 6190 bp |
| imperative | 134 | 3731 bp | 4552 bp |
| imperfect | 121 | 4380 bp | 4380 bp |
| infinitive | 53 | 10000 bp | 10000 bp |
| l-participle | 181 | 7679 bp | 7679 bp |
| present-future | 507 | 7928 bp | 7928 bp |
