# The tagger model

Trained by `cargo xtask train-tagger --epochs 8` on 2026-09-05 (never on the Bible).

- UD PROIEL train: 18327 sentences, 102552 annotated tokens
- Syntacticus: 19038 sentences (4953 held-out sentences removed), 116614 annotated tokens, gold among the readings 104259, examples with several readings 72899
- examples (tokens with several readings, the gold among them): 136985
- epoch 1: training accuracy 82.03%; epoch 2: training accuracy 90.47%; epoch 3: training accuracy 92.94%; epoch 4: training accuracy 94.30%; epoch 5: training accuracy 95.13%; epoch 6: training accuracy 95.51%; epoch 7: training accuracy 95.91%; epoch 8: training accuracy 96.15%
- UD PROIEL dev+test, tokens with several readings: 12623/14532 = 86.86% (the analyzer's first reading: 38.85%)
- by part of speech: a 816/1099 = 74.25% (first reading 47.32%); n 4653/5417 = 85.90% (first reading 37.44%); pron 4112/4621 = 88.99% (first reading 37.94%); v 3042/3395 = 89.60% (first reading 39.59%)
- features 220574, 2646896 bytes

Hashes in `tagger.sha256` (the model and the corpora it was trained on).
