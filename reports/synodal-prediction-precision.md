# Synodal prediction precision

The gate for the exploratory segmentation tier (`SYN-PREDICT-VERB-SEGMENTATION-V1`). Every reviewed verb is masked in turn: its generated surfaces are re-derived by the corpus-free predictor and the predicted cell is scored against the engine's own cell (present and simple future count as one pair). Only confidence buckets at or above the floor emit review candidates.

- Masked verb lexemes: 145
- Precision floor: 6000 bp (top prediction)
- Model: `SYN-PREDICT-VERB-SEGMENTATION-V1`

## Precision by confidence bucket

| Bucket (bp) | Surfaces | Top-1 precision | Any-prediction precision | Emits candidates |
|---|---:|---:|---:|---|
| 0-2399 | 150 | 5000 bp | 5000 bp | no |
| 2400-2999 | 277 | 6245 bp | 6570 bp | yes |
| 3000-3399 | 364 | 9038 bp | 9093 bp | yes |
| 3400+ | 268 | 8470 bp | 8470 bp | yes |

## Precision by system

| System | Surfaces | Top-1 precision | Any-prediction precision |
|---|---:|---:|---:|
| aorist | 370 | 6054 bp | 6054 bp |
| imperative | 117 | 3760 bp | 4700 bp |
| imperfect | 115 | 4434 bp | 4434 bp |
| infinitive | 44 | 10000 bp | 10000 bp |
| l-participle | 145 | 7724 bp | 7724 bp |
| present-future | 425 | 7741 bp | 7741 bp |
