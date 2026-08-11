# Synodal evaluation

Target recension: `synodal-russian`. Fixture: pinned passage-held-out Ponomar Elizabeth Bible rows across Matthew, Acts, Daniel, Apocalypse, Amos, and Deuteronomy (1187 held-out token cells).

| Metric | Returned | Top-1 | Top-k | Abstained | Total |
|---|---:|---:|---:|---:|---:|
| Expanded | 1187 | 1163 | 1187 | 0 | 1187 |
| Printed | 1187 | 1118 | 1187 | 0 | 1187 |

Analytic phrases: expanded 5/5, printed 5/5 (5 held-out phrases).

Typed abbreviations: top-1 55/64, top-k 64/64 (64 held-out contractions; reverse lookup also required).

Exact registry round trips (top-k, including reviewed variants): expanded 1876/1876, printed 1876/1876.

Masked cells: expanded 505/506, printed 505/506. Leave-one-Synodal-lexeme-out inherited cells: expanded 0/0, printed 0/0.

Accent agreement: 1091/1091 accent-bearing rows.

Inherited evidence contributed 0/1 returned held-out cells, with 0/0 exact expanded forms. The reviewed alignment registry has 5 accepted mappings, 5 aligned target lexemes, and 1 rejected negative controls.

## Expanded accuracy by generation policy

| Slice | Returned | Top-1 | Top-k | Abstained | Total |
|---|---:|---:|---:|---:|---:|
| `exploratory` | 1187 | 1163 | 1187 | 0 | 1187 |
| `productive` | 1187 | 1163 | 1187 | 0 | 1187 |
| `strict` | 1187 | 1163 | 1187 | 0 | 1187 |

## Expanded accuracy by attestation status

| Slice | Returned | Top-1 | Top-k | Abstained | Total |
|---|---:|---:|---:|---:|---:|
| `attested` | 1084 | 1061 | 1084 | 0 | 1084 |
| `expected-form-not-returned` | 0 | 0 | 0 | 0 | 0 |
| `predicted` | 103 | 102 | 103 | 0 | 103 |

## Expanded accuracy by morphological system

| Slice | Returned | Top-1 | Top-k | Abstained | Total |
|---|---:|---:|---:|---:|---:|
| `adjective` | 75 | 71 | 75 | 0 | 75 |
| `aorist` | 14 | 13 | 14 | 0 | 14 |
| `determiner` | 13 | 12 | 13 | 0 | 13 |
| `future` | 26 | 25 | 26 | 0 | 26 |
| `imperative` | 29 | 28 | 29 | 0 | 29 |
| `imperfect` | 3 | 3 | 3 | 0 | 3 |
| `indeclinable` | 64 | 64 | 64 | 0 | 64 |
| `infinitive` | 4 | 4 | 4 | 0 | 4 |
| `l-participle` | 8 | 8 | 8 | 0 | 8 |
| `lexical-form` | 408 | 408 | 408 | 0 | 408 |
| `noun` | 323 | 312 | 323 | 0 | 323 |
| `numeral` | 19 | 18 | 19 | 0 | 19 |
| `participle` | 23 | 23 | 23 | 0 | 23 |
| `past` | 73 | 69 | 73 | 0 | 73 |
| `present` | 36 | 36 | 36 | 0 | 36 |
| `pronoun` | 69 | 69 | 69 | 0 | 69 |

## Expanded accuracy by provenance path

| Slice | Returned | Top-1 | Top-k | Abstained | Total |
|---|---:|---:|---:|---:|---:|
| `exact-synodal-attestation` | 1083 | 1061 | 1083 | 0 | 1083 |
| `synodal-normative-table` | 83 | 81 | 83 | 0 | 83 |
| `synodal-productive-rule` | 21 | 21 | 21 | 0 | 21 |

## Expanded accuracy by regularity

| Slice | Returned | Top-1 | Top-k | Abstained | Total |
|---|---:|---:|---:|---:|---:|
| `closed-class-held-out-exact-cell` | 25 | 25 | 25 | 0 | 25 |
| `closed-class-held-out-normative` | 30 | 30 | 30 | 0 | 30 |
| `exact-held-out-lexical` | 384 | 384 | 384 | 0 | 384 |
| `irregular` | 7 | 7 | 7 | 0 | 7 |
| `irregular-held-out` | 10 | 9 | 10 | 0 | 10 |
| `irregular-participle` | 1 | 1 | 1 | 0 | 1 |
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
| `v06-held-out-lexical-identity` | 61 | 61 | 61 | 0 | 61 |
| `v06-held-out-manual-exact` | 13 | 12 | 13 | 0 | 13 |
| `v06-held-out-normative-table` | 5 | 5 | 5 | 0 | 5 |
| `v06-held-out-source-typed-exact` | 485 | 468 | 485 | 0 | 485 |

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
- Expanded `eval:v06:exact-42beb1ca352eb0f0` (`past:third:singular`): expected `хождаше`, top-1 `ходи`.
- Expanded `eval:v06:exact-42d54c80b1e9c917` (`noun:accusative:singular:animate`): expected `рабъ`, top-1 `раба`.
- Expanded `eval:v06:exact-4939a19e5ab27dde` (`future:third:singular`): expected `избавитъ`, top-1 `избави`.
- Expanded `eval:v06:exact-614fe964ac48a0dd` (`imperative:second:singular`): expected `иди`, top-1 `идеши`.
- Expanded `eval:v06:exact-6849b215c9f1b25b` (`past:second:singular`): expected `прїѧтъ`, top-1 `прїѧ`.
- Expanded `eval:v06:exact-70859d2e90da66bc` (`past:third:singular`): expected `прїѧтъ`, top-1 `прїѧ`.
- Expanded `eval:v06:exact-7def29284314daa1` (`adjective:nominative:singular:neuter:any:short:positive`): expected `малѡ`, top-1 `мало`.
- Expanded `eval:v06:exact-8ee7fc72e685652e` (`noun:dative:singular:inanimate`): expected `бѡлѣзни`, top-1 `болѣзни`.
- Expanded `eval:v06:exact-9bae23a5cc72b118` (`past:third:dual`): expected `рѣсте`, top-1 `рѣста`.
- Expanded `eval:v06:exact-a52b4e1dc0f77849` (`noun:accusative:singular:animate`): expected `рабъ`, top-1 `раба`.
- Expanded `eval:v06:exact-a9355c2daae719d5` (`noun:genitive:singular:inanimate`): expected `словеси`, top-1 `слова`.
- Expanded `eval:v06:exact-b058ee586f2816e8` (`noun:genitive:singular:inanimate`): expected `бѡлѣзни`, top-1 `болѣзни`.
- Expanded `eval:v06:exact-c123b072b6cbc74e` (`noun:accusative:plural:inanimate`): expected `бѡлѣзни`, top-1 `болѣзни`.
- Expanded `eval:v06:manual:resha` (`aorist:third:plural`): expected `рѣша`, top-1 `рекоша`.
- Expanded `eval:v06:slovo-gen-sg` (`noun:genitive:singular:inanimate`): expected `словесе`, top-1 `слова`.
- Printed `eval:acts-9-9-tri` (`numeral:cardinal:nominative:plural:masculine:inanimate`): expected `трѝ`, top-1 `трїѐ`.
- Printed `eval:v04:reshchi-aorist-acute` (`aorist:third:singular`): expected `рече́`, top-1 `речѐ`.
- Printed `eval:v04:ves-masc-nom-pl` (`determiner:nominative:plural:masculine:animate:short:positive`): expected `всѝ`, top-1 `вси́`.
- Printed `eval:v05:interjection-se-acute-before-li` (`indeclinable`): expected `се́`, top-1 `сѐ`.
- Printed `eval:v05:mnog-nominative-plural` (`adjective:nominative:plural:masculine:animate:short:positive`): expected `мнѡ́ги`, top-1 `Мно́зи`.
- Printed `eval:v05:on-capital-plural` (`pronoun:nominative:plural:masculine:third:any`): expected `Ѻ҆ни́`, top-1 `ѻ҆нѝ`.
- Printed `eval:v05:slovo-slovesa` (`noun:accusative:plural:inanimate`): expected `словеса̀`, top-1 `сло́ва`.
- Printed `eval:v05:syn-dative-plural` (`noun:dative:plural:animate`): expected `сынѡ́мъ`, top-1 `сыновѡ́мъ`.
- Printed `eval:v05:ves-genitive-masculine` (`determiner:genitive:singular:masculine:inanimate:short:positive`): expected `всегѡ̀`, top-1 `всего̀`.
- Printed `eval:v05:ves-instrumental-feminine` (`determiner:instrumental:singular:feminine:inanimate:short:positive`): expected `все́ю`, top-1 `Все́ю`.
- Printed `eval:v05:zapoved-omega` (`noun:genitive:singular:inanimate`): expected `за́пѡвѣди`, top-1 `за́повѣди`.
- Printed `eval:v06:dom-ins-sg` (`noun:instrumental:singular:inanimate`): expected `домѡ́мъ`, top-1 `до́момъ`.
- Printed `eval:v06:exact-04cb9846b4c8b6db` (`adjective:genitive:singular:masculine:any:short:positive`): expected `жи̑ва`, top-1 `жи́ва`.
- Printed `eval:v06:exact-0642d378eeb6d2a1` (`noun:genitive:plural:animate`): expected `ра̑бъ`, top-1 `ра́бъ`.
- Printed `eval:v06:exact-06a0b24fde2ef68e` (`participle:past:passive:nominative:singular:feminine:any:short:positive`): expected `дана̑`, top-1 `да́на`.
- Printed `eval:v06:exact-0cb4915a0058444d` (`adjective:genitive:singular:masculine:any:short:positive`): expected `жива̀`, top-1 `жи́ва`.
- Printed `eval:v06:exact-0eb4c7d99724ca76` (`adjective:nominative:singular:feminine:any:short:positive`): expected `жи̑ва`, top-1 `жи́ва`.
- Printed `eval:v06:exact-1a1c91bd40c26893` (`noun:nominative:plural:animate`): expected `лица̑`, top-1 `ли́ца`.
- Printed `eval:v06:exact-1bbb80834ad2a9e3` (`noun:nominative:plural:inanimate`): expected `кораблѝ`, top-1 `корабли̑`.
- Printed `eval:v06:exact-206a4cdecc4a38cd` (`past:third:singular`): expected `живѝ`, top-1 `жи́ви`.
- Printed `eval:v06:exact-228dbb9a4fc0132b` (`adjective:genitive:singular:neuter:any:short:positive`): expected `блага̀`, top-1 `бла̑га`.
- Printed `eval:v06:exact-29b870dbecaf8434` (`noun:nominative:plural:inanimate`): expected `стада̀`, top-1 `ста́да`.
- Printed `eval:v06:exact-2b01c52cf32cfd85` (`noun:genitive:plural:inanimate`): expected `бѡлѣ́зни`, top-1 `болѣ́зни`.
- Printed `eval:v06:exact-2b60f46e88d4822a` (`past:third:singular`): expected `родѝ`, top-1 `ро́ди`.
- Printed `eval:v06:exact-32b0b5e0e7691641` (`adjective:nominative:singular:feminine:any:short:positive`): expected `до́брѣ`, top-1 `добра̀`.
- Printed `eval:v06:exact-3dbd30879a2731eb` (`adjective:accusative:singular:neuter:inanimate:short:positive`): expected `ма́лѡ`, top-1 `ма́ло`.
- Printed `eval:v06:exact-3ebf5549ab4f08a3` (`noun:nominative:plural:inanimate`): expected `бѡлѣ́зни`, top-1 `болѣ́зни`.
- Printed `eval:v06:exact-42beb1ca352eb0f0` (`past:third:singular`): expected `хожда́ше`, top-1 `ходѝ`.
- Printed `eval:v06:exact-42d54c80b1e9c917` (`noun:accusative:singular:animate`): expected `ра̑бъ`, top-1 `раба̀`.
- Printed `eval:v06:exact-44358b64f9d85332` (`noun:accusative:plural:inanimate`): expected `си̑лы`, top-1 `си́лы`.
- Printed `eval:v06:exact-4939a19e5ab27dde` (`future:third:singular`): expected `и҆зба́витъ`, top-1 `и҆зба́ви`.
- Printed `eval:v06:exact-4ffbc7e7b44507f2` (`adjective:genitive:singular:neuter:any:short:positive`): expected `жи̑ва`, top-1 `жи́ва`.
- Printed `eval:v06:exact-50c0d10124bc5a8f` (`past:third:singular`): expected `ста́`, top-1 `ста̀`.
- Printed `eval:v06:exact-56b5d1fe76029244` (`noun:genitive:singular:animate`): expected `лица̑`, top-1 `ли́ца`.
- Printed `eval:v06:exact-5e52ffe3a03b4b11` (`noun:genitive:singular:inanimate`): expected `си̑лы`, top-1 `си́лы`.
- Printed `eval:v06:exact-614fe964ac48a0dd` (`imperative:second:singular`): expected `и҆дѝ`, top-1 `и҆́деши`.
- Printed `eval:v06:exact-6849b215c9f1b25b` (`past:second:singular`): expected `прїѧ́тъ`, top-1 `прїѧ̀`.
- Printed `eval:v06:exact-70859d2e90da66bc` (`past:third:singular`): expected `прїѧ́тъ`, top-1 `прїѧ̀`.
- Printed `eval:v06:exact-78d7153e6e0f1e23` (`noun:accusative:plural:animate`): expected `лица̑`, top-1 `ли́ца`.
- Printed `eval:v06:exact-79ad51711e4ee737` (`noun:genitive:singular:inanimate`): expected `бра̑ни`, top-1 `бра́ни`.
- Printed `eval:v06:exact-7def29284314daa1` (`adjective:nominative:singular:neuter:any:short:positive`): expected `ма́лѡ`, top-1 `ма́ло`.
- Printed `eval:v06:exact-85d3804bb10296b1` (`adjective:nominative:singular:feminine:any:short:positive`): expected `блага̀`, top-1 `бла̑га`.
- Printed `eval:v06:exact-8dd8e840c4c8884a` (`adjective:nominative:plural:masculine:any:short:positive`): expected `живѝ`, top-1 `жи́ви`.
- Printed `eval:v06:exact-8ee7fc72e685652e` (`noun:dative:singular:inanimate`): expected `бѡлѣ́зни`, top-1 `болѣ́зни`.
- Printed `eval:v06:exact-95a1539bff67061b` (`adjective:genitive:singular:neuter:any:short:positive`): expected `жива̀`, top-1 `жи́ва`.
- Printed `eval:v06:exact-9bae23a5cc72b118` (`past:third:dual`): expected `рѣ́сте`, top-1 `рѣ́ста`.
- Printed `eval:v06:exact-9cb2f89f209d1b7f` (`imperative:second:plural`): expected `творитѐ`, top-1 `твори́те`.
- Printed `eval:v06:exact-a52b4e1dc0f77849` (`noun:accusative:singular:animate`): expected `ра́бъ`, top-1 `раба̀`.
- Printed `eval:v06:exact-a6f75ecf9d155583` (`adjective:nominative:plural:neuter:any:short:positive`): expected `жива̀`, top-1 `жи́ва`.
- Printed `eval:v06:exact-a9355c2daae719d5` (`noun:genitive:singular:inanimate`): expected `словесѝ`, top-1 `сло́ва`.
- Printed `eval:v06:exact-af9553144d42df99` (`participle:past:passive:nominative:singular:feminine:any:short:positive`): expected `дана̀`, top-1 `да́на`.
- Printed `eval:v06:exact-b058ee586f2816e8` (`noun:genitive:singular:inanimate`): expected `бѡлѣ́зни`, top-1 `болѣ́зни`.
- Printed `eval:v06:exact-c123b072b6cbc74e` (`noun:accusative:plural:inanimate`): expected `бѡлѣ́зни`, top-1 `болѣ́зни`.
- Printed `eval:v06:exact-c906e786922f2016` (`adjective:accusative:plural:neuter:inanimate:short:positive`): expected `блага̀`, top-1 `бла̑га`.
- Printed `eval:v06:exact-cb89ed83a7628524` (`adjective:genitive:singular:masculine:any:short:positive`): expected `блага̀`, top-1 `бла̑га`.
- Printed `eval:v06:exact-cccf40d2cf68ed75` (`past:third:singular`): expected `ста̑`, top-1 `ста̀`.
- Printed `eval:v06:exact-d0b0242925123bef` (`noun:nominative:plural:inanimate`): expected `ри̑зы`, top-1 `ри́зы`.
- Printed `eval:v06:exact-de69e9e2163ace8a` (`noun:locative:singular:inanimate`): expected `кораблѝ`, top-1 `корабли̑`.
- Printed `eval:v06:exact-e2353dc398fa9ef6` (`noun:genitive:singular:inanimate`): expected `стада̀`, top-1 `ста́да`.
- Printed `eval:v06:exact-e5a8c9295ab08fc8` (`imperative:second:singular`): expected `живѝ`, top-1 `жи́ви`.
- Printed `eval:v06:exact-eb79ebbd741a24c4` (`noun:accusative:plural:inanimate`): expected `ри̑зы`, top-1 `ри́зы`.
- Printed `eval:v06:exact-f3432698526965a8` (`noun:nominative:singular:animate`): expected `ра̑бъ`, top-1 `ра́бъ`.
- Printed `eval:v06:exact-f4d20f271a374033` (`adjective:nominative:singular:feminine:any:short:positive`): expected `жива̀`, top-1 `жи́ва`.
- Printed `eval:v06:exact-f9412ea965195abe` (`present:second:plural`): expected `творитѐ`, top-1 `твори́те`.
- Printed `eval:v06:exact-fa7076f43321c9ce` (`adjective:nominative:plural:neuter:any:short:positive`): expected `жи̑ва`, top-1 `жи́ва`.
- Printed `eval:v06:manual:resha` (`aorist:third:plural`): expected `рѣ́ша`, top-1 `реко́ша`.
- Printed `eval:v06:noshch-gen-sg` (`noun:genitive:singular:inanimate`): expected `нощѝ`, top-1 `но́щи`.
- Printed `eval:v06:slovo-gen-sg` (`noun:genitive:singular:inanimate`): expected `словесѐ`, top-1 `сло́ва`.
- Printed `eval:v06:strana-gen-sg` (`noun:genitive:singular:inanimate`): expected `страны̑`, top-1 `страны̀`.

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
