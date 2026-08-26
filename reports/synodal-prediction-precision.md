# Synodal prediction precision

The gate for the exploratory segmentation tier (`SYN-PREDICT-VERB-SEGMENTATION-V1`). Every reviewed verb is masked in turn: its generated surfaces are re-derived by the corpus-free predictor and the predicted cell is scored against the engine's own cell (present and simple future count as one pair). Only confidence buckets at or above the floor emit review candidates.

- Masked verb lexemes: 158
- Precision floor: 6000 bp (top prediction)
- Model: `SYN-PREDICT-VERB-SEGMENTATION-V1`

## Precision by confidence bucket

| Bucket (bp) | Surfaces | Top-1 precision | Any-prediction precision | Emits candidates |
|---|---:|---:|---:|---|
| 0-2399 | 186 | 4838 bp | 4838 bp | no |
| 2400-2999 | 402 | 6542 bp | 6766 bp | yes |
| 3000-3399 | 530 | 9113 bp | 9150 bp | yes |
| 3400+ | 375 | 8533 bp | 8533 bp | yes |

## Precision by system

| System | Surfaces | Top-1 precision | Any-prediction precision |
|---|---:|---:|---:|
| aorist | 485 | 6288 bp | 6288 bp |
| imperative | 155 | 3612 bp | 4322 bp |
| imperfect | 144 | 4166 bp | 4166 bp |
| infinitive | 64 | 10000 bp | 10000 bp |
| l-participle | 228 | 7631 bp | 7631 bp |
| present-future | 617 | 8055 bp | 8055 bp |
