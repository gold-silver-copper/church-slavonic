# Synodal 100% strict top-k coverage baseline

This is the reproducible starting checkpoint for the continuing 100% program.
It is not a completion claim. All numbers in the baseline section were
reproduced on 2026-08-16 with `cargo xtask synodal-coverage --offline --check`
before the first recovery change.

## Locked inputs and runtime

The target is the union of the two normalized, pinned `synodal-russian` corpus
adapters below under `GenerationPolicy::Strict` and
`OrthographyProfile::SynodalLiturgical`.

| Artifact | SHA-256 |
|---|---|
| `data/intermediate/synodal/ponomar-elizabeth-bible-2026-08-09.jsonl` | `ef0323df940c93c9b72a3cbb6f7adfb062ba38ffcdcf401eff5cf369c4869c26` |
| `data/intermediate/synodal/wikisource-church-slavonic-bible-2026-08-09.jsonl` | `913d9781ef511988d8bcc5d19b1b8c63c7582cd5e476f62469eff199e7c2c08f` |
| generated morphology registry | `716eb7c74b23249428109bb7711d40e30bf9584db08e631e4e31aaef7f63b21a` |
| generated dictionary registry | `b2c1b080aafcd482fbe1b2ba457490dc88c20ee9069d7d67e206a6e9133ed4a4` |
| held-out evaluation report | `810e014bde25445da37ad1092b01e35854f95be3caa65b5ed4615ecd109d3312` |

The denominator is 74,130 passages, 1,313,344 tokens, and 57,476 normalized
token types. It must not be changed to raise the percentage. Genuine adapter or
source-transcription corrections require a separately audited before/after
denominator change.

## Starting coverage

| Measure | Tokens | Percent |
|---|---:|---:|
| Top-1 analyzed | 607,414 | 46.249421% |
| Top-k analyzed | 920,924 | 70.120547% |
| Ambiguous | 5,591 | 0.425707% |
| Unresolved | 390,555 | 29.737449% |

The old `top_k_uncovered_frequency_by_surface` map contained 55,678 surfaces
and 392,223 occurrences. It omitted 197 recognized Cyrillic-numeral occurrences
that were outside top-k but had no gap record, so the true initial outside-top-k
count was 392,420. The complete frontier introduced by the first program wave
must always satisfy:

```text
sum(frontier.token_frequency) = total_tokens - top_k_analyzed
```

| Corpus/source | Tokens | Top-1 | Top-k | Ambiguous | Unresolved |
|---|---:|---:|---:|---:|---:|
| Elizabeth Bible / Ponomar | 661,857 | 305,834 | 463,461 | 2,792 | 197,479 |
| Church Slavonic Bible / Wikisource | 651,487 | 301,580 | 457,463 | 2,799 | 193,076 |

## Starting status and gap inventory

| Resolver status | Tokens |
|---|---:|
| `abbreviation-expansion` | 43,664 |
| `ambiguous` | 5,591 |
| `cyrillic-numeral` | 197 |
| `exact-synodal-attestation` | 629,827 |
| `spelling-variant` | 1,668 |
| `synodal-irregular-override` | 37,172 |
| `synodal-normative-table` | 185,968 |
| `synodal-productive-rule` | 18,702 |
| `unresolved` | 390,555 |

| Primary gap | Tokens |
|---|---:|
| `ambiguity-or-spelling-variant` | 7,259 |
| `missing-accent-or-orthographic-metadata` | 21,067 |
| `missing-declension-or-class` | 43 |
| `missing-verb-principal-part` | 72 |
| `unknown-lexeme` | 369,373 |

| Diagnostic recovery route | Tokens |
|---|---:|
| `abbreviation-registry` | 12,329 |
| `exact-evidence` | 459 |
| `reviewed-class` | 43 |
| `reviewed-principal-part` | 72 |
| `spelling-variant` | 22,735 |
| `ungrouped-unknown` | 356,585 |

These route totals are projections, not analyses, and cannot be added to the
coverage numerator.

## First realized recovery wave

Canonical Cyrillic numerals previously had only a resolver status. The engine
now returns a typed `CyrillicNumeral` containing the parsed numeric value and
canonical spelling; only a successful canonical parse counts. This realizes an
exact +197 top-1 and +197 top-k gain without a lexical placeholder:

| Measure | Starting | After typed numerals | Delta |
|---|---:|---:|---:|
| Top-1 analyzed | 607,414 | 607,611 | +197 |
| Top-k analyzed | 920,924 | 921,121 | +197 |
| Top-k percent | 70.120547% | 70.135547% | +0.015000 pp |
| Unresolved | 390,555 | 390,555 | 0 |
| Outside strict top-k | 392,420 | 392,223 | -197 |

Ponomar realizes 92 of these occurrences and Wikisource 105. The complete
frontier now contains 60,085 mark-sensitive rows accounting for exactly the
remaining 392,223 occurrences. The older accentless diagnostic map has 55,678
keys; the difference is intentional because distinct printed accents,
breathings, capitalization, and superscripts must not be merged during review.

## Second realized recovery wave

The locked SCI Ponomar 2016 artifact contains both a frequency list and a
21,104-row dictionary workbook. The adapter now preserves the workbook as
structured candidate records without treating mixed-recension dictionary rows
as target grammatical cells. Ten high-frequency headwords were manually
reviewed against independent source-partition Ponomar Bible witnesses. Public
Polyakov grammatical-dictionary pages were used as a manual part-of-speech and
cell cross-check where an entry was available; they were not bulk imported or
redistributed.

The admitted forms are `жре́цъ`, `саꙋ́лъ`, `со́нмъ`, `совѣ́тъ`,
`і҆ѡнаѳа́нъ`, `ѕло̀`, `премꙋ́дрость`, `то́чїю`, `прѧ́мѡ`, and `сквозѣ̀`.
Every row is an exact target-recension lexical form or indeclinable form. No
productive noun or proper-name paradigm was inferred from a dictionary
headword.

| Measure | After typed numerals | After exact lexical review | Delta |
|---|---:|---:|---:|
| Top-1 analyzed | 607,611 | 610,586 | +2,975 |
| Top-k analyzed | 921,121 | 924,096 | +2,975 |
| Top-k percent | 70.135547% | 70.362068% | +0.226521 pp |
| Unresolved | 390,555 | 387,576 | -2,979 |
| Outside strict top-k | 392,223 | 389,248 | -2,975 |

Four related occurrences moved from `unresolved` to a diagnosed
`spelling-variant` state but remain outside strict top-k, so they are not
counted as recovered. The complete frontier now contains 60,068 rows whose
frequencies sum exactly to 389,248. The held-out evaluation remains unchanged
at 2,136/2,136 within top-k for both expanded and printed profiles.

| Current generated artifact | SHA-256 |
|---|---|
| morphology registry | `86940b57c49945aa844b862a78b0a671547b1e24c47e5131ed79ad4afb380411` |
| dictionary registry | `7f5723e8fc52817c1d218e8139267ae9d4eb2e328eadb4a8263a30d60f4e3ed7` |
| held-out evaluation report | `810e014bde25445da37ad1092b01e35854f95be3caa65b5ed4615ecd109d3312` |

## Third realized recovery wave

Two independently evidenced closed-class items were recovered without adding a
productive open-class paradigm. `Заⷱ҇` is now an exact non-reversible
abbreviation of `зачало` in its pericope-heading context. The derived
preposition `вослѣдъ` is a separate lexeme from the noun `слѣдъ`: Alypy §110
classifies the unabbreviated form, §197 prints `в̾слѣ́дъ`, and disjoint locked
Ponomar passages provide source and held-out target witnesses.

The realized +1,162 consists of 585 `Заⷱ҇` occurrences, 565 contracted
`в̾слѣ́дъ` occurrences, and 12 unabbreviated `вослѣ́дъ` occurrences. All three
were counted by the strict resolver; none came from an accentless fallback.

| Measure | After exact lexical review | After closed-class recovery | Delta |
|---|---:|---:|---:|
| Top-1 analyzed | 610,586 | 611,748 | +1,162 |
| Top-k analyzed | 924,096 | 925,258 | +1,162 |
| Top-k percent | 70.362068% | 70.450545% | +0.088477 pp |
| Unresolved | 387,576 | 386,414 | -1,162 |
| Outside strict top-k | 389,248 | 388,086 | -1,162 |

The complete frontier contains 60,064 rows whose frequencies sum exactly to
388,086. The expanded and printed held-out sets remain 2,137/2,137 correct
within top-k. Typed abbreviations are 75/75 within top-k, and phrases remain
14/14 in both profiles.

| Current generated artifact | SHA-256 |
|---|---|
| morphology registry | `41ffad570549626acbf779fdc4726f5c89ab00dfc8fc118c17aa2f96843b89e2` |
| dictionary registry | `7d486f9e8b028f13f649818a99f744e008d2726b63d32f22b25ccae9467035a5` |
| held-out evaluation report | `07825f3e25d8813985822e8d4da959fe34168d5f080ef5541aa81c4976e72055` |

## Fourth realized recovery wave

Frontier rank one `ѕла̑ѧ` is a substantivized neuter-plural form of the hard
adjective `ѕлый`, not a noun form of `ѕло`. Alypy §58 supplies the positive and
comparison stems, locked Ponomar passages supply disjoint nominative,
accusative, short-masculine, semantic, and held-out witnesses, and the complete
listed accent table was manually cross-checked without bulk-importing the
metadata-only Polyakov source.

The engine now types mobile-`о` short-masculine formation (`ѕл- : ѕол-`) and
word-left accent placement for vowel-less stems. The one registered lexeme
therefore generates the complete positive and comparison paradigms; exact
Synodal cells retain precedence for `ѕо́лъ`, `ѕлы́й`, `ѕлѣ́йшїй`, and the
kamora-bearing `ѕла̑ѧ`. The analysis layer preserves the legitimate `ѕло̀`
noun/adjective homonym instead of collapsing their identities.

| Measure | After closed-class recovery | After productive `ѕлый` | Delta |
|---|---:|---:|---:|
| Top-1 analyzed | 611,748 | 611,489 | -259 |
| Top-k analyzed | 925,258 | 926,282 | +1,024 |
| Top-k percent | 70.450545% | 70.528513% | +0.077968 pp |
| Unresolved | 386,414 | 385,390 | -1,024 |
| Outside strict top-k | 388,086 | 387,062 | -1,024 |

The top-1 decrease is an honest consequence of newly exposed ambiguity and
does not affect strict top-k recovery. The complete frontier contains 60,040
rows whose frequencies sum exactly to 387,062. The expanded and printed
held-out sets are 2,138/2,138 correct within top-k; the newly added `ѕла̑ѧ`
row is evaluation-only. Typed abbreviations remain 75/75 and phrases 14/14
within top-k.

| Current generated artifact | SHA-256 |
|---|---|
| morphology registry | `1d46a9db33ec8f0718f81a38fca64901549162331411121246bb4bee086ffd9d` |
| dictionary registry | `e4f1b260c800f3cde6a096843f7b25f92dd14ff8a0eb5e6edd14ab58e98ad03d` |
| held-out evaluation report | `8d672f15701068e57213bb09de8fa69fa85ec3b3b794e6168b47a43ab3021442` |

## Fifth realized recovery wave

Two already identifiable noun families were completed without admitting an
accentless fallback. The stable `дꙋша` identity was promoted from exact-only
rows to Alypy's complete second-mixed declension with a target-backed mobile
accent paradigm. Exact source rows retain precedence for the independently
typed singular, dual, and plural cells. In particular, the corpus distinguishes
ordinary nominative plural `дꙋ́ши` from the rare direct dual `дꙋши̑`; they are
not merged as interchangeable spellings.

The existing `адѡнаі` lexeme was already a complete indeclinable noun under
Alypy §37. A source-partition Ponomar formula now supplies its reusable initial
psili and final-grave presentation contract, `а҆дѡнаі̀`, while a disjoint
evaluation passage checks it. The engine therefore preserves one invariant
form across every typed case and number rather than adding 389 untyped token
exceptions.

Of the +1,199 recovered strict top-k occurrences, +810 come from the productive
`дꙋша` family and +389 from `а҆дѡнаі̀`.

| Measure | After productive `ѕлый` | After productive noun completion | Delta |
|---|---:|---:|---:|
| Top-1 analyzed | 611,489 | 611,367 | -122 |
| Top-k analyzed | 926,282 | 927,481 | +1,199 |
| Top-k percent | 70.528513% | 70.619807% | +0.091294 pp |
| Unresolved | 385,390 | 384,191 | -1,199 |
| Outside strict top-k | 387,062 | 385,863 | -1,199 |

The top-1 change again records newly visible legitimate ambiguity rather than a
loss of top-k analyses. The complete frontier contains 60,027 rows whose
frequencies sum exactly to 385,863. Expanded and printed held-out evaluation
are both 2,141/2,141 correct within top-k; the four new noun witnesses are
partition-disjoint from their runtime evidence.

| Current generated artifact | SHA-256 |
|---|---|
| morphology registry | `9d5628760a7b381e7c703cc16b5a975d5905e12c4d43ad1db76b5fbf9d0d33a3` |
| dictionary registry | `e4f1b260c800f3cde6a096843f7b25f92dd14ff8a0eb5e6edd14ab58e98ad03d` |
| held-out evaluation report | `458d8841eb237fd8be781d10e2982eec6bcf2dbdecba25d5eb68ee69edaa6556` |

## Cumulative checkpoint after waves 6–24

The 2026-08-16 checkpoint aggregates the subsequent reviewed release waves;
their individual rule, source, decision, and held-out evaluation records remain
in the versioned data and generated reports. Those waves added or completed
`ꙗзыкъ`; typed noun, adjective, and proper-name families; `вїно`;
`положити`; participial systems; closed-class adverbs, prepositions, and
an interjection; soft `-їе` nouns; `кънязь`; `доко́лѣ`; typed salvation
abbreviations; reusable lexical-animacy restrictions; the bounded
metathesized `жрецъ : жерц-` family; and the attested `рабѡ́въ`
plural variant.

The denominator, locked corpus artifacts, normalization policy, and strict
top-k criterion are unchanged. Runtime evidence remains source-partitioned
from held-out target-recension evaluation; evaluation rows do not admit
analyses. Potentially productive frontier families without sufficient lexical
identity, principal-part, or cell evidence remain explicitly deferred.

| Measure | After wave 5 | After wave 24 | Delta |
|---|---:|---:|---:|
| Top-1 analyzed | 611,367 | 615,722 | +4,355 |
| Top-k analyzed | 927,481 | 955,709 | +28,228 |
| Top-k percent | 70.619807% | 72.769130% | +2.149323 pp |
| Unresolved | 384,191 | 356,606 | -27,585 |
| Outside strict top-k | 385,863 | 357,635 | -28,228 |

The complete frontier contains 59,437 rows whose frequencies sum exactly to
357,635. Elizabeth Bible / Ponomar contributes 481,242 analyzed top-k tokens;
Church Slavonic Bible / Wikisource contributes 474,467. The deterministic
family gate is 200/200 reviewed. Expanded and printed held-out evaluation are
both 2,258/2,258 correct within top-k, and the exact registry round-trips
3,229/3,229 rows in both forms.

| Current generated artifact | SHA-256 |
|---|---|
| morphology registry | `4bf99fea5b4ec26c071c82127829f202c2eee24a6bf71d35dbe1d76acc5118f1` |
| dictionary registry | `bf240418a53abe7d4d8f88f782d7876a0382af3417f76b0c13bbf3fdd203a8a4` |
| held-out evaluation report | `5eca0f77d6d5b0ecaecc6d90912fb1a90a920ce7993adf12143dd91b9d04dc88` |

## Wave 25 checkpoint: `ѻтроча : ѻтрочат-`

Wave 25 admits the already registered fourth-neuter family only after Alypy
§43 supplied its complete singular, dual, and plural table and six
source-partition passages independently established short-stem varia,
extended-stem acute, and initial psili. The nominative- and
accusative-singular grave cells are exact; the source-attested titlecase acute
nominative remains a bounded exact variant; all remaining cells use the
reviewed reusable accent paradigm. Three passage-disjoint singular and plural
cells remain held out.

The same wave closed a fail-open invariant for restricted nouns. Caller
irregulars and committed exact rows can no longer bypass lexical number or
animacy inventories; untyped restricted-noun `lexical-form` rows fail closed.
The three older reviews for `людїе`, collective `братїѧ`, and `Кнѧ̑зи` were
migrated to their already established typed noun cells, removing same-identity
untyped duplicates without adding new surface forms.

| Measure | After wave 24 | After wave 25 | Delta |
|---|---:|---:|---:|
| Top-1 analyzed | 615,722 | 616,220 | +498 |
| Top-k analyzed | 955,709 | 955,900 | +191 |
| Top-k percent | 72.769130% | 72.783673% | +0.014543 pp |
| Unresolved | 356,606 | 356,415 | -191 |
| Outside strict top-k | 357,635 | 357,444 | -191 |

The complete frontier now contains 59,429 rows whose frequencies sum exactly
to 357,444. Elizabeth Bible / Ponomar contributes 481,339 analyzed top-k
tokens; Church Slavonic Bible / Wikisource contributes 474,561. The
deterministic family gate remains 200/200 reviewed after explicitly deferring
the newly entering `па́схꙋ` proposal pending an independent identity and
complete family decision.

Expanded and printed held-out evaluation are both 2,261/2,261 correct within
top-k; top-1 is 2,197 expanded and 2,113 printed. The exact registry
round-trips 3,232/3,232 rows in both profiles.

| Current generated artifact | SHA-256 |
|---|---|
| morphology registry | `d3542d95239d58a3dafe1aaf3decdf19f196ef003550a22963df23549d766792` |
| dictionary registry | `bf240418a53abe7d4d8f88f782d7876a0382af3417f76b0c13bbf3fdd203a8a4` |
| held-out evaluation report | `529bea72d199951817c8b249fd1797ab222054a3451b7cd79bbc77eaf633192d` |

## Wave 26 checkpoint: singular cardinal `єдинъ`

Wave 26 completes the singular-only cardinal-one presentation contract from
Alypy §62. A new numeral-specific accent scope applies fixed acute on `и` and
initial psili only to the source-licensed singular numeral inventory; Alypy's
dual and plural columns remain outside this identity because the source
explicitly describes them as pronominal or adjectival uses. Governed
source-partition phrases independently establish neuter instrumental
`є҆ди́нѣмъ` and feminine instrumental `є҆ди́ною`, while masculine and feminine
instrumental passages remain disjoint held-out evaluation.

The complete table recovered 254 tokens: the two leading frontier spellings
plus lower-frequency acute `є҆ди́номъ`, `є҆ди́ной`, and `є҆ди́ноѧ` forms. Top-1
drops because the completed paradigm now retains additional genuine
same-lexeme gender, animacy, and case homographs that were formerly absent;
this is an intentional precision-preserving ambiguity expansion, while top-k
recall increases and unresolved coverage decreases exactly.

The wave also hardened the family-admission gate: every
`exact-indeclinable-*` class label must now match both the runtime part of
speech and the sole `indeclinable` cell. The archaic `быти` participle label
must independently match Verb POS, participle support, and both reviewed
present-active principal parts.

| Measure | After wave 25 | After wave 26 | Delta |
|---|---:|---:|---:|
| Top-1 analyzed | 616,220 | 615,012 | -1,208 |
| Top-k analyzed | 955,900 | 956,154 | +254 |
| Top-k percent | 72.783673% | 72.803013% | +0.019340 pp |
| Ambiguous | 7,205 | 7,205 | 0 |
| Unresolved | 356,415 | 356,161 | -254 |
| Outside strict top-k | 357,444 | 357,190 | -254 |

The complete frontier now contains 59,423 rows whose frequencies sum exactly
to 357,190. Elizabeth Bible / Ponomar contributes 481,467 analyzed top-k
tokens; Church Slavonic Bible / Wikisource contributes 474,687. The
deterministic family gate is again 200/200 reviewed after explicitly deferring
the newly entering `бо́йсѧ` proposal pending a registered reflexive-verb
identity, typed imperative cell, principal parts, and source-bounded accent
contract.

Expanded and printed held-out evaluation are both 2,263/2,263 correct within
top-k; top-1 is 2,199 expanded and 2,115 printed. The exact registry
round-trips 3,234/3,234 rows in both profiles. Abbreviations remain 81/81 and
phrases 14/14 within top-k.

| Current generated artifact | SHA-256 |
|---|---|
| morphology registry | `dcdf1a8be48878842a2ef31bdc1620bc4f3240a24d8b9bb599e82850b729eeea` |
| dictionary registry | `bf240418a53abe7d4d8f88f782d7876a0382af3417f76b0c13bbf3fdd203a8a4` |
| held-out evaluation report | `3dbd9ae0c67a2e3ba6a84b23fa1322bdda015fc67f9e5fcb7793f2baf83f6ac1` |

## Wave 27 checkpoint: inanimate `престолъ`

Wave 27 completes the existing throne or altar identity as an inanimate
first-hard masculine. Alypy §34 supplies the complete ending inventory, while
ten governed source-partition passages independently establish the fixed acute
on the second stem vowel across singular and plural cases. The ordinary class
remains productive over singular, dual, and plural; the directly witnessed
wide-omega genitive plural `престо́лѡвъ` is an exact-first spelling alongside
the normative `престо́ловъ` and zero-ending variants, without introducing a
global omega rewrite.

Four disjoint evaluation passages test nominative and genitive singular,
instrumental singular, and inanimate accusative plural. Forward and reverse
tests also require every inanimate cell, reject animate readings, preserve the
exact wide-omega witness, and reject a deliberately misplaced accent.

| Measure | After wave 26 | After wave 27 | Delta |
|---|---:|---:|---:|
| Top-1 analyzed | 615,012 | 615,152 | +140 |
| Top-k analyzed | 956,154 | 956,440 | +286 |
| Top-k percent | 72.803013% | 72.824789% | +0.021776 pp |
| Ambiguous | 7,205 | 7,205 | 0 |
| Unresolved | 356,161 | 355,875 | -286 |
| Outside strict top-k | 357,190 | 356,904 | -286 |

The complete frontier now contains 59,413 rows whose frequencies sum exactly
to 356,904. Elizabeth Bible / Ponomar contributes 481,609 analyzed top-k
tokens; Church Slavonic Bible / Wikisource contributes 474,831. The recovery
exceeds the proposal's 213-token lower bound because the completed family also
licenses lower-frequency ordinary oblique and plural surfaces. The
deterministic family gate is again 200/200 reviewed after explicitly deferring
the newly entering `вои́стиннꙋ` proposal pending evidence that distinguishes an
indeclinable adverb from a frozen phrase.

Expanded and printed held-out evaluation are both 2,267/2,267 correct within
top-k; top-1 is 2,203 expanded and 2,119 printed. The exact registry
round-trips 3,237/3,237 rows in both profiles. Abbreviations remain 81/81 and
phrases 14/14 within top-k.

| Current generated artifact | SHA-256 |
|---|---|
| morphology registry | `2c651760a7e0795eb87381111f2e2c7f65f688af03ff1bfadb799752ecbf8aa0` |
| dictionary registry | `bf240418a53abe7d4d8f88f782d7876a0382af3417f76b0c13bbf3fdd203a8a4` |
| held-out evaluation report | `3ac815670b79cf40f4313767b2202aad04d331c1dbeab1b6eef0c05a383b2bc4` |

## Starting morphological-system slices

These slices describe tokens for which the resolver could infer a requested or
returned system. `unresolved` contains the remaining unclassified surfaces.

| System | Tokens | Top-1 | Top-k | Ambiguous | Unresolved |
|---|---:|---:|---:|---:|---:|
| `adjective` | 17,235 | 5,112 | 14,922 | 0 | 2,313 |
| `aorist` | 40,051 | 21,953 | 39,569 | 0 | 482 |
| `determiner` | 17,424 | 12,287 | 16,899 | 0 | 525 |
| `future` | 15,490 | 14,475 | 15,024 | 0 | 466 |
| `imperative` | 4,198 | 2,579 | 3,862 | 46 | 336 |
| `imperfect` | 6,179 | 2,986 | 6,058 | 0 | 121 |
| `indeclinable` | 368,125 | 365,585 | 367,703 | 2,117 | 422 |
| `infinitive` | 1,421 | 1,389 | 1,389 | 0 | 32 |
| `l-participle` | 1,360 | 1,231 | 1,231 | 0 | 129 |
| `lexical-form` | 93,374 | 50,096 | 92,341 | 1,991 | 1,033 |
| `noun` | 159,244 | 103,213 | 151,990 | 7 | 7,254 |
| `numeral` | 8,876 | 392 | 6,642 | 0 | 2,234 |
| `past-active-participle` | 2,603 | 211 | 2,044 | 0 | 559 |
| `past-passive-participle` | 735 | 0 | 492 | 0 | 243 |
| `present` | 23,269 | 20,372 | 22,243 | 1,415 | 1,026 |
| `present-active-participle` | 6,052 | 3,051 | 4,968 | 0 | 1,084 |
| `present-passive-participle` | 33 | 0 | 0 | 0 | 33 |
| `pronoun` | 176,281 | 2,482 | 173,547 | 15 | 2,734 |
| `unresolved` | 371,358 | 0 | 0 | 0 | 369,493 |
| `verbal-noun` | 36 | 0 | 0 | 0 | 36 |

## Held-out precision checkpoint

The locked evaluation contains 2,267 lexical/cell rows. Expanded and printed
profiles both return a correct result within top-k for 2,267/2,267 rows; top-1
is 2,203/2,267 expanded and 2,119/2,267 printed. Abbreviation top-k is 81/81,
and phrase top-k is 14/14 in both profiles. Every recovery wave must preserve
these results unless an evidence-backed correction explicitly updates the
contract.

## Completion gate

Completion requires exactly 1,313,344/1,313,344 strict top-k analyses, zero
unresolved tokens, an empty complete frontier, and 100% independently in every
corpus, source, partition, and source/partition slice. Candidate queues,
diagnostic families, accentless fallbacks, cross-recension forms, and held-out
occurrences used as their own evidence never count.

The executable gate is:

```sh
cargo xtask synodal-coverage --offline --check --require-complete
```

It also validates the two locked intermediate hashes and the 74,130-passage,
1,313,344-token, 57,476-type denominator, and refuses custom or truncated
inputs.

The target is deliberately corpus-bounded. Passing it will prove complete
analysis of the locked canonical corpus, not knowledge of every historical or
possible Church Slavonic word.
