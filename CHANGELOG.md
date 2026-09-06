# Changelog

## 3.4.0 (2026-09-05) — the verb that is several lexemes, the tagger's transfer

The plan is `V3.3-PROMPT.md` Parts 3–4. A minor release: the library is
3.3's; the constraint layer gained two eliminations (bare-loc, bare-voc),
the tools a measurement command (`tagger-transfer`); no id moved, no
model changed.

| Number | 3.3.0 | 3.4.0 |
|---|---|---|
| hand overlay (3,757 leaves): rules resolve / exclude | 459 / 0 | 466 / 0 |
| tagger on the overlay | 74.9% (1,228 of 1,639) | 74.6% (1,200 of 1,609) |
| five-fold transfer (measured, not shipped) | — | OCS + four folds 78.93%, OCS only 75.12%, bundled 74.58% |
| Bible treebank | one cell 240,672 (38.1%), sets 2,311, tagger 190,494 (30.1%), closed 179,009, several lexemes 12,880 (2.0%), verbatim 5,430 (0.9%) | one cell 244,589 (38.7%), sets 2,306 (0.4%), tagger 186,594 (29.5%), closed 179,009 (28.3%), several lexemes 12,868 (2.0%), verbatim 5,430 (0.9%); without the rules one cell 27.8%, sets 36.6%, several lexemes 6.2% |
| held-out recall (UD dev+test) | unchanged | unchanged |

### Part 3 — the verb that is several lexemes (2026-09-05)

- **`bare-loc`**: a reading whose every cell is locative, with no
  locative-governing preposition before it in the chunk, goes — from a
  set or from a several-lexeme token (ви́дѣ the aorist; one-subject then
  fires in Genesis 1:4). A leaf that is only locative and a chunk at the
  verse's start are left alone.
- **`bare-voc`**: voc-drop for a several-lexeme token.
- A genitive beside a nominative subject: not a rule (NOTES).
- `CS_DEBUG_VERSE=<ch>:<vs>` prints an overlay verse's auto tree in
  `score-disambiguation`.

| Number | before | after |
|---|---|---|
| hand overlay (3,757 leaves): rules resolve / exclude | 459 / 0 | 466 / 0 (bare-loc 26 alone, 120 with the others; bare-voc 3) |
| one-subject on the overlay | 10 resolved | 11 |
| tagger on the overlay | 74.92% (1,228 of 1,639) | 74.58% (1,200 of 1,609: the rules took 30 of its leaves) |
| Bible treebank | one cell 240,672 (38.1%), sets 2,311, tagger 190,494 (30.1%), closed 179,009, several lexemes 12,880 (2.0%), verbatim 5,430 (0.9%) | one cell 244,589 (38.7%), sets 2,306 (0.4%), tagger 186,594 (29.5%), closed 179,009 (28.3%), several lexemes 12,868 (2.0%), verbatim 5,430 (0.9%); zero mismatches |
| the same with `CS_NO_DISAMBIGUATE=1` | — | one cell 175,759 (27.8%), sets 231,334 (36.6%), tagger 0, several lexemes 39,264 (6.2%), verbatim 5,430 |

### Part 4 — the tagger's transfer, measured (2026-09-05)

- **`cargo xtask tagger-transfer`**: five folds of the overlay by
  chapter; a tagger trained on the OCS material plus four folds, scored
  on the fifth, beside the OCS-only training and the bundled model.
  Five-fold: OCS + the other folds 78.93% (1,307 of 1,656), the
  OCS-only training repeated 75.12%, the bundled model 74.58% — 3.8
  points for 1,300 Synodal examples. The shipped model is unchanged (OCS dev+test 86.9%).
- The tagger's errors on the overlay by kind are recorded (NOTES); no
  fold change, no retraining.

## 3.3.0 (2026-09-05) — the verbatim residue

The plan is `V3.3-PROMPT.md` Parts 0–2 (Parts 3–4 are 3.4.0's). A minor
release: no id moved (50 added), no letter of Old Church Slavonic
changed, the library's API is 3.2's — `Form::print` gained two rules of
the print (the izhitsa's kendema, the paerok as a letter), a titlo row
may name a closed lexeme, and the phonological word takes a pronoun's
clitic as its enclitic. The Bible's verbatim share fell from 5.7% to
0.9%.

| Number | 3.2.0 | 3.3.0 |
|---|---|---|
| Synodal lexemes: nouns / adjectives / verbs / pronouns / closed | 13,147 / 8,323 / 8,210 / 68 / 1,340 | 13,172 / 8,323 / 8,224 / 72 / 1,351 |
| titlo rows | 22 | 135 |
| quarantine | 1,442 lines | 1,433 |
| ids changed | 0 | 0 (50 added) |
| census verbatim (`census verbatim`) | 36,568 | 5,408: (a) 2,147 found by key, (b) 1,117 titlo, (c) 2,144 no reading |
| Bible treebank | one cell 214,958 (34.0%), sets 1,782 (0.3%), tagger 187,945 (29.7%), closed 178,215 (28.2%), several lexemes 12,298 (1.9%), verbatim 35,731 (5.7%), apparatus 1,017 | one cell 240,672 (38.1%), sets 2,311 (0.4%), tagger 190,494 (30.1%), closed 179,009 (28.3%), several lexemes 12,880 (2.0%), verbatim 5,430 (0.9%), apparatus 1,150; zero mismatches |
| hand overlay | 337 verses, 3,377 leaves; rules exclude none; tagger 75.6% (1,259 of 1,665) | 337 verses, 3,757 leaves (389 redrafted); rules exclude none (resolve 459); tagger 74.9% (1,228 of 1,639) |
| Polyakov cells reproduced by the primary: nouns / adjectives / verbs | 43,847 / 88,572 / 116,673 | 43,934 / 88,572 / 117,016 |
| held-out recall (UD dev+test) | 95.48 / 89.31 / 90.89 / 99.25 / 98.07 | unchanged |

### Part 0 — the census of the verbatim (2026-09-05)

`cargo xtask census verbatim`: 36,568 verbatim leaves — found by key
but not exactly 20,577 (marks only 13,908: во/ко/со 9,625 the lifter
never lifts as the prepositions' variants; і/ї in loanwords 3,071; the
izhitsa's kendema 2,055; the genitive's wide -гѡ on the н-forms and
ѹ҆̀бо 1,144; head ѧ 161), titlo tokens with no row 7,187 (гдⷭ҇нь 1,615,
нн҃ѣ 903, цр҃ь 407, првⷣный 283, блгⷣть 255, …), no reading 8,804 (the
cardinal numerals and their compounds, є҆ди́нъ's spelling, всѧ́кїй's and
и҆ны́й's long forms, хотѣти's хощ- stem, любо́вь, проти́вꙋ, бли́з̾, ѡ҆б̾,
ра́звѣ, а҆дѡнаі̀). The verb that is several lexemes: 135 tokens.

### Part 1 — the print's last letters (2026-09-05)

- **The izhitsa's kendema** is a rule of `Form::print` (`ѷ` for an
  unstressed non-initial ѵ read as a vowel; bare after а/е, the psili
  at the head); `from_print` folds it back.
- **The paerok** (U+033E, an elided jer: в̾слѣ́дъ, ѡ҆б̾) is a letter of
  the word (`ꙿ` in the letters, `ъ` in the comparison key).
- **The loanword's ї** before a consonant (кївѡ́тъ, вїно̀) is written by
  the importer from `data/loanword-iota.tsv` (`census verbatim --write`)
  with the lifted prints as the veto; the id folds ї/і and ѷ/ѵ.
- **The n-forms' genitive** -негѡ → -него (the pronoun class); ѹ҆̀бо,
  в̾слѣ́дъ, ѡ҆б̾ as hand variants; a form's own varia survives `realise`.
- **The vocalised prepositions** во/ко/со are the leaf's alternative
  `(f въ.x.2 :alt 1)`, never verbatim.
- **Titlo rows**: 113 (22 → 135) added by hand with the Bible count (a row may
  name a closed lexeme, pos `x`); `titlo::abbreviate` keeps a mark with
  the skeleton's last letter and the skeleton keeps ѡ a letter; an abbr
  node carries the row's skeleton, `(abbr "гл҃" "гла" …)`, and renders
  through the row that names its lexeme; `cargo xtask titlo <surface>`
  prints the index's entries.

| Number | before | after |
|---|---|---|
| census verbatim | 36,568: (a) 20,577, (b) 7,187, (c) 8,804 | 14,069: (a) 4,508, (b) 1,141, (c) 8,420 |
| (a) marks only / і-ї / kendema / wide-narrow о / head ѧ | 13,908 / 3,071 / 2,055 / 1,144 / 161 | 3,896 / 128 / 8 / 85 / 161 |
| Bible treebank | one cell 214,958 (34.0%), sets 1,782, tagger 187,945 (29.7%), closed 178,215, several lexemes 12,298 (1.9%), verbatim 35,731 (5.7%) | one cell 237,565 (37.6%), sets 2,093 (0.3%), tagger 186,780 (29.6%), closed 178,215 (28.2%), several lexemes 12,858 (2.0%), verbatim 13,418 (2.1%); zero mismatches |
| held-out recall | 95.48 / 89.31 / 90.89 / 99.25 | unchanged |
| ids | — | unchanged |

### Part 2 — the lexicon's gaps (2026-09-05)

- **The pronominal adjectives** є҆ди́нъ, всѧ́къ, и҆́нъ, ѻ҆́нъ decline with
  the adjective's endings beside the pronominal ones (є҆ди́нагѡ, є҆ди́ныѧ,
  и҆ны́хъ, ѻ҆́ный): PA1/PA1n carry both, всѧ́къ the velar twin PA1nk.
- **The numerals**: пѧ́ть … де́сѧть, сто̀ and the -десѧть compounds as
  nouns from Polyakov's NUM entries (27 lexemes); два̀, ѻ҆́ба, трѝ, четы́ре
  as pronoun-class lexemes from Alypy (PNdva, PNoba, PNtri, PNcet); the
  hundreds as closed words (`num`).
- **The composite verb class** `A+B` (хотѣ́ти V22t+V12t, спа́ти
  V12p+V22p): A's paradigm, B's present; **Vved** on the cut stem
  (вѣ́мъ, вѣ́ждь); бы́ти's imperfect бѣ́хꙋ; 14 verbs out of the quarantine.
- **любы̀**: the class N5ov* (любвѐ, любо́вь).
- **Closed words by hand**: проти́вꙋ, ра́звѣ, вы́ше, при́снѡ, а҆дѡнаі̀,
  бли́з̾; **the apparatus** `*`, `*↑`, є҆вр, гре́ч.
- **The pronoun clitics** мѧ, тѧ, ми, ти unaccented after their host are
  the phonological word's enclitics, `(pwa host (pn азъ.pron … :clit yes))`.
- **`cargo xtask redraft-hand`**: the overlay's verbatim leaves become
  leaves when the lexicon catches up; the scorer counts a hand set apart.

| Number | before | after |
|---|---|---|
| census verbatim | 14,069: (a) 4,508, (b) 1,141, (c) 8,420 | 5,408: (a) 2,147, (b) 1,117, (c) 2,144 |
| quarantine | 1,442 lines | 1,433 (14 lemmas out) |
| hand overlay | 3,377 leaves, 381 verbatim; rules exclude 0; tagger 75.62% | 3,757 leaves, 6 verbatim (389 redrafted); rules exclude 0 (resolve 459); tagger 74.92% (1,228 of 1,639) |
| Bible treebank | one cell 237,565 (37.6%), sets 2,093, tagger 186,780 (29.6%), closed 178,215, several lexemes 12,858 (2.0%), verbatim 13,418 (2.1%), apparatus 1,017 | one cell 240,672 (38.1%), sets 2,311 (0.4%), tagger 190,494 (30.1%), closed 179,009 (28.3%), several lexemes 12,880 (2.0%), verbatim 5,430 (0.9%), apparatus 1,150; zero mismatches |
| held-out recall | 95.48 / 89.31 / 90.89 / 99.25 | unchanged |
| ids | — | 50 added (27 nouns, 14 verbs, 9 closed), none moved |

## 3.2.0 (2026-09-05) — the clause

The plan is `V3.0-PROMPT.md` Parts 5–6. A minor release: the library is
3.1's; the constraint layer gained a clause rule, the tools a calibration
command.

### Part 5 — the structure the flat tree lacks (2026-09-05)

- **`one-subject`**: a clause (between punctuation, conjunctions and the
  relative pronoun) with one finite transitive verb (the `tran` note)
  and one noun or pronoun that can only be nominative in the verb's
  number: every other noun reading nominative or accusative drops the
  nominative; a verb that cannot be third person has no noun subject.
  Precision 100% on the 3,377-leaf overlay.
- **`cargo xtask tagger-curve`**: the bundled tagger by confidence tenth
  on UD PROIEL dev+test; no threshold applied (see NOTES).

| Number | before | after |
|---|---|---|
| hand overlay: leaves the rules resolve / exclude | np-agree 165, prep-gov 123, subj-verb 33, voc-drop 51 / 0 | + one-subject 28 / 0 |
| tagger on the overlay | 75.62% (1,278 of 1,690) | 75.62% (1,259 of 1,665: the rules took 25 of its leaves) |
| tagger on UD dev+test by share | — | p ≥ 0.9: 13,694 of 14,532 at 89.42%; below 0.9 the tenths are 42–50% |
| Bible treebank | one cell 212,707 (33.7%), tagger 190,196 (30.1%) | one cell 214,958 (34.0%), sets 1,782, tagger 187,945 (29.7%), closed 178,215 (28.2%), several lexemes 12,298 (1.9%), verbatim 35,731 (5.7%); one-subject narrowed 2,545 leaves; zero mismatches |

### Part 6 — the generator's placement (2026-09-05)

Deferred: no placement function exists (the 2.2 record was inexact),
and no consumer generates an enclitic. `prosody::words` and
`Form::print_unit` stay what a generator would call.

## 3.1.0 (2026-09-05) — the lexicon's twins, the print's letters, the gold by register

The plan is `V3.0-PROMPT.md` Parts 3–4, executed through
`V3.0-CONTINUE-PROMPT.md` step 4. A minor release: no id moved (now a
rule of the importer), no letter of Old Church Slavonic changed, the
library's API is 3.0's; the Synodal print writes the initial uk as ѹ and
the prefix от- as ѿ, and every consumer that pasted a string with «оу҆»
re-pastes it (the game did: four strings).

| Number | 3.0.0 | 3.1.0 |
|---|---|---|
| Synodal lexemes: nouns / adjectives / verbs / closed | 13,205 / 8,348 / 8,285 / 1,342 | 13,147 / 8,323 / 8,210 / 1,340 (158 twins absorbed, `data/twins.tsv`) |
| ids changed | — | 0 (`restore_ids`) |
| Bible treebank | one cell 204,769 (32.4%), sets 1,775, tagger 185,183 (29.3%), closed 177,091 (28.0%), several lexemes 13,128 (2.1%; 5.6% without the tagger), verbatim 48,983 (7.8%) | one cell 212,707 (33.7%), sets 1,782 (0.3%; 26.9%), tagger 190,196 (30.1%), closed 178,215 (28.2%), several lexemes 12,298 (1.9%; 5.5%), verbatim **35,731 (5.7%)**; zero mismatches |
| analyzer, tokens with no reading | 71,870 (11.39%) | 61,044 (9.68%) before the ligature |
| hand overlay | 211 verses, 2,097 leaves; tagger 74.8% | 337 verses, 3,377 leaves, 0 findings; rules exclude none; tagger 75.6% |
| held-out recall (UD dev+test) | 95.48 / 89.31 / 90.89 / 99.25 / 98.07 | unchanged |

### Part 3 — the lexicon's twins (2026-09-05)

- **The rule of identity** in the Polyakov importer, after the fit (ids
  never move): two fitted lexemes with the same accent-blind lemma, part
  of speech, gender and animacy whose every shared attested cell prints
  the same primary are one lexeme — the lowest id survives, refitted from
  the union of the entries' forms, provenance joined, `twin: <id>` in the
  note; `data/twins.tsv` maps absorbed → survivor and the hand overlay
  was remapped from it. A pair whose shared cell differs stays two lines
  (дѣло.n / .n.2; господь.n / .n.2 by го́споди against госпо́дїе — the
  nomina sacra stay two lines and `:amb` through the titlo index).

| Number | before | after |
|---|---|---|
| lexemes: nouns / adjectives / verbs | 13,205 / 8,348 / 8,285 | 13,147 / 8,323 / 8,210 (158 absorbed; 134 pairs kept apart) |
| `census homonymy`: identical-paradigm pairs / subset pairs / differing pairs sharing a surface | 43 (307 tokens) / 36 (1,135) / 435 (6,616) | 11 (93: gender or animacy differs) / 12 (638) / 370 (5,310) |
| several-lexeme tokens (`:amb`) | 13,183 | 11,814 |
| Bible treebank | one cell 204,769 (32.4%), tagger 185,183, several lexemes 13,128 (2.1%; 5.6% without the tagger) | one cell 205,403 (32.5%), sets 1,779, tagger 185,910 (29.4%), closed 177,091, several lexemes 11,774 (1.9%; 5.4% without the tagger), verbatim 48,972; zero mismatches |
| held-out recall | unchanged | unchanged |
| hand overlay | 2,097 leaves, rules exclude none, tagger 74.8% | 2,097 leaves, rules exclude none, tagger 74.95% (820 of 1,094) |
| ids changed | — | 0 (158 absorbed ids listed in `data/twins.tsv`) |

### Part 4 — the gold by register: the lexicon findings first (2026-09-05)

- **The word-initial uk is the one letter ѹ.** The pinned Bible never
  writes «оу»; the crate realised it so, and 11,777 + 229 tokens stayed
  verbatim by typography. `realise` writes ѹ, `Form::from_print` folds a
  print's «оу», `comparison_key` folds ѹ, `orthography::id_stem` keeps
  every id's «оу» spelling; the Synodal lexicon files re-imported
  (1,191 lines re-spelled), `write_outcome` re-realises kept variants.

- **The prefix от- is the ligature ѿ**, written into the lexeme's letters
  by the importer with the Bible as arbiter (ѡ҆трѐ, о-тре, keeps «ѡ҆т»);
  **ids never move** as a rule (`restore_ids`: a lemma group's lexemes
  take the existing ids in order, absorbed twins holding their places);
  1 Corinthians 13 lifted (100 leaves, 0 findings; любы̀ and трѝ are
  missing lexemes).

| Number | before | after the uk | after the ligature |
|---|---|---|---|
| Bible treebank | one cell 205,403 (32.5%), sets 1,779, tagger 185,910 (29.4%), closed 177,091 (28.0%), several lexemes 11,774 (1.9%), verbatim 48,972 (7.7%) | one cell 211,123 (33.4%), sets 1,790, tagger 189,408 (30.0%), closed 178,214 (28.2%), several lexemes 12,180 (1.9%), verbatim 38,214 (6.0%) | one cell 212,450 (33.6%), sets 1,791 (0.3%), tagger 190,428 (30.1%), closed 178,213 (28.2%), several lexemes 12,329 (2.0%), verbatim **35,718 (5.7%)**; zero mismatches |
| analyzer, tokens with no reading | 71,870 (11.39%) | 61,044 (9.68%) | — |
| hand overlay | 2,097 leaves, out of reach 54 | 2,097, out of reach 42 (tagger 74.86%) | 2,197 leaves (1 Corinthians 13), rules exclude none, tagger 74.51% |
| held-out recall / ids changed | — | unchanged / 0 | unchanged / 0 |

The gold itself (2026-09-05): Leviticus 1, Isaiah 53, Luke 2, Romans 1,
1 Corinthians 13 hand-lifted; two np-agree defects fixed at the rule
(the relative pronoun after a noun, the preposed converb).

| Number | before | after |
|---|---|---|
| hand overlay | 211 verses, 2,097 leaves | 337 verses, **3,377 leaves** (2,035 narrow a set), 0 findings; the gate asked 4,200 |
| constraint rules on the overlay | exclude none | exclude none (np-agree 165/165 resolved, prep-gov 123/125, subj-verb 33/33, voc-drop 51/70) |
| tagger on the overlay | 74.8% (810 of 1,083) | 75.62% (1,278 of 1,690); nom/acc 126 of 392 wrong cells |
| Bible treebank | one cell 212,450, several lexemes 12,329, verbatim 35,718 | one cell 212,707 (33.7%), sets 1,782, tagger 190,196 (30.1%), closed 178,215, several lexemes 12,298 (1.9%), verbatim 35,731 (5.7%); zero mismatches |

## 3.0.0 (2026-09-05) — the accent-paradigm inventory and weighted evidence

The plan is `V2.2-PROMPT.md` Part 6, then `V3.0-PROMPT.md` Parts 0–2 and
`V3.0-CONTINUE-PROMPT.md`; the analysis behind it is `docs/OPEN-DESIGNS.md`
5. A major release because every Synodal stress column reads differently
(a paradigm of the inventory, 47 named, with a list only where none
fits), `Analysis` and `Reading` gained `weight`, `Letters` and the stress
resolution gained the vowel counts a place resolves against (`Vowels`,
`resolve_in`), and the stems column gained `tail=`. No letter and no
held-out number changed.

| Number | 2.3.0 | 3.0.0 |
|---|---|---|
| stress columns with an exception list: nouns / adjectives / verbs / pronouns | 395 / 243 / 1,202 / 31 = 1,871 | **210 / 213 / 567 / 20 = 1,010** (one number moved counted apart: 71) |
| named paradigms in `lexicon/stress.tsv` (beside the built-in `a`, `b`) | 2 (`c`, `d`) | 47 |
| the -надесѧть numerals' overrides | 44 | 0 (`tail=на́десѧть`) |
| closed lines (adverbs an adjective prints are its `adv` cell) | 1,383 | 1,342; 887 adjectives carry Polyakov's adverb as the cell's evidence |
| Polyakov cells reproduced by the primary: nouns / adjectives / verbs | 94.7% / 94.1% / 91.5% | 94.67% / 93.17% (the arbiter's wide-letter primaries are overrides) / 91.63% |
| primaries the pinned Bible outnumbers | 167 | 135 (115 letter variants, 10 citation cells, 8 stress twins of names and small counts) |
| held-out recall (UD dev+test) | 95.48 / 89.31 / 90.89 / 99.25 / 98.07 | unchanged |
| Bible treebank | one cell 32.4%, sets 0.3%, tagger 29.3%, closed 28.0%, several lexemes 2.1%, verbatim 7.8% | one cell 32.4% (204,769), sets 0.3% (1,775), tagger 29.3% (185,183), closed 28.0% (177,091), several lexemes 2.1% (13,128), verbatim 7.8% (48,983); zero mismatches |
| ids changed | — | 0 |

### Part 6 — the inventory, the weights, the print as arbiter (2026-09-05)

- **The inventory** (`lexicon/stress.tsv`, columns `name spec exemplar
  count`): 31 named paradigms, every one a shape the Part 0.5 census
  showed — nouns `a.gpl` `a.dpl` `a.ipl` `a.gdpl` `a.gipl` `a.gdipl`
  `c.na` (the plural cells that go to the ending), `b.acc` (рꙋка̀ :
  рꙋ́кꙋ), `b.voc` (вра́же), `b.npl`, `b.gen`, `a.dat`, `a.nom`, `a.obl`;
  adjectives `a.short`, `a.shortn`, `b.shortn`, `a.comp`, `a.compn`,
  `a.plL`; verbs `b.pres` (вожꙋ̀ : во́диши, with the first-plural
  imperative), `b.2pl` (веселитѐ: the Bible confirms вмѣнитѐ,
  вселите́сѧ), `b.part` `b.part2` `b.part3` (влекі́й, веды́й), `a.aor3`;
  pronouns `pr.obl`, `pr.kto`, `pr.moj`. Two places and two keys the
  format needed: `F` the word's last vowel; `pres`/`aor`/`impf` and
  `impv` for a whole tense.
- **Two crate fixes the census exposed.** A solid enclitic's vowels
  never carry the stress (возда́стсѧ, блюсти́сѧ: `compose` stops the
  ending's count before the enclitic — 277 reflexive verbs carried
  exception lists that said only this). The fitter compares resolved
  indices, not places (`Evidence::Either` cells had been written as
  exceptions).
- **The fitter** (`fit::stress_column`): every paradigm of the inventory
  is tried bare and with one number moved, the fewest exceptions win,
  ties go to the simpler column and the inventory's order; `cargo xtask
  refit-stress --pos <pos> [--write]` re-fits a file's lines from their
  own forms and keeps a column only when every form prints the same (a
  no-op check on Polyakov's files — 0 changes — and the way the
  inventory reached the pronouns, whose lines Alypy's tables made).
- **Weights.** `variants` carries `×n` (Polyakov's count); `Lexeme::
  variant_weight`, `Analysis::weight`, `Reading::weight`; the analyzer
  ranks exact, then the primary, then weight, then the form's place.
- **The print as arbiter.** `cargo xtask census forms --write` counts
  what the treebank's one-cell leaves print per (lexeme, cell) and what
  its set leaves hide (`data/treebank-forms.tsv`, committed); the
  importer makes the Bible's commonest form the primary where a source's
  stress twins disagree — only among forms the Bible never prints inside
  a set (дре́ва beside древа̀ stays Polyakov's), never in the citation
  cell (lemmas and ids stable: 0 ids changed), never across a letter
  difference. `write_outcome` now keeps another source's lines instead
  of dropping them (Polyakov's pronoun import had dropped Alypy's).
- Re-imported: nouns, adjectives, verbs (Polyakov); pronouns refitted.

Measured (2.3.0 → Part 6 as it stands):

| Number | before | after |
|---|---|---|
| stress columns with an exception list: nouns / adjectives / verbs / pronouns | 395 / 243 / 1,202 / 31 = 1,871 | **219 / 218 / 886 / 21 = 1,344** (in 155 / 193 / 629 / 21 shapes) |
| lines a named paradigm absorbs (nouns / adjectives / verbs / pronouns) | c 24, d 11 | nouns 205 (c 26, a.gdpl 17, b.voc 16, a.ipl 16, d 13, a.dpl 11, c.na 11, a.nom 11, a.gpl 10, b.acc 10, b.npl 10, a.obl 8, b.gen 7, a.gipl 6, a.dat 5, a.gdipl 5) / adjectives 26 / verbs 295 (b.pres 110, b.2pl 57, b.part 30, d 27, a.aor3 18, b.part2 17, c 14, b.part3 12) / pronouns 10 |
| Polyakov cells reproduced by the primary: nouns / adjectives / verbs | 94.7% / 94.1% / 91.5% | 94.67% (43,847/46,315) / 93.33% (87,896/94,181) / 91.63% (116,673/127,332) — adjectives fall where the print's wide-letter forms became primaries the class does not spell; true exceptions 1,628 → 1,570 / 3,712 → 3,601 / 9,057 → 8,952 |
| primaries the Bible outnumbers (`census forms`, one-cell leaves) | 167 of 27,544 disputed pairs | 135 |
| held-out recall (UD dev+test) | 95.48 / 89.31 / 90.89 / 99.25 / 98.07 | unchanged (measured after the first re-import of this part; the later passes changed primaries only, which recall does not see) |
| Bible treebank | one cell 204,650, tagger 185,118, several lexemes 13,107 | one cell 204,726 (32.4%), one lexeme several cells 1,775, tagger 185,169 (29.3%), closed 177,234, several lexemes 12,988 (2.1%), verbatim 49,037; zero mismatches; 389,575 leaves complete; hand overlay re-rendered after `fix-hand-alts` (21 stale `:alt`) |
| ids changed by the re-import | — | 0 (the citation cell keeps the headword's form) |

### V3.0 Part 0 — the censuses (2026-09-05)

The plan is `V3.0-PROMPT.md`. Numbers on record before Part 1 moves
anything (the findings are in NOTES):

| Census | Number |
|---|---|
| `census stress`, exception cells only the candidate place `P` names | verbs 4,035 (886 lists; `P` names every stuck cell of 323 lists, some of 66), adjectives 13 (the short comparative's masculine), nouns 0, pronouns 0 |
| -надесѧть numerals | 7 lines, 44 overrides (all stressed on на́), 7 variants |
| `adv-of=` closed lines | 61: 33 printed by the adjective's `adv` cell already, 12 on the adjective's stem, 6 on its ending, 8 on none of the places, 2 unaccented |
| `census forms`, the outnumbered primaries | 135: 115 letter variants, 10 citation cells, 1 stress twin inside a set, 9 stress twins never inside a set (pronouns, three names) |
| `census homonymy`, pairs of one part of speech sharing a surface | 43 twins (307 tokens), 36 subset pairs (1,135), 435 differing pairs (6,616); гдⷭ҇ь and kin 5,767 tokens through the titlo index (господь.n / .n.2) |
| `score-disambiguation`, the tagger's 274 errors by kind | nom/acc 97, several features 37, gender 31, another feature 30, gen/acc 29, по 14, number 14, another case 10, another pos 9, another lexeme 3 |

The crate gained `Place::Pre` (`P`), `Letters::pre_vowels` and
`stress::resolve_in` so the census could measure it; no paradigm names
it yet.

### V3.0 Part 1 — the residue's places (2026-09-05)

- **The stem place through a derivation** is now a rule of the crate
  (`stress::resolve_in` with `Vowels { base, pre, stem, total }`): the
  lemma's stressed vowel while the stem has it; where a derivation
  removed it (-ова- → -ꙋ-, the iotated -ати presents) the derived stem's
  last vowel, never the extension's (цѣлꙋ́ющїй, пи́шꙋщїй); a lemma stressed
  on its ending keeps the thematic index (твори́мый).
- **`P`**, the last vowel of the stem before the class's extension, in
  the column grammar; twelve named paradigms from the census (`b.pres.ppm`
  и҆зго́нимъ, `b.pres.part` благохва́лѧщїй, `a.ov` for what the rule does
  not reach, `b.inf`, `b.ppf`, …); the most specific block rule wins
  (`part.pres=P` over `part=S`).
- **The -надесѧть numerals** carry `stems=tail=на́десѧть`: a stressed
  solid tail after the inflected first element (`Letters::tail_stress`),
  the compound's one stress; 44 overrides gone.
- Verbs, adjectives and nouns re-imported; the treebank rebuilt.

| Number | before | after |
|---|---|---|
| stress columns with an exception list: nouns / adjectives / verbs / pronouns | 219 / 218 / 886 / 21 = 1,344 | **219 / 211 / 632 / 21 = 1,083** (shapes 155 / 187 / 475 / 21) |
| named paradigms in `lexicon/stress.tsv` | 31 | 43 |
| -надесѧть numerals: overrides / lists | 44 / 7 | 0 / 0 |
| Polyakov cells reproduced by the primary: nouns / adjectives / verbs | 43,847 / 87,896 / 116,673 | 43,847 / 87,938 / 116,673 |
| held-out recall (UD dev+test) | 95.48 / 89.31 / 90.89 / 99.25 / 98.07 | unchanged |
| Bible treebank | one cell 204,726, sets 1,775, tagger 185,169, several lexemes 12,988, verbatim 49,037 | one cell 204,710 (32.4%), sets 1,775, tagger 185,206 (29.3%), closed 177,234, several lexemes 13,027 (2.1%), verbatim 48,977 (7.8%); zero mismatches |
| ids changed | — | 0 |

Step 3 — the `adv-of=` adverbs (2026-09-05): Polyakov's adverb enters
the adjective's `adv` cell as attested evidence (887 adjectives; `b.adv`,
`a.adv` named; 243 `adv=` overrides); the closed re-import deleted 41
lines, 22 `adv-of=` lines stay (another letter); `(adv прѧмый.a)` in the
overlay.

| Number | before | after |
|---|---|---|
| closed lines / `adv-of=` lines | 1,383 / 61 | 1,342 / 22 |
| adjective cells attested / reproduced | 94,181 / 87,938 | 95,063 / 88,572 |
| adjective lists | 211 | 215 (four adverb accents no paradigm names) |
| treebank one cell / closed / several lexemes | 204,710 / 177,234 (28.0%) / 13,027 (2.1%) | 204,769 / 177,091 (28.0%) / 13,128 (2.1%); zero mismatches |

Step 4 — the residue recorded (2026-09-05): one-number moves counted
apart (71: paradigms the notation spells inline), `a.ppp`/`a.pppn`
named, онъ.pron.2's two twins decided by hand (ѻ҆нꙋ̀, ѡ҆́ны), the residue
described by kind in `docs/OPEN-DESIGNS.md` 5.

| Number | before | after |
|---|---|---|
| exception lists: nouns / adjectives / verbs / pronouns | 219 / 215 / 632 / 21 = 1,087 | **210 / 213 / 567 / 20 = 1,010** (shapes 148 / 188 / 459 / 20); one number moved 9 / 2 / 59 / 1 |
| named paradigms | 43 | 47 |
| stress twins the Bible outnumbers, never inside a set | 9 | 7 |

## 2.3.0 (2026-09-05) — the constraint layer of homonymy, the gold, the tagger

The plan is `V2.2-PROMPT.md` Parts 4–5 (Part 6, the accent inventory, is
release 3.0.0); the analysis behind it is `docs/OPEN-DESIGNS.md` 1b. The
library's API is unchanged from 2.2.0; the release adds the crate
`church-slavonic-tagger`, the constraint layer and the hand overlay in
the tools, and the numbers below.

### Part 4 — the constraint layer of homonymy, and the gold (2026-09-05)

- **The gold.** The hand overlay grew from Genesis 1 (31 verses) to 211
  verses: Genesis 2–3, Exodus 1, Proverbs 1 (the pinned Psalter is one
  chapter of psalm groups, so the poetic register is Proverbs), Matthew
  1, John 1 — flat trees, every leaf fully specified from the analyzer's
  readings, drafted by `cargo xtask hand-draft <book> <chapter>` (the
  auto-lifted trees before the constraint layer, every set and
  several-lexeme token listed with its readings) and decided verse by
  verse. 2,095 hand leaves, 1,280 of them a narrowing of a larger set;
  `narrow-hand` at 0 findings; the linter clean; `fix-hand-alts` starts a
  stale alternative index again from the primary. A hand cell the
  lexicon does not print stays verbatim with `:lemma` and `:case` (a
  lexicon finding, listed in NOTES). Genesis 1:21's гадѡ́въ was corrected
  from the accusative to the genitive (ζῴων ἑρπετῶν).
- **The constraints** (`treebank/disambiguate.rs`): rules over a verse's
  flat tree that eliminate and never select, each named on the leaf it
  narrowed (`:by prep-gov :from nom|acc|voc.sg`; a several-lexeme token
  reduced to one lexeme carries `:from-lexemes n`), each leaving
  everything when it would leave nothing. `prep-gov`: a preposition's
  `gov=` frame narrows the nominal after it, and a second nominal when
  the first is an adjective or a participle (never after a pronoun: въ
  не́мже льстѝ нѣ́сть has the genitive of negation). `np-agree`: an
  adjective-like leaf beside a noun leaf, each kept to the cells that
  agree with some cell of the other in case, number and the noun's
  gender. `subj-verb`: a noun whose every reading is nominative beside a
  finite verb: third person, the noun's number. `voc-drop`: the vocative
  leaves a set with other members unless an imperative or an
  interjection stands beside the token. A narrowed leaf's alternative
  index is recomputed for its new first cell from the token, or the leaf
  is left alone. `build-treebank` runs the layer (`CS_NO_DISAMBIGUATE=1`
  turns it off); `check-treebank`'s leaf census accepts a narrowed leaf
  when its `:from` set is the lexicon's and its cells a subset.
- **Scoring** (`cargo xtask score-disambiguation`): every hand verse is
  auto-lifted and constrained, each hand leaf aligned with the auto word
  at its position; precision is the auto set containing the hand's cell,
  resolution the set being that cell alone; by rule; a rule that ever
  excluded a hand cell was fixed (prep-gov's second target after a
  pronoun) — never tuned.

Measured (2.2.0 → 2.3 Part 4):

| Number | before | after |
|---|---|---|
| hand overlay: verses / leaves / narrowings | 31 / 284 / 179 | 211 / 2,095 / 1,280 |
| **precision on the gold** (hand cell inside the auto set) | — | **100%: 0 excluded of 2,095** (1,902 inside, 193 out of the layer's reach — auto `:amb` or verbatim) |
| resolution on the gold (auto set = hand cell) | — | 45.0% (943); by rule: np-agree 98 of 150, prep-gov 55 of 76, subj-verb 28 of 28, voc-drop 33 of 298 (+17 with np-agree) |
| Bible treebank: one cell / one lexeme several cells / closed / several lexemes / verbatim | 23.8 / 34.3 / 28.0 / 6.0 / 7.8 % | **32.4 (204,650) / 26.0 (164,438) / 28.0 / 5.6 (35,562) / 7.8 %**; zero mismatches; 366,993 leaves complete |
| leaves narrowed by rule (whole Bible) | — | voc-drop 57,615, np-agree 41,542, prep-gov 18,082, subj-verb 2,767; 1,931 several-lexeme tokens reduced to one lexeme (prep-gov) |

### Part 5 — the statistical tagger (2026-09-05)

- **The crate** `church-slavonic-tagger` (the library stays
  dependency-free): an averaged perceptron over the (part of speech,
  cell) readings the analyzer returns for a token, features from the
  surface under a manuscript fold (accent-blind, jer-blind: де́нь ~ дьнь,
  сотворѝ ~ сътвори), its suffixes, the neighbouring surfaces and lemmas
  (the abbreviations differ between the recensions — бг҃ъ, б҃ъ — the
  lemmas do not), the readings' part of speech, case, number, person,
  and the previous token's choice; never a lexeme id, so the model
  transfers between the recensions. Greedy left to right; the choice's
  softmax share written on the leaf as `:prob` (`:p` is a verb leaf's
  person); a tie is an abstention. The model is a committed binary
  (`data/models/tagger.bin`, feature hashes and weights, 2.6 MB) with
  its record (`tagger.md`) and the sha256 of the model and of the
  archives it was trained on (`tagger.sha256`); `cargo xtask
  train-tagger` rebuilds it reproducibly (fixed-seed shuffle).
- **Training material**: the gold morphology of UD PROIEL **train** and
  of Syntacticus with the 4,953 sentences UD holds out removed (they are
  the same Codex Marianus); the Bible is never training material. The
  gold follows the overlay's convention where the treebanks' differs:
  a direct object (`obj`) the treebanks tag in the genitive — the
  genitive-accusative, сътворимъ чловѣка `Case=Gen` — is the accusative
  when the readings offer it. Trained only on tokens with several
  readings among which the gold stands.
- **In the treebank** (`treebank/tag.rs`): applied AFTER the constraint
  layer and only where it left several readings — a leaf with a set is
  narrowed to the tagger's cell (`:by voc-drop+tagger :from nom|acc.sg
  :prob 0.98`), a several-lexeme token whose readings the tagger tells
  apart by cell becomes a leaf (`:by tagger :from-lexemes 2`). The
  coverage table reports "Tagger" as its own column, never folded into
  the analysed share; `CS_NO_TAGGER=1` rebuilds without it and gets the
  Part 4 numbers back exactly; the leaf census and the round-trip hold.
  `score-disambiguation` scores the tagger apart, by confidence.
- **A repository finding, fixed.** `.gitignore`'s `treebank/` (the built
  Bible) also ignored `crates/church-slavonic-tools/src/treebank/`: the
  whole treebank module (lifter, nodes, linter, runner, constraints) had
  never been committed, and tags v2.0.0–v2.2.0 do not build from a
  clean clone. The pattern is anchored (`/treebank/`) and the module is
  in this commit.

Measured (2.3 Part 4 → Part 5):

| Number | value |
|---|---|
| training: UD PROIEL train / Syntacticus (held-out sentences removed) | 18,327 sentences, 102,552 annotated tokens / 19,038 sentences, 116,614 tokens; 136,985 examples with several readings; 8 epochs; 220,574 features |
| **OCS, UD dev+test, tokens with several readings** (the overlay's object convention) | **86.86% (12,623 of 14,532)**; the analyzer's first reading 38.85%; by part of speech: verbs 89.60%, pronouns 88.99%, nouns 85.90%, adjectives 74.25%; by set size: 2 readings 91.82%, 3 86.03%, 4 86.95% |
| the same under the treebanks' own convention (genitive-accusative as genitive) | 88.61% (12,876 of 14,531) |
| **hand overlay: the tagger's precision** | **74.72%: 810 right of 1,084 chosen** (262 a wrong cell, 12 a wrong lexeme) of 2,095 hand leaves; at `:prob` ≥ 0.9 77.72% (764 of 983) — the perceptron's shares are not calibrated |
| hand overlay, the whole auto tree: contains the hand cell / resolves it | 84.44% (1,769) / 83.68% (1,753) — Part 4 alone 100% / 45.0% |
| commonest confusions (hand → tagger) | acc.sg → nom.sg 25, nom.sg → acc.sg 19, acc.sg → gen.sg 14, nom.pl → acc.pl 8, long n.sg.acc → nom 8, the neuter singular pronoun → plural 7, loc.sg → dat.sg 6 (по землѝ: the treebanks read по with the dative) |
| Bible treebank: one cell / one lexeme several cells / **tagger** / closed / several lexemes / verbatim | 32.4% (204,650, unchanged) / 0.3% (1,775) / **29.3% (185,118)** / 28.0% / 2.1% (13,107) / 7.8%; 163,668 leaves chosen, 22,598 several-lexeme tokens reduced; zero mismatches; 389,448 leaves complete |
| `CS_NO_TAGGER=1` | one lexeme several cells 164,438, several lexemes 35,562 — Part 4 exactly |

## 2.2.0 (2026-09-05) — the verb's whole two-stem system, the closed lexicon, the phonological word

The plan is `V2.2-PROMPT.md` (Parts 0–3 of six; Parts 4–6 are the 2.3.0
and 3.0.0 releases); the analysis behind it is `docs/OPEN-DESIGNS.md`.
Tag `v2.2.0`. The library's API grew by `Cell::Adv`, `Lexeme::government/
prosody/subcategory`, `grammar::Prosody`, `Form::with_enclitic/print_unit/
print_hosting`, the `prosody` module; nothing was removed. The closed
lexicon lost 1,123 lines to the adjectives' adverb cell. The game
re-tested against 2.2.0 without a content change.

### The close (2026-09-05)

- `docs/DESIGN.md`: the non-present verb cells by type; the closed
  lexicon's columns and the adverb cell; the `(adv …)`, `(pw …)`, `(pwa …)`
  leaves; the fifth stage. `docs/OPEN-DESIGNS.md`: 3 and 4 executed with
  their numbers, 2 completed; 1b and 5 open with their censuses.
  `HANDOFF-PROMPT.md` and the README regenerated from the final `eval`
  and `check-treebank`.
- Version 2.2.0 for `church-slavonic`.

The release in numbers (2.1 → 2.2):

| Number | 2.1 | 2.2 |
|---|---|---|
| held-out recall, UD dev+test: nouns / adjectives / verbs / pronouns / npron | 95.48 / 89.31 / 90.59 / 99.25 / 98.07 | 95.48 / 89.31 / 90.89 / 99.25 / 98.07 |
| Syntacticus verbs | 94.91 | 95.02 |
| Bible treebank: one cell / one lexeme several cells / closed / several lexemes / verbatim | 23.6 / 34.0 / 28.1 / 6.0 / 8.2 | 23.8 / 34.3 / 28.0 / 6.0 / 7.8 |
| phonological words in the treebank: solid / apart | — | 441 / 1,854 |
| closed lines / prepositions with a frame | 2,503 / 0 | 1,383 / 29 |
| OCS verb cells against the type (aorist, imperfect, l-participle) | 88, 144, 90 of 153 | 153, 153, 153 |
| stored stems: OCS / Synodal | 56 / 358 | 56 / 344 |
| Bible coverage (analyzer): one / one lexeme several / several / none | 320,930 / 205,888 / 31,785 / 72,119 | 321,134 / 206,024 / 31,827 / 71,939 |

### Part 0 — the censuses (2026-09-05)

`cargo xtask census <verb-cells --ocs | closed | clitics | homonymy |
stress>` beside `census stems`; the module is `tools/src/census/`.

| Census | Value |
|---|---|
| OCS verb classes: aorist / imperfect / l-participle cells against the Leskien type | aorist 88 agree / 65 disagree, imperfect 144 / 9, l-participle 90 / 63 (of 153 cells each); every disagreement is Kaikki's template (класс II косехъ/кослъ for коснѫхъ/коснѫлъ; velars стрѣжехъ for стрѣгохъ; dentals грѧдехъ; nasals кльнхъ; V:I:C's imperfect понесахъ for несѣахъ) |
| UD variants on OCS verb lines in those blocks | 501 lines: aorist 674 forms, imperfect 266, l-participle 63, past participles 783 |
| closed lines by tag | adv 2,313, advpro 99, conj 36, pr 29, part 18, intj 7, pred 1 (2,503) |
| adverbs an adjective already produces | letters and accent 1,435 (857 printed with the wide ѡ), letters only 67, no adjective 811 |
| prepositions' case frames from the treebank | 23 prepositions, 39,000 tokens; the frames as expected (къ dat 1,458 unambiguous; съ ins 1,563 / gen 203; на acc 1,566 / loc 1,193; ѿ gen 3,707), with the syncretic sets counted apart |
| solid enclitics | 43,063 tokens ending in an enclitic string analyse whole (ordinary endings: -ти, -ми, -сѧ); 524 analyse only as host + enclitic (и҆̀хже 270, ѻ҆́ньже 147, во́ньже 16, Землѧ́же among the singletons; -же 441, -сѧ 28, -ми 18, -си 12); 3,954 neither, dominated by ѹ҆̀бо 766 (a spelling of ꙋ҆́бо) and негѡ́же 63 |
| several-lexeme tokens | 37,647: several parts of speech 19,402, several lexemes of one part of speech 17,569, a closed word beside an inflected form 676; commonest гдⷭ҇ь 3,579, а҆́зъ 1,939, ва́мъ 1,358, мнѣ̀ 1,260 |
| underspecified sets by size | 2 cells 99,341; 3 cells 47,924; 4 cells 14,033; 6 cells 17,285; 8 cells 10,720; 12 cells 5,165; 16 cells 3,078; commonest aor.2|3.sg 14,488, nom|acc.sg 13,521, nom|acc|voc.sg 12,599, gen|acc.sg 8,163 |
| stress columns with an exception list | nouns 395 (of 13,205) in 174 shapes, the twelve commonest absorb 145; adjectives 242 in 199 shapes / 37; verbs 1,202 in 702 shapes / 242; pronouns 31 in 24 / 19 |

### Part 1 — the non-present verb cells by type (2026-09-05)

- **OCS** (`scripts/kaikki-to-classes.py`, `type_cell`): each Leskien type
  declares its aorist, imperfect and l-participle cells outright, and
  Kaikki's majority is not kept where it disagrees — the sigmatic aorist
  on a vowel stem (дѣлахъ, дѣла; любихъ, люби; кꙑпѣхъ; клѧхъ), the -ох-
  aorist on a consonant stem with the palatalised velar before the bare
  е of the second and third person singular (несохъ, несе; рекохъ, рече;
  грѧдохъ, грѧде), class II with -нѫ- first and the root aorist as the
  alternative on a new `13=pal1` stem (двигнѫхъ | двигохъ, двигнѫ |
  движе); the imperfect -ѣа- after a consonant stem (несѣахъ, кльнѣахъ,
  грѧдѣахъ), -аа- after the palatalised velar and the a-types (речаахъ,
  лежаахъ, писаахъ), -ꙗа- on the iotated stem of class IV -ити and the
  jer type (хождаахъ, пьꙗахъ), -ѣа- on -ѣти (кꙑпѣахъ), -а- after a vowel
  stem (дѣлаахъ, вѣроваахъ); the l-participle on the infinitive stem
  (неслъ, реклъ, двигнѫлъ, клѧлъ). `census verb-cells --ocs` (its
  predictor is the same statement in Rust) reads 153/153 agree in all
  three blocks. The exemplar test covers one aorist, imperfect and
  l-participle per type.
- **Synodal** (`scripts/legend-adj-verb-pron.py`): бити's cells whose
  ending opens with a vowel or й are on the ї-stem (бі́й, бі́йте, бїю́щїй
  beside би́хъ, би́лъ, би́ти); the 19 `1=бі`-type lines are gone. A 2.1
  defect went with them: those lines printed their infinitive as вбітѝ.
  The нн/н long-participle lines stay (Polyakov's own variation).

Measured (2.1 → 2.2 Part 1):

| Number | 2.1 | 2.2 Part 1 |
|---|---|---|
| **UD dev+test verb recall** | 90.59% (7,935/8,759) | **90.89% (7,961)** |
| **Syntacticus verb recall** | 94.91% (43,422/45,749) | **95.02% (43,469)** |
| UD dev+test verb misses by block (not guessed) | aorist 280, past active participle short 243, present 197 | aorist 139, past active participle short 145, present 111, imperfect 24 |
| UD variants on OCS verb lines: aorist / imperfect / l-participle | 674 / 266 / 63 (501 lines) | 644 / 266 / 59 (the rest are manuscript spellings: ѣвихъ, бъихъ, вьзꙙхъ, въвръгохъ) |
| OCS verb cells against the type: aorist / imperfect / l-participle | 88 / 144 / 90 agree of 153 | 153 / 153 / 153 |
| Kaikki verb cells reproduced | 74,667 / 78,063 (95.7%) | 73,562 (94.2%); reachable 73,675 — 1,105 cells are Kaikki's template against the type (косехъ, кослъ, кльнхъ, стрѣжехъ, грѧдехъ) and are counted, not kept |
| OCS stored stems | 56 | 56 |
| Synodal stored stems | 358 | 344 (no `1=бі` line) |
| Polyakov verb cells reproduced by the primary | 116,548 / 127,329 | 116,677 / 127,332 |

### Part 2 — the closed-class lexicon: government, prosody, the adverb cell (2026-09-05)

- **Subcategory as class.** A closed lexeme's class column is its
  subcategory (`prep`, `conj`, `part`, `adv`, `advpro`, `intj`, `pred`);
  `lexicon/classes/closed.tsv` names them with their one cell;
  `Lexeme::subcategory()`.
- **Government.** Every preposition line carries `gov=<case>|…`
  (`Lexeme::government()`): the grammar's frames (Alypy's inventory, a
  table in the importer) ordered by the treebank's unambiguous counts
  (`data/prep-frames.tsv`, written by `cargo xtask census closed --write`);
  a case the print attests in at least a twentieth of a preposition's
  unambiguous tokens but the grammar does not know is a note (`gov? dat:336`
  on въ, `gov? acc:9` on при), never a frame. 29 prepositions, every one
  of the Bible's 23 among them. The linter's ten-preposition table is
  deleted; `lint_pp` reads the lexicon by id or by print (the frames of
  every closed lexeme printing the word joined) and leaves a word without
  a frame unchecked.
- **Prosody.** `pros=encl` on же, бо, ли, ꙋ҆́бѡ; `pros=procl` on every
  preposition and on не, ни (`Lexeme::prosody()`, `grammar::Prosody`).
- **The adverb cell.** `Cell::Adv(AdvCell { degree })` under the adjective
  (`adv`, `comp.adv`); the Synodal adjective classes carry `adv=1-о^|1-ѣ`
  (the neuter short nominative's ending with the mark that prints the
  wide ѡ — мꙋ́дрѡ, до́брѡ — beside the short locative до́брѣ) and
  `comp.adv=@short.comp.n.sg.nom` (мꙋдрѣ́е); the treebank leaf `(adv
  мꙋдрый.a [:deg comp])`. A closed adverb line an adjective prints exactly
  is gone (1,123 lines; the adjective's note carries `adv P:<count>`, 936
  adjectives); one the adjective prints with another accent or letter
  stays with `adv-of=<id>` (61); the rest stay (the primary adverbs and
  the compounds, 1,129).

Measured (2.1 → 2.2 Part 2):

| Number | 2.1 | 2.2 Part 2 |
|---|---|---|
| closed lines | 2,503 | 1,380 (prep 29, conj 36, part 18, adv 1,190, advpro 99, intj 7, pred 1) |
| adverbs that are an adjective's cell | — (closed lines) | 1,123 deleted, 61 kept with `adv-of=`, 807 without an adjective |
| prepositions with a frame / of the Bible's | 10 (the linter's table) | 29 / 23 of 23 |
| Bible treebank: analysed one cell / one lexeme several cells / closed / several lexemes / verbatim | 23.6 / 34.0 / 28.1 / 6.0 / 8.2 % | 23.7 (149,943) / 34.1 (215,226) / 27.9 (176,553) / 6.0 (37,660) / 8.2 (51,547) %; zero mismatches, 364,886 leaves complete |
| analysed + closed | 326,664 | 326,496 (−168: adverb tokens that are now a syncretic set of their adjective, до́брѣ = adv | short loc; +39 several lexemes) |
| Bible coverage (analyzer): one / one lexeme several cells / several lexemes / none | 320,930 / 205,888 / 31,785 / 72,119 | 320,930 / 206,027 / 31,824 / 72,143 |
| held-out adjective recall | 89.31% | 89.31% (nouns 95.48, verbs 90.89, pronouns 99.25, npron 98.07) |

The closed-class share fell by the adverbs that became cells, as the
design intends; the gate's substance — no coverage lost, the
several-lexemes share not above 6.0% — holds. The game's 35 tests pass
unchanged.

### Part 3 — the phonological word (2026-09-05)

- **The rule in the crate.** `Form::with_enclitic(enclitic, recension)`
  builds the accentual unit — the host's letters with the enclitic's,
  the host's jer dropped before it in the Synodal print (ихъ + же =
  и҆̀хже; OCS keeps имъже), the number mark skipping the enclitic's
  vowels — and `Form::print_unit` prints it: землѧ̀ + же = землѧ́же, the
  host's final varia an oxia because the unit's last vowel is the
  enclitic's. `Form::print_hosting` is the host's print when the enclitic
  is written apart (Землѧ́ же): the final varia an oxia, nothing else
  touched. The 2.0 `encl=` lexemes are this rule applied at the letters
  stage and print unchanged (the consistency test). `church_slavonic::prosody::words`
  groups a token sequence into phonological words by the lexicon's
  prosody (a proclitic to the next tonic word, an enclitic to the
  previous unit) for a renderer or a generator; second-position
  placement is the generator's call.
- **The treebank.** `(pw host (f же.x.2))` a unit written solid, `(pwa host
  (f же.x.2))` one written apart; the host an analyzed leaf or a closed
  lexeme, every enclitic a closed lexeme with `pros=encl`, rendered
  through the unit rule. The lifter reads a token with no whole reading
  as host + enclitic (the enclitic stripped, the host's final oxia read
  as the standalone varia or its jer restored, one lexeme), and a token
  with no whole reading followed by an enclitic token as a unit written
  apart; the probe renders the token(s) back. The linter and the
  coverage count a unit as its host. Three contractions entered the
  closed lexicon by hand (`H:`): во́нь, на́нь, за́нь (въ/на/за + нь), so that
  во́ньже reads as two lexemes (вонѧ's genitive plural beside the
  contraction) and stays `:amb` rather than lifting as вонѧ.
- **The hand overlay.** Genesis 1:2's «Землѧ́ же» is `(cap (pwa (n землѧ.n
  :case nom :num sg) (f же.x.2)))`; the ceiling row rises by one.

Measured (2.2 Part 2 → Part 3):

| Number | before | after |
|---|---|---|
| tokens ending in an enclitic that analyse only as host + enclitic (census) | 524 | 441 lifted as `(pw …)` (онъ.pron 417 — и҆̀хже, ѻ҆́ньже; closed hosts 17 — на́ньже, за́ньже); the rest read as several lexemes or none |
| hosts with a final oxia before an enclitic written apart (Bible print) | 2,538 (Рече́ же 289, Є҆гда́ же 138, ты́ же 110, Ѻ҆ни́ же 81) | 1,854 lifted as `(pwa …)` (verbs 693, closed hosts 436, pronouns 419, nouns 291); the rest read as several lexemes |
| Bible treebank: analysed one cell / one lexeme several cells / closed / several lexemes / verbatim | 23.7 / 34.1 / 27.9 / 6.0 / 8.2 % | 23.8 (150,319) / 34.3 (216,689) / 28.0 (177,210) / 6.0 (37,663) / 7.8 (49,048) %; zero mismatches; 366,724 leaves complete |
| Genesis 1 hand overlay | 283 analysed, Землѧ́ verbatim | 284 analysed, Землѧ́ же a unit |

## 2.1.0 (2026-09-05) — present stems by derivation, syncretism by underspecification

The plan is `V2.1-PROMPT.md` (executed; its postscript lists the
departures); the analysis behind it is `docs/OPEN-DESIGNS.md`. Tag
`v2.1.0`. The library's API grew by `Lexicon::readings`, `Reading`,
`cell::CellSet`, `Cell::case/gender/person` and the `jer` derivation;
nothing was removed. The game (`~/Desktop/code/vertograd`) re-tested
against 2.1.0 without a content change.

### Part 3 — the close (2026-09-05)

- `docs/DESIGN.md`: the two-stem verb model as class-level derivations
  and the Leskien types; the spelling rule after a husher; readings,
  the underspecified cell and the leaf grammar with disjunctions and
  `:cell`; the four-way Bible coverage; two standing rules (syncretism as
  the set, homonymy as `:amb`; a derivable stem is never stored).
  `docs/OPEN-DESIGNS.md`: 2 and 1a marked executed with their numbers;
  1b, 3, 4, 5 open, 1b's census in hand. `HANDOFF-PROMPT.md` and the
  README regenerated from the final `eval` and `check-treebank`.
- Version 2.1.0 for `church-slavonic`.

The release in numbers (2.0 → 2.1):

| Number | 2.0 | 2.1 |
|---|---|---|
| held-out recall, UD dev+test: nouns / adjectives / verbs / pronouns / npron | 94.87 / 89.35 / 85.79 / 99.25 / 97.84 | 95.48 / 89.31 / 90.59 / 99.25 / 98.07 |
| Syntacticus: nouns / adjectives / verbs / pronouns / npron | 95.20 / 95.18 / 93.68 / 99.20 / 95.90 | 95.33 / 95.17 / 94.91 / 99.20 / 96.18 |
| Bible treebank: one cell / one lexeme several cells / closed / several lexemes / verbatim | 23.4 / — / 28.1 / 40.2 / 8.1 | 23.6 / 34.0 / 28.1 / 6.0 / 8.2 |
| stored present stems: OCS / Synodal | 1,442 / 636 | 56 / 358 |
| guessed present cells: OCS / Synodal verbs | 22.72 / 46.46 | 78.99 / 46.51 |
| Kaikki cells reproduced: nouns / adjectives / verbs | 99.1 / 97.8 / 94.5 | 98.6 / 97.8 / 95.7 |
| OCS verb classes | 55 | 27 |

### Part 0 — the census (2026-09-05)

- `cargo xtask census stems --pos verb [--ocs]` classifies every stored
  numbered stem by its relation to the lemma's infinitive stem; `cargo
  xtask eval --guess verbs [--ocs]` is the leave-one-out guessed-present
  number (each verb hidden, its own vote left out of the ending map,
  every present, imperative and present-participle cell compared).

Measured:

| Census | Value |
|---|---|
| OCS verb lines with a stored stem | 1,442 of 2,456: stem 2 `theme` 323, `iot` 36, `ov` 11, `artefact` 491 (a whole present form as the stem), `suppletive` 581 (mostly the seeding's short prefixes: блазнити 2=бла) |
| Synodal verb lines with a stored stem | 636 of 8,279: participle stems read off attested forms with an empty ending (`9=алчꙋщꙋ` 233, `12=…нн` 109, `7=…ш` 93), 100 genuine present suppletions (взѧ́ти 2=возм), 97 stem-1 corrections (би́ти 1=бі) |
| guessed present, OCS verbs (leave-one-out) | class 37.91%, cells 22.72% (68,531/301,680) |
| guessed present, Synodal verbs | class 46.67%, cells 46.46% (1,027,052/2,210,493) |

### Part 1 — present stems by derivation (2026-09-05)

- **OCS verb classes by Leskien** (`scripts/kaikki-to-classes.py`): a class
  is (infinitive type, present type) and its stems column is derivations
  — `V:IV:i` (любити: `2=iot`, first person on 2, the rest on 1 with -и-,
  the imperfect on 2, the past passive `8=ext:ен:iot`), `V:IV:ě`,
  `V:IV:a` (лежати), `V:III:j` (писати: the whole present on `2=iot`),
  `V:III:aje` (дѣлати), `V:III:ja` (таꙗти), `V:III:ov` (`2=ov`),
  `V:III:jer` (пити: `2=jer`, the tense jer before j), `V:I:C` (нести),
  `V:I:к`/`V:I:г` (рещи: `2=ext:к`, `3=pal1:ext:к`, `4=pal2:ext:к`),
  `V:I:т`/`V:I:д`/`V:I:з` (грѧсти: the dental hidden by -сти),
  `V:I:ьн`/`V:I:ьм` (клѧти: `2=ext:ьн:cut`), `V:I:a` (ковати), `V:II`
  (двигнѫти). Each Kaikki entry is placed by predicting its attested
  first and third person singular from the derived stems; every present
  cell reads its ending against the stem the type declares, so members
  that iotate and members that do not vote the same ending. 27 classes
  (55 before), 12 of them residue classes (`V:res:<ending>`) for the
  suppletive entries, which keep their stem on the line and are never
  offered to a lexeme the seeding did not place.
- **The spelling rule after a husher (OCS).** ѭ/ѥ/ѩ/ꙗ are written
  ѫ/е/ѧ/а after ж ч ш щ ц and жд, at the ending and inside a derivation
  (пишѫ, пишетъ, рождѫ, хождаахъ beside люблѭ, глаголѥтъ) — the rule that
  lets one class name `2-ѭ` once. Kaikki's own forms agree (thousands to
  a few hundred; the exceptions are variants now). `iotate` is vacuous
  on a stem already palatal (дъждѫ); `ov` gives -ю- after -ева- and after
  a vowel (воюю, оу҆треню́ю), -ꙋ- otherwise; new derivation `jer` (и → ь,
  ы → ъ).
- **A 2.0 defect found by the new exemplar test**: participle blocks
  delegated to the *Synodal* adjective table for every recension, so no
  OCS past participle was ever produced; `table_of(Adjective, recension)`.
- **UD import**: the stem read off the attested present (`present_stem`)
  is a fallback, tried only when no class derivation reproduces the
  attested present, kept only when it reproduces more, and never a stem
  as long as a form; the tie-break no longer favours the table's first
  class for a lexeme the seeding did not place (it favoured `V:I:C`),
  the verb table is written in order of class size so a tie goes to the
  commonest class of the lemma's shape, and a residue class is out of the
  running. Provenance: an import replaces its own earlier `K:`/`U:`/`P:`
  tokens instead of accumulating them.
- **Synodal** (`scripts/legend-adj-verb-pron.py`): the participle-stem
  inference skips cells with an empty ending and never votes where an
  alternative of the class already produces the form; class-level
  derivations for what the census showed stored: the archaic past active
  participle on the soft stem (`14=ext:ьш:iot` V21p и҆зба́вльшїй, `ext:ьш`
  V21n, `ext:ш:iot` V21t вмѣ́щшїй, `ext:ш` V21s), the -нꙋ-less past active
  participle of V13k/V13t (`14=ext:ш`, воздви́гшїй, воскре́сшїй, nominative
  `1-ъ`), бити's ї-stems (`9=ext:ѧ:iota`, imperfect on 2, `14=ext:ен:iota`
  бїе́нъ, `15` its long form). Re-imported: polyakov, alypy, ruwiktionary
  verbs; kaikki and ud for every OCS part of speech (the husher rule
  touches nouns and adjectives too).

Measured (gate in bold; before → after):

| Number | 2.0 | 2.1 Part 1 |
|---|---|---|
| **OCS verb lines with a stored stem** | 1,442 (of 2,456) | **56** (of 2,455): 53 suppletive (възьмати 2=въземл, бьрати 2=бер, пѣти 2=по, …), 1 artefact (роути, Kaikki's ров), клеветати (Kaikki's клевещѣтъ), дъждити |
| OCS verb classes | 55 | 27 (12 residue) |
| **Kaikki verb cells reproduced** | 73,791 / 78,063 (94.5%) | **74,667 (95.7%)**; reachable 74,823 |
| Kaikki noun / adjective cells reproduced | 99.1% / 97.8% | 98.6% (39,139/39,678; Kaikki's ѩ after ц/щ) / 97.8% (38,225/39,096) |
| **UD dev+test recall: verbs** | 85.79% (7,514/8,759) | **90.59% (7,935)** |
| UD dev+test recall: nouns / adjectives / pronouns / npron | 94.87 / 89.35 / 99.25 / 97.84 | 95.48 / 89.31 / 99.25 / 98.07 |
| **Syntacticus recall: verbs** | 93.68% | **94.91% (43,422/45,749)**; nouns 95.33, adjectives 95.17, pronouns 99.20, npron 96.18 |
| **guessed present, OCS verbs (leave-one-out)** | class 37.91%, cells 22.72% | **class 79.51%, cells 78.99% (238,519/301,965)** |
| Synodal verb lines with a stored stem | 636 (of 8,279) | 358 (of 8,284): stem 12 `…нн`/`…н` spelling 105, stem 2 suppletive 94 (взѧ́ти 2=возм, зва́ти 2=зов, и҆ма́ти 2=емл), stem 1 90 (би́ти 1=бі: Polyakov's ї before a vowel in the imperfect and passive), stem 7 82, … |
| Polyakov verb cells reproduced by the primary | 91.5% | 91.5% (116,548/127,329); reachable 118,254 |
| guessed present, Synodal verbs | class 46.67%, cells 46.46% | class 46.72%, cells 46.51% |
| Bible coverage, Synodal (one reading / several / none) | 321,046 / 237,567 / 72,311 | 321,132 / 237,673 / 72,119 |
| Bible treebank: analyzed / closed / ambiguous / verbatim | 23.4 / 28.1 / 40.2 / 8.1 % | 23.4 / 28.1 / 40.2 / 8.2 % (51,556), zero mismatches |
| OCS exemplar test | — | любити → люблѭ/любиши/любѧтъ/люблꙗахъ/любленъ, просити → прошѫ/прошаахъ, ходити → хождѫ/хожденъ, писати → пишѫ/пишеши/пиши, глаголати → глаголѭ/глаголѥтъ, дѣлати → дѣлаѭ, вѣровати → вѣроуѭ, нести → несѫ/несеши, рещи → рекѫ/речеши/реци/рекꙑ, мощи → могѫ/можетъ/моѕи, грѧсти → грѧдѫ, двигнѫти → двигнѫ/двигнеши, пити → пьѭ |

The two OCS gates the prompt set (stored stems ≤ 60; Kaikki cells ≥
94.5%; UD verbs ≥ 85.79%; Syntacticus ≥ 93.68%; the guessed number
rising) are met. рьци (the jer grade of the root in the imperative) is a
lexeme override, not a derivation.

### Part 2 — syncretism by underspecification (2026-09-05)

- **Readings.** `Lexicon::readings(surface) -> Vec<Reading>` beside
  `analyze`: the analyses grouped by (lexeme, print) — one lexeme and every
  cell whose form prints the surface, each with its alternative index;
  `Reading::cell_set()`.
- **The underspecified cell.** `cell::CellSet`: a sorted, deduplicated set
  of cells of one part of speech; `name()` factors the shared components
  and writes the disjunction where they differ (`nom|acc|voc.sg`,
  `long.pos.m|n.sg.gen`, `aor.2|3.sg`, `aor|impv.2|3.sg`), lists the
  cells in cell order where the set is not such a product
  (`nom.pl|gen.sg|acc.pl`); `parse(pos, text)` is its inverse (a factored
  name that would read as a list of whole cells is written listed:
  `3.m.sg.gen|3.m.sg.dat`). `Cell::case/gender/person` accessors.
- **The treebank leaf** carries the set: `Node::Lex { id, cells, alt }`;
  a product set as disjunctive features (`(n свѣтъ.n :case nom|acc :num
  sg)`, `(adj мꙋдрый.a :case gen :num sg :g m|n :series long)`, `(v рещи.v
  :t aor :p 2|3 :num sg)`), any other set as `:cell` with the set's name
  (`(n жена.n :cell nom.pl|gen.sg|acc.pl)`); `:alt` is the first cell's,
  rendering goes through the first cell. The reader expands a product
  and rejects a member the part of speech has not.
- **The lifter**: a token whose exact readings are one lexeme is
  *analysed* — one cell, or the set (`TokenFate::Underspecified`); a
  titlo-written token groups the expansions of one lexeme under one row
  (дх҃ъ is nom.sg|gen.pl|acc.pl of дꙋхъ: the abbreviation erases the
  accent that tells дꙋ́хъ from дꙋ̑хъ); several lexemes stay verbatim with
  `:amb n` (homonymy: дꙋ́хъ the noun, дꙋ́хъ дꙋти's aorist).
- **The linter** treats a disjunctive feature as satisfied when any
  member agrees (np agreement, the subject's number and person, a
  preposition's case); it never narrows a set.
- **`check-treebank`** asserts, over every auto-lifted leaf, that the
  leaf names every cell of its lexeme that prints the token (through the
  titlo index for an abbreviated token) — 364,073 leaves, none
  incomplete — and splits the table into analysed (one cell) / one
  lexeme, several cells / closed / several lexemes / verbatim.
  **`narrow-hand`** reports, for each Genesis 1 hand leaf, whether the
  hand's cell is inside the lexicon's set. `eval` reports Bible coverage
  four ways.

Measured (2.0 → 2.1 Part 2):

| Number | 2.0 | 2.1 Part 2 |
|---|---|---|
| Bible treebank: analysed (one cell) | 23.4% (147,670) | **23.6% (149,269)** |
| Bible treebank: one lexeme, several cells | — (inside "ambiguous") | **34.0% (215,087)** |
| Bible treebank: closed-class | 28.1% | 28.1% (177,395) |
| Bible treebank: several lexemes (`:amb`) | 40.2% "ambiguous" (254,308) | **6.0% (37,621)** |
| Bible treebank: verbatim | 8.1% | 8.2% (51,557) |
| round trip | zero mismatches | zero mismatches over 34,470 verses; 364,073 auto-lifted leaves complete |
| Bible coverage (analyzer, exact): one reading / one lexeme several cells / several lexemes / none | 50.90% / — / 37.67% / 11.43% | 50.90% (321,132) / 32.63% (205,888) / 5.04% (31,785) / 11.43% (72,119) |
| Genesis 1 hand overlay (`narrow-hand`) | — | 283 leaves, 179 narrow a larger set, 0 outside the lexicon's set |
| Genesis 1 auto-lift, commonest noun sets | — | gen|acc.sg 633, nom|acc.sg 552, nom|acc|voc.sg 493 |

The honest size of the homonymy problem (`docs/OPEN-DESIGNS.md` 1b) is
6.0% of the Bible's tokens; the 40.2% "ambiguous" of 2.0 was five parts
syncretism to one part homonymy.

## 2.0.0 (2026-09-05) — the lexicon-first rewrite

Executed from V2-PROMPT.md; the design is docs/DESIGN.md. Each part
records the three eval numbers (held-out recall, Bible coverage,
guesser accuracy) — the gates that replace the 1.x 100.00%/0 gate.

### Part 0 — freeze, scaffold, baselines (2026-09-04)

- Tag `v1.2.0-final` at the freeze point. The 1.x crates moved to
  `legacy/` as `church-slavonic-legacy`, `church-slavonic-core-legacy`,
  `church-slavonic-syntax-legacy`, `extractor-legacy`, `xtask-legacy`
  (`cargo xtask-legacy accuracy | check-treebank` are the baseline
  instruments until Part 5). All legacy suites pass after the rename.
- New crates: `church-slavonic` (2.0.0-dev; `grammar`, `cell` — typed
  cells with one canonical name each, `form` — letters + stress +
  number mark with `print`/`key`/`from_print`, `orthography` ported
  unchanged except that `ї` is kept as typed and placed by
  `Form::print`, `lexicon` — the tsv parser and the embedded files,
  and stubs for `paradigm`, `stress`, `inflect`, `analyze`, `guess`)
  and `church-slavonic-tools` (`cargo xtask`: the source parsers ported
  as `sources::{polyakov,alypy,kaikki,ruwiktionary,ud}` — the UD/PROIEL
  loader onto typed cells — and the treebank ported as
  `treebank::{sexpr,node,lint,bible,closed,titlo,lift,runner}` with the
  leaf grammar accepting a lexeme id; the lemma-keyed leaves render
  through the legacy crate until Part 2).
- `docs/DESIGN.md` written. `cargo xtask eval` prints the three numbers
  as `n/a`; `--legacy` runs the legacy harness.
- `cargo xtask check-treebank` (ported) re-renders all 34,470 verses at
  zero mismatches.

Baselines (1.2.0, the numbers every later part is gated against):

| Number | 1.2 baseline |
|---|---|
| Held-out recall, UD PROIEL dev+test: nouns / adjectives / verbs / pronouns / npron | 92.04 / 83.82 / 85.58 / 99.25 / 93.21 % |
| Bible treebank (631,946 tokens): analyzed / closed / ambiguous / verbatim | 21.5 / 27.1 / 31.0 / 20.2 % |
| Bible treebank, analyzed by leaf kind: n / v / pers / pn / adj / lp / part / refl | 57,986 / 49,998 / 13,308 / 9,383 / 2,035 / 1,864 / 804 / 528 tokens |
| Bible treebank, noun share of all tokens (the Part 2 gate) | 9.18 % (57,986) |
| Guesser accuracy | not measured in 1.x |

### Part 1 — Synodal nouns (2026-09-04)

The design proved on the richest slice. `crates/church-slavonic/lexicon/
syn/nouns.tsv`: 13,092 lexemes, one line each (1.09 MB), imported from
Polyakov's S entries by `cargo xtask import polyakov --pos noun --write`.

- **Class tables** (`lexicon/classes/noun.tsv`, 49 classes): seeded from
  Polyakov's paradigm legend by `scripts/polyakov-legend-to-classes.py`
  (stem derivations `base`/`drop`/`insert`/`pal1`/`pal2`/`ext`/`cut`
  named per class; the twin classes `N1g`/`N1x`/`N2g`/`N1c` and the
  indeclinable `0` added), then corrected by measurement: the number
  mark per cell from the attested primaries (`--fix-marks`: 14 cells the
  legend left plain, e.g. every `ins.pl` of the masculines and the
  `nom.pl` of the neuters), the primary alternative per cell from the
  alternative-preference census, and the print's second series of
  plural endings (-ахъ/-амъ/-ами, -ове, -ови, the zero genitive plural
  on the full stem of the fleeting classes: ѻ҆тє́цъ) added as
  alternatives across the classes.
- **Stress paradigms** (`lexicon/stress.tsv`): measured, not assumed.
  Census over the 13,092 fitted lexemes: `a` 12,082, `b` 619, `a{pl=E}`
  20, `b{voc.sg=S}` 16, `a{ins.pl=E}` 18, `a{gen.pl=E;dat.pl=E}` 18;
  172 distinct specs in all, none other above 11. Named: `a`, `b`
  (built in), `c` = `S;pl=E`, `d` = `E;pl=S`; the residue stays inline.
  The 1.x verdict (v0.8: mobile-stress tokens rejected, commonest shape
  on 15 rows) holds under the class prior too — Synodal noun stress is
  fixed or ending-stressed in all but ~1% of Polyakov's lexemes.
- **Importer** (`church-slavonic-tools::import`): a source is compared
  under what it can encode (`translit_equal`: і for the print's ї, я
  for ѧ/ꙗ, ѡт for ѿ — the single largest lever, 12,000 cells); a form
  tagged for several cells (`gen/acc`) never outranks one tagged for the
  cell alone and any alternative satisfies it; the coded class competes
  with its fleeting-vowel and velar twins (Polyakov codes ѻ҆се́лъ N1t)
  and numbered stems are read off the attested forms (`stems=1=льв`);
  stored forms are canonicalised to the print's typography. Noise
  skipped and counted: titlo spellings 1,549, abbreviation marks on a
  consonant 870, unaccented forms 128, two stress marks, unanalysed
  398. Quarantined 585 entries with a reason (no analysed forms 429,
  attested nominative ≠ lemma 112, class does not produce the lemma 33,
  adjectival classes 7, no class 3, unknown class 1).
- **Library**: `Form::print` gained the `ї` rule and `ѿ`; the class
  engine (`paradigm`), `stress`, `inflect` (`Lexeme::inflect`, `forms`,
  `paradigm`), `guess` (`Lexicon::guess`), the consistency test
  (`tests/lexicon_consistency.rs`: every override and variant reproduced,
  every nominative the lemma) and the tools' reproduction floor test
  (`tests/polyakov_nouns.rs`).

Measured (the Part 1 gate was ≥ 99% of Polyakov's noun forms reproduced
with overrides on < 5% of entries):

| Number | Value |
|---|---|
| Polyakov noun cells attested (after the noise filters) | 45,876 |
| reproduced as the primary form (`inflect`) | 43,521 (94.87%) |
| reachable through any alternative or variant (`forms`, the analyzer's view) | 44,412 (96.81%) |
| true exceptions (no class alternative fits): cells / lexemes | 1,464 / 964 (7.36% of lexemes) |
| alternative preferences (an override naming a non-primary alternative) | 973 cells |
| lexemes with any override | 1,650 (12.60%) |
| guesser, leave-one-out over 13,028 lexemes: class / cells | 94.18% / 93.52% |
| `ра́бъ` | one line: `N1t`, `b{voc.sg=S}`, no override, gen.pl variant |

**The gate is not met and the design is not refuted.** The residue, read
in samples (NOTES.md, Part 1): the source's own spelling variants (ль/л
before н and ц: нача́лства, слꙋжи́телницꙋ; ѵ/и/в: наѵи́нъ : навѵ́номъ;
ѣ/е: премѣне́ніе : премене́ній), per-lexeme alternative preferences (the
second plural series, the gen-shaped accusative), the suppletive and
mobile handful (ѻ҆́ко : ѻ҆чеса̀, сло́во : словесѐ, де́нь : днѝ) — lexical
facts, which is what the columns are for. Against 1.x: the same source
took 8,313 rows of stored cells plus five fact-cell mechanisms to reach
its by-construction 100%; 2.0 states the same knowledge as 13,092 lines
of class + stress + provenance, and reads back 94.9% of the cells from
two named paradigms. Recorded as a shortfall against the written gate;
Part 2 proceeds.

### Part 2 — the analyzer, the eval, the treebank (2026-09-04)

- **Analyzer** (`church_slavonic::analyze`): `Lexicon::analyze(surface)
  -> Vec<Analysis { lexeme, cell, alt, exact, print }>` over an index of
  every lexeme × every cell × every alternative and variant, keyed by
  the accent-blind comparison key, built lazily on first use (373,036
  entries for the Synodal nouns in 3.4 s, release), ranked exact print
  first, then the primary form before other alternatives. Ambiguity is
  returned, never resolved.
- **Treebank** (`cargo xtask build-treebank`, 6.0 s): stored trees are
  re-lifted IN PLACE through the 2.0 lexicon — a verbatim leaf and a
  1.x noun leaf become an id leaf `(n землѧ.n :case acc :num sg [:alt n])`
  when exactly one EXACT reading exists and the leaf renders the token
  back byte-for-byte, an ambiguous 1.x leaf keeps its count (its other
  readings may be parts of speech the lexicon does not hold yet),
  everything else stays and renders through the legacy crate until
  Part 3. 55,453 leaves changed; zero mismatches over 34,470 verses.
  The Genesis 1 hand overlay points its 131 noun leaves at ids
  (`fix-hand-alts` chose the `:alt` of гадѡ́въ); two «є҆́же» that the 1.x
  overlay had rendered through the vocative of є҆́жъ — a false analysis
  the round-trip could not see — are the relative pronoun now.
- **Eval** (`cargo xtask eval`): Bible coverage through the analyzer
  and the guesser number; held-out recall waits for the OCS lexicon
  (Part 4).

Measured (gate: noun Bible coverage ≥ the 1.2 treebank's 9.18%,
`build-treebank` < 10 s, zero mismatches, the three numbers for nouns):

| Number | Value | 1.2 |
|---|---|---|
| Bible tokens with one exact noun reading / several / none | 76,030 (12.05%) / 76,962 (12.20%) / 477,932 | — |
| treebank: analyzed / by lexeme id / closed / ambiguous / verbatim | 24.1% / 11.3% (71,587) / 27.1% / 34.5% / 14.1% | 21.5% / — / 27.1% / 31.0% / 20.2% |
| noun leaves by id against the 1.2 noun share | 71,587 | 57,986 (9.18%) |
| `build-treebank` / index build | 6.0 s / 3.4 s | ~180 s |
| held-out recall | n/a until Part 4 | 92.04% nouns |
| guesser, Synodal nouns: class / cells | 94.18% / 93.52% | — |

The gate is met. The ambiguous share rose (31.0% → 34.5%) because the
2.0 lexicon holds every Polyakov noun, homographs included, where 1.x
indexed only lemmas with an irregular cell; a token whose noun readings
are several is recorded, never guessed.

### Part 3 — adjectives, verbs, participles, pronouns, closed classes (2026-09-05)

- **Adjectives** (`classes/adj.tsv`, 16 classes seeded from Polyakov's
  legend by `scripts/legend-adj-verb-pron.py`): the short and long series
  and the comparative are blocks of ONE lexeme; the possessives' nominal
  cells are a class (A2t, A2j); the -ск- adjectives their own velar twin
  (A1sk: -стїи, not -цыи); measured alternatives and number marks per
  (class, cell) by census and `--fix-marks`. 8,344 lexemes, 94.05%
  primary / 96.07% reachable over 94,138 Polyakov cells.
- **Verbs** (`classes/verb.tsv`, 49 classes): stems 1 infinitive, 2
  present, 3 palatalised imperative, 5–8 the four participle stems
  declined as adjective classes by delegation (`5~A1s`), 9 the bare
  present participle, 11 the past active short stem, 12 the long past
  passive; the athematic бы́ти (present, future бꙋ́дꙋ, imperfect бѧ́хъ,
  the two aorists), да́ти, ꙗ҆́сти, вѣ́дѣти, и҆мѣ́ти as literal-cell classes;
  reflexive verbs carry `stems=encl=сѧ` and the class writes the enclitic
  solid, the jer before it dropped. 8,239 lexemes, 91.4% primary over
  125,932 cells.
- **Pronouns** (`classes/pronoun.tsv`, 21 classes; 68 lexemes): the
  personal matrix (азъ/мы/ты/вы with per-lexeme stems, the third person
  as a literal class on the empty stem with its н- and clitic
  alternatives), the reflexive, the relative и҆́же (the third person's
  obliques with its own nominatives, `encl=же`), the pronominal
  adjectives (PA1…, the fleeting PA1*, the velar PA1tk for такі́й), the
  nominal declension PN/PNk for тако́въ, толи́къ; кто̀/что̀ and their
  compounds on stems named per lexeme, `encl=же|либо|ждо` for the
  compounds. Imported from the 1.x arbitrated tables (Alypy §47/§48 with
  the witnesses folded in) through the same fit as every source, with
  `P:` provenance where Polyakov lists the lemma; the temporary importer
  and the legacy dependency were then deleted.
- **Closed classes** (`syn/closed.tsv`, 2,503 lexemes): Polyakov's ADV,
  ADVPRO, CONJ, PR, PART, INTJ, PRED entries as one-cell lexemes, spelling
  variants as variants.
- **Form** gained three flags the print's own choices need to round-trip:
  `mark_skip` (the number mark stays off a solid enclitic: є҆гѡ́же,
  тѣ̑мже), `varia` (и҆̀хъ the accusative against и҆́хъ the genitive, ꙗ҆̀же)
  and `kamora` (своѧ̑ beside свѡѧ̀); `from_print` folds an initial ѻ/є
  so ids are bare letters (755 noun ids changed); the attested citation
  form replaces a source headword that spells it otherwise (тьма̀ →
  тма̀, 216 lexemes, the headword kept in `note`).
- **Cross-checking sources** (`cargo xtask import alypy|ruwiktionary|
  witnesses --pos <pos>`): every source cell is reproduced, reachable,
  added as a variant with the source's token, or quarantined with a
  reason; a Polyakov re-import keeps those variants. Alypy's `-ѣ` dual
  alternative was mis-expanded by the 1.x loader (пи́шеѣ for пи́шетѣ) and
  is fixed.
- **Treebank**: leaves for every part of speech carry a lexeme id and a
  cell (`(adj мꙋдрый.a :case nom :num sg :g m :series long)`, `(v рещи.v
  :t aor :p 3 :num sg)`, `(part …)`, `(lp …)`, `(pn азъ.pron :p 1 :num sg
  :case dat :clit yes)`, `(f и.x)`); the treebank is lifted from the
  print every time (22 s, index 14 s built in parallel); a titlo token
  lifts through a small index of the titlo rows' paradigms; closed-class
  homographs (Polyakov's и҆ as conjunction and particle) are one function
  word. The Genesis 1 hand overlay's 1.x leaves were converted to ids
  (33 left verbatim with their old features as notes: бы́сть before the
  бы́ти class existed, forms the sources do not attest).
- **Analyzer**: `Lexeme::all_forms` derives stems and the stress
  paradigm once per lexeme; the index is built in parallel chunks
  (7.84 M entries, 14 s, was 95 s).

Measured (gate: every 1.2 source reproduced/variant/quarantined with
counts; held-out per POS — n/a until Part 4; Bible coverage ≥ 48.6%
mechanical with verbatim ≤ 20.2%; zero mismatches):

| Number | Value | 1.2 |
|---|---|---|
| Polyakov cells reproduced / reachable: nouns | 43,842 / — of 46,315 (94.7%) | — |
| adjectives | 88,540 / 90,442 of 94,138 (94.05% / 96.07%) | — |
| verbs | 115,144 of 125,932 (91.4%) | — |
| Alypy cells reproduced / reachable / variants added / quarantined | nouns 380/58/79/14; adjectives 355/20/25/63; verbs 183/51/132/12; pronouns 274/86/46/0 | 100.00% / 0 by construction |
| ru.wiktionary | nouns 442/82/149/42; verbs 38/11/17/0 | 100.00% / 0 |
| witnesses | nouns 0/1/0/2; pronouns 21/0/2/0 | 100.00% / 0 |
| treebank: analyzed / closed / ambiguous / verbatim | 23.4% / 28.1% / 39.1% / 9.3% (mechanical 51.5%) | 21.5% / 27.1% / 31.0% / 20.2% (48.6%) |
| `build-treebank` / index build | 22 s / 14 s | ~180 s |
| Bible tokens with one exact reading / several / none (all POS) | 50.86% / 36.59% / 12.55% | — |
| guesser, Synodal nouns: class / cells | 93.94% / 93.29% | — |

The gate is met on the treebank and the source accounting; the noun
gate of Part 1 (99% / 5%) stays unmet as recorded. `data/witnesses.tsv`
stays until Part 5: the legacy baseline instrument reads it.

### Part 4 — Old Church Slavonic (2026-09-05)

- **OCS class tables** (`classes/ocs/{noun,adj,verb,pronoun}.tsv`),
  seeded from Kaikki's own paradigm tables by
  `scripts/kaikki-to-classes.py`: 44 noun classes (Kaikki's stem class :
  nominative ending : gender, velar twins with the second palatalisation
  written ѕ), 6 adjective classes (short and long series, the incomplete
  soft table filled from its complete twin, the contracted -ꙑ/-и long
  nominatives as alternatives), 55 verb classes (the infinitive's ending,
  the present's first- and third-person endings, whether the present stem
  is the infinitive's — `stems=2=пь` on пити's line — with the past
  participles derived on stems 7/8/11 and declined by delegation to the
  adjective classes), 17 hand-written pronoun classes (тъ/сь hard and
  soft, вьсь, the possessives, къто/чьто, the relative иже and the third
  person on the empty stem, азъ/тꙑ/себе/мꙑ/вꙑ with per-lexeme stems, the
  nominal PN/PNk). The class tables are per recension (`table_of`), a
  lexeme carries its recension, the derivations know it (ѕ), the
  enclitic keeps its jer in OCS (имъже).
- **Kaikki import** (`cargo xtask import kaikki --pos <pos>`): 2,826 entries
  read cell by cell from `data/intermediate/kaikki-cells.jsonl`, each
  fitted to its seeded class and to every class producing its lemma, the
  best kept (ties noted); 1,921 nouns (99.1% of 39,678 cells reproduced),
  311 adjectives (97.8% of 39,096), 517 verbs (94.5% of 78,063), 15
  pronouns; Kaikki's typo class quarantined as `kaikki-nom-mismatch` (46
  nouns, 6 adjectives, 15 pronoun form-of headwords).
- **UD PROIEL train import** (`cargo xtask filter-ud`, `import ud --pos
  <pos>`): 30,379 attestations → variants with `U:` on Kaikki's lexemes
  and 4,794 new lexemes fitted to the inventory (a class's exemplar
  ending breaks ties; the present stem read off the attested present
  forms; the citation cell overridden to the lemma where the class fails
  it). Lexicon: 3,493 nouns, 1,527 adjectives, 2,456 verbs, 82 pronouns.
- **The OCS guesser** (`Lexicon::class_by_ending`): the commonest class
  among the lexicon's own lexemes sharing the lemma's last three, two,
  one letters — the lexicon instead of a hand rule.
- **Held-out recall** in `cargo xtask eval`: UD PROIEL dev+test and
  Syntacticus through the 1.2 harness's manuscript fold (шт/шч ~ щ, the
  jers, ѣ/ⱕ/ѧ ~ е, contracted double vowels, a titlo abbreviation as an
  ordered subsequence, the third person's post-prepositional н- and its
  aphaeresis), a pronoun slot also answered by its clitic twin, бꙑти's
  imperfect-tagged aorist by the aorist cell, a lemma the lexicon lacks
  by the guesser. `CS_RECALL_MISSES=n` samples the misses,
  `CS_RECALL_BLOCKS=1` counts them by cell block.
- `Lexeme::forms` deduplicates prints in the lexeme's own recension (the
  Synodal print folded ѥ/ѩ variants away); `xtask analyze --ocs`.

Measured (gate: UD dev+test recall ≥ the 1.2 baseline for every POS;
Syntacticus reported):

| Recall | 2.0 | 1.2 |
|---|---|---|
| UD PROIEL dev+test nouns | 94.87% (8,366/8,818) | 92.04% |
| adjectives | 89.35% (2,291/2,564) | 83.82% |
| verbs | 85.79% (7,514/8,759) | 85.58% |
| personal pronouns | 99.25% (3,983/4,013) | 99.25% |
| non-personal pronouns | 97.84% (1,268/1,296) | 93.21% |
| Syntacticus nouns / adjectives / verbs / pronouns / npron | 95.20% / 95.17% / 93.68% / 99.20% / 95.90% | — |

The gate is met (verbs by 0.21 points, the personal pronoun at par). The
remaining misses are the guessed lemmas' present stems (the guesser
cannot read an iotated stem off a lemma), the past participles of the
classes seeded without them, and manuscript spellings the fold does not
reach.

### Part 5 — the cutover (2026-09-05)

- Deleted: `legacy/` (the 1.x crates and their extractor), the executed
  1.x prompt files, `deprecated/`, `experiments/`, `data/witnesses.tsv`
  (converted to `W:` provenance in Part 3), the `xtask-legacy` alias and
  `eval --legacy`. `data/titlo.tsv` moved into the library as
  `lexicon/titlo.tsv` and the titlo layer with it (`church_slavonic::titlo`:
  `rows`, `abbreviate`, `skeleton`); a row for the gospel (є҆ѵⷢ҇лїе, 17
  print tokens) added.
- The workspace is two crates: `church-slavonic` 2.0.0 (dependency
  `unicode-normalization` only) and `church-slavonic-tools` (the `cargo
  xtask` binary, not published). README rewritten around the four
  stages, the lexicon format and the three eval numbers; HANDOFF-PROMPT.md
  rewritten for the 2.x program.
- Findings on the way: a verb's participles no longer outvote its finite
  cells when the stress paradigm is chosen (дои́ти is `b`, дои́лъ); и҆тѝ
  and its 41 compounds get the Viti class (и҆дꙋ̀, и҆до́хъ, ше́лъ); что̀ is
  neuter in both recensions; the third person's plural dative and
  accusative keep the print's varia (и҆̀мъ, и҆̀хъ); a guessed verb stressed
  on its theme keeps the stress on the ending (затепли́лъ); the Synodal
  guesser reads verb and adjective classes off the lexicon's lemma
  endings.
- `~/Desktop/code/vertograd` migrated: `slavonic.rs` adapts the game's
  lemma-keyed calls to the lexicon (`find`, `inflect`, `print`; a
  capitalised lemma keeps its capital; a homograph is chosen by its exact
  spelling; a lemma under a titlo declines through the titlo row); seven
  content strings re-pasted from the crate's real output (ѻ҆́вцꙋ — the
  print's 14 against 1; ѕе́лїе, Ѳеодо́сїе, є҆ѵⷢ҇лїе — the print's ї; хартїю̀,
  ѳи́та, ꙗ҆ицѐ — Polyakov's headwords); its 35 tests and
  `./scripts/headless-test.sh` green.

Measured at the close (`cargo xtask eval`, `check-treebank`):

| Number | 2.0.0 |
|---|---|
| held-out recall, UD dev+test: nouns / adjectives / verbs / personal / other pronouns | 94.87% / 89.35% / 85.79% / 99.25% / 97.84% |
| Bible treebank: analyzed / closed / ambiguous / verbatim | 23.4% / 28.1% / 40.2% / 8.1% (zero mismatches) |
| guesser, Synodal nouns: class / cells | 93.94% / 93.29% |
| Polyakov cells reproduced by the primary: nouns / adjectives / verbs | 94.7% / 94.1% / 91.5% |

## 1.2.0 — the Synodal pronoun release

The commonest words of the language render from cells: Synodal
non-personal pronouns (и҆́же, ве́сь, се́й, то́й, the possessives, the
interrogatives), the reflexive себѐ and the enclitics (мѝ, тѧ̀, ны̀, ѧ҆̀)
were empty or unaddressable in 1.1; the whole-Bible treebank's verbatim
share fell from 29.0% to 20.2% on this release alone. Every part is
measured in NOTES.md (v1.2 parts 0–5).

### Added
- **Synodal non-personal pronouns**: `ChurchSlavonic::npron` answers in
  the Synodal recension (`church_slavonic_core::npron_syn`: the print's
  closed lexicon from Alypy §47/§48 — то́й/се́й/ве́сь/кі́й/кто̀/что̀
  tables, the ending-stressed possessives, the stem-stressed soft and
  hard classes with velar softening, the relative и҆́же as the
  third-person row + же with the plural varia, ни-/нѣ́- prefixes and
  же/жде/ждо enclitics); 158 attested rows from Polyakov's APRO entries
  (short series; the long series joins the ADJECTIVE table under its own
  citation form: всѧ́кїй, ѻ҆́вый), the кто̀/что̀ family and Alypy §48.
  `lemmas(NonPersonalPronoun, Synodal)` enumerates 60 table lemmas and
  the rule's closed lexicon (`ChurchSlavonicCore::npron_lexicon`).
- **The reflexive and the clitic cells**: the personal matrix grows
  append-only from 90 to 119 cells — `reflexive(case, recension)`
  (себє̀/себѣ̀/себѐ/собо́ю), `clitic(person, number, gender, case,
  recension) -> Option` (мѝ/мѧ̀/тѝ/тѧ̀/ны̀/вы̀; и҆̀/ю҆̀/є҆̀/ѧ҆̀; in OCS the
  clitic is the primary accusative), `reflexive_clitic` (сѝ/сѧ̀), each
  with a `_sense` twin for the variant keys; `schema::reflexive_cell`,
  `clitic_cell`, `reflexive_clitic_cell`, `pronoun_features`.
- **Witness rows for pronouns**: `data/witnesses.tsv` accepts `pronoun`
  and `npron` with symbolic cells (`3.f.sg.dat`, `clit.1.pl.acc`,
  `m.pl.gen`); 23 new rows, every one a verified line of the Bible.
- **Orthography**: `realise` keeps an explicit varia on a monosyllable's
  only vowel — and on the host of a solid enclitic — so the print's
  «и҆̀хъ»/«и҆̀мъ» (against the genitive «и҆́хъ») and «и҆̀же»/«ꙗ҆̀же»/
  «и҆̀хже» (the relative's plural) are spellable;
  `transliteration_equivalent`, `number_mark_equivalent`,
  `is_vowel_letter`.

### Fixed
- **The Synodal personal row reads as the print**: «ꙗ҆̀» (Polyakov's
  civil «я́») gave way to «ѧ҆̀» in the dual and neuter-plural accusatives;
  the nominatives are the ѻ҆́нъ series (the anaphor «и҆̀»/«ꙗ҆̀»/«є҆̀»
  had been attested as nominatives by the dictionary's `nom/acc`
  bundles); «мы̀» is the plural nominative (not «ны̀»), «тебє̀» the
  genitive, «є҆́й»/«є҆́ю» the feminine dative/instrumental, «ни́хъ» the
  plural locative, «и҆̀мъ»/«и҆̀хъ» the plural dative/accusative.
- **Lookup invariant 5 — the print outranks the transliteration**: a
  print-exact source (Alypy, a witness) decides the letters a civil
  transliteration cannot encode (ꙗ/ѧ, the monosyllable's oxia/varia);
  a transliterated form differing from the rule only in those is the
  rule's form and is not stored; each source is scored under what it
  can encode. On the pronoun rows the print's number mark (the kamora of
  всѧ̑, на̑ша) and an enclitic's mark presence («ны» for «ны̀») are the
  rule's too — the dictionary's tag bundles cannot attest them per cell.
- **Bare-key assignment**: the shared personal row's bare key goes to
  its primary readings; a row storing an accentless Synodal spelling
  never outranks a clean one (всѧ́кій's «всякую» had taken the key from
  «всѧ́кꙋю»). Tried for every lemma-keyed row and refuted by the
  treebank (Polyakov's counts are per form, not per cell; «га́ды»
  outranked «гадѡ́въ»): recorded, deferred to the Bible-as-source design.
- **Source reading**: Polyakov's erok/abbreviation marks on a consonant
  («нас̑», «нбс̑нѣй»; 3,762 spellings) are no longer forms — 366 noun,
  449 adjective and 337 verb rows that existed only for them are gone;
  the anaphor's nominative analyses are skipped; `X+же` headwords are
  the print's one word (то́йже); UD `Reflex=Yes` (minus the possessive)
  and PROIEL `Pk` feed the reflexive cells.

### Measured
- Accuracy 100.00% / gap 0 on every source including the new rows:
  Non-personal pronouns Synodal (Alypy) 252/252, (Polyakov) 1,651/1,651,
  (witnessed print) 9/9; Pronouns Synodal (witnessed print) 14/14.
  Corpus recall now counts the reflexives it used to skip: pronouns UD
  dev+test 5,984/6,029 (99.25%), Syntacticus 32,525/32,800 (99.16%).
- The treebank (church-slavonic-syntax): zero mismatches over 34,470
  verses; analyzed 20.0% → 21.5%, ambiguous 23.7% → 31.0%, verbatim
  29.0% → 20.2%; Genesis 1's hand ceiling 79.4% → 85.1%.

## 1.1.0 — the consumer-defect release

The first real consumer (the `vertograd` monastery game) audited hundreds
of generated forms; its rejections diagnosed into three defect classes,
each fixed in its proper layer — and twice the audit itself was wrong and
the crate's attested answers stand guarded.

### Fixed
- **Lookup folding**: `ѷ`-spelled Synodal input now reaches the
  `ѵ`-spelled table key (the kendema is positional typography, as
  `comparison_key` always held); new lookup invariant 4 in the crate docs.
  `кѷпарі́съ` finds its attested inanimate accusative.
- **End-stressed verbs can hypothesize**: the class/present-stem override
  inference also strips accented endings («стриже́ши», «дои́ши»), and it
  re-runs over each candidate's UNIONED cells — sources that attest one
  cell per entry (Polyakov form-of) previously starved it.
- **The stems the infinitive hid**: a Second-class fact on an `-ити`
  lemma re-derives the stems as a true i-verb (`дои́ти` "to milk" is not
  `до` + `и҆тѝ`: aorist «доѝ», imperative «доѝ», the «напоѝ» print
  type); a present-stem fact on a `-щи` lemma restores the neutralized
  velar to the aorist and l-participle (`стрищѝ` : «стри́глъ»,
  «стрижѐ»). The l-participle now enters the fact engine on class/stem
  facts, and unaccented derived stems thread the lemma's accent like the
  plain rule path.
- **The accusative-shape fact teaches both ways**: any attested
  nominative-shaped accusative (singular, dual or plural) marks the row
  inanimate for the others (`ѻ҆гꙋре́цъ`), and the extractor's re-store
  pass reads its sources live so mutually-derivable cells are stored
  once.

### Added
- **The witness source** (`data/witnesses.tsv`): curated single cells
  from running Synodal print, each citing a verbatim line of a pinned
  text, ingested like any source (own 100.00%/0 accuracy row) and
  verified by `cargo xtask check-witnesses`. First admissions: the
  inanimate accusative of `ѡ҆́блакъ` (Lk 9:34), `ꙗ҆́блонь`'s nominative
  (Joel 1:12) and feminine instrumental (Song 8:5).
- The consumer-defect ledger (`tests/consumer_defects.rs`): every
  diagnosed form as a test, including the guards for the two forms where
  the AUDIT was wrong — «пожа́тъ» (the attested `-ѧти` aorist) and
  «вожжѝ» (an attested imperative spelling; the guard asserts the
  attested set across sense keys, since sense numbers may renumber).

### Measured, unchanged
- The Synodal unattested-masculine animate accusative default stands:
  Polyakov's masculines are 72.9% animate by lemma / 53.3% by token, and
  no Synodal held-out corpus exists to arbitrate further. Unattested and
  unwitnessed accusatives (`прꙋ́дъ`, `ко́локолъ`) keep the default.
- Held-out corpus recall: unchanged except OCS dev+test verbs
  7414 → 7413 (−1 slot, recorded in NOTES).

## 1.0.0 — the schema close

### Added
- The l-participle (resultative): `ChurchSlavonic::l_participle(key,
  &gender, &number, &recension)`. The verb row grows append-only from 549
  to 558 cells (a nominative-only gender/number block); the rule builds it
  on the infinitive stem (`бꙑти` : `бꙑлъ`, `вести` : `велъ`, `рещи` :
  `реклъ`), Synodal accents ride the lemma's stress, and the reflexive
  `-сѧ` stays outside. Newly attested cells: Polyakov's `partcp,perf`
  forms (4,082 slots), the UD PROIEL train split's `PartRes` tokens, and
  the Wiktionary l-participle pages. The held-out treebank evaluations
  score the new cells.
- The non-personal pronouns: `ChurchSlavonic::npron(key, &gender,
  &number, &case, &recension)` over a new 54-cell lemma-keyed table
  (`npron_phf.rs`) and a pronominal-declension rule — the hard `тъ` type,
  the soft `сь`/`мои`/`нашь` type, mixed `вьсь`, the relative `иже` as
  the anaphoric series plus `же`, the singular-only `къто`/`чьто`, and
  the `ни-`/`нѣ-` compounds. Sources: the Wiktionary pronoun tables and
  form-of pages and the UD PROIEL train split; both treebank evaluations
  score the class (93.2% and 94.3% held-out recall on first contact).

### Fixed
- Two same-signature candidates now union their attested raw cells, so a
  rule-equal form shadowed by a stored bare cell is re-materialised on a
  variant key instead of silently dropped (`еиже` under `иже`).

### Notes
- 1.0.0 closes the schema scope. The residual held-out corpus-recall gap
  is documented data ceiling, not backlog: forms enter the tables only
  when a pinned source attests them past the gates, and no further
  qualifying machine-readable source exists today (see `NOTES.md` for the
  candidates examined). The research diary lives in `NOTES.md` from this
  release on.

## 0.9.0 — the accusative-shape fact

### Added
- The noun resolution reads a row's stored LOWER accusative as a fact
  (`schema::NOUN_SHAPE_SOURCE_CELLS`, cells 3 and 10): a stored
  nominative-shaped accusative — an inanimate, where the Synodal masculine
  rule answers the genitive shape — teaches the row's higher accusative
  cells the same shape (`вѣне́цъ` : `вѣнцы̀`, not `вѣнце́въ`). Sources
  derive upward only, so the lowest stored accusative is the anchor and
  never subtracts itself. One engine as always: the facade, the
  extractor's subtraction, the reachability passes and the audits all read
  it; no new cell, no arity change. 39 stored accusative-plural cells are
  now derived, and ~1,500 rows whose only attested accusative is the
  singular now answer their unattested plural in the attested shape.


## 0.8.0 — convention-aware accent tokens

### Changed
- The accent-pattern token now rides inside the accent pass
  (`core::accent::with_accent_pattern`) on every rule path, instead of
  bare-re-stressing the finished form: the print's stress-coupled
  conventions — the wide `ѡ`/`є`, the kamora, the word-final varia, the
  carried stem marks — follow the token's position exactly as they follow
  the lemma's. The skeleton-level stem/override paths, whose endings carry
  no convention marker, keep the bare re-stress.
- The convention itself was corrected against the corpus: the widening
  targets the last narrow `о`/`е` at or after the stress; a form stressed
  on its final vowel widens the last narrow `о`/`е` anywhere instead
  (`вѡнѝ`, `верєѝ`); and a lexical wide letter no longer excuses the
  kamora (`а҆арѡ̑нимъ`, `а҆вессалѡ̑мли` — the print writes the kamora
  anyway). 643 attested kamora-bearing cells that were stored are now
  reproduced by rule; noun bare accuracy +0.09 (Polyakov) and +0.20
  (Alypy), adjectives +0.02, nothing regresses, OCS byte-identical.


## 0.7.0 — one resolution engine and the accent-pattern cells

### Changed
- The fact-resolution order (own exact cell -> bare exact cell -> facts
  read own-else-bare -> rule) is consolidated into
  `church_slavonic_core::resolution` and `church_slavonic_core::schema`;
  the runtime facade, the extractor's subtraction and reachability passes,
  and both dead-weight audits call the one engine. The refactor is
  byte-identical: same tables, same accuracy report.

### Added
- Synodal accent-pattern cells (noun 21, adjective 126, verb 548; arities
  22/127/549): a derived `s<N>`/`e` token adopted only when every attested
  accented form of the row shares the stress shape and the re-accented
  rule reproduces it exactly. ~370 rows adopt; mobile and mixed-convention
  paradigms (~6,500 accented Synodal rows) stay stored — the finding of
  this build is that Synodal storage couples stress with the print's
  plural-letter conventions, so pure stress patterns have limited reach.
  The refresh summary reports the token/Polyakov-class agreement rate
  (85/128 across 29 classes on this data).

## 0.6.0 — class cells and the slice tables

### Added
- Per-verb conjugation-class and present-stem override cells (546/547):
  derived facts, inferred by the extractor from the attested present cells
  and validated form by form, that re-run the finite (and present
  participle) rule with the verb's true class — a misclassed verb's finite
  block collapses to two cells. The runtime resolves exact cell -> bare
  row's cell -> class/present-stem override (own, else the bare row's) ->
  rule, and every audit mirrors that order.

### Changed
- The generated tables are sorted static slices looked up by binary search;
  the `phf` dependency is gone. Same information, simpler artifact:
  byte-identical accuracy output, table-hit throughput unchanged (~6.8M
  pronoun calls/s), and the runtime crate rebuilds faster (0.65s against
  1.46s on a table touch).

## 0.5.0 — declined participles

### Added
- The full declined participle system: present and past, active and
  passive, short and long series, over the adjective-style agreement
  features. New facade call `ChurchSlavonic::participle(key, &tense,
  &voice, &series, &case, &number, &gender, &recension)` with the new
  `Voice` and `Series` enums; `verb(..., Form::Participle)` still returns
  the two citation cells. The verb schema grows append-only from 38 to 546
  cells: 504 declined-participle cells and four participle-STEM cells the
  extractor derives from the attested declensions — a regular declension of
  an irregular stem costs four cells, not five hundred, and the runtime
  expands the stem through the same declension rule.
- Table sources for the new cells: the Kaikki participle sub-tables read in
  full, Polyakov's participle declensions (corpus-frequency gated at ≥5 —
  its hapax analyses are where OCR and analysis noise lives; the citation
  cells stay ungated), and the UD PROIEL train split under the existing
  gates. The two treebank evaluations score the declined cells too.
- Participle rules per recension: the OCS long-series shapes (`-щиимь`,
  `-щиꙗ`, `-щеи`), the Synodal print's own mixed declension for the active
  stems (Alypy pp. 95–96: `-щагѡ` but `-щихъ`), citation contractions
  derived from the stem (`-ѫщ` -> `-ꙑ`, `-ꙋщ` -> `-ый`), and reflexive
  participles.

### Changed
- The generated sparse rows index cells as `u16` (the schema outgrew
  `u8`); `check-registry`'s dead-weight audit is stem-aware.

## 0.4.0 — the english-parity release (breaking)

The workspace is rebuilt in the shape of `gold-silver-copper/english`: four
crates, generated PHF tables as the whole artifact, three `xtask` commands,
one README.

### Changed (breaking)
- `church-slavonic` is one `lib.rs`: table-first, rule-fallback, case
  restoration, deterministic `_n` sense keys assigned by a pure sort (keys
  may renumber on refresh). Every call takes `&Recension`
  (`OldChurchSlavonic` | `Synodal`); the scoped handles, profiles and
  identity layer are gone.
- `church-slavonic-core` is rules only (no data): `grammar`, `noun`, `adj`,
  `verb`, `pronoun`, `orthography`, `sense_key`, `utils`; depends on
  `unicode-normalization` alone.
- The tables regenerate from two pinned sources (Kaikki OCS Wiktionary,
  the Alypy grammar pages) with `cargo xtask refresh-data`; no curated
  data files, ledgers, overrides or lockfiles remain.

### Removed
- Crates `old-church-slavonic-core`, `church-slavonic-orthography`,
  `church-slavonic-dictionary`, `synodal-church-slavonic-core`,
  `synodal-church-slavonic`, `synodal-church-slavonic-dictionary` and the
  old extractor. Each published name gets a final empty patch release
  pointing here (`deprecated/`); their sources are at tag
  `pre-english-parity`.
- `data/` (except the gitignored `data/intermediate/`), `reports/`, `docs/`,
  the root prompt files and the non-CI workflows.
- Every `xtask` command except `refresh-data`, `check-registry`, `accuracy`.

### Moved
- The Synodal text analyzer is an unmaintained experiment under
  `experiments/analyzer/`, built against the published 0.6 crates.

Earlier history (0.1–0.3 and the synodal 0.4–0.6 program) is in the git
history of `CHANGELOG.md` before this release.
