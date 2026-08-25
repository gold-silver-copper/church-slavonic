# Synodal prediction precision

The gate for the exploratory segmentation tier (`SYN-PREDICT-VERB-SEGMENTATION-V1`). Every reviewed verb is masked in turn: its generated surfaces are re-derived by the corpus-free predictor and the predicted cell is scored against the engine's own cell (present and simple future count as one pair). Only confidence buckets at or above the floor emit review candidates.

- Masked verb lexemes: 155
- Precision floor: 6000 bp (top prediction)
- Model: `SYN-PREDICT-VERB-SEGMENTATION-V1`

## Precision by confidence bucket

| Bucket (bp) | Surfaces | Top-1 precision | Any-prediction precision | Emits candidates |
|---|---:|---:|---:|---|
| 0-2399 | 178 | 4943 bp | 4943 bp | no |
| 2400-2999 | 368 | 6413 bp | 6657 bp | yes |
| 3000-3399 | 495 | 9050 bp | 9090 bp | yes |
| 3400+ | 352 | 8437 bp | 8437 bp | yes |

## Precision by system

| System | Surfaces | Top-1 precision | Any-prediction precision |
|---|---:|---:|---:|
| aorist | 460 | 6239 bp | 6239 bp |
| imperative | 147 | 3673 bp | 4421 bp |
| imperfect | 144 | 4166 bp | 4166 bp |
| infinitive | 59 | 10000 bp | 10000 bp |
| l-participle | 209 | 7655 bp | 7655 bp |
| present-future | 564 | 7960 bp | 7960 bp |
