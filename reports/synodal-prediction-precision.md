# Synodal prediction precision

The gate for the exploratory segmentation tier (`SYN-PREDICT-VERB-SEGMENTATION-V1`). Every reviewed verb is masked in turn: its generated surfaces are re-derived by the corpus-free predictor and the predicted cell is scored against the engine's own cell (present and simple future count as one pair). Only confidence buckets at or above the floor emit review candidates.

- Masked verb lexemes: 154
- Precision floor: 6000 bp (top prediction)
- Model: `SYN-PREDICT-VERB-SEGMENTATION-V1`

## Precision by confidence bucket

| Bucket (bp) | Surfaces | Top-1 precision | Any-prediction precision | Emits candidates |
|---|---:|---:|---:|---|
| 0-2399 | 174 | 4942 bp | 4942 bp | no |
| 2400-2999 | 356 | 6404 bp | 6657 bp | yes |
| 3000-3399 | 477 | 9098 bp | 9140 bp | yes |
| 3400+ | 340 | 8441 bp | 8441 bp | yes |

## Precision by system

| System | Surfaces | Top-1 precision | Any-prediction precision |
|---|---:|---:|---:|
| aorist | 446 | 6255 bp | 6255 bp |
| imperative | 143 | 3636 bp | 4405 bp |
| imperfect | 138 | 4202 bp | 4202 bp |
| infinitive | 57 | 10000 bp | 10000 bp |
| l-participle | 201 | 7661 bp | 7661 bp |
| present-future | 547 | 7952 bp | 7952 bp |
