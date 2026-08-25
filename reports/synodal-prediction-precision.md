# Synodal prediction precision

The gate for the exploratory segmentation tier (`SYN-PREDICT-VERB-SEGMENTATION-V1`). Every reviewed verb is masked in turn: its generated surfaces are re-derived by the corpus-free predictor and the predicted cell is scored against the engine's own cell (present and simple future count as one pair). Only confidence buckets at or above the floor emit review candidates.

- Masked verb lexemes: 152
- Precision floor: 6000 bp (top prediction)
- Model: `SYN-PREDICT-VERB-SEGMENTATION-V1`

## Precision by confidence bucket

| Bucket (bp) | Surfaces | Top-1 precision | Any-prediction precision | Emits candidates |
|---|---:|---:|---:|---|
| 0-2399 | 169 | 4970 bp | 4970 bp | no |
| 2400-2999 | 334 | 6347 bp | 6616 bp | yes |
| 3000-3399 | 443 | 9119 bp | 9164 bp | yes |
| 3400+ | 315 | 8507 bp | 8507 bp | yes |

## Precision by system

| System | Surfaces | Top-1 precision | Any-prediction precision |
|---|---:|---:|---:|
| aorist | 426 | 6197 bp | 6197 bp |
| imperative | 135 | 3703 bp | 4518 bp |
| imperfect | 126 | 4285 bp | 4285 bp |
| infinitive | 53 | 10000 bp | 10000 bp |
| l-participle | 185 | 7675 bp | 7675 bp |
| present-future | 511 | 7925 bp | 7925 bp |
