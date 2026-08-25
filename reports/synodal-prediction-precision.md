# Synodal prediction precision

The gate for the exploratory segmentation tier (`SYN-PREDICT-VERB-SEGMENTATION-V1`). Every reviewed verb is masked in turn: its generated surfaces are re-derived by the corpus-free predictor and the predicted cell is scored against the engine's own cell (present and simple future count as one pair). Only confidence buckets at or above the floor emit review candidates.

- Masked verb lexemes: 151
- Precision floor: 6000 bp (top prediction)
- Model: `SYN-PREDICT-VERB-SEGMENTATION-V1`

## Precision by confidence bucket

| Bucket (bp) | Surfaces | Top-1 precision | Any-prediction precision | Emits candidates |
|---|---:|---:|---:|---|
| 0-2399 | 166 | 5000 bp | 5000 bp | no |
| 2400-2999 | 321 | 6323 bp | 6604 bp | yes |
| 3000-3399 | 430 | 9139 bp | 9186 bp | yes |
| 3400+ | 305 | 8524 bp | 8524 bp | yes |

## Precision by system

| System | Surfaces | Top-1 precision | Any-prediction precision |
|---|---:|---:|---:|
| aorist | 414 | 6183 bp | 6183 bp |
| imperative | 132 | 3712 bp | 4545 bp |
| imperfect | 121 | 4380 bp | 4380 bp |
| infinitive | 52 | 10000 bp | 10000 bp |
| l-participle | 177 | 7683 bp | 7683 bp |
| present-future | 497 | 7907 bp | 7907 bp |
