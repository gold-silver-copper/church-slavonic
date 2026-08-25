# Synodal prediction precision

The gate for the exploratory segmentation tier (`SYN-PREDICT-VERB-SEGMENTATION-V1`). Every reviewed verb is masked in turn: its generated surfaces are re-derived by the corpus-free predictor and the predicted cell is scored against the engine's own cell (present and simple future count as one pair). Only confidence buckets at or above the floor emit review candidates.

- Masked verb lexemes: 155
- Precision floor: 6000 bp (top prediction)
- Model: `SYN-PREDICT-VERB-SEGMENTATION-V1`

## Precision by confidence bucket

| Bucket (bp) | Surfaces | Top-1 precision | Any-prediction precision | Emits candidates |
|---|---:|---:|---:|---|
| 0-2399 | 178 | 4943 bp | 4943 bp | no |
| 2400-2999 | 375 | 6453 bp | 6693 bp | yes |
| 3000-3399 | 497 | 9054 bp | 9094 bp | yes |
| 3400+ | 356 | 8455 bp | 8455 bp | yes |

## Precision by system

| System | Surfaces | Top-1 precision | Any-prediction precision |
|---|---:|---:|---:|
| aorist | 462 | 6255 bp | 6255 bp |
| imperative | 147 | 3673 bp | 4421 bp |
| imperfect | 144 | 4166 bp | 4166 bp |
| infinitive | 60 | 10000 bp | 10000 bp |
| l-participle | 212 | 7641 bp | 7641 bp |
| present-future | 572 | 7972 bp | 7972 bp |
