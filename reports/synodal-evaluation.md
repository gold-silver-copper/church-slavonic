# Synodal evaluation

Target recension: `synodal-russian`. Fixture: pinned passage-held-out Ponomar Elizabeth Bible rows across Matthew, Acts, Daniel, Apocalypse, Amos, and Deuteronomy (2496 held-out token cells).

The correction ledger excludes 8 historically preserved but grammatically retracted evaluation rows from scoring.

| Metric | Returned | Top-1 | Top-k | Abstained | Total |
|---|---:|---:|---:|---:|---:|
| Expanded | 2496 | 2428 | 2492 | 0 | 2496 |
| Printed | 2496 | 2343 | 2488 | 0 | 2496 |

Analytic phrases: expanded 14/14, printed 14/14 (14 held-out phrases).

Typed abbreviations: top-1 71/81, top-k 81/81 (81 held-out contractions; reverse lookup also required).

Exact registry round trips (top-k, including reviewed variants): expanded 3245/3245, printed 3245/3245.

Masked cells: expanded 759/762, printed 755/762. Leave-one-Synodal-lexeme-out inherited cells: expanded 0/0, printed 0/0.

Accent agreement: 2341/2349 accent-bearing rows.

Inherited evidence contributed 0/1 returned held-out cells, with 0/0 exact expanded forms. The reviewed alignment registry has 5 accepted mappings, 5 aligned target lexemes, and 1 rejected negative controls.

## Expanded accuracy by generation policy

| Slice | Returned | Top-1 | Top-k | Abstained | Total |
|---|---:|---:|---:|---:|---:|
| `exploratory` | 2496 | 2428 | 2492 | 0 | 2496 |
| `productive` | 2496 | 2428 | 2492 | 0 | 2496 |
| `strict` | 2496 | 2428 | 2492 | 0 | 2496 |

## Expanded accuracy by attestation status

| Slice | Returned | Top-1 | Top-k | Abstained | Total |
|---|---:|---:|---:|---:|---:|
| `attested` | 2124 | 2062 | 2124 | 0 | 2124 |
| `expected-form-not-returned` | 4 | 0 | 0 | 0 | 4 |
| `predicted` | 368 | 366 | 368 | 0 | 368 |

## Expanded accuracy by morphological system

| Slice | Returned | Top-1 | Top-k | Abstained | Total |
|---|---:|---:|---:|---:|---:|
| `adjective` | 140 | 136 | 140 | 0 | 140 |
| `aorist` | 238 | 232 | 237 | 0 | 238 |
| `determiner` | 21 | 20 | 21 | 0 | 21 |
| `future` | 106 | 102 | 105 | 0 | 106 |
| `imperative` | 73 | 71 | 73 | 0 | 73 |
| `imperfect` | 35 | 33 | 35 | 0 | 35 |
| `indeclinable` | 88 | 82 | 88 | 0 | 88 |
| `infinitive` | 28 | 28 | 28 | 0 | 28 |
| `l-participle` | 21 | 21 | 21 | 0 | 21 |
| `lexical-form` | 435 | 430 | 434 | 0 | 435 |
| `noun` | 1030 | 996 | 1029 | 0 | 1030 |
| `numeral` | 26 | 25 | 26 | 0 | 26 |
| `participle` | 80 | 80 | 80 | 0 | 80 |
| `present` | 76 | 75 | 76 | 0 | 76 |
| `pronoun` | 99 | 97 | 99 | 0 | 99 |

## Expanded accuracy by provenance path

| Slice | Returned | Top-1 | Top-k | Abstained | Total |
|---|---:|---:|---:|---:|---:|
| `exact-synodal-attestation` | 2122 | 2062 | 2119 | 0 | 2122 |
| `synodal-irregular-override` | 19 | 16 | 19 | 0 | 19 |
| `synodal-normative-table` | 78 | 74 | 78 | 0 | 78 |
| `synodal-productive-rule` | 277 | 276 | 276 | 0 | 277 |

## Expanded accuracy by regularity

| Slice | Returned | Top-1 | Top-k | Abstained | Total |
|---|---:|---:|---:|---:|---:|
| `closed-class-held-out-exact-cell` | 25 | 25 | 25 | 0 | 25 |
| `closed-class-held-out-normative` | 30 | 30 | 30 | 0 | 30 |
| `exact-held-out-animate-accusative-plural-omega` | 1 | 0 | 1 | 0 | 1 |
| `exact-held-out-aorist-third-singular-grave` | 1 | 1 | 1 | 0 | 1 |
| `exact-held-out-cardinal-one-instrumental-singular-feminine` | 1 | 1 | 1 | 0 | 1 |
| `exact-held-out-conclusive-conjunction` | 1 | 1 | 1 | 0 | 1 |
| `exact-held-out-fleeting-vowel-place-name` | 1 | 1 | 1 | 0 | 1 |
| `exact-held-out-fourth-neuter-at-accusative-singular` | 1 | 1 | 1 | 0 | 1 |
| `exact-held-out-future-ending-stress` | 1 | 1 | 1 | 0 | 1 |
| `exact-held-out-future-first-singular-grave` | 1 | 1 | 1 | 0 | 1 |
| `exact-held-out-genitive-plural-kamora` | 1 | 1 | 1 | 0 | 1 |
| `exact-held-out-imperative-homograph` | 1 | 1 | 1 | 0 | 1 |
| `exact-held-out-infinitive` | 1 | 1 | 1 | 0 | 1 |
| `exact-held-out-interrogative-adverb` | 1 | 1 | 1 | 0 | 1 |
| `exact-held-out-interrogative-temporal-adverb` | 1 | 1 | 1 | 0 | 1 |
| `exact-held-out-invariant-adverb` | 1 | 1 | 1 | 0 | 1 |
| `exact-held-out-invariant-interjection` | 1 | 1 | 1 | 0 | 1 |
| `exact-held-out-lexical` | 383 | 383 | 383 | 0 | 383 |
| `exact-held-out-no-yat-locative-variant` | 1 | 1 | 1 | 0 | 1 |
| `exact-held-out-nominative-plural-variant` | 1 | 0 | 1 | 0 | 1 |
| `exact-held-out-normative-preposition` | 1 | 1 | 1 | 0 | 1 |
| `exact-held-out-positional-plural` | 2 | 2 | 2 | 0 | 2 |
| `exact-held-out-preposition` | 1 | 1 | 1 | 0 | 1 |
| `exact-held-out-primary-preposition` | 1 | 1 | 1 | 0 | 1 |
| `exact-held-out-primary-preposition-with-yerok` | 1 | 1 | 1 | 0 | 1 |
| `exact-held-out-pronominal-adverb` | 1 | 1 | 1 | 0 | 1 |
| `exact-held-out-temporal-adverb` | 1 | 1 | 1 | 0 | 1 |
| `exact-held-out-vocalized-preposition` | 1 | 1 | 1 | 0 | 1 |
| `exact-held-out-wide-e-plural` | 1 | 1 | 1 | 0 | 1 |
| `exact-held-out-wide-omega-soft-ie-neuter` | 1 | 1 | 1 | 0 | 1 |
| `held-out-exact-wide-omega-cell` | 1 | 1 | 1 | 0 | 1 |
| `held-out-productive-velar-cell` | 1 | 1 | 1 | 0 | 1 |
| `held-out-velar-palatalization-cell` | 1 | 1 | 1 | 0 | 1 |
| `irregular` | 7 | 7 | 7 | 0 | 7 |
| `irregular-held-out` | 10 | 9 | 10 | 0 | 10 |
| `irregular-participle` | 1 | 1 | 1 | 0 | 1 |
| `normative-held-out-wide-omega-animate-accusative-plural` | 1 | 0 | 1 | 0 | 1 |
| `productive-held-out-animate-hard-masculine` | 2 | 2 | 2 | 0 | 2 |
| `productive-held-out-cardinal-one-instrumental-singular-masculine` | 1 | 1 | 1 | 0 | 1 |
| `productive-held-out-city-name-locative` | 1 | 1 | 1 | 0 | 1 |
| `productive-held-out-city-name-nominative` | 1 | 1 | 1 | 0 | 1 |
| `productive-held-out-first-conjugation-future-accent` | 1 | 1 | 1 | 0 | 1 |
| `productive-held-out-first-conjugation-future-principal-part` | 1 | 1 | 1 | 0 | 1 |
| `productive-held-out-first-hard-masculine-accusative-plural` | 1 | 1 | 1 | 0 | 1 |
| `productive-held-out-first-hard-masculine-genitive-singular` | 1 | 1 | 1 | 0 | 1 |
| `productive-held-out-first-hard-masculine-instrumental-singular` | 1 | 1 | 1 | 0 | 1 |
| `productive-held-out-first-hard-masculine-nominative-singular` | 1 | 1 | 1 | 0 | 1 |
| `productive-held-out-first-soft-masculine-instrumental-singular` | 1 | 1 | 1 | 0 | 1 |
| `productive-held-out-fixed-accent` | 4 | 4 | 4 | 0 | 4 |
| `productive-held-out-fixed-accent-and-breathing` | 3 | 3 | 3 | 0 | 3 |
| `productive-held-out-fixed-oblique-accent` | 2 | 2 | 2 | 0 | 2 |
| `productive-held-out-fourth-neuter-at-dative-singular` | 1 | 1 | 1 | 0 | 1 |
| `productive-held-out-fourth-neuter-at-nominative-plural` | 1 | 1 | 1 | 0 | 1 |
| `productive-held-out-future-third-plural` | 1 | 1 | 1 | 0 | 1 |
| `productive-held-out-future-third-singular` | 1 | 1 | 1 | 0 | 1 |
| `productive-held-out-genitive-ending-accent` | 1 | 1 | 1 | 0 | 1 |
| `productive-held-out-hard-adjective-fixed-accent` | 1 | 1 | 1 | 0 | 1 |
| `productive-held-out-hard-adjective-sk-st-alternation` | 1 | 1 | 1 | 0 | 1 |
| `productive-held-out-hard-masculine-fixed-accent` | 2 | 2 | 2 | 0 | 2 |
| `productive-held-out-hard-masculine-oblique` | 1 | 1 | 1 | 0 | 1 |
| `productive-held-out-hard-masculine-wide-yat` | 1 | 1 | 1 | 0 | 1 |
| `productive-held-out-hard-neuter-ending-accent` | 1 | 1 | 1 | 0 | 1 |
| `productive-held-out-hard-neuter-fixed-accent` | 1 | 1 | 1 | 0 | 1 |
| `productive-held-out-hard-neuter-zero-ending` | 1 | 1 | 1 | 0 | 1 |
| `productive-held-out-historical-j-possessive-citation` | 1 | 1 | 1 | 0 | 1 |
| `productive-held-out-historical-j-possessive-hard-vowel-ending` | 1 | 1 | 1 | 0 | 1 |
| `productive-held-out-indeclinable-accent` | 1 | 1 | 1 | 0 | 1 |
| `productive-held-out-instrumental-acute` | 1 | 1 | 1 | 0 | 1 |
| `productive-held-out-l-participle` | 2 | 2 | 2 | 0 | 2 |
| `productive-held-out-locative-ending-accent` | 1 | 1 | 1 | 0 | 1 |
| `productive-held-out-long-adjective-accusative-plural-neuter` | 1 | 1 | 1 | 0 | 1 |
| `productive-held-out-long-adjective-genitive-plural` | 1 | 1 | 1 | 0 | 1 |
| `productive-held-out-long-adjective-nominative-plural` | 1 | 1 | 1 | 0 | 1 |
| `productive-held-out-long-adjective-nominative-plural-neuter` | 1 | 1 | 1 | 0 | 1 |
| `productive-held-out-mixed-ts-masculine-nominative-plural` | 1 | 1 | 1 | 0 | 1 |
| `productive-held-out-mobile-accent` | 2 | 2 | 2 | 0 | 2 |
| `productive-held-out-perfective-finite-accent-and-breathing` | 1 | 1 | 1 | 0 | 1 |
| `productive-held-out-perfective-finite-principal-part` | 1 | 1 | 1 | 0 | 1 |
| `productive-held-out-personal-name-accusative` | 2 | 2 | 2 | 0 | 2 |
| `productive-held-out-personal-name-dative` | 5 | 5 | 5 | 0 | 5 |
| `productive-held-out-personal-name-genitive` | 1 | 1 | 1 | 0 | 1 |
| `productive-held-out-personal-name-nominative` | 7 | 7 | 7 | 0 | 7 |
| `productive-held-out-personal-name-vocative` | 3 | 3 | 3 | 0 | 3 |
| `productive-held-out-possessive-in-adjective` | 1 | 1 | 1 | 0 | 1 |
| `productive-held-out-possessive-in-adjective-fixed-accent` | 1 | 1 | 1 | 0 | 1 |
| `productive-held-out-present-active-participle-accent` | 1 | 1 | 1 | 0 | 1 |
| `productive-held-out-proper-name-dative-ovi` | 1 | 1 | 1 | 0 | 1 |
| `productive-held-out-relational-adjective` | 2 | 2 | 2 | 0 | 2 |
| `productive-held-out-river-name-genitive` | 1 | 1 | 1 | 0 | 1 |
| `productive-held-out-second-conjugation-imperative` | 2 | 2 | 2 | 0 | 2 |
| `productive-held-out-short-adjective-accusative-singular` | 1 | 1 | 1 | 0 | 1 |
| `productive-held-out-short-adjective-nominative-singular` | 1 | 1 | 1 | 0 | 1 |
| `productive-held-out-short-relational-adjective` | 1 | 1 | 1 | 0 | 1 |
| `productive-held-out-soft-ie-fixed-accent` | 5 | 5 | 5 | 0 | 5 |
| `productive-held-out-soft-ie-genitive` | 1 | 1 | 1 | 0 | 1 |
| `productive-held-out-soft-ie-neuter-fixed-accent` | 1 | 1 | 1 | 0 | 1 |
| `productive-held-out-soft-ie-neuter-genitive-singular` | 2 | 2 | 2 | 0 | 2 |
| `productive-held-out-soft-ie-neuter-instrumental-singular` | 1 | 1 | 1 | 0 | 1 |
| `productive-held-out-soft-ie-neuter-nominative-singular` | 2 | 2 | 2 | 0 | 2 |
| `productive-held-out-substantivized-adjective` | 1 | 1 | 1 | 0 | 1 |
| `productive-held-out-vowel-aorist` | 1 | 1 | 1 | 0 | 1 |
| `regular` | 2 | 2 | 2 | 0 | 2 |
| `regular-held-out` | 17 | 17 | 17 | 0 | 17 |
| `regular-inherited` | 1 | 1 | 1 | 0 | 1 |
| `reviewed-real-passage-normative` | 15 | 15 | 15 | 0 | 15 |
| `v04-held-out-exact-irregular` | 7 | 7 | 7 | 0 | 7 |
| `v04-held-out-normative-family` | 1 | 1 | 1 | 0 | 1 |
| `v05-held-out-cardinal-cell` | 1 | 1 | 1 | 0 | 1 |
| `v05-held-out-closed-class` | 5 | 5 | 5 | 0 | 5 |
| `v05-held-out-determiner-variant` | 1 | 1 | 1 | 0 | 1 |
| `v05-held-out-exact-adjective` | 1 | 0 | 1 | 0 | 1 |
| `v05-held-out-exact-irregular` | 1 | 1 | 1 | 0 | 1 |
| `v05-held-out-exact-noun` | 1 | 1 | 1 | 0 | 1 |
| `v05-held-out-exact-numeral` | 2 | 2 | 2 | 0 | 2 |
| `v05-held-out-exact-verb` | 5 | 5 | 5 | 0 | 5 |
| `v05-held-out-irregular-plural` | 1 | 0 | 1 | 0 | 1 |
| `v05-held-out-mixed-determiner` | 8 | 7 | 8 | 0 | 8 |
| `v05-held-out-negative-irregular` | 1 | 1 | 1 | 0 | 1 |
| `v05-held-out-noun-variant` | 4 | 3 | 4 | 0 | 4 |
| `v05-held-out-positional-accent` | 1 | 1 | 1 | 0 | 1 |
| `v05-held-out-positional-variant` | 1 | 1 | 1 | 0 | 1 |
| `v05-held-out-proper-name` | 2 | 2 | 2 | 0 | 2 |
| `v05-held-out-relative-pronoun` | 1 | 1 | 1 | 0 | 1 |
| `v05-held-out-third-person-variant` | 1 | 1 | 1 | 0 | 1 |
| `v06-held-out-exact-cell` | 81 | 80 | 81 | 0 | 81 |
| `v06-held-out-exact-irregular` | 5 | 5 | 5 | 0 | 5 |
| `v06-held-out-lexical-identity` | 61 | 59 | 60 | 0 | 61 |
| `v06-held-out-manual-exact` | 13 | 12 | 13 | 0 | 13 |
| `v06-held-out-normative-table` | 5 | 5 | 5 | 0 | 5 |
| `v06-held-out-source-typed-exact` | 473 | 456 | 473 | 0 | 473 |
| `v07-held-out-exact-cell` | 913 | 881 | 913 | 0 | 913 |
| `v07-held-out-explicit-accent-case-variant` | 44 | 40 | 44 | 0 | 44 |
| `v07-reviewed-identity-correction` | 1 | 1 | 1 | 0 | 1 |
| `v12-productive-held-out` | 221 | 219 | 219 | 0 | 221 |
| `v13-demonstration` | 2 | 2 | 2 | 0 | 2 |
| `v14-broad-on-exploitation` | 3 | 2 | 2 | 0 | 3 |
| `v14-uk-exploitation` | 7 | 7 | 7 | 0 | 7 |
| `v18-held-out-productive-accent` | 2 | 2 | 2 | 0 | 2 |
| `v18-held-out-source-typed-exact` | 1 | 1 | 1 | 0 | 1 |
| `v19-held-out-normative-short-feminine-accusative` | 1 | 1 | 1 | 0 | 1 |

## Top-1 disagreements

- Expanded `eval:acts-9-9-tri` (`numeral:cardinal:nominative:plural:masculine:inanimate`): expected `три`, top-1 `трїе`.
- Expanded `eval:v05:mnog-nominative-plural` (`adjective:nominative:plural:masculine:animate:short:positive`): expected `мнѡги`, top-1 `мнози`.
- Expanded `eval:v05:slovo-slovesa` (`noun:accusative:plural:inanimate`): expected `словеса`, top-1 `слова`.
- Expanded `eval:v05:syn-dative-plural` (`noun:dative:plural:animate`): expected `сыномъ`, top-1 `сыновомъ`.
- Expanded `eval:v05:ves-genitive-masculine` (`determiner:genitive:singular:masculine:inanimate:short:positive`): expected `всегѡ`, top-1 `всего`.
- Expanded `eval:v06:exact-2b01c52cf32cfd85` (`noun:genitive:plural:inanimate`): expected `бѡлѣзни`, top-1 `болѣзни`.
- Expanded `eval:v06:exact-32b0b5e0e7691641` (`adjective:nominative:singular:feminine:any:short:positive`): expected `добрѣ`, top-1 `добра`.
- Expanded `eval:v06:exact-3dbd30879a2731eb` (`adjective:accusative:singular:neuter:inanimate:short:positive`): expected `малѡ`, top-1 `мало`.
- Expanded `eval:v06:exact-3ebf5549ab4f08a3` (`noun:nominative:plural:inanimate`): expected `бѡлѣзни`, top-1 `болѣзни`.
- Expanded `eval:v06:exact-42d54c80b1e9c917` (`noun:accusative:singular:animate`): expected `рабъ`, top-1 `раба`.
- Expanded `eval:v06:exact-4939a19e5ab27dde` (`future:third:singular`): expected `избавитъ`, top-1 `избави`.
- Expanded `eval:v06:exact-614fe964ac48a0dd` (`imperative:second:singular`): expected `иди`, top-1 `идеши`.
- Expanded `eval:v06:exact-6996fc14230302dc` (`noun:locative:singular:inanimate`): expected `тьмѣ`, top-1 `тмѣ`.
- Expanded `eval:v06:exact-70859d2e90da66bc` (`aorist:third:singular`): expected `прїѧтъ`, top-1 `прїѧ`.
- Expanded `eval:v06:exact-7def29284314daa1` (`adjective:nominative:singular:neuter:any:short:positive`): expected `малѡ`, top-1 `мало`.
- Expanded `eval:v06:exact-8ee7fc72e685652e` (`noun:dative:singular:inanimate`): expected `бѡлѣзни`, top-1 `болѣзни`.
- Expanded `eval:v06:exact-9bae23a5cc72b118` (`aorist:third:dual`): expected `рѣсте`, top-1 `рѣста`.
- Expanded `eval:v06:exact-a52b4e1dc0f77849` (`noun:accusative:singular:animate`): expected `рабъ`, top-1 `раба`.
- Expanded `eval:v06:exact-a9355c2daae719d5` (`noun:genitive:singular:inanimate`): expected `словеси`, top-1 `слова`.
- Expanded `eval:v06:exact-b058ee586f2816e8` (`noun:genitive:singular:inanimate`): expected `бѡлѣзни`, top-1 `болѣзни`.
- Expanded `eval:v06:exact-c123b072b6cbc74e` (`noun:accusative:plural:inanimate`): expected `бѡлѣзни`, top-1 `болѣзни`.
- Expanded `eval:v06:exact-ec9f03ea8db3f50f` (`aorist:first:singular`): expected `Рѣхъ`, top-1 `рѣкъ`.
- Expanded `eval:v06:lexical-8897d4e2e9679e01` (`lexical-form`): expected `илїи`, top-1 `илїа`.
- Expanded `eval:v06:lexical-c36c119a628a329c` (`lexical-form`): expected `послꙋша`, top-1 `послꙋшати`.
- Expanded `eval:v06:manual:resha` (`aorist:third:plural`): expected `рѣша`, top-1 `рекоша`.
- Expanded `eval:v06:slovo-gen-sg` (`noun:genitive:singular:inanimate`): expected `словесе`, top-1 `слова`.
- Expanded `eval:v07:variant-369f932e734ff9e7` (`noun:instrumental:plural:inanimate`): expected `ᲂусты`, top-1 `ѹсты`.
- Expanded `eval:v07:variant-601b77a623ca9eaf` (`indeclinable`): expected `ᲂуже`, top-1 `оуже`.
- Expanded `eval:v07:variant-b8c1513b6bda88c9` (`indeclinable`): expected `ѹже`, top-1 `оуже`.
- Expanded `eval:v07:variant-8df2b6ab8a91ffa0` (`noun:genitive:plural:inanimate`): expected `ꙗзыкъ`, top-1 `ѧзыкъ`.
- Expanded `eval:v15:noun:tsar-nominative-plural-tsarie` (`noun:nominative:plural:animate`): expected `царїе`, top-1 `цари`.
- Expanded `eval:v17:noun:levit-accusative-plural-omega` (`noun:accusative:plural:animate`): expected `леѵітѡвъ`, top-1 `леѵіты`.
- Expanded `eval:v24:noun:rab-wide-omega-accusative-plural` (`noun:accusative:plural:animate`): expected `рабѡвъ`, top-1 `рабы`.
- Expanded `eval:v07:05506371f37c63d6` (`aorist:third:singular`): expected `бѣжа`, top-1 `Бѣжа`.
- Expanded `eval:v07:0cc4625dd2086f15` (`noun:genitive:singular:inanimate`): expected `родꙋ`, top-1 `рода`.
- Expanded `eval:v07:12d883c01f5428e4` (`noun:instrumental:plural:inanimate`): expected `ᲂусты`, top-1 `ѹсты`.
- Expanded `eval:v07:264e1313b5a9fce0` (`future:second:plural`): expected `бꙋдите`, top-1 `бꙋдете`.
- Expanded `eval:v07:3ae1ab6559aeee91` (`noun:accusative:plural:inanimate`): expected `ᲂуста`, top-1 `ѹста`.
- Expanded `eval:v07:3ed087dcfeef0e16` (`present:first:singular`): expected `глаголꙋ`, top-1 `глаголю`.
- Expanded `eval:v07:44aeeb943ebc7c26` (`indeclinable`): expected `ѹ`, top-1 `оу`.
- Expanded `eval:v07:465e4d0cb2d9e2cc` (`noun:genitive:singular:inanimate`): expected `любви`, top-1 `любве`.
- Expanded `eval:v07:4d5b70b162c95606` (`indeclinable`): expected `ѹже`, top-1 `оуже`.
- Expanded `eval:v07:52a51adda1edae2c` (`imperfect:third:plural`): expected `имѧхꙋ`, top-1 `имѣѧхꙋ`.
- Expanded `eval:v07:5b2b9d69eb8735e0` (`indeclinable`): expected `ᲂуже`, top-1 `оуже`.
- Expanded `eval:v07:63fbc155df7d7d1a` (`noun:dative:singular:inanimate`): expected `словꙋ`, top-1 `словеси`.
- Expanded `eval:v07:645ba104b4d03aef` (`future:second:singular`): expected `ѡбрѧщеши`, top-1 `ѡбрѧщете`.
- Expanded `eval:v07:6d0cb86a0d9aee04` (`imperative:second:plural`): expected `речете`, top-1 `рцыте`.
- Expanded `eval:v07:7748137afb8161c2` (`lexical-form`): expected `послати`, top-1 `посланъ`.
- Expanded `eval:v07:7da0b9f08c2294aa` (`noun:genitive:singular:inanimate`): expected `ᲂутра`, top-1 `ѹтра`.
- Expanded `eval:v07:88de1a03e5c7bd6a` (`noun:locative:singular:inanimate`): expected `родꙋ`, top-1 `родѣ`.
- Expanded `eval:v07:922e6f2723af3ee9` (`noun:nominative:singular:inanimate`): expected `любы`, top-1 `любовь`.
- Expanded `eval:v07:9261a7d8e2639922` (`indeclinable`): expected `ᲂу`, top-1 `оу`.
- Expanded `eval:v07:94d144f9d3ff2a2e` (`noun:genitive:plural:inanimate`): expected `ꙗзыкъ`, top-1 `ѧзыкъ`.
- Expanded `eval:v07:95f004c8142282be` (`pronoun:dative:singular:neuter:none:any`): expected `немꙋже`, top-1 `ємꙋже`.
- Expanded `eval:v07:9a678cab7ac6aafd` (`lexical-form`): expected `взыти`, top-1 `взыдоша`.
- Expanded `eval:v07:a24b407b362af140` (`noun:genitive:singular:inanimate`): expected `ꙗзыка`, top-1 `ѧзыка`.
- Expanded `eval:v07:b345a6c547f9d2a7` (`noun:accusative:singular:inanimate`): expected `ᲂумъ`, top-1 `ѹмъ`.
- Expanded `eval:v07:c105b5928b76e40e` (`pronoun:genitive:singular:feminine:none:any`): expected `неѧже`, top-1 `єѧже`.
- Expanded `eval:v07:cb4107f699fcc5d7` (`imperfect:third:singular`): expected `имѧше`, top-1 `имѣѧше`.
- Expanded `eval:v07:cc64afc6ffc35f45` (`noun:locative:plural:inanimate`): expected `словесѣхъ`, top-1 `словеси`.
- Expanded `eval:v07:cf6bac0302bf3133` (`lexical-form`): expected `сꙋдити`, top-1 `сꙋди`.
- Expanded `eval:v07:d5495b7b716d9f44` (`noun:genitive:singular:inanimate`): expected `волѧ`, top-1 `воли`.
- Expanded `eval:v07:e48eca6f14f7c825` (`noun:accusative:singular:inanimate`): expected `любы`, top-1 `любовь`.
- Expanded `eval:v07:e72eb04dce8a642d` (`noun:nominative:singular:inanimate`): expected `ᲂумъ`, top-1 `ѹмъ`.
- Expanded `eval:v07:f6a8afc17586cb5c` (`noun:genitive:singular:inanimate`): expected `тьмы`, top-1 `тмы`.
- Expanded `eval:v12:fb522502444b462d` (`future:second:singular`): expected `оумреши`, top-1 `ѹмреши`.
- Expanded `eval:v12:5ddb6c02cd48d6bf` (`aorist:third:singular`): expected `оумре`, top-1 `ѹмре`.
- Expanded `eval:v14:ef37fe236af5406e` (`noun:genitive:singular:inanimate`): expected `орꙋжіѧ`, top-1 `орꙋжїѧ`.
- Printed `eval:acts-3-16-dati-aorist` (`aorist:third:singular`): expected `дадѐ`, top-1 `даде́`.
- Printed `eval:acts-9-9-tri` (`numeral:cardinal:nominative:plural:masculine:inanimate`): expected `трѝ`, top-1 `трїѐ`.
- Printed `eval:v04:reshchi-aorist-acute` (`aorist:third:singular`): expected `рече́`, top-1 `речѐ`.
- Printed `eval:v04:ves-masc-nom-pl` (`determiner:nominative:plural:masculine:animate:short:positive`): expected `всѝ`, top-1 `вси́`.
- Printed `eval:v05:interjection-se-acute-before-li` (`indeclinable`): expected `се́`, top-1 `сѐ`.
- Printed `eval:v05:mnog-nominative-plural` (`adjective:nominative:plural:masculine:animate:short:positive`): expected `мнѡ́ги`, top-1 `мно́зи`.
- Printed `eval:v05:on-capital-plural` (`pronoun:nominative:plural:masculine:third:any`): expected `Ѻ҆ни́`, top-1 `ѻ҆нѝ`.
- Printed `eval:v05:slovo-slovesa` (`noun:accusative:plural:inanimate`): expected `словеса̀`, top-1 `сло́ва`.
- Printed `eval:v05:syn-dative-plural` (`noun:dative:plural:animate`): expected `сынѡ́мъ`, top-1 `сыновѡ́мъ`.
- Printed `eval:v05:ves-genitive-masculine` (`determiner:genitive:singular:masculine:inanimate:short:positive`): expected `всегѡ̀`, top-1 `всего̀`.
- Printed `eval:v05:zapoved-omega` (`noun:genitive:singular:inanimate`): expected `за́пѡвѣди`, top-1 `за́повѣди`.
- Printed `eval:v06:dom-ins-sg` (`noun:instrumental:singular:inanimate`): expected `домѡ́мъ`, top-1 `до́момъ`.
- Printed `eval:v06:exact-04cb9846b4c8b6db` (`adjective:genitive:singular:masculine:any:short:positive`): expected `жи̑ва`, top-1 `жи́ва`.
- Printed `eval:v06:exact-0642d378eeb6d2a1` (`noun:genitive:plural:animate`): expected `ра̑бъ`, top-1 `ра́бъ`.
- Printed `eval:v06:exact-06a0b24fde2ef68e` (`participle:past:passive:nominative:singular:feminine:any:short:positive`): expected `дана̑`, top-1 `да́на`.
- Printed `eval:v06:exact-0cb4915a0058444d` (`adjective:genitive:singular:masculine:any:short:positive`): expected `жива̀`, top-1 `жи́ва`.
- Printed `eval:v06:exact-0eb4c7d99724ca76` (`adjective:nominative:singular:feminine:any:short:positive`): expected `жи̑ва`, top-1 `жи́ва`.
- Printed `eval:v06:exact-1a1c91bd40c26893` (`noun:nominative:plural:animate`): expected `лица̑`, top-1 `ли́ца`.
- Printed `eval:v06:exact-1bbb80834ad2a9e3` (`noun:nominative:plural:inanimate`): expected `кораблѝ`, top-1 `корабли̑`.
- Printed `eval:v06:exact-228dbb9a4fc0132b` (`adjective:genitive:singular:neuter:any:short:positive`): expected `блага̀`, top-1 `бла̑га`.
- Printed `eval:v06:exact-29b870dbecaf8434` (`noun:nominative:plural:inanimate`): expected `стада̀`, top-1 `ста́да`.
- Printed `eval:v06:exact-2b01c52cf32cfd85` (`noun:genitive:plural:inanimate`): expected `бѡлѣ́зни`, top-1 `болѣ́зни`.
- Printed `eval:v06:exact-2b60f46e88d4822a` (`aorist:third:singular`): expected `родѝ`, top-1 `ро́ди`.
- Printed `eval:v06:exact-32b0b5e0e7691641` (`adjective:nominative:singular:feminine:any:short:positive`): expected `до́брѣ`, top-1 `добра̀`.
- Printed `eval:v06:exact-3dbd30879a2731eb` (`adjective:accusative:singular:neuter:inanimate:short:positive`): expected `ма́лѡ`, top-1 `ма́ло`.
- Printed `eval:v06:exact-3ebf5549ab4f08a3` (`noun:nominative:plural:inanimate`): expected `бѡлѣ́зни`, top-1 `болѣ́зни`.
- Printed `eval:v06:exact-42d54c80b1e9c917` (`noun:accusative:singular:animate`): expected `ра̑бъ`, top-1 `раба̀`.
- Printed `eval:v06:exact-44358b64f9d85332` (`noun:accusative:plural:inanimate`): expected `си̑лы`, top-1 `си́лы`.
- Printed `eval:v06:exact-4939a19e5ab27dde` (`future:third:singular`): expected `и҆зба́витъ`, top-1 `и҆зба́ви`.
- Printed `eval:v06:exact-4ffbc7e7b44507f2` (`adjective:genitive:singular:neuter:any:short:positive`): expected `жи̑ва`, top-1 `жи́ва`.
- Printed `eval:v06:exact-50c0d10124bc5a8f` (`aorist:third:singular`): expected `ста́`, top-1 `ста̀`.
- Printed `eval:v06:exact-56b5d1fe76029244` (`noun:genitive:singular:animate`): expected `лица̑`, top-1 `ли́ца`.
- Printed `eval:v06:exact-5e52ffe3a03b4b11` (`noun:genitive:singular:inanimate`): expected `си̑лы`, top-1 `си́лы`.
- Printed `eval:v06:exact-614fe964ac48a0dd` (`imperative:second:singular`): expected `и҆дѝ`, top-1 `и҆́деши`.
- Printed `eval:v06:exact-6996fc14230302dc` (`noun:locative:singular:inanimate`): expected `тьмѣ̀`, top-1 `тмѣ̀`.
- Printed `eval:v06:exact-70859d2e90da66bc` (`aorist:third:singular`): expected `прїѧ́тъ`, top-1 `прїѧ̀`.
- Printed `eval:v06:exact-78d7153e6e0f1e23` (`noun:accusative:plural:animate`): expected `лица̑`, top-1 `ли́ца`.
- Printed `eval:v06:exact-79ad51711e4ee737` (`noun:genitive:singular:inanimate`): expected `бра̑ни`, top-1 `бра́ни`.
- Printed `eval:v06:exact-7def29284314daa1` (`adjective:nominative:singular:neuter:any:short:positive`): expected `ма́лѡ`, top-1 `ма́ло`.
- Printed `eval:v06:exact-85d3804bb10296b1` (`adjective:nominative:singular:feminine:any:short:positive`): expected `блага̀`, top-1 `бла̑га`.
- Printed `eval:v06:exact-8ee7fc72e685652e` (`noun:dative:singular:inanimate`): expected `бѡлѣ́зни`, top-1 `болѣ́зни`.
- Printed `eval:v06:exact-95a1539bff67061b` (`adjective:genitive:singular:neuter:any:short:positive`): expected `жива̀`, top-1 `жи́ва`.
- Printed `eval:v06:exact-9bae23a5cc72b118` (`aorist:third:dual`): expected `рѣ́сте`, top-1 `рѣ́ста`.
- Printed `eval:v06:exact-9cb2f89f209d1b7f` (`imperative:second:plural`): expected `творитѐ`, top-1 `твори́те`.
- Printed `eval:v06:exact-a52b4e1dc0f77849` (`noun:accusative:singular:animate`): expected `ра́бъ`, top-1 `раба̀`.
- Printed `eval:v06:exact-a6f75ecf9d155583` (`adjective:nominative:plural:neuter:any:short:positive`): expected `жива̀`, top-1 `жи́ва`.
- Printed `eval:v06:exact-a9355c2daae719d5` (`noun:genitive:singular:inanimate`): expected `словесѝ`, top-1 `сло́ва`.
- Printed `eval:v06:exact-af9553144d42df99` (`participle:past:passive:nominative:singular:feminine:any:short:positive`): expected `дана̀`, top-1 `да́на`.
- Printed `eval:v06:exact-b058ee586f2816e8` (`noun:genitive:singular:inanimate`): expected `бѡлѣ́зни`, top-1 `болѣ́зни`.
- Printed `eval:v06:exact-c123b072b6cbc74e` (`noun:accusative:plural:inanimate`): expected `бѡлѣ́зни`, top-1 `болѣ́зни`.
- Printed `eval:v06:exact-c906e786922f2016` (`adjective:accusative:plural:neuter:inanimate:short:positive`): expected `блага̀`, top-1 `бла̑га`.
- Printed `eval:v06:exact-cb89ed83a7628524` (`adjective:genitive:singular:masculine:any:short:positive`): expected `блага̀`, top-1 `бла̑га`.
- Printed `eval:v06:exact-cccf40d2cf68ed75` (`aorist:third:singular`): expected `ста̑`, top-1 `ста̀`.
- Printed `eval:v06:exact-d0b0242925123bef` (`noun:nominative:plural:inanimate`): expected `ри̑зы`, top-1 `ри́зы`.
- Printed `eval:v06:exact-de69e9e2163ace8a` (`noun:locative:singular:inanimate`): expected `кораблѝ`, top-1 `корабли̑`.
- Printed `eval:v06:exact-e2353dc398fa9ef6` (`noun:genitive:singular:inanimate`): expected `стада̀`, top-1 `ста́да`.
- Printed `eval:v06:exact-eb79ebbd741a24c4` (`noun:accusative:plural:inanimate`): expected `ри̑зы`, top-1 `ри́зы`.
- Printed `eval:v06:exact-ec9f03ea8db3f50f` (`aorist:first:singular`): expected `Рѣ́хъ`, top-1 `рѣ́къ`.
- Printed `eval:v06:exact-f3432698526965a8` (`noun:nominative:singular:animate`): expected `ра̑бъ`, top-1 `ра́бъ`.
- Printed `eval:v06:exact-f4d20f271a374033` (`adjective:nominative:singular:feminine:any:short:positive`): expected `жива̀`, top-1 `жи́ва`.
- Printed `eval:v06:exact-f9412ea965195abe` (`present:second:plural`): expected `творитѐ`, top-1 `твори́те`.
- Printed `eval:v06:exact-fa7076f43321c9ce` (`adjective:nominative:plural:neuter:any:short:positive`): expected `жи̑ва`, top-1 `жи́ва`.
- Printed `eval:v06:lexical-8897d4e2e9679e01` (`lexical-form`): expected `и҆лїѝ`, top-1 `и҆лїа̀`.
- Printed `eval:v06:lexical-c36c119a628a329c` (`lexical-form`): expected `послꙋ́ша`, top-1 `послꙋ́шати`.
- Printed `eval:v06:manual:resha` (`aorist:third:plural`): expected `рѣ́ша`, top-1 `реко́ша`.
- Printed `eval:v06:noshch-gen-sg` (`noun:genitive:singular:inanimate`): expected `нощѝ`, top-1 `но́щи`.
- Printed `eval:v06:slovo-gen-sg` (`noun:genitive:singular:inanimate`): expected `словесѐ`, top-1 `сло́ва`.
- Printed `eval:v06:strana-gen-sg` (`noun:genitive:singular:inanimate`): expected `страны̑`, top-1 `страны̀`.
- Printed `eval:v07:variant-09f6b1e7fbe3a545` (`noun:accusative:plural:inanimate`): expected `грѣхѝ`, top-1 `грѣ́хи`.
- Printed `eval:v07:variant-213e7569dddbe2b5` (`noun:locative:singular:inanimate`): expected `ѻ҆лтарѝ`, top-1 `ѻ҆лтари̑`.
- Printed `eval:v07:variant-369f932e734ff9e7` (`noun:instrumental:plural:inanimate`): expected `ᲂу҆сты̑`, top-1 `ѹ҆́сты̑`.
- Printed `eval:v07:variant-4b91c8d4008feaa0` (`noun:genitive:singular:inanimate`): expected `любвѐ`, top-1 `любве́`.
- Printed `eval:v07:variant-4ed0042b1a68e15f` (`noun:accusative:plural:animate`): expected `врагѝ`, top-1 `враги̑`.
- Printed `eval:v07:variant-52b05b9208c3a6b2` (`imperative:second:singular`): expected `Помѧнѝ`, top-1 `помѧни́`.
- Printed `eval:v07:variant-601b77a623ca9eaf` (`indeclinable`): expected `ᲂу҆жѐ`, top-1 `оу҆же́`.
- Printed `eval:v07:variant-6c1255166a61a421` (`aorist:third:singular`): expected `созда̀`, top-1 `созда́`.
- Printed `eval:v07:variant-7a034e348d0738c3` (`aorist:third:singular`): expected `собра̀`, top-1 `собра́`.
- Printed `eval:v07:variant-8fc3f8466c081cab` (`aorist:third:singular`): expected `воста̀`, top-1 `воста́`.
- Printed `eval:v07:variant-924bfd9ad3f7f425` (`aorist:third:singular`): expected `поразѝ`, top-1 `порази́`.
- Printed `eval:v07:variant-979c59365a14a1a3` (`aorist:third:singular`): expected `возопѝ`, top-1 `возопи́`.
- Printed `eval:v07:variant-9faaff4bafd1c794` (`aorist:third:singular`): expected `написа̀`, top-1 `написа́`.
- Printed `eval:v07:variant-b06686b3ddba8e8e` (`noun:genitive:plural:animate`): expected `ча̑дъ`, top-1 `ча́дъ`.
- Printed `eval:v07:variant-b8c1513b6bda88c9` (`indeclinable`): expected `ѹ҆жѐ`, top-1 `оу҆же́`.
- Printed `eval:v07:variant-ca80624a2c268f53` (`imperative:second:singular`): expected `сохранѝ`, top-1 `сохрани́`.
- Printed `eval:v07:variant-cb34b395c9a499b6` (`aorist:third:singular`): expected `поживѐ`, top-1 `поживе́`.
- Printed `eval:v07:variant-f714d916e66f6a5f` (`noun:genitive:singular:inanimate`): expected `зла̑та`, top-1 `зла́та`.
- Printed `eval:v07:variant-f9a8c4cf53df071f` (`noun:nominative:plural:animate`): expected `человѣ̑ки`, top-1 `человѣ́ки`.
- Printed `eval:v07:variant-fc9821f7609714ae` (`noun:instrumental:plural:inanimate`): expected `ѹ҆сты̀`, top-1 `ѹ҆́сты̑`.
- Printed `eval:v07:variant-10c14f5a929ecc82` (`noun:genitive:singular:inanimate`): expected `стѣны̀`, top-1 `стѣ́ны`.
- Printed `eval:v07:variant-8df2b6ab8a91ffa0` (`noun:genitive:plural:inanimate`): expected `ꙗ҆зы̑къ`, top-1 `ѧ҆зы́къ`.
- Printed `eval:v07:variant-90b0d8951b2488ed` (`aorist:third:singular`): expected `поѧ̀`, top-1 `поѧ́`.
- Printed `eval:v07:variant-9f52884b1d8373d0` (`noun:genitive:singular:inanimate`): expected `горы̀`, top-1 `го́ры`.
- Printed `eval:v07:variant-a826101106287cb0` (`noun:genitive:singular:inanimate`): expected `рѣкѝ`, top-1 `рѣ́ки`.
- Printed `eval:v07:variant-c44fbc8c0073c063` (`aorist:third:singular`): expected `ѿпꙋстѝ`, top-1 `ѿпꙋсти́`.
- Printed `eval:v07:variant-ecc08addaf23ce48` (`aorist:third:singular`): expected `прострѐ`, top-1 `простре́`.
- Printed `eval:v07:variant-f0dda60622d21b36` (`aorist:third:singular`): expected `посла́`, top-1 `посла̀`.
- Printed `eval:v07:variant-f827d8f7c574fe77` (`aorist:third:singular`): expected `сꙋдѝ`, top-1 `сꙋди́`.
- Printed `eval:v07:variant-fa5da20a40757fc4` (`noun:accusative:plural:inanimate`): expected `десѧти̑ны`, top-1 `десѧти́ны`.
- Printed `eval:v07:variant-ff0719a52d611730` (`aorist:third:singular`): expected `вопросѝ`, top-1 `вопроси́`.
- Printed `eval:v07:variant-249ff5c20632363a` (`noun:genitive:singular:inanimate`): expected `бра̑шна`, top-1 `бра́шна`.
- Printed `eval:v07:variant-3b2eb3cfafa850e4` (`noun:nominative:singular:animate`): expected `жена́`, top-1 `жена̀`.
- Printed `eval:v07:variant-714e96e847d769ec` (`noun:locative:singular:inanimate`): expected `ра́зꙋмѣ`, top-1 `разꙋмѣ́`.
- Printed `eval:v07:variant-88f72685d9323d5d` (`imperative:second:plural`): expected `сꙋди́те`, top-1 `сꙋ́дите`.
- Printed `eval:v07:variant-8994884681e33a81` (`noun:nominative:plural:animate`): expected `ры̑бы`, top-1 `ры́бы`.
- Printed `eval:v07:variant-d27aebd153616ed5` (`noun:accusative:plural:inanimate`): expected `да̑ни`, top-1 `да́ни`.
- Printed `eval:v07:variant-334f7c3d9fbe2389` (`noun:locative:singular:animate`): expected `царѝ`, top-1 `цари̑`.
- Printed `eval:v11:preposition:vosled` (`indeclinable`): expected `в̾слѣ́дъ`, top-1 `вослѣ́дъ`.
- Printed `eval:v15:noun:tsar-nominative-plural-tsarie` (`noun:nominative:plural:animate`): expected `ца́рїе`, top-1 `цари̑`.
- Printed `eval:v17:noun:levit-accusative-plural-omega` (`noun:accusative:plural:animate`): expected `леѵі́тѡвъ`, top-1 `леѵі́ты`.
- Printed `eval:v24:noun:rab-wide-omega-accusative-plural` (`noun:accusative:plural:animate`): expected `рабѡ́въ`, top-1 `рабы̑`.
- Printed `eval:v07:05506371f37c63d6` (`aorist:third:singular`): expected `бѣжа̀`, top-1 `бѣжа́`.
- Printed `eval:v07:0bc7c0760a8f536c` (`adjective:accusative:plural:neuter:any:short:positive`): expected `Бла́га`, top-1 `бла̑га`.
- Printed `eval:v07:0cc4625dd2086f15` (`noun:genitive:singular:inanimate`): expected `ро́дꙋ`, top-1 `ро́да`.
- Printed `eval:v07:0e2a9ef0e3118fa9` (`pronoun:dative:plural:neuter:none:any`): expected `Си́мъ`, top-1 `си̑мъ`.
- Printed `eval:v07:12d883c01f5428e4` (`noun:instrumental:plural:inanimate`): expected `ᲂу҆сты̀`, top-1 `ѹ҆́сты̑`.
- Printed `eval:v07:264e1313b5a9fce0` (`future:second:plural`): expected `Бꙋ́дите`, top-1 `бꙋ́дете`.
- Printed `eval:v07:3ae1ab6559aeee91` (`noun:accusative:plural:inanimate`): expected `ᲂу҆ста̀`, top-1 `ѹ҆ста̀`.
- Printed `eval:v07:3ed087dcfeef0e16` (`present:first:singular`): expected `глаго́лꙋ`, top-1 `глаго́лю`.
- Printed `eval:v07:44aeeb943ebc7c26` (`indeclinable`): expected `Ѹ҆`, top-1 `оу҆`.
- Printed `eval:v07:465e4d0cb2d9e2cc` (`noun:genitive:singular:inanimate`): expected `любвѝ`, top-1 `любве́`.
- Printed `eval:v07:4d5b70b162c95606` (`indeclinable`): expected `Ѹ҆же́`, top-1 `оу҆же́`.
- Printed `eval:v07:52a51adda1edae2c` (`imperfect:third:plural`): expected `и҆мѧ́хꙋ`, top-1 `и҆мѣ́ѧхꙋ`.
- Printed `eval:v07:5b2b9d69eb8735e0` (`indeclinable`): expected `ᲂу҆́же`, top-1 `оу҆же́`.
- Printed `eval:v07:63fbc155df7d7d1a` (`noun:dative:singular:inanimate`): expected `сло́вꙋ`, top-1 `словесѝ`.
- Printed `eval:v07:645ba104b4d03aef` (`future:second:singular`): expected `ѡ҆брѧ́щеши`, top-1 `ѡ҆брѧ́щете`.
- Printed `eval:v07:6d0cb86a0d9aee04` (`imperative:second:plural`): expected `рече́те`, top-1 `рцы́те`.
- Printed `eval:v07:7748137afb8161c2` (`lexical-form`): expected `посла́ти`, top-1 `по́сланъ`.
- Printed `eval:v07:7da0b9f08c2294aa` (`noun:genitive:singular:inanimate`): expected `ᲂу҆́тра`, top-1 `ѹ҆́тра`.
- Printed `eval:v07:88de1a03e5c7bd6a` (`noun:locative:singular:inanimate`): expected `ро́дꙋ`, top-1 `ро́дѣ`.
- Printed `eval:v07:922e6f2723af3ee9` (`noun:nominative:singular:inanimate`): expected `Любы̀`, top-1 `любо́вь`.
- Printed `eval:v07:9261a7d8e2639922` (`indeclinable`): expected `ᲂу҆`, top-1 `оу҆`.
- Printed `eval:v07:94d144f9d3ff2a2e` (`noun:genitive:plural:inanimate`): expected `ꙗ҆зы́къ`, top-1 `ѧ҆зы́къ`.
- Printed `eval:v07:95f004c8142282be` (`pronoun:dative:singular:neuter:none:any`): expected `немꙋ́же`, top-1 `є҆мꙋ́же`.
- Printed `eval:v07:9a678cab7ac6aafd` (`lexical-form`): expected `взы́ти`, top-1 `взыдо́ша`.
- Printed `eval:v07:a24b407b362af140` (`noun:genitive:singular:inanimate`): expected `ꙗ҆зы́ка`, top-1 `ѧ҆зы́ка`.
- Printed `eval:v07:b345a6c547f9d2a7` (`noun:accusative:singular:inanimate`): expected `ᲂу҆́мъ`, top-1 `ѹ҆́мъ`.
- Printed `eval:v07:c105b5928b76e40e` (`pronoun:genitive:singular:feminine:none:any`): expected `неѧ́же`, top-1 `є҆ѧ́же`.
- Printed `eval:v07:cb4107f699fcc5d7` (`imperfect:third:singular`): expected `и҆мѧ́ше`, top-1 `и҆мѣ́ѧше`.
- Printed `eval:v07:cc64afc6ffc35f45` (`noun:locative:plural:inanimate`): expected `словесѣ́хъ`, top-1 `словесѝ`.
- Printed `eval:v07:cf6bac0302bf3133` (`lexical-form`): expected `сꙋди́ти`, top-1 `сꙋди́`.
- Printed `eval:v07:d5495b7b716d9f44` (`noun:genitive:singular:inanimate`): expected `во́лѧ`, top-1 `во́ли`.
- Printed `eval:v07:e48eca6f14f7c825` (`noun:accusative:singular:inanimate`): expected `Любы̀`, top-1 `любо́вь`.
- Printed `eval:v07:e72eb04dce8a642d` (`noun:nominative:singular:inanimate`): expected `ᲂу҆́мъ`, top-1 `ѹ҆́мъ`.
- Printed `eval:v07:f2da2578054f271e` (`pronoun:dative:plural:masculine:none:any`): expected `Си́мъ`, top-1 `си̑мъ`.
- Printed `eval:v07:f6a8afc17586cb5c` (`noun:genitive:singular:inanimate`): expected `тьмы̀`, top-1 `тмы̀`.
- Printed `eval:v07:fdcd7d9ef5505c86` (`pronoun:dative:plural:feminine:none:any`): expected `Си́мъ`, top-1 `си̑мъ`.
- Printed `eval:v12:fb522502444b462d` (`future:second:singular`): expected `ᲂу҆́мреши`, top-1 `ѹ҆́мреши`.
- Printed `eval:v12:5ddb6c02cd48d6bf` (`aorist:third:singular`): expected `ᲂу҆́мре`, top-1 `ѹ҆́мре`.
- Printed `eval:v14:140e49c2040c5a43` (`noun:nominative:dual:inanimate`): expected `ѹ҆стнѣ̀`, top-1 `ᲂу҆стнѣ̀`.
- Printed `eval:v14:d8c92587f265eb94` (`aorist:third:singular`): expected `ѹ҆твердѝ`, top-1 `ᲂу҆твердѝ`.
- Printed `eval:v14:36d45eb16f577d41` (`aorist:first:singular`): expected `ѹ҆гото́вахъ`, top-1 `ᲂу҆гото́вахъ`.
- Printed `eval:v14:99a5e5294b3701a4` (`present:third:singular`): expected `ѹ҆гото́витъ`, top-1 `ᲂу҆гото́витъ`.
- Printed `eval:v14:bfa2cdd5694d8467` (`noun:nominative:singular:animate`): expected `ѻ҆зїи́лъ`, top-1 `ѻ҆зіи́лъ`.

## Inherited OCS evaluation

The accepted registry contains 2 explicit identity mappings and 3 transformed mappings. The structural Productive-policy admission check has 5 true-positive admissions, 0 false-positive admissions, and precision 10000/10,000 basis points on the reviewed gold registry. This is a policy guard, not an independent estimate of automatic alignment quality.


## Inherited cells by identity/transformed mapping

| Slice | Returned | Top-1 | Top-k | Abstained | Total |
|---|---:|---:|---:|---:|---:|

## Inherited cells by morphological system

| Slice | Returned | Top-1 | Top-k | Abstained | Total |
|---|---:|---:|---:|---:|---:|

## Inherited cells by confidence band

| Slice | Returned | Top-1 | Top-k | Abstained | Total |
|---|---:|---:|---:|---:|---:|

Returned inherited confidence: n/a basis points; empirical exact expanded agreement: n/a basis points; absolute descriptive calibration gap: n/a basis points.

## Abstention

No held-out row abstained in this reviewed fixture. Unsupported and missing-metadata behavior is exercised separately by paradigms and guard witnesses.

## Interpretation and limitations

- The corpus passages are evaluation-only; they are not generation inputs.
- The current real-text slice is intentionally small and reports counts, not statistical confidence.
- No legally cleared, machine-readable non-biblical Synodal liturgical corpus is currently pinned; catalog-only and unresolved-rights editions are intentionally excluded from held-out scoring.
- Productive liturgical rendering abstains when accent metadata is absent.
- One participle and one analytic perfect are covered by independent corpus witnesses; other analytic constructions remain typed unit fixtures until their lexical registries grow.
- Abbreviation, numeral, malformed-mark, and hostile-Unicode regressions are deterministic utility fixtures, not corpus-accuracy rows.
- Gold admission precision is a structural policy check over the reviewed registry, not an independently estimated automatic-alignment precision.
- The single inherited held-out cell is insufficient to assess confidence calibration; the reported gap is descriptive only.
