# Synodal corpus coverage

## Type-disjoint holdout

This is the headline measure. The corpus partition split is passage-disjoint,
so an exact row sourced from a `source` passage closes its own held-out twin.
This slice holds out normalized *types* instead, selected by a content hash that
cannot be tuned, and is the only measurement here that shows generalisation to
surfaces the reviewed data has never seen. Coverage that arrives as
`exact-synodal-attestation` is a row citing the held-out type itself and is
memorisation; `synodal-normative-table`, `synodal-productive-rule` and
`synodal-irregular-override` coverage is generalisation. Corpus-wide top-k
rising while `generalised` stays flat is memorising.

- Held-out types present: 2924
- Held-out tokens: 44251

| Outcome | Tokens | Share of held-out |
|---|---:|---:|
| **generalised** (by rule) | 13740 | 3105 bp |
| memorised (exact row) | 14998 | 3389 bp |
| ambiguous | 265 | 59 bp |
| unresolved | 14443 | 3263 bp |
| top-k (any analysis) | 29797 | 6733 bp |
| top-1 | 11688 | 2641 bp |

### Held-out tokens by resolver status

| Resolver status | Tokens | Share of held-out |
|---|---:|---:|
| `abbreviation-expansion` | 794 | 179 bp |
| `ambiguous` | 265 | 59 bp |
| `exact-synodal-attestation` | 14998 | 3389 bp |
| `spelling-variant` | 11 | 2 bp |
| `synodal-irregular-override` | 253 | 57 bp |
| `synodal-normative-table` | 5867 | 1325 bp |
| `synodal-productive-rule` | 7620 | 1721 bp |
| `unresolved` | 14443 | 3263 bp |

### Held-out tokens by morphological system

A wave aimed at one system must be visible landing in that system.

| System | Held-out | Generalised | Memorised | Unresolved |
|---|---:|---:|---:|---:|
| `adjective` | 1721 | 1055 | 406 | 53 |
| `aorist` | 2289 | 730 | 1536 | 23 |
| `compound-cardinal-word` | 401 | 401 | 0 | 0 |
| `determiner` | 147 | 0 | 147 | 0 |
| `future` | 856 | 692 | 161 | 3 |
| `imperative` | 283 | 213 | 64 | 6 |
| `imperfect` | 340 | 68 | 270 | 2 |
| `indeclinable` | 1561 | 0 | 1534 | 27 |
| `infinitive` | 1536 | 1033 | 437 | 66 |
| `l-participle` | 18 | 16 | 0 | 2 |
| `lexical-form` | 1514 | 0 | 909 | 20 |
| `noun` | 13594 | 3924 | 8976 | 556 |
| `numeral` | 494 | 357 | 0 | 8 |
| `past-active-participle` | 238 | 19 | 199 | 20 |
| `past-passive-participle` | 49 | 2 | 45 | 2 |
| `present` | 1031 | 685 | 314 | 32 |
| `present-active-participle` | 369 | 357 | 0 | 12 |
| `pronoun` | 4210 | 4188 | 0 | 22 |
| `unresolved` | 13576 | 0 | 0 | 13565 |
| `verbal-noun` | 24 | 0 | 0 | 24 |

## Corpus-wide coverage

- Passages: 74130
- Tokens: 1313344
- Types: 57341
- Top-1 analyzed: 619481 (4716 bp)
- Top-k analyzed: 989543 (7534 bp)
- Ambiguous: 9792
- Unresolved: 322594

## Gap categories

| Category | Tokens |
|---|---:|
| `unknown-lexeme` | 309041 |
| `missing-declension-or-class` | 70 |
| `missing-verb-principal-part` | 54 |
| `unsupported-formation` | 36 |
| `missing-accent-or-orthographic-metadata` | 13393 |
| `ambiguity-or-spelling-variant` | 10999 |

## Coverage composition

Strict top-k counts tokens that have *any* analysis. These measures describe what
that coverage is made of, so recall cannot be bought with rows that commit to no
morphology, and so a fall in unique-reading counts can be attributed rather than
assumed. `morphology-free` tokens carry only `lexical-form` readings.
`lemma-unique` is not capped by syncretism the way top-1 is.

| Measure | Tokens | Share of top-k |
|---|---:|---:|
| morphologically typed | 939797 | 9497 bp |
| morphology-free | 49746 | 502 bp |
| lemma-unique | 979751 | 9901 bp |
| within-lexeme ambiguous (syncretism) | 357312 | 3610 bp |
| cross-lexeme ambiguous (homonymy) | 9792 | 98 bp |

## Estimated recovery routes

These are diagnostic estimates, not admitted lexical identities or guaranteed recoveries.

| Route | Tokens |
|---|---:|
| `exact-evidence` | 437 |
| `reviewed-class` | 70 |
| `reviewed-principal-part` | 54 |
| `abbreviation-registry` | 10677 |
| `spelling-variant` | 14600 |
| `unsupported-formation` | 36 |
| `ungrouped-unknown` | 297927 |

## Exploratory predictions over the unresolved remainder

Diagnostic only. These tokens have no reviewed reading; the corpus-free
segmentation tier (`SYN-PREDICT-VERB-SEGMENTATION-V1`, reachable only under
`GenerationPolicy::Exploratory`) can offer a typed hypothesis for them. They
never count toward strict top-k and no sealed floor reads this table; the
masked precision gate lives in `reports/synodal-prediction-precision.md`.

| Top prediction's system | Tokens |
|---|---:|
| `aorist` | 46809 |
| `imperative` | 27418 |
| `imperfect` | 6441 |
| `infinitive` | 9836 |
| `l-participle` | 7911 |
| `present` | 37380 |

| Confidence bucket (bp) | Tokens |
|---|---:|
| 0-2399 | 48799 |
| 2400-2999 | 19476 |
| 3000-3399 | 45230 |
| 3400+ | 22290 |

## Unresolved tokens by probable family

| Family diagnostic | Tokens | Documents | Route | Surfaces |
|---|---:|---:|---|---|
| `family:synodal:verb:vozvestiti` | 558 | 530 | `spelling-variant` | Возвѣсти́, Возвѣсти́сѧ, Возвѣсти́те, Возвѣсти́ти, Возвѣстѝ, Возвѣщꙋ̀, Возвѣщꙋ́, возвѣ́сти, возвѣсти́, возвѣсти́мъ, возвѣсти́сѧ, возвѣсти́те, возвѣсти́ти, возвѣсти́хъ, возвѣститѐ, возвѣстѝ, возвѣстѧ́тсѧ, возвѣстѧ́тъ, возвѣщꙋ̀, возвѣщꙋ́ |
| `diagnostic-family:господь` | 503 | 484 | `abbreviation-registry` | Гдⷭ҇а, Гдⷭ҇еви, Гдⷭ҇емъ, Гдⷭ҇не, Гдⷭ҇нѧ, Гдⷭ҇ꙋ, Госпо́дствꙋюще, гдⷪ҇ꙋ, гдⷭа, гдⷭеви, гдⷭемъ, гдⷭень, гдⷭи, гдⷭне, гдⷭней, гдⷭни, гдⷭню, гдⷭь, гдⷭѣ, гдⷭ҇ви, гдⷭ҇и́на, гдⷭ҇и́нъ, гдⷭ҇и́нꙋ, гдⷭ҇ней, гдⷭ҇немъ, гдⷭ҇нею, гдⷭ҇нима, гдⷭ҇ними, гдⷭ҇нь, гдⷭ҇скꙋю, гдⷭ҇ствꙋй, гдⷭ҇ьі҆́ѡвꙋ, гдⷭꙋ, го́сподемъ, госпо́дства, госпо́дствова, госпо́дствовасте, госпо́дствовати, госпо́дствоваше, госпо́дствѣ, госпо́дствꙋ, госпо́дствꙋемъ, госпо́дствꙋетъ, госпо́дствꙋйте, госпо́дствꙋютъ, госпо́дствꙋюща, госпо́дствꙋющихъ, госпо́дствꙋѧ, госпо́дствꙋѧй, госпо́дїй, госпо́дїємъ, госпо́дїѧми, господе́мъ, господи́нома, господи́номъ, господи́нꙋ |
| `family:synodal:verb:byti` | 369 | 361 | `spelling-variant` | Бы́вшей, Бы́вши, Бы́вшымъ, бы̀сть, бы̀ша, бы́вша, бы́вшаго, бы́вшагѡ, бы́вшаѧ, бы́вшей, бы́вшемъ, бы́вшемꙋ, бы́вшею, бы́вши, бы́вшихъ, бы́вшымъ, бы́вшыѧ, бы́вшїи, бы́вшїй, бы́вшїѧ, бы́вшꙋю, бы̑вша, бы̑вшаѧ, бывшїи, была́, сꙋ̑ща, сꙋ̑щаѧ, є҆́си, є҆́смы, є҆си, є҆смь |
| `family:synodal:verb:v06-vzeti` | 323 | 311 | `spelling-variant` | Взѧ́, Возми́те, взе́мши, взе́мшѧ, взѧ́, взѧ́ста, взѧ́сте, взѧ́ти, взѧ́то, взѧ́тое, взѧ́тїе, взѧ́тїѧ, взѧ́хомъ, взѧ̑та, взѧ̑ты, взѧта̀, взѧта̑, взѧты̀, возми́, возми́те, возми́тесѧ, возмꙋ́ |
| `family:synodal:noun:v12-lakot` | 253 | 146 | `spelling-variant` | Ла́коть, Ла̑кти, ла́коть, ла́ктей, ла́кти, ла̑кти, лакте́й |
| `family:synodal:noun:v06-673b2df93b4f89a8` | 249 | 241 | `spelling-variant` | Зна́менїѧ, Зна́мєнїѧ, зна́меньми, зна́менїемъ, зна́менїи, зна́менїихъ, зна́менїй, зна́менїє, зна́менїємъ, зна́менїѧ, зна́мєнїи, зна́мєнїѧ, знаме́нїе, знамє́нїѧ |
| `family:synodal:adverb:wikt-4f4b6240f36e` | 239 | 199 | `spelling-variant` | Го́ре, го́ре, горѣ́ |
| `family:synodal:pronoun:inyi` | 234 | 229 | `spelling-variant` | И҆ны̑ѧ, и҆́наго, и҆́нымъ, и҆́ныхъ, и҆́ныѧ, и҆на́го, и҆на́ѧ, и҆на̑ѧ, и҆ны́мъ, и҆ны́хъ, и҆ны́ѧ, и҆ны̑мъ, и҆ны̑ѧ |
| `family:synodal:adjective:wikt-e406458f5df6` | 227 | 220 | `spelling-variant` | вели́кагѡ, вели́ки, вели́цѣй, вели̑ка, вели̑ки, вели̑кїѧ, велика̀ |
| `family:synodal:noun:v12-nepravda` | 225 | 215 | `spelling-variant` | непра́вды, непра̑вды |
| `ungrouped:двєри` | 218 | 208 | `ungrouped-unknown` | Двє́ри, двє́ри |
| `ungrouped:ѡполчишасѧ` | 213 | 197 | `ungrouped-unknown` | ѡ҆полчи́шасѧ |
| `ungrouped:ꙗвисѧ` | 213 | 205 | `ungrouped-unknown` | Ꙗ҆ви́сѧ, ꙗ҆ви́сѧ |
| `family:synodal:verb:v12-slyshati` | 211 | 206 | `spelling-variant` | Слы́шавше, Слы́шавши, Слы́шавъ, слы́шавша, слы́шавше, слы́шавши, слы́шавшими, слы́шавшихъ, слы́шавъ |
| `ungrouped:нѣцыи` | 211 | 203 | `ungrouped-unknown` | Нѣ́цыи, нѣ́цыи |
| `ungrouped:предаде` | 207 | 203 | `ungrouped-unknown` | предаде́, предадѐ |
| `ungrouped:востани` | 204 | 183 | `ungrouped-unknown` | Воста́ни, воста́ни, востанѝ |
| `ungrouped:воньже` | 198 | 186 | `ungrouped-unknown` | Во́ньже, во́ньже |
| `family:synodal:pronoun:nekii` | 194 | 188 | `spelling-variant` | нѣ́каѧ, нѣ́кїй, нѣ̑каѧ |
| `ungrouped:жєртвы` | 193 | 187 | `ungrouped-unknown` | Жє́ртвы, жє́ртвы |
| `ungrouped:іꙋдеє` | 192 | 192 | `ungrouped-unknown` | І҆ꙋде́є, і҆ꙋде́є |
| `ungrouped:іꙋдеи` | 190 | 188 | `ungrouped-unknown` | і҆ꙋде́и |
| `ungrouped:ѹмретъ` | 186 | 169 | `ungrouped-unknown` | ѹ҆́мретъ |
| `ungrouped:лакѡтъ` | 185 | 116 | `ungrouped-unknown` | лакѡ́тъ |
| `ungrouped:нечестивыхъ` | 184 | 182 | `ungrouped-unknown` | нечести́выхъ |
| `ungrouped:иноплемєнницы` | 181 | 167 | `ungrouped-unknown` | И҆ноплемє́нницы, и҆ноплемє́нницы |
| `ungrouped:изведе` | 178 | 178 | `ungrouped-unknown` | И҆зведе́, и҆зведе́, и҆зведѐ |
| `ungrouped:оустъ` | 177 | 166 | `ungrouped-unknown` | ᲂу҆́стъ |
| `ungrouped:нача` | 176 | 174 | `ungrouped-unknown` | нача̀, нача́ |
| `family:synodal:determiner:sam` | 175 | 175 | `spelling-variant` | са́мымъ, са́мыѧ, са́мѣмъ, са́мꙋю, сама̀, сама́го, сама́гѡ, сама̑, самаго̀, самаго́, самагѡ̀, самагѡ́, само̀, само́й, само́мъ, самого̀, самогѡ̀, самогѡ́, самомꙋ̀, самомꙋ́, самы́мъ, самы̑ѧ, самѣ́мъ, самѣ̑мъ, самꙋ̀, самꙋ́ю |
| `ungrouped:єлицы` | 173 | 165 | `ungrouped-unknown` | Є҆ли́цы, є҆ли́цы, є҆ли̑цы, є҆лѝцы |
| `ungrouped:далече` | 171 | 169 | `ungrouped-unknown` | Дале́че, дале́че |
| `ungrouped:премꙋдрости` | 170 | 169 | `ungrouped-unknown` | премꙋ́дрости |
| `ungrouped:єзекіа` | 170 | 160 | `ungrouped-unknown` | Є҆зекі́а, є҆зекі́а |
| `ungrouped:ѻлтарѧ` | 169 | 145 | `ungrouped-unknown` | ѻлтарѧ̀, ѻ҆лтарѧ̀ |
| `ungrouped:паѵелъ` | 166 | 166 | `ungrouped-unknown` | Па́ѵелъ, па́ѵелъ |
| `ungrouped:творѧй` | 166 | 162 | `ungrouped-unknown` | Творѧ́й, творѧ́й |
| `ungrouped:воцарисѧ` | 164 | 164 | `ungrouped-unknown` | воцари́сѧ |
| `ungrouped:ѡправданїѧ` | 164 | 158 | `ungrouped-unknown` | Ѡ҆правда̑нїѧ, ѡ҆правда́нїѧ, ѡ҆правда̑нїѧ |
| `ungrouped:лꙋчше` | 163 | 161 | `ungrouped-unknown` | Лꙋ́чше, лꙋ́чше |
| `family:synodal:verb:v07-553feeb14b8be67e` | 157 | 151 | `spelling-variant` | со́брано, со́браныхъ, собери́, соберꙋ́, собра́нїе, собра́нїемъ, собра́нїи, собра́нїй, собра́нїю, собра́нїѧ, собра̑вша, собра̑нїѧ, собраша́сѧ |
| `ungrouped:богатство` | 157 | 154 | `ungrouped-unknown` | Бога́тство, бога́тство |
| `family:synodal:adjective:chist` | 156 | 149 | `spelling-variant` | чи́ста, чи́сты, чи́стыма, чи́стымъ, чи́стыѧ, чи́стїи, чи̑ста, чи̑стаѧ, чи̑сты, чи̑стымъ, чи̑стыѧ, чиста̀ |
| `ungrouped:блгⷭ҇венъ` | 156 | 155 | `abbreviation-registry` | Блгⷭ҇ве́нъ, Блгⷭ҇венъ, блгⷭ҇ве́нъ |
| `ungrouped:введе` | 155 | 153 | `ungrouped-unknown` | Введе́, введе́, введѐ |
| `ungrouped:каменїемъ` | 154 | 150 | `ungrouped-unknown` | Ка́менїемъ, ка́менїемъ |
| `ungrouped:сонмѡмъ` | 154 | 152 | `ungrouped-unknown` | со́нмѡмъ |
| `ungrouped:ѻвна` | 154 | 148 | `ungrouped-unknown` | ѻ҆вна̀, ѻ҆вна́, ѻ҆вна̑ |
| `ungrouped:слꙋжити` | 153 | 148 | `ungrouped-unknown` | слꙋжи́ти |
| `ungrouped:сꙋббѡты` | 153 | 138 | `ungrouped-unknown` | Сꙋббѡ̑ты, сꙋббѡ́ты, сꙋббѡ̑ты |
| `ungrouped:погꙋбити` | 152 | 150 | `ungrouped-unknown` | погꙋби́ти |
| `ungrouped:ѻрꙋжїе` | 150 | 142 | `ungrouped-unknown` | ѻ҆рꙋ́жїе |
| `ungrouped:навꙋходоносоръ` | 149 | 145 | `ungrouped-unknown` | Навꙋходоно́соръ, навꙋходоно́соръ |
| `ungrouped:ѹслыша` | 149 | 148 | `ungrouped-unknown` | Ѹ҆слы́ша, ѹ҆слы́ша |
| `family:synodal:pronoun:toi` | 148 | 140 | `spelling-variant` | То́, Тогѡ́, то́, того́, тогѡ́, томꙋ́, тоѧ́, ты̑ѧ |
| `ungrouped:царева` | 148 | 144 | `ungrouped-unknown` | царе́ва |
| `ungrouped:ѻньже` | 147 | 141 | `ungrouped-unknown` | ѻ҆́ньже |
| `family:synodal:numeral:vtoryi` | 145 | 142 | `spelling-variant` | Втора́ѧ, втора́ѧ, втора̑ѧ, второ́мꙋ, вторы́ми, вторы́мъ, вторы́хъ, вторы́ѧ, вторы̑мъ, вторѣ́й, вторѣ́мъ, вторꙋ́ю |
| `ungrouped:богѡвъ` | 145 | 137 | `ungrouped-unknown` | Богѡ́въ, богѡ́въ |
| `ungrouped:велїй` | 145 | 144 | `ungrouped-unknown` | Ве́лїй, ве́лїй |
| `ungrouped:врагѡвъ` | 144 | 140 | `ungrouped-unknown` | врагѡ́въ |
| `family:synodal:adjective:zlyi` | 143 | 139 | `spelling-variant` | ѕла̑, ѕлы̑, ѕлы̑мъ, ѕлы̑ѧ, ѕлѣ́е, ѕлѣ́йша, ѕлѣ́йшагѡ, ѕлѣ́йшихъ, ѕлѣ́йшїи, ѕлѣ̑йшаѧ |
| `family:synodal:numeral:pervyi` | 143 | 142 | `spelling-variant` | пе́рваѧ, пе́рвомꙋ, пе́рвыми, пе́рвымъ, пе́рвыѧ, пе́рвїи, пе́рвѣй, пе́рвѣмъ, пе́рвꙋю |
| `ungrouped:ѡдеснꙋю` | 143 | 141 | `ungrouped-unknown` | ѡ҆деснꙋ́ю |
| `family:synodal:noun:v07-4ea57e089679d12a` | 140 | 136 | `spelling-variant` | ско́рби, скорбѝ |
| `family:synodal:verb:v12-pogibnuti` | 140 | 140 | `spelling-variant` | Поги́бе, поги́бе, поги́блъ, поги́бнꙋти, погибо́хъ |
| `ungrouped:веселїе` | 140 | 138 | `ungrouped-unknown` | Весе́лїе, весе́лїе |
| `ungrouped:дверїй` | 140 | 127 | `ungrouped-unknown` | две́рїй |
| `ungrouped:сімѡнъ` | 140 | 140 | `ungrouped-unknown` | Сі́мѡнъ, сі́мѡнъ |
| `ungrouped:вєси` | 138 | 103 | `ungrouped-unknown` | вє́си |
| `ungrouped:сонъ` | 138 | 122 | `ungrouped-unknown` | Со́нъ, со̀нъ, со́нъ |
| `ungrouped:тꙋкъ` | 138 | 100 | `ungrouped-unknown` | Тꙋ́къ, тꙋ́къ |
| `ungrouped:ѻтцы` | 138 | 122 | `ungrouped-unknown` | ѻ҆тцы́, ѻ҆тцы̑ |
| `family:synodal:pronoun:on` | 137 | 135 | `spelling-variant` | ѻ҆нꙋ̀ |
| `ungrouped:восташа` | 136 | 134 | `ungrouped-unknown` | Воста́ша, воста́ша |
| `ungrouped:прѡчаѧ` | 136 | 136 | `ungrouped-unknown` | Прѡ́чаѧ, прѡ́чаѧ |
| `ungrouped:талантъ` | 136 | 101 | `ungrouped-unknown` | тала́нтъ, тала̑нтъ |
| `ungrouped:вѣнецъ` | 134 | 134 | `ungrouped-unknown` | Вѣне́цъ, вѣне́цъ |
| `ungrouped:велїимъ` | 132 | 131 | `ungrouped-unknown` | ве́лїимъ |
| `ungrouped:ѿкꙋдꙋ` | 131 | 123 | `ungrouped-unknown` | Ѿкꙋ́дꙋ, ѿкꙋ́дꙋ |
| `family:synodal:numeral:pyatyi` | 130 | 124 | `spelling-variant` | Пѧ́тагѡ, Пѧ́тый, пѧ́таго, пѧ́тагѡ, пѧ́тое, пѧ́томꙋ, пѧ́тый, пѧто́ю |
| `ungrouped:воздвигошасѧ` | 130 | 128 | `ungrouped-unknown` | Воздвиго́шасѧ, воздвиго́шасѧ |
| `ungrouped:нечистъ` | 130 | 118 | `ungrouped-unknown` | Нечи́стъ, нечи́стъ |
| `ungrouped:ѿстꙋпи` | 130 | 126 | `ungrouped-unknown` | Ѿстꙋпѝ, ѿстꙋпи, ѿстꙋпѝ |
| `ungrouped:внезапꙋ` | 129 | 127 | `ungrouped-unknown` | Внеза́пꙋ, внеза́пꙋ |
| `ungrouped:мꙋки` | 129 | 125 | `ungrouped-unknown` | мꙋ́ки, мꙋ̑ки, мꙋкѝ |
| `ungrouped:имѣнїѧ` | 128 | 128 | `ungrouped-unknown` | и҆мѣ́нїѧ, и҆мѣ̑нїѧ |
| `ungrouped:прїѧша` | 128 | 128 | `ungrouped-unknown` | Прїѧ́ша, прїѧ́ша |
| `ungrouped:разгнѣвасѧ` | 128 | 126 | `ungrouped-unknown` | Разгнѣ́васѧ, разгнѣ́васѧ |
| `ungrouped:самаріи` | 128 | 126 | `ungrouped-unknown` | самарі́и |
| `ungrouped:скоты` | 128 | 122 | `ungrouped-unknown` | ско́ты, скоты̀, скоты̑ |
| `ungrouped:цр҃ь` | 128 | 117 | `abbreviation-registry` | Цр҃ь, цр҃ь |
| `ungrouped:ꙗвитсѧ` | 128 | 128 | `ungrouped-unknown` | Ꙗ҆ви́тсѧ, ꙗ҆ви́тсѧ |
| `ungrouped:воздвиже` | 127 | 127 | `ungrouped-unknown` | Воздви́же, воздви́же |
| `ungrouped:прейде` | 127 | 121 | `ungrouped-unknown` | Пре́йде, пре́йде |
| `family:synodal:pronoun:sei` | 126 | 124 | `spelling-variant` | Се́гѡ, Сего́, Сегѡ́, Семꙋ́, Сеѧ́, Сїѧ́, се́гѡ, се́ю, се́ѧ, сего́, сегѡ́, семꙋ́, сею̀, сїю́, сїю̑, сїѧ́, сѣ́й |
| `ungrouped:поклонисѧ` | 126 | 126 | `ungrouped-unknown` | поклони́сѧ |
| `ungrouped:дрꙋгій` | 125 | 123 | `ungrouped-unknown` | Дрꙋгі́й, дрꙋгі́й |
| `ungrouped:кꙋпнѡ` | 125 | 123 | `ungrouped-unknown` | Кꙋ́пнѡ, кꙋ́пнѡ |
| `ungrouped:єлей` | 125 | 116 | `ungrouped-unknown` | є҆ле́й |

## Coverage by corpus

| Corpus | Tokens | Top-1 | Top-k | Ambiguous | Unresolved |
|---|---:|---:|---:|---:|---:|
| Church Slavonic Bible | 651487 | 307604 | 490149 | 4887 | 160736 |
| Elizabeth Bible | 661857 | 311877 | 499394 | 4905 | 161858 |

## Coverage by source

| Source | Tokens | Top-1 | Top-k | Ambiguous | Unresolved |
|---|---:|---:|---:|---:|---:|
| `ponomar-elizabeth-bible-2026-08-09` | 661857 | 311877 | 499394 | 4905 | 161858 |
| `wikisource-church-slavonic-bible-2026-08-09` | 651487 | 307604 | 490149 | 4887 | 160736 |

## Coverage by partition

| Partition | Tokens | Top-1 | Top-k | Ambiguous | Unresolved |
|---|---:|---:|---:|---:|---:|
| `evaluation` | 265701 | 125279 | 200627 | 2091 | 64842 |
| `source` | 1047643 | 494202 | 788916 | 7701 | 257752 |

## Coverage by source and partition

| Source/partition | Tokens | Top-1 | Top-k | Ambiguous | Unresolved |
|---|---:|---:|---:|---:|---:|
| `ponomar-elizabeth-bible-2026-08-09:evaluation` | 134910 | 63593 | 101878 | 1083 | 32918 |
| `ponomar-elizabeth-bible-2026-08-09:source` | 526947 | 248284 | 397516 | 3822 | 128940 |
| `wikisource-church-slavonic-bible-2026-08-09:evaluation` | 130791 | 61686 | 98749 | 1008 | 31924 |
| `wikisource-church-slavonic-bible-2026-08-09:source` | 520696 | 245918 | 391400 | 3879 | 128812 |

## Gap categories by source

| Source | Category | Tokens |
|---|---|---:|
| `ponomar-elizabeth-bible-2026-08-09` | `unknown-lexeme` | 155041 |
| `ponomar-elizabeth-bible-2026-08-09` | `missing-declension-or-class` | 49 |
| `ponomar-elizabeth-bible-2026-08-09` | `missing-verb-principal-part` | 27 |
| `ponomar-elizabeth-bible-2026-08-09` | `unsupported-formation` | 18 |
| `ponomar-elizabeth-bible-2026-08-09` | `missing-accent-or-orthographic-metadata` | 6723 |
| `ponomar-elizabeth-bible-2026-08-09` | `ambiguity-or-spelling-variant` | 5510 |
| `wikisource-church-slavonic-bible-2026-08-09` | `unknown-lexeme` | 154000 |
| `wikisource-church-slavonic-bible-2026-08-09` | `missing-declension-or-class` | 21 |
| `wikisource-church-slavonic-bible-2026-08-09` | `missing-verb-principal-part` | 27 |
| `wikisource-church-slavonic-bible-2026-08-09` | `unsupported-formation` | 18 |
| `wikisource-church-slavonic-bible-2026-08-09` | `missing-accent-or-orthographic-metadata` | 6670 |
| `wikisource-church-slavonic-bible-2026-08-09` | `ambiguity-or-spelling-variant` | 5489 |

## Gap categories by partition

| Partition | Category | Tokens |
|---|---|---:|
| `evaluation` | `unknown-lexeme` | 62025 |
| `evaluation` | `missing-declension-or-class` | 12 |
| `evaluation` | `missing-verb-principal-part` | 15 |
| `evaluation` | `unsupported-formation` | 6 |
| `evaluation` | `missing-accent-or-orthographic-metadata` | 2784 |
| `evaluation` | `ambiguity-or-spelling-variant` | 2323 |
| `source` | `unknown-lexeme` | 247016 |
| `source` | `missing-declension-or-class` | 58 |
| `source` | `missing-verb-principal-part` | 39 |
| `source` | `unsupported-formation` | 30 |
| `source` | `missing-accent-or-orthographic-metadata` | 10609 |
| `source` | `ambiguity-or-spelling-variant` | 8676 |

## Review queue

| Rank | Gap | Token | Frequency | Documents | Action |
|---:|---|---|---:|---:|---|
| 1 | `ambiguity-or-spelling-variant` | `ни` | 1658 | 1285 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 2 | `ambiguity-or-spelling-variant` | `твоѝ` | 816 | 709 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 3 | `ambiguity-or-spelling-variant` | `ты́сѧщъ` | 634 | 503 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 4 | `ambiguity-or-spelling-variant` | `и҆́мать` | 557 | 497 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 5 | `ambiguity-or-spelling-variant` | `є҆̀` | 469 | 412 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 6 | `ambiguity-or-spelling-variant` | `бра́тїѧ` | 464 | 453 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 7 | `ambiguity-or-spelling-variant` | `тꙋ̀` | 373 | 360 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 8 | `ambiguity-or-spelling-variant` | `сотвори́лъ` | 343 | 322 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 9 | `ambiguity-or-spelling-variant` | `ты́сѧщы` | 333 | 311 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 10 | `ambiguity-or-spelling-variant` | `і҆а́кѡвъ` | 297 | 279 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 11 | `ambiguity-or-spelling-variant` | `ѕло̀` | 261 | 255 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 12 | `ambiguity-or-spelling-variant` | `а҆арѡ́нъ` | 253 | 243 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 13 | `ambiguity-or-spelling-variant` | `и҆́мꙋтъ` | 242 | 223 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 14 | `ambiguity-or-spelling-variant` | `го́ре` | 238 | 198 | review whether this is a permitted Synodal spelling variant, abbreviation, or distinct lexeme |
| 15 | `missing-accent-or-orthographic-metadata` | `непра́вды` | 225 | 215 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 16 | `ambiguity-or-spelling-variant` | `вїно̀` | 221 | 207 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 17 | `unknown-lexeme` | `двє́ри` | 218 | 208 | review the token against target-recension evidence and create or reject a lexical candidate |
| 18 | `ambiguity-or-spelling-variant` | `вѣ́сть` | 213 | 207 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 19 | `unknown-lexeme` | `ꙗ҆ви́сѧ` | 213 | 205 | review the token against target-recension evidence and create or reject a lexical candidate |
| 20 | `unknown-lexeme` | `ѡ҆полчи́шасѧ` | 213 | 197 | review the token against target-recension evidence and create or reject a lexical candidate |
| 21 | `unknown-lexeme` | `нѣ́цыи` | 211 | 203 | review the token against target-recension evidence and create or reject a lexical candidate |
| 22 | `unknown-lexeme` | `предадѐ` | 207 | 203 | review the token against target-recension evidence and create or reject a lexical candidate |
| 23 | `unknown-lexeme` | `воста́ни` | 204 | 183 | review the token against target-recension evidence and create or reject a lexical candidate |
| 24 | `missing-accent-or-orthographic-metadata` | `ла́ктей` | 200 | 119 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 25 | `unknown-lexeme` | `во́ньже` | 198 | 186 | review the token against target-recension evidence and create or reject a lexical candidate |
| 26 | `unknown-lexeme` | `жє́ртвы` | 193 | 187 | review the token against target-recension evidence and create or reject a lexical candidate |
| 27 | `ambiguity-or-spelling-variant` | `своѝ` | 193 | 187 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 28 | `unknown-lexeme` | `і҆ꙋде́є` | 192 | 192 | review the token against target-recension evidence and create or reject a lexical candidate |
| 29 | `unknown-lexeme` | `і҆ꙋде́и` | 190 | 188 | review the token against target-recension evidence and create or reject a lexical candidate |
| 30 | `ambiguity-or-spelling-variant` | `и҆́мамъ` | 188 | 174 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 31 | `unknown-lexeme` | `ѹ҆́мретъ` | 186 | 169 | review the token against target-recension evidence and create or reject a lexical candidate |
| 32 | `unknown-lexeme` | `лакѡ́тъ` | 185 | 116 | review the token against target-recension evidence and create or reject a lexical candidate |
| 33 | `unknown-lexeme` | `нечести́выхъ` | 184 | 182 | review the token against target-recension evidence and create or reject a lexical candidate |
| 34 | `missing-accent-or-orthographic-metadata` | `возвѣстѝ` | 182 | 174 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 35 | `unknown-lexeme` | `и҆ноплемє́нницы` | 181 | 167 | review the token against target-recension evidence and create or reject a lexical candidate |
| 36 | `unknown-lexeme` | `и҆зведѐ` | 178 | 178 | review the token against target-recension evidence and create or reject a lexical candidate |
| 37 | `unknown-lexeme` | `ᲂу҆́стъ` | 177 | 166 | review the token against target-recension evidence and create or reject a lexical candidate |
| 38 | `unknown-lexeme` | `нача̀` | 176 | 174 | review the token against target-recension evidence and create or reject a lexical candidate |
| 39 | `unknown-lexeme` | `є҆ли́цы` | 173 | 165 | review the token against target-recension evidence and create or reject a lexical candidate |
| 40 | `unknown-lexeme` | `дале́че` | 171 | 169 | review the token against target-recension evidence and create or reject a lexical candidate |
| 41 | `unknown-lexeme` | `премꙋ́дрости` | 170 | 169 | review the token against target-recension evidence and create or reject a lexical candidate |
| 42 | `unknown-lexeme` | `господи́нꙋ` | 170 | 162 | review the token against target-recension evidence and create or reject a lexical candidate |
| 43 | `unknown-lexeme` | `є҆зекі́а` | 170 | 160 | review the token against target-recension evidence and create or reject a lexical candidate |
| 44 | `unknown-lexeme` | `ѻ҆лтарѧ̀` | 169 | 145 | review the token against target-recension evidence and create or reject a lexical candidate |
| 45 | `unknown-lexeme` | `па́ѵелъ` | 166 | 166 | review the token against target-recension evidence and create or reject a lexical candidate |
| 46 | `unknown-lexeme` | `творѧ́й` | 166 | 162 | review the token against target-recension evidence and create or reject a lexical candidate |
| 47 | `unknown-lexeme` | `воцари́сѧ` | 164 | 164 | review the token against target-recension evidence and create or reject a lexical candidate |
| 48 | `unknown-lexeme` | `ѡ҆правда̑нїѧ` | 164 | 158 | review the token against target-recension evidence and create or reject a lexical candidate |
| 49 | `unknown-lexeme` | `Лꙋ́чше` | 163 | 161 | review the token against target-recension evidence and create or reject a lexical candidate |
| 50 | `ambiguity-or-spelling-variant` | `и҆́мате` | 163 | 148 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 51 | `unknown-lexeme` | `бога́тство` | 157 | 154 | review the token against target-recension evidence and create or reject a lexical candidate |
| 52 | `unknown-lexeme` | `блгⷭ҇ве́нъ` | 156 | 155 | review the token against target-recension evidence and create or reject a lexical candidate |
| 53 | `unknown-lexeme` | `введѐ` | 155 | 153 | review the token against target-recension evidence and create or reject a lexical candidate |
| 54 | `unknown-lexeme` | `со́нмѡмъ` | 154 | 152 | review the token against target-recension evidence and create or reject a lexical candidate |
| 55 | `unknown-lexeme` | `ка́менїемъ` | 154 | 150 | review the token against target-recension evidence and create or reject a lexical candidate |
| 56 | `unknown-lexeme` | `ѻ҆вна̀` | 154 | 148 | review the token against target-recension evidence and create or reject a lexical candidate |
| 57 | `unknown-lexeme` | `слꙋжи́ти` | 153 | 148 | review the token against target-recension evidence and create or reject a lexical candidate |
| 58 | `unknown-lexeme` | `сꙋббѡ́ты` | 153 | 138 | review the token against target-recension evidence and create or reject a lexical candidate |
| 59 | `unknown-lexeme` | `погꙋби́ти` | 152 | 150 | review the token against target-recension evidence and create or reject a lexical candidate |
| 60 | `ambiguity-or-spelling-variant` | `и҆́маши` | 150 | 143 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 61 | `unknown-lexeme` | `ѻ҆рꙋ́жїе` | 150 | 142 | review the token against target-recension evidence and create or reject a lexical candidate |
| 62 | `unknown-lexeme` | `ѹ҆слы́ша` | 149 | 148 | review the token against target-recension evidence and create or reject a lexical candidate |
| 63 | `unknown-lexeme` | `навꙋходоно́соръ` | 149 | 145 | review the token against target-recension evidence and create or reject a lexical candidate |
| 64 | `unknown-lexeme` | `царе́ва` | 148 | 144 | review the token against target-recension evidence and create or reject a lexical candidate |
| 65 | `unknown-lexeme` | `ѻ҆́ньже` | 147 | 141 | review the token against target-recension evidence and create or reject a lexical candidate |
| 66 | `unknown-lexeme` | `ве́лїй` | 145 | 144 | review the token against target-recension evidence and create or reject a lexical candidate |
| 67 | `unknown-lexeme` | `богѡ́въ` | 145 | 137 | review the token against target-recension evidence and create or reject a lexical candidate |
| 68 | `unknown-lexeme` | `врагѡ́въ` | 144 | 140 | review the token against target-recension evidence and create or reject a lexical candidate |
| 69 | `unknown-lexeme` | `ѡ҆деснꙋ́ю` | 143 | 141 | review the token against target-recension evidence and create or reject a lexical candidate |
| 70 | `ambiguity-or-spelling-variant` | `по́ли` | 142 | 142 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 71 | `unknown-lexeme` | `сі́мѡнъ` | 140 | 140 | review the token against target-recension evidence and create or reject a lexical candidate |
| 72 | `unknown-lexeme` | `весе́лїе` | 140 | 138 | review the token against target-recension evidence and create or reject a lexical candidate |
| 73 | `missing-accent-or-orthographic-metadata` | `ско́рби` | 140 | 136 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 74 | `unknown-lexeme` | `две́рїй` | 140 | 127 | review the token against target-recension evidence and create or reject a lexical candidate |
| 75 | `ambiguity-or-spelling-variant` | `нѣ́кїй` | 138 | 138 | review whether this is a permitted Synodal spelling variant, abbreviation, or distinct lexeme |
| 76 | `unknown-lexeme` | `со́нъ` | 138 | 122 | review the token against target-recension evidence and create or reject a lexical candidate |
| 77 | `unknown-lexeme` | `ѻ҆тцы̑` | 138 | 122 | review the token against target-recension evidence and create or reject a lexical candidate |
| 78 | `unknown-lexeme` | `вє́си` | 138 | 103 | review the token against target-recension evidence and create or reject a lexical candidate |
| 79 | `unknown-lexeme` | `тꙋ́къ` | 138 | 100 | review the token against target-recension evidence and create or reject a lexical candidate |
| 80 | `missing-accent-or-orthographic-metadata` | `ѻ҆нꙋ̀` | 137 | 135 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 81 | `unknown-lexeme` | `прѡ́чаѧ` | 136 | 136 | review the token against target-recension evidence and create or reject a lexical candidate |
| 82 | `unknown-lexeme` | `Воста́ша` | 136 | 134 | review the token against target-recension evidence and create or reject a lexical candidate |
| 83 | `missing-accent-or-orthographic-metadata` | `зна́мєнїѧ` | 136 | 134 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 84 | `unknown-lexeme` | `тала̑нтъ` | 136 | 101 | review the token against target-recension evidence and create or reject a lexical candidate |
| 85 | `unknown-lexeme` | `вѣне́цъ` | 134 | 134 | review the token against target-recension evidence and create or reject a lexical candidate |
| 86 | `unknown-lexeme` | `ве́лїимъ` | 132 | 131 | review the token against target-recension evidence and create or reject a lexical candidate |
| 87 | `unknown-lexeme` | `ѿкꙋ́дꙋ` | 131 | 123 | review the token against target-recension evidence and create or reject a lexical candidate |
| 88 | `unknown-lexeme` | `воздвиго́шасѧ` | 130 | 128 | review the token against target-recension evidence and create or reject a lexical candidate |
| 89 | `unknown-lexeme` | `ѿстꙋпѝ` | 130 | 126 | review the token against target-recension evidence and create or reject a lexical candidate |
| 90 | `ambiguity-or-spelling-variant` | `а҆арѡ́на` | 130 | 126 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 91 | `unknown-lexeme` | `нечи́стъ` | 130 | 118 | review the token against target-recension evidence and create or reject a lexical candidate |
| 92 | `unknown-lexeme` | `внеза́пꙋ` | 129 | 127 | review the token against target-recension evidence and create or reject a lexical candidate |
| 93 | `unknown-lexeme` | `мꙋкѝ` | 129 | 125 | review the token against target-recension evidence and create or reject a lexical candidate |
| 94 | `ambiguity-or-spelling-variant` | `ты́сѧща` | 129 | 119 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 95 | `unknown-lexeme` | `и҆мѣ́нїѧ` | 128 | 128 | review the token against target-recension evidence and create or reject a lexical candidate |
| 96 | `unknown-lexeme` | `прїѧ́ша` | 128 | 128 | review the token against target-recension evidence and create or reject a lexical candidate |
| 97 | `unknown-lexeme` | `ꙗ҆ви́тсѧ` | 128 | 128 | review the token against target-recension evidence and create or reject a lexical candidate |
| 98 | `unknown-lexeme` | `разгнѣ́васѧ` | 128 | 126 | review the token against target-recension evidence and create or reject a lexical candidate |
| 99 | `unknown-lexeme` | `самарі́и` | 128 | 126 | review the token against target-recension evidence and create or reject a lexical candidate |
| 100 | `unknown-lexeme` | `ско́ты` | 128 | 122 | review the token against target-recension evidence and create or reject a lexical candidate |
| 101 | `unknown-lexeme` | `цр҃ь` | 128 | 117 | review the token against target-recension evidence and create or reject a lexical candidate |
| 102 | `unknown-lexeme` | `воздви́же` | 127 | 127 | review the token against target-recension evidence and create or reject a lexical candidate |
| 103 | `unknown-lexeme` | `пре́йде` | 127 | 121 | review the token against target-recension evidence and create or reject a lexical candidate |
| 104 | `unknown-lexeme` | `поклони́сѧ` | 126 | 126 | review the token against target-recension evidence and create or reject a lexical candidate |
| 105 | `missing-accent-or-orthographic-metadata` | `ты̑ѧ` | 126 | 118 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 106 | `unknown-lexeme` | `дрꙋгі́й` | 125 | 123 | review the token against target-recension evidence and create or reject a lexical candidate |
| 107 | `unknown-lexeme` | `кꙋ́пнѡ` | 125 | 123 | review the token against target-recension evidence and create or reject a lexical candidate |
| 108 | `unknown-lexeme` | `є҆ле́й` | 125 | 116 | review the token against target-recension evidence and create or reject a lexical candidate |
| 109 | `missing-accent-or-orthographic-metadata` | `взѧ́ти` | 124 | 121 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 110 | `unknown-lexeme` | `поко́й` | 123 | 119 | review the token against target-recension evidence and create or reject a lexical candidate |
| 111 | `unknown-lexeme` | `є҆ле́а` | 122 | 122 | review the token against target-recension evidence and create or reject a lexical candidate |
| 112 | `missing-accent-or-orthographic-metadata` | `поги́бе` | 122 | 122 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 113 | `unknown-lexeme` | `ѡ҆ста́нокъ` | 122 | 110 | review the token against target-recension evidence and create or reject a lexical candidate |
| 114 | `unknown-lexeme` | `про́чїи` | 121 | 121 | review the token against target-recension evidence and create or reject a lexical candidate |
| 115 | `unknown-lexeme` | `і҆еремі́а` | 121 | 119 | review the token against target-recension evidence and create or reject a lexical candidate |
| 116 | `unknown-lexeme` | `созадѝ` | 121 | 115 | review the token against target-recension evidence and create or reject a lexical candidate |
| 117 | `unknown-lexeme` | `всели́сѧ` | 121 | 113 | review the token against target-recension evidence and create or reject a lexical candidate |
| 118 | `ambiguity-or-spelling-variant` | `послꙋ́ша` | 120 | 120 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 119 | `unknown-lexeme` | `а҆́може` | 120 | 118 | review the token against target-recension evidence and create or reject a lexical candidate |
| 120 | `unknown-lexeme` | `наведꙋ̀` | 120 | 118 | review the token against target-recension evidence and create or reject a lexical candidate |
| 121 | `unknown-lexeme` | `возда́стъ` | 120 | 112 | review the token against target-recension evidence and create or reject a lexical candidate |
| 122 | `unknown-lexeme` | `со́нма` | 117 | 117 | review the token against target-recension evidence and create or reject a lexical candidate |
| 123 | `unknown-lexeme` | `приложѝ` | 116 | 114 | review the token against target-recension evidence and create or reject a lexical candidate |
| 124 | `unknown-lexeme` | `прїиди́те` | 116 | 114 | review the token against target-recension evidence and create or reject a lexical candidate |
| 125 | `unknown-lexeme` | `повѣ́да` | 116 | 110 | review the token against target-recension evidence and create or reject a lexical candidate |
| 126 | `unknown-lexeme` | `наказа́нїе` | 115 | 113 | review the token against target-recension evidence and create or reject a lexical candidate |
| 127 | `ambiguity-or-spelling-variant` | `и҆́мамы` | 115 | 113 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 128 | `unknown-lexeme` | `воста́нетъ` | 114 | 114 | review the token against target-recension evidence and create or reject a lexical candidate |
| 129 | `unknown-lexeme` | `приложи́тъ` | 114 | 114 | review the token against target-recension evidence and create or reject a lexical candidate |
| 130 | `unknown-lexeme` | `вои́стиннꙋ` | 114 | 113 | review the token against target-recension evidence and create or reject a lexical candidate |
| 131 | `unknown-lexeme` | `і҆исꙋ́сꙋ` | 114 | 108 | review the token against target-recension evidence and create or reject a lexical candidate |
| 132 | `unknown-lexeme` | `послѣ̑днѧѧ` | 113 | 113 | review the token against target-recension evidence and create or reject a lexical candidate |
| 133 | `unknown-lexeme` | `а҆ммѡ́нихъ` | 112 | 110 | review the token against target-recension evidence and create or reject a lexical candidate |
| 134 | `unknown-lexeme` | `вавѷлѡ́нскїй` | 112 | 110 | review the token against target-recension evidence and create or reject a lexical candidate |
| 135 | `missing-accent-or-orthographic-metadata` | `вели́кагѡ` | 112 | 110 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 136 | `unknown-lexeme` | `вы́шше` | 110 | 108 | review the token against target-recension evidence and create or reject a lexical candidate |
| 137 | `unknown-lexeme` | `заповѣ́даю` | 110 | 108 | review the token against target-recension evidence and create or reject a lexical candidate |
| 138 | `unknown-lexeme` | `зане́же` | 109 | 109 | review the token against target-recension evidence and create or reject a lexical candidate |
| 139 | `unknown-lexeme` | `саꙋ́лꙋ` | 109 | 109 | review the token against target-recension evidence and create or reject a lexical candidate |
| 140 | `unknown-lexeme` | `колєсни́цы` | 108 | 108 | review the token against target-recension evidence and create or reject a lexical candidate |
| 141 | `unknown-lexeme` | `высо́кихъ` | 108 | 94 | review the token against target-recension evidence and create or reject a lexical candidate |
| 142 | `unknown-lexeme` | `ѡ҆крє́стнаѧ` | 108 | 70 | review the token against target-recension evidence and create or reject a lexical candidate |
| 143 | `unknown-lexeme` | `повелѣ̑нїѧ` | 107 | 103 | review the token against target-recension evidence and create or reject a lexical candidate |
| 144 | `ambiguity-or-spelling-variant` | `чесѡ̀` | 107 | 101 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 145 | `unknown-lexeme` | `пе́рвѣе` | 106 | 106 | review the token against target-recension evidence and create or reject a lexical candidate |
| 146 | `unknown-lexeme` | `по́йдꙋтъ` | 106 | 106 | review the token against target-recension evidence and create or reject a lexical candidate |
| 147 | `unknown-lexeme` | `і҆еремі́и` | 106 | 106 | review the token against target-recension evidence and create or reject a lexical candidate |
| 148 | `unknown-lexeme` | `по́йте` | 106 | 100 | review the token against target-recension evidence and create or reject a lexical candidate |
| 149 | `unknown-lexeme` | `нача́ша` | 106 | 99 | review the token against target-recension evidence and create or reject a lexical candidate |
| 150 | `unknown-lexeme` | `небє́сныѧ` | 105 | 105 | review the token against target-recension evidence and create or reject a lexical candidate |
| 151 | `unknown-lexeme` | `пребꙋ́детъ` | 105 | 105 | review the token against target-recension evidence and create or reject a lexical candidate |
| 152 | `unknown-lexeme` | `вавѷлѡ́нска` | 105 | 103 | review the token against target-recension evidence and create or reject a lexical candidate |
| 153 | `unknown-lexeme` | `писа́нїе` | 105 | 103 | review the token against target-recension evidence and create or reject a lexical candidate |
| 154 | `unknown-lexeme` | `поклони́шасѧ` | 104 | 104 | review the token against target-recension evidence and create or reject a lexical candidate |
| 155 | `unknown-lexeme` | `пойдꙋ̀` | 104 | 102 | review the token against target-recension evidence and create or reject a lexical candidate |
| 156 | `unknown-lexeme` | `си́льныхъ` | 104 | 102 | review the token against target-recension evidence and create or reject a lexical candidate |
| 157 | `unknown-lexeme` | `рабы̑ни` | 104 | 101 | review the token against target-recension evidence and create or reject a lexical candidate |
| 158 | `unknown-lexeme` | `фарїсе́є` | 104 | 99 | review the token against target-recension evidence and create or reject a lexical candidate |
| 159 | `unknown-lexeme` | `ѻ҆рꙋ́жїемъ` | 104 | 99 | review the token against target-recension evidence and create or reject a lexical candidate |
| 160 | `unknown-lexeme` | `помоли́сѧ` | 104 | 98 | review the token against target-recension evidence and create or reject a lexical candidate |
| 161 | `unknown-lexeme` | `є҆ле́емъ` | 104 | 96 | review the token against target-recension evidence and create or reject a lexical candidate |
| 162 | `unknown-lexeme` | `погꙋби́тъ` | 104 | 93 | review the token against target-recension evidence and create or reject a lexical candidate |
| 163 | `unknown-lexeme` | `сі̑кль` | 104 | 68 | review the token against target-recension evidence and create or reject a lexical candidate |
| 164 | `missing-accent-or-orthographic-metadata` | `Слы́шавше` | 103 | 102 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 165 | `missing-accent-or-orthographic-metadata` | `древа̀` | 103 | 101 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 166 | `unknown-lexeme` | `Житїѐ` | 102 | 99 | review the token against target-recension evidence and create or reject a lexical candidate |
| 167 | `unknown-lexeme` | `послы̀` | 102 | 98 | review the token against target-recension evidence and create or reject a lexical candidate |
| 168 | `unknown-lexeme` | `воню̀` | 101 | 101 | review the token against target-recension evidence and create or reject a lexical candidate |
| 169 | `unknown-lexeme` | `вы́нꙋ` | 101 | 101 | review the token against target-recension evidence and create or reject a lexical candidate |
| 170 | `unknown-lexeme` | `молю́` | 101 | 95 | review the token against target-recension evidence and create or reject a lexical candidate |
| 171 | `unknown-lexeme` | `а҆вессалѡ́мъ` | 101 | 89 | review the token against target-recension evidence and create or reject a lexical candidate |
| 172 | `unknown-lexeme` | `преста̀` | 100 | 99 | review the token against target-recension evidence and create or reject a lexical candidate |
| 173 | `unknown-lexeme` | `свы́ше` | 100 | 99 | review the token against target-recension evidence and create or reject a lexical candidate |
| 174 | `missing-accent-or-orthographic-metadata` | `кни̑ги` | 100 | 95 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 175 | `unknown-lexeme` | `пе́рвенца` | 100 | 79 | review the token against target-recension evidence and create or reject a lexical candidate |
| 176 | `unknown-lexeme` | `блгⷭ҇вѝ` | 99 | 99 | review the token against target-recension evidence and create or reject a lexical candidate |
| 177 | `unknown-lexeme` | `словесє́мъ` | 99 | 99 | review the token against target-recension evidence and create or reject a lexical candidate |
| 178 | `unknown-lexeme` | `ᲂу҆чн҃цы̀` | 99 | 98 | review the token against target-recension evidence and create or reject a lexical candidate |
| 179 | `unknown-lexeme` | `првⷣныхъ` | 99 | 95 | review the token against target-recension evidence and create or reject a lexical candidate |
| 180 | `unknown-lexeme` | `и҆здале́ча` | 98 | 98 | review the token against target-recension evidence and create or reject a lexical candidate |
| 181 | `unknown-lexeme` | `покры̀` | 98 | 98 | review the token against target-recension evidence and create or reject a lexical candidate |
| 182 | `unknown-lexeme` | `првⷣнъ` | 98 | 96 | review the token against target-recension evidence and create or reject a lexical candidate |
| 183 | `unknown-lexeme` | `си́ленъ` | 98 | 96 | review the token against target-recension evidence and create or reject a lexical candidate |
| 184 | `unknown-lexeme` | `і҆ѡсафа́тъ` | 98 | 94 | review the token against target-recension evidence and create or reject a lexical candidate |
| 185 | `unknown-lexeme` | `ѻ҆рꙋ̑жїѧ` | 98 | 94 | review the token against target-recension evidence and create or reject a lexical candidate |
| 186 | `unknown-lexeme` | `Є҆ли́кѡ` | 98 | 92 | review the token against target-recension evidence and create or reject a lexical candidate |
| 187 | `unknown-lexeme` | `воевѡ́ды` | 96 | 96 | review the token against target-recension evidence and create or reject a lexical candidate |
| 188 | `unknown-lexeme` | `ѻ҆бои́хъ` | 96 | 89 | review the token against target-recension evidence and create or reject a lexical candidate |
| 189 | `unknown-lexeme` | `снѣдѧ́тъ` | 96 | 84 | review the token against target-recension evidence and create or reject a lexical candidate |
| 190 | `unknown-lexeme` | `по́йде` | 95 | 95 | review the token against target-recension evidence and create or reject a lexical candidate |
| 191 | `unknown-lexeme` | `чрез̾` | 95 | 93 | review the token against target-recension evidence and create or reject a lexical candidate |
| 192 | `unknown-lexeme` | `помо́литсѧ` | 95 | 91 | review the token against target-recension evidence and create or reject a lexical candidate |
| 193 | `unknown-lexeme` | `и҆са́ѵъ` | 95 | 85 | review the token against target-recension evidence and create or reject a lexical candidate |
| 194 | `unknown-lexeme` | `ꙗ҆́тъ` | 94 | 94 | review the token against target-recension evidence and create or reject a lexical candidate |
| 195 | `unknown-lexeme` | `дабы̀` | 94 | 93 | review the token against target-recension evidence and create or reject a lexical candidate |
| 196 | `unknown-lexeme` | `за́повѣдїй` | 94 | 93 | review the token against target-recension evidence and create or reject a lexical candidate |
| 197 | `unknown-lexeme` | `поѧ́стъ` | 94 | 92 | review the token against target-recension evidence and create or reject a lexical candidate |
| 198 | `unknown-lexeme` | `саꙋ́ла` | 94 | 92 | review the token against target-recension evidence and create or reject a lexical candidate |
| 199 | `unknown-lexeme` | `ѡ҆ста́сѧ` | 94 | 92 | review the token against target-recension evidence and create or reject a lexical candidate |
| 200 | `unknown-lexeme` | `сю́дꙋ` | 94 | 46 | review the token against target-recension evidence and create or reject a lexical candidate |
| 201 | `missing-accent-or-orthographic-metadata` | `Слы́шавъ` | 93 | 93 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 202 | `unknown-lexeme` | `возмо́жетъ` | 92 | 92 | review the token against target-recension evidence and create or reject a lexical candidate |
| 203 | `unknown-lexeme` | `си́льнїи` | 92 | 92 | review the token against target-recension evidence and create or reject a lexical candidate |
| 204 | `unknown-lexeme` | `ѧ҆зы́комъ` | 92 | 92 | review the token against target-recension evidence and create or reject a lexical candidate |
| 205 | `unknown-lexeme` | `поѧ́тъ` | 92 | 91 | review the token against target-recension evidence and create or reject a lexical candidate |
| 206 | `unknown-lexeme` | `і҆иꙋ́й` | 92 | 82 | review the token against target-recension evidence and create or reject a lexical candidate |
| 207 | `unknown-lexeme` | `вско́рѣ` | 91 | 91 | review the token against target-recension evidence and create or reject a lexical candidate |
| 208 | `unknown-lexeme` | `млⷭ҇тивъ` | 91 | 91 | review the token against target-recension evidence and create or reject a lexical candidate |
| 209 | `unknown-lexeme` | `погꙋблю̀` | 91 | 91 | review the token against target-recension evidence and create or reject a lexical candidate |
| 210 | `missing-accent-or-orthographic-metadata` | `бо́ги` | 91 | 90 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 211 | `unknown-lexeme` | `ѹ҆бѝ` | 91 | 86 | review the token against target-recension evidence and create or reject a lexical candidate |
| 212 | `unknown-lexeme` | `сокрѡ́вища` | 91 | 83 | review the token against target-recension evidence and create or reject a lexical candidate |
| 213 | `unknown-lexeme` | `заповѣ́дахъ` | 90 | 90 | review the token against target-recension evidence and create or reject a lexical candidate |
| 214 | `unknown-lexeme` | `и҆збра́нныхъ` | 90 | 88 | review the token against target-recension evidence and create or reject a lexical candidate |
| 215 | `ambiguity-or-spelling-variant` | `живы́й` | 90 | 88 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 216 | `unknown-lexeme` | `сотвори́вый` | 90 | 87 | review the token against target-recension evidence and create or reject a lexical candidate |
| 217 | `unknown-lexeme` | `всѧ́чєскаѧ` | 90 | 80 | review the token against target-recension evidence and create or reject a lexical candidate |
| 218 | `unknown-lexeme` | `ст҃ы́ни` | 89 | 88 | review the token against target-recension evidence and create or reject a lexical candidate |
| 219 | `unknown-lexeme` | `мѡа́вли` | 89 | 87 | review the token against target-recension evidence and create or reject a lexical candidate |
| 220 | `unknown-lexeme` | `ко́ль` | 89 | 83 | review the token against target-recension evidence and create or reject a lexical candidate |
| 221 | `unknown-lexeme` | `нача́токъ` | 89 | 83 | review the token against target-recension evidence and create or reject a lexical candidate |
| 222 | `unknown-lexeme` | `кѡры́сти` | 89 | 77 | review the token against target-recension evidence and create or reject a lexical candidate |
| 223 | `missing-accent-or-orthographic-metadata` | `чи́ста` | 88 | 86 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 224 | `unknown-lexeme` | `воста́нꙋтъ` | 88 | 85 | review the token against target-recension evidence and create or reject a lexical candidate |
| 225 | `unknown-lexeme` | `і҆ѡнаѳа́на` | 88 | 84 | review the token against target-recension evidence and create or reject a lexical candidate |
| 226 | `missing-accent-or-orthographic-metadata` | `ничесѡ́же` | 87 | 87 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 227 | `unknown-lexeme` | `прїидꙋ̀` | 87 | 85 | review the token against target-recension evidence and create or reject a lexical candidate |
| 228 | `unknown-lexeme` | `і҆ꙋде́ѡмъ` | 87 | 85 | review the token against target-recension evidence and create or reject a lexical candidate |
| 229 | `unknown-lexeme` | `И҆мѣ́ѧй` | 87 | 83 | review the token against target-recension evidence and create or reject a lexical candidate |
| 230 | `unknown-lexeme` | `всꙋ́е` | 87 | 81 | review the token against target-recension evidence and create or reject a lexical candidate |
| 231 | `unknown-lexeme` | `комꙋ́ждо` | 87 | 81 | review the token against target-recension evidence and create or reject a lexical candidate |
| 232 | `unknown-lexeme` | `помышлє́нїѧ` | 86 | 86 | review the token against target-recension evidence and create or reject a lexical candidate |
| 233 | `unknown-lexeme` | `і҆ꙋде́ю` | 86 | 86 | review the token against target-recension evidence and create or reject a lexical candidate |
| 234 | `unknown-lexeme` | `ѡ҆снова̑нїѧ` | 86 | 86 | review the token against target-recension evidence and create or reject a lexical candidate |
| 235 | `unknown-lexeme` | `плѣне́нїе` | 86 | 84 | review the token against target-recension evidence and create or reject a lexical candidate |
| 236 | `unknown-lexeme` | `царє́вы` | 86 | 84 | review the token against target-recension evidence and create or reject a lexical candidate |
| 237 | `unknown-lexeme` | `є҆зекі́и` | 86 | 84 | review the token against target-recension evidence and create or reject a lexical candidate |
| 238 | `unknown-lexeme` | `ѡ҆полчи́сѧ` | 86 | 84 | review the token against target-recension evidence and create or reject a lexical candidate |
| 239 | `unknown-lexeme` | `є҆гѵ́птѧне` | 86 | 82 | review the token against target-recension evidence and create or reject a lexical candidate |
| 240 | `unknown-lexeme` | `преда́стъ` | 86 | 80 | review the token against target-recension evidence and create or reject a lexical candidate |
| 241 | `unknown-lexeme` | `чл҃вѣ́ческїй` | 86 | 80 | review the token against target-recension evidence and create or reject a lexical candidate |
| 242 | `missing-accent-or-orthographic-metadata` | `высота̀` | 86 | 80 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 243 | `unknown-lexeme` | `ᲂу҆́шы` | 86 | 78 | review the token against target-recension evidence and create or reject a lexical candidate |
| 244 | `ambiguity-or-spelling-variant` | `про́клѧтъ` | 86 | 78 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 245 | `unknown-lexeme` | `пе́рвенецъ` | 85 | 85 | review the token against target-recension evidence and create or reject a lexical candidate |
| 246 | `unknown-lexeme` | `сꙋдїи̑` | 85 | 85 | review the token against target-recension evidence and create or reject a lexical candidate |
| 247 | `unknown-lexeme` | `царе́въ` | 85 | 83 | review the token against target-recension evidence and create or reject a lexical candidate |
| 248 | `unknown-lexeme` | `ви́дитъ` | 84 | 84 | review the token against target-recension evidence and create or reject a lexical candidate |
| 249 | `unknown-lexeme` | `саваѡ́ѳъ` | 84 | 84 | review the token against target-recension evidence and create or reject a lexical candidate |
| 250 | `unknown-lexeme` | `ѕвѣ́зды` | 84 | 84 | review the token against target-recension evidence and create or reject a lexical candidate |
| 251 | `missing-accent-or-orthographic-metadata` | `и҆ны̑мъ` | 84 | 84 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 252 | `missing-accent-or-orthographic-metadata` | `и҆ны́хъ` | 84 | 83 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 253 | `missing-accent-or-orthographic-metadata` | `возвѣщꙋ̀` | 84 | 82 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 254 | `ambiguity-or-spelling-variant` | `ѻ҆́ный` | 84 | 82 | review whether this is a permitted Synodal spelling variant, abbreviation, or distinct lexeme |
| 255 | `unknown-lexeme` | `є҆леаза́ръ` | 84 | 80 | review the token against target-recension evidence and create or reject a lexical candidate |
| 256 | `unknown-lexeme` | `постыдѧ́тсѧ` | 84 | 78 | review the token against target-recension evidence and create or reject a lexical candidate |
| 257 | `unknown-lexeme` | `поми́лꙋй` | 84 | 77 | review the token against target-recension evidence and create or reject a lexical candidate |
| 258 | `unknown-lexeme` | `снѣ́сте` | 84 | 76 | review the token against target-recension evidence and create or reject a lexical candidate |
| 259 | `unknown-lexeme` | `блгⷣти` | 83 | 83 | review the token against target-recension evidence and create or reject a lexical candidate |
| 260 | `missing-accent-or-orthographic-metadata` | `возми́те` | 83 | 81 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 261 | `unknown-lexeme` | `ѡ҆шꙋ́юю` | 83 | 80 | review the token against target-recension evidence and create or reject a lexical candidate |
| 262 | `unknown-lexeme` | `весе́лїемъ` | 82 | 82 | review the token against target-recension evidence and create or reject a lexical candidate |
| 263 | `unknown-lexeme` | `и҆зы́ти` | 82 | 82 | review the token against target-recension evidence and create or reject a lexical candidate |
| 264 | `unknown-lexeme` | `сокрꙋшѝ` | 82 | 82 | review the token against target-recension evidence and create or reject a lexical candidate |
| 265 | `missing-accent-or-orthographic-metadata` | `бо̀` | 82 | 82 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 266 | `unknown-lexeme` | `даві́дова` | 82 | 80 | review the token against target-recension evidence and create or reject a lexical candidate |
| 267 | `unknown-lexeme` | `повелѣ́нїе` | 82 | 80 | review the token against target-recension evidence and create or reject a lexical candidate |
| 268 | `unknown-lexeme` | `послѣдѝ` | 82 | 79 | review the token against target-recension evidence and create or reject a lexical candidate |
| 269 | `unknown-lexeme` | `а҆вїмеле́хъ` | 82 | 78 | review the token against target-recension evidence and create or reject a lexical candidate |
| 270 | `ambiguity-or-spelling-variant` | `востѡ́къ` | 82 | 78 | review whether this is a permitted Synodal spelling variant, abbreviation, or distinct lexeme |
| 271 | `unknown-lexeme` | `ве́лїе` | 81 | 81 | review the token against target-recension evidence and create or reject a lexical candidate |
| 272 | `unknown-lexeme` | `мно́жае` | 81 | 81 | review the token against target-recension evidence and create or reject a lexical candidate |
| 273 | `unknown-lexeme` | `житїѧ̀` | 81 | 77 | review the token against target-recension evidence and create or reject a lexical candidate |
| 274 | `unknown-lexeme` | `мздꙋ̀` | 81 | 77 | review the token against target-recension evidence and create or reject a lexical candidate |
| 275 | `unknown-lexeme` | `пїла́тъ` | 81 | 76 | review the token against target-recension evidence and create or reject a lexical candidate |
| 276 | `unknown-lexeme` | `а҆рхїере́є` | 81 | 74 | review the token against target-recension evidence and create or reject a lexical candidate |
| 277 | `unknown-lexeme` | `і҆а́кѡвль` | 80 | 80 | review the token against target-recension evidence and create or reject a lexical candidate |
| 278 | `unknown-lexeme` | `завѣща̀` | 80 | 78 | review the token against target-recension evidence and create or reject a lexical candidate |
| 279 | `unknown-lexeme` | `кни́жницы` | 80 | 78 | review the token against target-recension evidence and create or reject a lexical candidate |
| 280 | `unknown-lexeme` | `мно́жествѣ` | 80 | 78 | review the token against target-recension evidence and create or reject a lexical candidate |
| 281 | `unknown-lexeme` | `помощѝ` | 80 | 78 | review the token against target-recension evidence and create or reject a lexical candidate |
| 282 | `unknown-lexeme` | `пшени́чны` | 80 | 78 | review the token against target-recension evidence and create or reject a lexical candidate |
| 283 | `ambiguity-or-spelling-variant` | `и҆мы́й` | 80 | 78 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 284 | `ambiguity-or-spelling-variant` | `сѧ̀` | 80 | 76 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 285 | `unknown-lexeme` | `ю҆́гꙋ` | 80 | 75 | review the token against target-recension evidence and create or reject a lexical candidate |
| 286 | `unknown-lexeme` | `і҆еровоа́мъ` | 80 | 74 | review the token against target-recension evidence and create or reject a lexical candidate |
| 287 | `unknown-lexeme` | `зла̑ты` | 80 | 73 | review the token against target-recension evidence and create or reject a lexical candidate |
| 288 | `missing-accent-or-orthographic-metadata` | `возвѣсти́те` | 80 | 71 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 289 | `unknown-lexeme` | `ко́жи` | 80 | 65 | review the token against target-recension evidence and create or reject a lexical candidate |
| 290 | `missing-accent-or-orthographic-metadata` | `плѡ́ти` | 80 | 62 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 291 | `unknown-lexeme` | `дв҃дꙋ` | 79 | 79 | review the token against target-recension evidence and create or reject a lexical candidate |
| 292 | `unknown-lexeme` | `пе́рсть` | 79 | 79 | review the token against target-recension evidence and create or reject a lexical candidate |
| 293 | `unknown-lexeme` | `прⷭ҇нѡ` | 79 | 79 | review the token against target-recension evidence and create or reject a lexical candidate |
| 294 | `unknown-lexeme` | `і҆ꙋдє́й` | 79 | 79 | review the token against target-recension evidence and create or reject a lexical candidate |
| 295 | `unknown-lexeme` | `нача̑лницы` | 79 | 75 | review the token against target-recension evidence and create or reject a lexical candidate |
| 296 | `unknown-lexeme` | `нача̑льницы` | 79 | 75 | review the token against target-recension evidence and create or reject a lexical candidate |
| 297 | `unknown-lexeme` | `є҆ди́ныѧ` | 79 | 72 | review the token against target-recension evidence and create or reject a lexical candidate |
| 298 | `unknown-lexeme` | `де́бри` | 79 | 71 | review the token against target-recension evidence and create or reject a lexical candidate |
| 299 | `unknown-lexeme` | `даві́довѣ` | 78 | 78 | review the token against target-recension evidence and create or reject a lexical candidate |
| 300 | `unknown-lexeme` | `нечести́вїи` | 78 | 78 | review the token against target-recension evidence and create or reject a lexical candidate |
| 301 | `unknown-lexeme` | `посе́мъ` | 78 | 78 | review the token against target-recension evidence and create or reject a lexical candidate |
| 302 | `unknown-lexeme` | `є҆́здра` | 78 | 78 | review the token against target-recension evidence and create or reject a lexical candidate |
| 303 | `unknown-lexeme` | `ѱало́мъ` | 78 | 78 | review the token against target-recension evidence and create or reject a lexical candidate |
| 304 | `unknown-lexeme` | `ѿсю́дꙋ` | 78 | 78 | review the token against target-recension evidence and create or reject a lexical candidate |
| 305 | `unknown-lexeme` | `лжꙋ̀` | 78 | 76 | review the token against target-recension evidence and create or reject a lexical candidate |
| 306 | `missing-accent-or-orthographic-metadata` | `согрѣши́хомъ` | 78 | 76 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 307 | `unknown-lexeme` | `воззрѣ́въ` | 78 | 75 | review the token against target-recension evidence and create or reject a lexical candidate |
| 308 | `unknown-lexeme` | `а҆ммѡ̑ни` | 78 | 74 | review the token against target-recension evidence and create or reject a lexical candidate |
| 309 | `unknown-lexeme` | `тꙋ́не` | 78 | 74 | review the token against target-recension evidence and create or reject a lexical candidate |
| 310 | `missing-accent-or-orthographic-metadata` | `достоѧ́нїе` | 78 | 74 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 311 | `unknown-lexeme` | `ᲂу҆го́дно` | 77 | 77 | review the token against target-recension evidence and create or reject a lexical candidate |
| 312 | `unknown-lexeme` | `возлїѧ́нїѧ` | 77 | 73 | review the token against target-recension evidence and create or reject a lexical candidate |
| 313 | `unknown-lexeme` | `вы́ше` | 77 | 73 | review the token against target-recension evidence and create or reject a lexical candidate |
| 314 | `unknown-lexeme` | `возра́дꙋютсѧ` | 77 | 71 | review the token against target-recension evidence and create or reject a lexical candidate |
| 315 | `unknown-lexeme` | `ѹ҆́шы` | 77 | 68 | review the token against target-recension evidence and create or reject a lexical candidate |
| 316 | `unknown-lexeme` | `мно́гꙋ` | 76 | 76 | review the token against target-recension evidence and create or reject a lexical candidate |
| 317 | `unknown-lexeme` | `плѣне́нїѧ` | 76 | 76 | review the token against target-recension evidence and create or reject a lexical candidate |
| 318 | `unknown-lexeme` | `ване́а` | 76 | 74 | review the token against target-recension evidence and create or reject a lexical candidate |
| 319 | `unknown-lexeme` | `вѣ́съ` | 76 | 74 | review the token against target-recension evidence and create or reject a lexical candidate |
| 320 | `unknown-lexeme` | `по́йдемъ` | 76 | 72 | review the token against target-recension evidence and create or reject a lexical candidate |
| 321 | `unknown-lexeme` | `ст҃ы̑мъ` | 76 | 72 | review the token against target-recension evidence and create or reject a lexical candidate |
| 322 | `unknown-lexeme` | `ꙗ҆́звою` | 76 | 72 | review the token against target-recension evidence and create or reject a lexical candidate |
| 323 | `unknown-lexeme` | `и҆спе́рва` | 76 | 70 | review the token against target-recension evidence and create or reject a lexical candidate |
| 324 | `unknown-lexeme` | `златы́хъ` | 76 | 64 | review the token against target-recension evidence and create or reject a lexical candidate |
| 325 | `unknown-lexeme` | `возглаго́лю` | 75 | 75 | review the token against target-recension evidence and create or reject a lexical candidate |
| 326 | `unknown-lexeme` | `поги́бель` | 75 | 73 | review the token against target-recension evidence and create or reject a lexical candidate |
| 327 | `unknown-lexeme` | `стра́жꙋ` | 75 | 73 | review the token against target-recension evidence and create or reject a lexical candidate |
| 328 | `unknown-lexeme` | `ѡ҆держа́нїе` | 75 | 71 | review the token against target-recension evidence and create or reject a lexical candidate |
| 329 | `unknown-lexeme` | `бл҃гъ` | 74 | 74 | review the token against target-recension evidence and create or reject a lexical candidate |
| 330 | `unknown-lexeme` | `царе́во` | 74 | 74 | review the token against target-recension evidence and create or reject a lexical candidate |
| 331 | `unknown-lexeme` | `ѻ҆внѡ́въ` | 74 | 74 | review the token against target-recension evidence and create or reject a lexical candidate |
| 332 | `unknown-lexeme` | `воззва̀` | 74 | 73 | review the token against target-recension evidence and create or reject a lexical candidate |
| 333 | `unknown-lexeme` | `даві́довъ` | 74 | 73 | review the token against target-recension evidence and create or reject a lexical candidate |
| 334 | `unknown-lexeme` | `сребро́мъ` | 74 | 70 | review the token against target-recension evidence and create or reject a lexical candidate |
| 335 | `unknown-lexeme` | `ды́мъ` | 74 | 68 | review the token against target-recension evidence and create or reject a lexical candidate |
| 336 | `unknown-lexeme` | `и҆збра́хъ` | 74 | 66 | review the token against target-recension evidence and create or reject a lexical candidate |
| 337 | `unknown-lexeme` | `подо́бїе` | 74 | 66 | review the token against target-recension evidence and create or reject a lexical candidate |
| 338 | `unknown-lexeme` | `прїе́млетъ` | 74 | 57 | review the token against target-recension evidence and create or reject a lexical candidate |
| 339 | `unknown-lexeme` | `мно́жествꙋ` | 73 | 73 | review the token against target-recension evidence and create or reject a lexical candidate |
| 340 | `unknown-lexeme` | `царє́мъ` | 73 | 73 | review the token against target-recension evidence and create or reject a lexical candidate |
| 341 | `unknown-lexeme` | `а҆рхїере́й` | 73 | 71 | review the token against target-recension evidence and create or reject a lexical candidate |
| 342 | `ambiguity-or-spelling-variant` | `ꙗ҆́вѣ` | 73 | 71 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 343 | `unknown-lexeme` | `сꙋббѡ́тꙋ` | 73 | 69 | review the token against target-recension evidence and create or reject a lexical candidate |
| 344 | `unknown-lexeme` | `црⷭ҇тво` | 73 | 60 | review the token against target-recension evidence and create or reject a lexical candidate |
| 345 | `unknown-lexeme` | `восхо́щетъ` | 72 | 72 | review the token against target-recension evidence and create or reject a lexical candidate |
| 346 | `unknown-lexeme` | `всели́шасѧ` | 72 | 72 | review the token against target-recension evidence and create or reject a lexical candidate |
| 347 | `unknown-lexeme` | `и҆спо́лнь` | 72 | 72 | review the token against target-recension evidence and create or reject a lexical candidate |
| 348 | `unknown-lexeme` | `манассі́ина` | 72 | 72 | review the token against target-recension evidence and create or reject a lexical candidate |
| 349 | `unknown-lexeme` | `і҆еровоа́ма` | 72 | 72 | review the token against target-recension evidence and create or reject a lexical candidate |
| 350 | `unknown-lexeme` | `ѡ҆скꙋдѣ́етъ` | 72 | 72 | review the token against target-recension evidence and create or reject a lexical candidate |
| 351 | `unknown-lexeme` | `ѳѷмїа́мъ` | 72 | 72 | review the token against target-recension evidence and create or reject a lexical candidate |
| 352 | `unknown-lexeme` | `дрꙋгѡ́въ` | 72 | 70 | review the token against target-recension evidence and create or reject a lexical candidate |
| 353 | `unknown-lexeme` | `повелѣ́нїемъ` | 72 | 70 | review the token against target-recension evidence and create or reject a lexical candidate |
| 354 | `unknown-lexeme` | `ѡ҆бита́ющихъ` | 72 | 70 | review the token against target-recension evidence and create or reject a lexical candidate |
| 355 | `unknown-lexeme` | `по́йдетъ` | 72 | 68 | review the token against target-recension evidence and create or reject a lexical candidate |
| 356 | `unknown-lexeme` | `полцѣ̀` | 72 | 68 | review the token against target-recension evidence and create or reject a lexical candidate |
| 357 | `unknown-lexeme` | `ѡ҆чи́ститъ` | 72 | 66 | review the token against target-recension evidence and create or reject a lexical candidate |
| 358 | `unknown-lexeme` | `прозва̀` | 72 | 64 | review the token against target-recension evidence and create or reject a lexical candidate |
| 359 | `unknown-lexeme` | `і҆́ѡвъ` | 72 | 63 | review the token against target-recension evidence and create or reject a lexical candidate |
| 360 | `unknown-lexeme` | `стѡѧ́ла` | 72 | 49 | review the token against target-recension evidence and create or reject a lexical candidate |
| 361 | `unknown-lexeme` | `возмо́гꙋтъ` | 71 | 71 | review the token against target-recension evidence and create or reject a lexical candidate |
| 362 | `unknown-lexeme` | `сни́детъ` | 71 | 71 | review the token against target-recension evidence and create or reject a lexical candidate |
| 363 | `missing-accent-or-orthographic-metadata` | `возвратѧ́тсѧ` | 71 | 69 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 364 | `unknown-lexeme` | `ви́диши` | 70 | 70 | review the token against target-recension evidence and create or reject a lexical candidate |
| 365 | `unknown-lexeme` | `хеврѡ́нъ` | 70 | 70 | review the token against target-recension evidence and create or reject a lexical candidate |
| 366 | `unknown-lexeme` | `і҆и҃левѣ` | 70 | 70 | review the token against target-recension evidence and create or reject a lexical candidate |
| 367 | `unknown-lexeme` | `ѡ҆бита́ти` | 70 | 70 | review the token against target-recension evidence and create or reject a lexical candidate |
| 368 | `unknown-lexeme` | `валаа́мъ` | 70 | 68 | review the token against target-recension evidence and create or reject a lexical candidate |
| 369 | `unknown-lexeme` | `пла́касѧ` | 70 | 68 | review the token against target-recension evidence and create or reject a lexical candidate |
| 370 | `unknown-lexeme` | `повѣ́даша` | 70 | 68 | review the token against target-recension evidence and create or reject a lexical candidate |
| 371 | `unknown-lexeme` | `писа́нїѧ` | 70 | 67 | review the token against target-recension evidence and create or reject a lexical candidate |
| 372 | `unknown-lexeme` | `же́ртвенникъ` | 70 | 66 | review the token against target-recension evidence and create or reject a lexical candidate |
| 373 | `unknown-lexeme` | `премꙋ́дръ` | 70 | 66 | review the token against target-recension evidence and create or reject a lexical candidate |
| 374 | `unknown-lexeme` | `рахи́ль` | 70 | 66 | review the token against target-recension evidence and create or reject a lexical candidate |
| 375 | `ambiguity-or-spelling-variant` | `даѧ́ти` | 70 | 66 | review whether this is a permitted Synodal spelling variant, abbreviation, or distinct lexeme |
| 376 | `unknown-lexeme` | `и҆ноплеме́нникѡвъ` | 69 | 69 | review the token against target-recension evidence and create or reject a lexical candidate |
| 377 | `unknown-lexeme` | `блгⷭ҇ве́нїе` | 69 | 67 | review the token against target-recension evidence and create or reject a lexical candidate |
| 378 | `unknown-lexeme` | `проро́цы` | 69 | 67 | review the token against target-recension evidence and create or reject a lexical candidate |
| 379 | `unknown-lexeme` | `скотѡ́въ` | 69 | 67 | review the token against target-recension evidence and create or reject a lexical candidate |
| 380 | `unknown-lexeme` | `благово́нїѧ` | 68 | 68 | review the token against target-recension evidence and create or reject a lexical candidate |
| 381 | `unknown-lexeme` | `заха́рїа` | 68 | 68 | review the token against target-recension evidence and create or reject a lexical candidate |
| 382 | `unknown-lexeme` | `и҆зведо́хъ` | 68 | 68 | review the token against target-recension evidence and create or reject a lexical candidate |
| 383 | `unknown-lexeme` | `пришле́цъ` | 68 | 68 | review the token against target-recension evidence and create or reject a lexical candidate |
| 384 | `unknown-lexeme` | `созида́ти` | 68 | 68 | review the token against target-recension evidence and create or reject a lexical candidate |
| 385 | `unknown-lexeme` | `вселѧ́тсѧ` | 68 | 66 | review the token against target-recension evidence and create or reject a lexical candidate |
| 386 | `unknown-lexeme` | `повелѣ́нїю` | 68 | 66 | review the token against target-recension evidence and create or reject a lexical candidate |
| 387 | `unknown-lexeme` | `трꙋба́ми` | 68 | 66 | review the token against target-recension evidence and create or reject a lexical candidate |
| 388 | `unknown-lexeme` | `а҆вени́ръ` | 68 | 64 | review the token against target-recension evidence and create or reject a lexical candidate |
| 389 | `unknown-lexeme` | `со́лнцемъ` | 68 | 64 | review the token against target-recension evidence and create or reject a lexical candidate |
| 390 | `unknown-lexeme` | `снѣ́стъ` | 68 | 60 | review the token against target-recension evidence and create or reject a lexical candidate |
| 391 | `unknown-lexeme` | `безче́стїе` | 67 | 67 | review the token against target-recension evidence and create or reject a lexical candidate |
| 392 | `unknown-lexeme` | `ле́стїю` | 67 | 67 | review the token against target-recension evidence and create or reject a lexical candidate |
| 393 | `unknown-lexeme` | `бѣ́хꙋ` | 67 | 65 | review the token against target-recension evidence and create or reject a lexical candidate |
| 394 | `unknown-lexeme` | `возда́мъ` | 67 | 65 | review the token against target-recension evidence and create or reject a lexical candidate |
| 395 | `unknown-lexeme` | `стєзѝ` | 67 | 65 | review the token against target-recension evidence and create or reject a lexical candidate |
| 396 | `unknown-lexeme` | `ѻ҆де́ждꙋ` | 67 | 65 | review the token against target-recension evidence and create or reject a lexical candidate |
| 397 | `unknown-lexeme` | `ѿи́метъ` | 67 | 63 | review the token against target-recension evidence and create or reject a lexical candidate |
| 398 | `unknown-lexeme` | `сожжѐ` | 67 | 61 | review the token against target-recension evidence and create or reject a lexical candidate |
| 399 | `unknown-lexeme` | `а҆́гнцєвъ` | 66 | 66 | review the token against target-recension evidence and create or reject a lexical candidate |
| 400 | `unknown-lexeme` | `посла́нїе` | 66 | 66 | review the token against target-recension evidence and create or reject a lexical candidate |
| 401 | `unknown-lexeme` | `совершѝ` | 66 | 66 | review the token against target-recension evidence and create or reject a lexical candidate |
| 402 | `unknown-lexeme` | `царе́вꙋ` | 66 | 66 | review the token against target-recension evidence and create or reject a lexical candidate |
| 403 | `unknown-lexeme` | `ѻ҆́троки` | 66 | 66 | review the token against target-recension evidence and create or reject a lexical candidate |
| 404 | `missing-accent-or-orthographic-metadata` | `свидѣ̑нїѧ` | 66 | 66 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 405 | `ambiguity-or-spelling-variant` | `ѻ҆смы́й` | 66 | 66 | review whether this is a permitted Synodal spelling variant, abbreviation, or distinct lexeme |
| 406 | `missing-accent-or-orthographic-metadata` | `возвѣсти́ти` | 66 | 65 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 407 | `unknown-lexeme` | `вѣкѡ́въ` | 66 | 64 | review the token against target-recension evidence and create or reject a lexical candidate |
| 408 | `unknown-lexeme` | `гедеѡ́нъ` | 66 | 64 | review the token against target-recension evidence and create or reject a lexical candidate |
| 409 | `unknown-lexeme` | `непоро́ченъ` | 66 | 64 | review the token against target-recension evidence and create or reject a lexical candidate |
| 410 | `unknown-lexeme` | `сто́гнахъ` | 66 | 64 | review the token against target-recension evidence and create or reject a lexical candidate |
| 411 | `unknown-lexeme` | `трапе́зꙋ` | 66 | 64 | review the token against target-recension evidence and create or reject a lexical candidate |
| 412 | `unknown-lexeme` | `бл҃же́ни` | 66 | 63 | review the token against target-recension evidence and create or reject a lexical candidate |
| 413 | `unknown-lexeme` | `нача́тки` | 66 | 62 | review the token against target-recension evidence and create or reject a lexical candidate |
| 414 | `unknown-lexeme` | `ѹ҆чн҃цы̀` | 66 | 58 | review the token against target-recension evidence and create or reject a lexical candidate |
| 415 | `unknown-lexeme` | `а҆ма́нъ` | 66 | 56 | review the token against target-recension evidence and create or reject a lexical candidate |
| 416 | `unknown-lexeme` | `бли́жнѧгѡ` | 66 | 54 | review the token against target-recension evidence and create or reject a lexical candidate |
| 417 | `unknown-lexeme` | `і҆а́кѡвлѧ` | 65 | 65 | review the token against target-recension evidence and create or reject a lexical candidate |
| 418 | `unknown-lexeme` | `ѡ҆быдо́ша` | 65 | 65 | review the token against target-recension evidence and create or reject a lexical candidate |
| 419 | `unknown-lexeme` | `ѹ҆́зрѧтъ` | 65 | 64 | review the token against target-recension evidence and create or reject a lexical candidate |
| 420 | `unknown-lexeme` | `златы̑ѧ` | 65 | 63 | review the token against target-recension evidence and create or reject a lexical candidate |
| 421 | `unknown-lexeme` | `нечести́вый` | 65 | 63 | review the token against target-recension evidence and create or reject a lexical candidate |
| 422 | `unknown-lexeme` | `сꙋббѡ́тный` | 65 | 59 | review the token against target-recension evidence and create or reject a lexical candidate |
| 423 | `unknown-lexeme` | `творѧ́щымъ` | 65 | 59 | review the token against target-recension evidence and create or reject a lexical candidate |
| 424 | `unknown-lexeme` | `прорцы̀` | 65 | 55 | review the token against target-recension evidence and create or reject a lexical candidate |
| 425 | `unknown-lexeme` | `ве́лїѧ` | 64 | 64 | review the token against target-recension evidence and create or reject a lexical candidate |
| 426 | `unknown-lexeme` | `возвесели́тсѧ` | 64 | 64 | review the token against target-recension evidence and create or reject a lexical candidate |
| 427 | `unknown-lexeme` | `вчера̀` | 64 | 64 | review the token against target-recension evidence and create or reject a lexical candidate |
| 428 | `unknown-lexeme` | `првⷣный` | 64 | 64 | review the token against target-recension evidence and create or reject a lexical candidate |
| 429 | `unknown-lexeme` | `преда́мъ` | 64 | 64 | review the token against target-recension evidence and create or reject a lexical candidate |
| 430 | `unknown-lexeme` | `сꙋдїѧ̀` | 64 | 64 | review the token against target-recension evidence and create or reject a lexical candidate |
| 431 | `unknown-lexeme` | `ѹ҆би́ти` | 64 | 64 | review the token against target-recension evidence and create or reject a lexical candidate |
| 432 | `missing-accent-or-orthographic-metadata` | `возвѣстѧ́тъ` | 64 | 64 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 433 | `missing-accent-or-orthographic-metadata` | `пѧ́тый` | 64 | 64 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 434 | `unknown-lexeme` | `ѹ҆́зритъ` | 64 | 63 | review the token against target-recension evidence and create or reject a lexical candidate |
| 435 | `unknown-lexeme` | `вала́къ` | 64 | 62 | review the token against target-recension evidence and create or reject a lexical candidate |
| 436 | `unknown-lexeme` | `даві́дѡвы` | 64 | 62 | review the token against target-recension evidence and create or reject a lexical candidate |
| 437 | `unknown-lexeme` | `и҆сповѣ́мсѧ` | 64 | 62 | review the token against target-recension evidence and create or reject a lexical candidate |
| 438 | `unknown-lexeme` | `про́йде` | 64 | 62 | review the token against target-recension evidence and create or reject a lexical candidate |
| 439 | `unknown-lexeme` | `со́нмꙋ` | 64 | 62 | review the token against target-recension evidence and create or reject a lexical candidate |
| 440 | `unknown-lexeme` | `вотщѐ` | 64 | 61 | review the token against target-recension evidence and create or reject a lexical candidate |
| 441 | `unknown-lexeme` | `а҆вра́мъ` | 64 | 60 | review the token against target-recension evidence and create or reject a lexical candidate |
| 442 | `unknown-lexeme` | `нечи́ста` | 64 | 59 | review the token against target-recension evidence and create or reject a lexical candidate |
| 443 | `unknown-lexeme` | `лава́нъ` | 64 | 58 | review the token against target-recension evidence and create or reject a lexical candidate |
| 444 | `unknown-lexeme` | `пи́ръ` | 64 | 58 | review the token against target-recension evidence and create or reject a lexical candidate |
| 445 | `unknown-lexeme` | `пома́за` | 64 | 58 | review the token against target-recension evidence and create or reject a lexical candidate |
| 446 | `unknown-lexeme` | `ровоа́мъ` | 64 | 58 | review the token against target-recension evidence and create or reject a lexical candidate |
| 447 | `unknown-lexeme` | `ѻ҆лтарю̀` | 64 | 57 | review the token against target-recension evidence and create or reject a lexical candidate |
| 448 | `unknown-lexeme` | `а҆хаа́въ` | 64 | 54 | review the token against target-recension evidence and create or reject a lexical candidate |
| 449 | `unknown-lexeme` | `десни́цꙋ` | 63 | 63 | review the token against target-recension evidence and create or reject a lexical candidate |
| 450 | `unknown-lexeme` | `слꙋ́жатъ` | 63 | 63 | review the token against target-recension evidence and create or reject a lexical candidate |
| 451 | `unknown-lexeme` | `сп҃сѐ` | 63 | 63 | review the token against target-recension evidence and create or reject a lexical candidate |
| 452 | `unknown-lexeme` | `чꙋжди́хъ` | 63 | 63 | review the token against target-recension evidence and create or reject a lexical candidate |
| 453 | `unknown-lexeme` | `земны́хъ` | 63 | 61 | review the token against target-recension evidence and create or reject a lexical candidate |
| 454 | `unknown-lexeme` | `быва́ютъ` | 63 | 60 | review the token against target-recension evidence and create or reject a lexical candidate |
| 455 | `unknown-lexeme` | `пре́йдетъ` | 63 | 57 | review the token against target-recension evidence and create or reject a lexical candidate |
| 456 | `unknown-lexeme` | `а҆ссѷрі́йска` | 62 | 62 | review the token against target-recension evidence and create or reject a lexical candidate |
| 457 | `unknown-lexeme` | `возмого́ша` | 62 | 62 | review the token against target-recension evidence and create or reject a lexical candidate |
| 458 | `unknown-lexeme` | `и҆збере́тъ` | 62 | 62 | review the token against target-recension evidence and create or reject a lexical candidate |
| 459 | `unknown-lexeme` | `мы́шцею` | 62 | 62 | review the token against target-recension evidence and create or reject a lexical candidate |
| 460 | `unknown-lexeme` | `ра́доватисѧ` | 62 | 62 | review the token against target-recension evidence and create or reject a lexical candidate |
| 461 | `unknown-lexeme` | `созда́ти` | 62 | 62 | review the token against target-recension evidence and create or reject a lexical candidate |
| 462 | `missing-accent-or-orthographic-metadata` | `жи́вꙋ` | 62 | 62 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 463 | `unknown-lexeme` | `погꙋбле́нїе` | 62 | 61 | review the token against target-recension evidence and create or reject a lexical candidate |
| 464 | `unknown-lexeme` | `слꙋже́нїе` | 62 | 60 | review the token against target-recension evidence and create or reject a lexical candidate |
| 465 | `ambiguity-or-spelling-variant` | `и҆но́мꙋ` | 62 | 58 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 466 | `unknown-lexeme` | `мо́гꙋтъ` | 62 | 57 | review the token against target-recension evidence and create or reject a lexical candidate |
| 467 | `unknown-lexeme` | `Матѳ` | 61 | 61 | review the token against target-recension evidence and create or reject a lexical candidate |
| 468 | `unknown-lexeme` | `превозноси́те` | 61 | 61 | review the token against target-recension evidence and create or reject a lexical candidate |
| 469 | `unknown-lexeme` | `воздаѧ́нїе` | 61 | 59 | review the token against target-recension evidence and create or reject a lexical candidate |
| 470 | `unknown-lexeme` | `ро́въ` | 61 | 57 | review the token against target-recension evidence and create or reject a lexical candidate |
| 471 | `unknown-lexeme` | `а҆нтїо́хъ` | 60 | 60 | review the token against target-recension evidence and create or reject a lexical candidate |
| 472 | `unknown-lexeme` | `безꙋ́мныхъ` | 60 | 60 | review the token against target-recension evidence and create or reject a lexical candidate |
| 473 | `unknown-lexeme` | `венїамі́ни` | 60 | 60 | review the token against target-recension evidence and create or reject a lexical candidate |
| 474 | `unknown-lexeme` | `вонмѝ` | 60 | 60 | review the token against target-recension evidence and create or reject a lexical candidate |
| 475 | `unknown-lexeme` | `высѡ́каѧ` | 60 | 60 | review the token against target-recension evidence and create or reject a lexical candidate |
| 476 | `unknown-lexeme` | `крѣ́пкїй` | 60 | 60 | review the token against target-recension evidence and create or reject a lexical candidate |
| 477 | `unknown-lexeme` | `навꙋходоно́сора` | 60 | 60 | review the token against target-recension evidence and create or reject a lexical candidate |
| 478 | `unknown-lexeme` | `помышле́нїе` | 60 | 60 | review the token against target-recension evidence and create or reject a lexical candidate |
| 479 | `unknown-lexeme` | `по́слетъ` | 60 | 60 | review the token against target-recension evidence and create or reject a lexical candidate |
| 480 | `unknown-lexeme` | `совѣ́та` | 60 | 60 | review the token against target-recension evidence and create or reject a lexical candidate |
| 481 | `unknown-lexeme` | `ѳѷмїа́ма` | 60 | 60 | review the token against target-recension evidence and create or reject a lexical candidate |
| 482 | `unknown-lexeme` | `ѻ҆зі́а` | 60 | 60 | review the token against target-recension evidence and create or reject a lexical candidate |
| 483 | `unknown-lexeme` | `ѻ҆́трочищь` | 60 | 60 | review the token against target-recension evidence and create or reject a lexical candidate |
| 484 | `unknown-lexeme` | `наказа́нїѧ` | 60 | 59 | review the token against target-recension evidence and create or reject a lexical candidate |
| 485 | `unknown-lexeme` | `ѹ҆слы́шитъ` | 60 | 59 | review the token against target-recension evidence and create or reject a lexical candidate |
| 486 | `unknown-lexeme` | `воздвиго́ша` | 60 | 58 | review the token against target-recension evidence and create or reject a lexical candidate |
| 487 | `unknown-lexeme` | `возлюби́хъ` | 60 | 58 | review the token against target-recension evidence and create or reject a lexical candidate |
| 488 | `unknown-lexeme` | `пожрѐ` | 60 | 58 | review the token against target-recension evidence and create or reject a lexical candidate |
| 489 | `unknown-lexeme` | `по́йдеши` | 60 | 58 | review the token against target-recension evidence and create or reject a lexical candidate |
| 490 | `unknown-lexeme` | `поста́вихъ` | 60 | 58 | review the token against target-recension evidence and create or reject a lexical candidate |
| 491 | `unknown-lexeme` | `самарі́ю` | 60 | 58 | review the token against target-recension evidence and create or reject a lexical candidate |
| 492 | `unknown-lexeme` | `са́рра` | 60 | 58 | review the token against target-recension evidence and create or reject a lexical candidate |
| 493 | `unknown-lexeme` | `є҆динолѣ́тныхъ` | 60 | 58 | review the token against target-recension evidence and create or reject a lexical candidate |
| 494 | `unknown-lexeme` | `є҆фре́мли` | 60 | 58 | review the token against target-recension evidence and create or reject a lexical candidate |
| 495 | `unknown-lexeme` | `ѕло́бꙋ` | 60 | 58 | review the token against target-recension evidence and create or reject a lexical candidate |
| 496 | `unknown-lexeme` | `є҆сѳи́рь` | 60 | 57 | review the token against target-recension evidence and create or reject a lexical candidate |
| 497 | `unknown-lexeme` | `верєѝ` | 60 | 56 | review the token against target-recension evidence and create or reject a lexical candidate |
| 498 | `unknown-lexeme` | `вложѝ` | 60 | 56 | review the token against target-recension evidence and create or reject a lexical candidate |
| 499 | `unknown-lexeme` | `семе́й` | 60 | 56 | review the token against target-recension evidence and create or reject a lexical candidate |
| 500 | `unknown-lexeme` | `десни́ца` | 60 | 54 | review the token against target-recension evidence and create or reject a lexical candidate |
