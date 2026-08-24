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
| **generalised** (by rule) | 9868 | 2230 bp |
| memorised (exact row) | 14998 | 3389 bp |
| ambiguous | 265 | 59 bp |
| unresolved | 18315 | 4138 bp |
| top-k (any analysis) | 25925 | 5858 bp |
| top-1 | 10796 | 2439 bp |

### Held-out tokens by resolver status

| Resolver status | Tokens | Share of held-out |
|---|---:|---:|
| `abbreviation-expansion` | 794 | 179 bp |
| `ambiguous` | 265 | 59 bp |
| `exact-synodal-attestation` | 14998 | 3389 bp |
| `spelling-variant` | 11 | 2 bp |
| `synodal-irregular-override` | 253 | 57 bp |
| `synodal-normative-table` | 5867 | 1325 bp |
| `synodal-productive-rule` | 3748 | 846 bp |
| `unresolved` | 18315 | 4138 bp |

### Held-out tokens by morphological system

A wave aimed at one system must be visible landing in that system.

| System | Held-out | Generalised | Memorised | Unresolved |
|---|---:|---:|---:|---:|
| `adjective` | 914 | 299 | 406 | 2 |
| `aorist` | 1913 | 354 | 1536 | 23 |
| `compound-cardinal-word` | 401 | 401 | 0 | 0 |
| `determiner` | 147 | 0 | 147 | 0 |
| `future` | 728 | 502 | 161 | 65 |
| `imperative` | 167 | 57 | 64 | 46 |
| `imperfect` | 272 | 0 | 270 | 2 |
| `indeclinable` | 1561 | 0 | 1534 | 27 |
| `infinitive` | 1464 | 1027 | 437 | 0 |
| `l-participle` | 4 | 4 | 0 | 0 |
| `lexical-form` | 1514 | 0 | 909 | 20 |
| `noun` | 11455 | 2058 | 8976 | 283 |
| `numeral` | 494 | 357 | 0 | 8 |
| `past-active-participle` | 217 | 4 | 199 | 14 |
| `past-passive-participle` | 49 | 2 | 45 | 2 |
| `present` | 805 | 465 | 314 | 26 |
| `present-active-participle` | 156 | 150 | 0 | 6 |
| `pronoun` | 4210 | 4188 | 0 | 22 |
| `unresolved` | 17780 | 0 | 0 | 17769 |

## Corpus-wide coverage

- Passages: 74130
- Tokens: 1313344
- Types: 57341
- Top-1 analyzed: 614857 (4681 bp)
- Top-k analyzed: 968073 (7371 bp)
- Ambiguous: 9502
- Unresolved: 344207

## Gap categories

| Category | Tokens |
|---|---:|
| `unknown-lexeme` | 332444 |
| `missing-declension-or-class` | 70 |
| `missing-verb-principal-part` | 72 |
| `unsupported-formation` | 0 |
| `missing-accent-or-orthographic-metadata` | 11621 |
| `ambiguity-or-spelling-variant` | 10566 |

## Coverage composition

Strict top-k counts tokens that have *any* analysis. These measures describe what
that coverage is made of, so recall cannot be bought with rows that commit to no
morphology, and so a fall in unique-reading counts can be attributed rather than
assumed. `morphology-free` tokens carry only `lexical-form` readings.
`lemma-unique` is not capped by syncretism the way top-1 is.

| Measure | Tokens | Share of top-k |
|---|---:|---:|
| morphologically typed | 917935 | 9482 bp |
| morphology-free | 50138 | 517 bp |
| lemma-unique | 958571 | 9901 bp |
| within-lexeme ambiguous (syncretism) | 340756 | 3519 bp |
| cross-lexeme ambiguous (homonymy) | 9502 | 98 bp |

## Estimated recovery routes

These are diagnostic estimates, not admitted lexical identities or guaranteed recoveries.

| Route | Tokens |
|---|---:|
| `exact-evidence` | 437 |
| `reviewed-class` | 70 |
| `reviewed-principal-part` | 72 |
| `abbreviation-registry` | 10677 |
| `spelling-variant` | 12685 |
| `unsupported-formation` | 0 |
| `ungrouped-unknown` | 321330 |

## Unresolved tokens by probable family

| Family diagnostic | Tokens | Documents | Route | Surfaces |
|---|---:|---:|---|---|
| `family:synodal:verb:vozvestiti` | 558 | 530 | `spelling-variant` | Возвѣсти́, Возвѣсти́сѧ, Возвѣсти́те, Возвѣсти́ти, Возвѣстѝ, Возвѣщꙋ̀, Возвѣщꙋ́, возвѣ́сти, возвѣсти́, возвѣсти́мъ, возвѣсти́сѧ, возвѣсти́те, возвѣсти́ти, возвѣсти́хъ, возвѣститѐ, возвѣстѝ, возвѣстѧ́тсѧ, возвѣстѧ́тъ, возвѣщꙋ̀, возвѣщꙋ́ |
| `diagnostic-family:господь` | 503 | 484 | `abbreviation-registry` | Гдⷭ҇а, Гдⷭ҇еви, Гдⷭ҇емъ, Гдⷭ҇не, Гдⷭ҇нѧ, Гдⷭ҇ꙋ, Госпо́дствꙋюще, гдⷪ҇ꙋ, гдⷭа, гдⷭеви, гдⷭемъ, гдⷭень, гдⷭи, гдⷭне, гдⷭней, гдⷭни, гдⷭню, гдⷭь, гдⷭѣ, гдⷭ҇ви, гдⷭ҇и́на, гдⷭ҇и́нъ, гдⷭ҇и́нꙋ, гдⷭ҇ней, гдⷭ҇немъ, гдⷭ҇нею, гдⷭ҇нима, гдⷭ҇ними, гдⷭ҇нь, гдⷭ҇скꙋю, гдⷭ҇ствꙋй, гдⷭ҇ьі҆́ѡвꙋ, гдⷭꙋ, го́сподемъ, госпо́дства, госпо́дствова, госпо́дствовасте, госпо́дствовати, госпо́дствоваше, госпо́дствѣ, госпо́дствꙋ, госпо́дствꙋемъ, госпо́дствꙋетъ, госпо́дствꙋйте, госпо́дствꙋютъ, госпо́дствꙋюща, госпо́дствꙋющихъ, госпо́дствꙋѧ, госпо́дствꙋѧй, госпо́дїй, госпо́дїємъ, госпо́дїѧми, господе́мъ, господи́нома, господи́номъ, господи́нꙋ |
| `family:synodal:verb:byti` | 369 | 361 | `spelling-variant` | Бы́вшей, Бы́вши, Бы́вшымъ, бы̀сть, бы̀ша, бы́вша, бы́вшаго, бы́вшагѡ, бы́вшаѧ, бы́вшей, бы́вшемъ, бы́вшемꙋ, бы́вшею, бы́вши, бы́вшихъ, бы́вшымъ, бы́вшыѧ, бы́вшїи, бы́вшїй, бы́вшїѧ, бы́вшꙋю, бы̑вша, бы̑вшаѧ, бывшїи, была́, сꙋ̑ща, сꙋ̑щаѧ, є҆́си, є҆́смы, є҆си, є҆смь |
| `family:synodal:verb:v07-ff04037d9da0c605` | 348 | 338 | `spelling-variant` | Возврати́сѧ, возврати́сѧ |
| `family:synodal:verb:v06-vzeti` | 345 | 333 | `spelling-variant` | Взѧ́, Возми́те, взе́мши, взе́мшѧ, взѧ́, взѧ́ста, взѧ́сте, взѧ́сѧ, взѧ́ти, взѧ́то, взѧ́тое, взѧ́тїе, взѧ́тїѧ, взѧ́хомъ, взѧ̑та, взѧ̑ты, взѧта̀, взѧта̑, взѧты̀, возми́, возми́те, возми́тесѧ, возмꙋ́ |
| `ungrouped:старѣйшины` | 252 | 242 | `ungrouped-unknown` | Старѣ̑йшины, старѣ́йшины, старѣ̑йшины |
| `family:synodal:noun:v06-673b2df93b4f89a8` | 249 | 241 | `spelling-variant` | Зна́менїѧ, Зна́мєнїѧ, зна́меньми, зна́менїемъ, зна́менїи, зна́менїихъ, зна́менїй, зна́менїє, зна́менїємъ, зна́менїѧ, зна́мєнїи, зна́мєнїѧ, знаме́нїе, знамє́нїѧ |
| `family:synodal:adverb:wikt-4f4b6240f36e` | 239 | 199 | `spelling-variant` | Го́ре, го́ре, горѣ́ |
| `family:synodal:pronoun:inyi` | 234 | 229 | `spelling-variant` | И҆ны̑ѧ, и҆́наго, и҆́нымъ, и҆́ныхъ, и҆́ныѧ, и҆на́го, и҆на́ѧ, и҆на̑ѧ, и҆ны́мъ, и҆ны́хъ, и҆ны́ѧ, и҆ны̑мъ, и҆ны̑ѧ |
| `ungrouped:неправды` | 225 | 215 | `ungrouped-unknown` | непра́вды, непра̑вды |
| `ungrouped:царствова` | 224 | 208 | `ungrouped-unknown` | ца́рствова |
| `ungrouped:двєри` | 218 | 208 | `ungrouped-unknown` | Двє́ри, двє́ри |
| `ungrouped:свѧщенницы` | 213 | 209 | `ungrouped-unknown` | Свѧще́нницы, свѧще́нницы |
| `ungrouped:ѡполчишасѧ` | 213 | 197 | `ungrouped-unknown` | ѡ҆полчи́шасѧ |
| `ungrouped:ꙗвисѧ` | 213 | 205 | `ungrouped-unknown` | Ꙗ҆ви́сѧ, ꙗ҆ви́сѧ |
| `ungrouped:нѣцыи` | 211 | 203 | `ungrouped-unknown` | Нѣ́цыи, нѣ́цыи |
| `ungrouped:предаде` | 207 | 203 | `ungrouped-unknown` | предаде́, предадѐ |
| `ungrouped:востани` | 204 | 183 | `ungrouped-unknown` | Воста́ни, воста́ни, востанѝ |
| `ungrouped:лактей` | 200 | 119 | `ungrouped-unknown` | ла́ктей, лакте́й |
| `ungrouped:внидетъ` | 199 | 193 | `ungrouped-unknown` | Вни́детъ, вни́детъ |
| `ungrouped:воньже` | 198 | 186 | `ungrouped-unknown` | Во́ньже, во́ньже |
| `family:synodal:pronoun:nekii` | 194 | 188 | `spelling-variant` | нѣ́каѧ, нѣ́кїй, нѣ̑каѧ |
| `ungrouped:жєртвы` | 193 | 187 | `ungrouped-unknown` | Жє́ртвы, жє́ртвы |
| `ungrouped:глаголаша` | 192 | 184 | `ungrouped-unknown` | Глаго́лаша, глаго́лаша |
| `ungrouped:книзѣ` | 192 | 188 | `ungrouped-unknown` | кни́зѣ |
| `ungrouped:іꙋдеє` | 192 | 192 | `ungrouped-unknown` | І҆ꙋде́є, і҆ꙋде́є |
| `ungrouped:іꙋдеи` | 190 | 188 | `ungrouped-unknown` | і҆ꙋде́и |
| `ungrouped:ѹмретъ` | 186 | 169 | `ungrouped-unknown` | ѹ҆́мретъ |
| `ungrouped:лакѡтъ` | 185 | 116 | `ungrouped-unknown` | лакѡ́тъ |
| `ungrouped:нечестивыхъ` | 184 | 182 | `ungrouped-unknown` | нечести́выхъ |
| `ungrouped:иноплемєнницы` | 181 | 167 | `ungrouped-unknown` | И҆ноплемє́нницы, и҆ноплемє́нницы |
| `ungrouped:слышахъ` | 180 | 172 | `ungrouped-unknown` | Слы́шахъ, слы́шахъ |
| `ungrouped:изведе` | 178 | 178 | `ungrouped-unknown` | И҆зведе́, и҆зведе́, и҆зведѐ |
| `ungrouped:оустъ` | 177 | 166 | `ungrouped-unknown` | ᲂу҆́стъ |
| `ungrouped:нача` | 176 | 174 | `ungrouped-unknown` | нача̀, нача́ |
| `family:synodal:determiner:sam` | 175 | 175 | `spelling-variant` | са́мымъ, са́мыѧ, са́мѣмъ, са́мꙋю, сама̀, сама́го, сама́гѡ, сама̑, самаго̀, самаго́, самагѡ̀, самагѡ́, само̀, само́й, само́мъ, самого̀, самогѡ̀, самогѡ́, самомꙋ̀, самомꙋ́, самы́мъ, самы̑ѧ, самѣ́мъ, самѣ̑мъ, самꙋ̀, самꙋ́ю |
| `ungrouped:єлицы` | 173 | 165 | `ungrouped-unknown` | Є҆ли́цы, є҆ли́цы, є҆ли̑цы, є҆лѝцы |
| `ungrouped:слышите` | 172 | 165 | `ungrouped-unknown` | Слы́шите, слы́шите |
| `family:synodal:adjective:wikt-e406458f5df6` | 171 | 164 | `spelling-variant` | вели́кагѡ, вели̑ка, вели̑каѧ, вели̑кимъ, велика̀ |
| `ungrouped:далече` | 171 | 169 | `ungrouped-unknown` | Дале́че, дале́че |
| `ungrouped:премꙋдрости` | 170 | 169 | `ungrouped-unknown` | премꙋ́дрости |
| `ungrouped:єзекіа` | 170 | 160 | `ungrouped-unknown` | Є҆зекі́а, є҆зекі́а |
| `ungrouped:ѻлтарѧ` | 169 | 145 | `ungrouped-unknown` | ѻлтарѧ̀, ѻ҆лтарѧ̀ |
| `ungrouped:паѵелъ` | 166 | 166 | `ungrouped-unknown` | Па́ѵелъ, па́ѵелъ |
| `ungrouped:творѧй` | 166 | 162 | `ungrouped-unknown` | Творѧ́й, творѧ́й |
| `ungrouped:воцарисѧ` | 164 | 164 | `ungrouped-unknown` | воцари́сѧ |
| `ungrouped:ѡправданїѧ` | 164 | 158 | `ungrouped-unknown` | Ѡ҆правда̑нїѧ, ѡ҆правда́нїѧ, ѡ҆правда̑нїѧ |
| `ungrouped:лꙋчше` | 163 | 161 | `ungrouped-unknown` | Лꙋ́чше, лꙋ́чше |
| `ungrouped:слыша` | 162 | 156 | `ungrouped-unknown` | Слы́ша, слы́ша |
| `ungrouped:єлїссей` | 158 | 136 | `ungrouped-unknown` | Є҆лїссе́й, є҆лїссе́й |
| `ungrouped:богатство` | 157 | 154 | `ungrouped-unknown` | Бога́тство, бога́тство |
| `family:synodal:adjective:chist` | 156 | 149 | `spelling-variant` | чи́ста, чи́сты, чи́стыма, чи́стымъ, чи́стыѧ, чи́стїи, чи̑ста, чи̑стаѧ, чи̑сты, чи̑стымъ, чи̑стыѧ, чиста̀ |
| `family:synodal:verb:wikt-332544509d0c` | 156 | 152 | `spelling-variant` | принесе́, принесе́сѧ, принесе́те, принесетѐ, принесо́хъ, принесꙋ́тсѧ, принесꙋ́тъ |
| `family:synodal:verb:wikt-3ca5f600ecca` | 156 | 146 | `spelling-variant` | Ѡ҆брати́, Ѡ҆брати́сѧ, ѡ҆брати́, ѡ҆брати́сѧ |
| `ungrouped:блгⷭ҇венъ` | 156 | 155 | `abbreviation-registry` | Блгⷭ҇ве́нъ, Блгⷭ҇венъ, блгⷭ҇ве́нъ |
| `ungrouped:введе` | 155 | 153 | `ungrouped-unknown` | Введе́, введе́, введѐ |
| `ungrouped:каменїемъ` | 154 | 150 | `ungrouped-unknown` | Ка́менїемъ, ка́менїемъ |
| `ungrouped:сонмѡмъ` | 154 | 152 | `ungrouped-unknown` | со́нмѡмъ |
| `ungrouped:ѻвна` | 154 | 148 | `ungrouped-unknown` | ѻ҆вна̀, ѻ҆вна́, ѻ҆вна̑ |
| `ambiguous-family:synodal:verb:imati+synodal:verb:wikt-0c6c8db63b7c` | 153 | 148 | `spelling-variant` | И҆́мамъ, И҆́мамы, И҆́мать, И҆ма́мъ, И҆мѣ́йте, И҆мꙋ́щемꙋ, И҆мꙋ́щїи, и҆́мамъ, и҆́мамы, и҆́мате, и҆́мать, и҆́маши, и҆́мꙋтсѧ, и҆́мꙋтъ, и҆ма́ть, и҆ма́ши, и҆мы́й, и҆мѣ̀, и҆мѣ̀ѧхꙋ, и҆мѣ́вшаго, и҆мѣ́вшымъ, и҆мѣ́й, и҆мѣ́йте, и҆мѣ́ла, и҆мѣ́ли, и҆мѣ́лъ, и҆мѣ́ста, и҆мѣ́ѧста, и҆мѣѧ́хꙋ, и҆мꙋ́щаго, и҆мꙋ́щагѡ, и҆мꙋ́щаѧ, и҆мꙋ́щее, и҆мꙋ́щей, и҆мꙋ́щема, и҆мꙋ́щемъ, и҆мꙋ́щемꙋ, и҆мꙋ́щи, и҆мꙋ́щими, и҆мꙋ́щихъ, и҆мꙋ́що, и҆мꙋ́щымъ, и҆мꙋ́щыѧ, и҆мꙋ́щїи, и҆мꙋ́щїѧ, и҆мꙋ́щѧ, и҆мꙋ́щꙋ, и҆мꙋ́щꙋю, и҆мꙋ̑ща |
| `ungrouped:слꙋжити` | 153 | 148 | `ungrouped-unknown` | слꙋжи́ти |
| `ungrouped:сꙋббѡты` | 153 | 138 | `ungrouped-unknown` | Сꙋббѡ̑ты, сꙋббѡ́ты, сꙋббѡ̑ты |
| `ungrouped:погꙋбити` | 152 | 150 | `ungrouped-unknown` | погꙋби́ти |
| `family:synodal:verb:wikt-6ceeefbe4e9e` | 151 | 147 | `spelling-variant` | снѣде́нїе, Ꙗ҆ды́й, ꙗ҆́дѐ, ꙗ҆де́, ꙗ҆дитѐ, ꙗ҆до́ста, ꙗ҆до́сте, ꙗ҆до́хомъ, ꙗ҆до́хъ, ꙗ҆ды́й, ꙗ҆дꙋ́ща, ꙗ҆дꙋ́щи |
| `ungrouped:оуслыша` | 150 | 149 | `ungrouped-unknown` | Оу҆слы́ша, ᲂу҆слы́ша |
| `ungrouped:ѻрꙋжїе` | 150 | 142 | `ungrouped-unknown` | ѻ҆рꙋ́жїе |
| `ungrouped:навꙋходоносоръ` | 149 | 145 | `ungrouped-unknown` | Навꙋходоно́соръ, навꙋходоно́соръ |
| `ungrouped:ѹслыша` | 149 | 148 | `ungrouped-unknown` | Ѹ҆слы́ша, ѹ҆слы́ша |
| `family:synodal:pronoun:toi` | 148 | 140 | `spelling-variant` | То́, Тогѡ́, то́, того́, тогѡ́, томꙋ́, тоѧ́, ты̑ѧ |
| `ungrouped:царева` | 148 | 144 | `ungrouped-unknown` | царе́ва |
| `ungrouped:ѻньже` | 147 | 141 | `ungrouped-unknown` | ѻ҆́ньже |
| `ungrouped:вышнѧгѡ` | 146 | 146 | `ungrouped-unknown` | вы́шнѧгѡ |
| `ungrouped:царствовати` | 146 | 144 | `ungrouped-unknown` | ца́рствовати |
| `family:synodal:numeral:vtoryi` | 145 | 142 | `spelling-variant` | Втора́ѧ, втора́ѧ, втора̑ѧ, второ́мꙋ, вторы́ми, вторы́мъ, вторы́хъ, вторы́ѧ, вторы̑мъ, вторѣ́й, вторѣ́мъ, вторꙋ́ю |
| `ungrouped:богѡвъ` | 145 | 137 | `ungrouped-unknown` | Богѡ́въ, богѡ́въ |
| `ungrouped:велїй` | 145 | 144 | `ungrouped-unknown` | Ве́лїй, ве́лїй |
| `ungrouped:книгꙋ` | 145 | 133 | `ungrouped-unknown` | кни́гꙋ |
| `ungrouped:врагѡвъ` | 144 | 140 | `ungrouped-unknown` | врагѡ́въ |
| `ungrouped:клѧтсѧ` | 144 | 144 | `ungrouped-unknown` | Клѧ́тсѧ, клѧ́тсѧ |
| `family:synodal:adjective:zlyi` | 143 | 139 | `spelling-variant` | ѕла̑, ѕлы̑, ѕлы̑мъ, ѕлы̑ѧ, ѕлѣ́е, ѕлѣ́йша, ѕлѣ́йшагѡ, ѕлѣ́йшихъ, ѕлѣ́йшїи, ѕлѣ̑йшаѧ |
| `family:synodal:numeral:pervyi` | 143 | 142 | `spelling-variant` | пе́рваѧ, пе́рвомꙋ, пе́рвыми, пе́рвымъ, пе́рвыѧ, пе́рвїи, пе́рвѣй, пе́рвѣмъ, пе́рвꙋю |
| `ungrouped:погибнетъ` | 143 | 139 | `ungrouped-unknown` | Поги́бнетъ, поги́бнетъ |
| `ungrouped:ѡдеснꙋю` | 143 | 141 | `ungrouped-unknown` | ѡ҆деснꙋ́ю |
| `ungrouped:возвратитсѧ` | 142 | 134 | `ungrouped-unknown` | возврати́тсѧ |
| `family:synodal:noun:v07-4ea57e089679d12a` | 140 | 136 | `spelling-variant` | ско́рби, скорбѝ |
| `ungrouped:веселїе` | 140 | 138 | `ungrouped-unknown` | Весе́лїе, весе́лїе |
| `ungrouped:дверїй` | 140 | 127 | `ungrouped-unknown` | две́рїй |
| `ungrouped:лꙋкавое` | 140 | 136 | `ungrouped-unknown` | лꙋка́вое |
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

## Coverage by corpus

| Corpus | Tokens | Top-1 | Top-k | Ambiguous | Unresolved |
|---|---:|---:|---:|---:|---:|
| Church Slavonic Bible | 651487 | 305567 | 480091 | 4742 | 170866 |
| Elizabeth Bible | 661857 | 309290 | 487982 | 4760 | 173341 |

## Coverage by source

| Source | Tokens | Top-1 | Top-k | Ambiguous | Unresolved |
|---|---:|---:|---:|---:|---:|
| `ponomar-elizabeth-bible-2026-08-09` | 661857 | 309290 | 487982 | 4760 | 173341 |
| `wikisource-church-slavonic-bible-2026-08-09` | 651487 | 305567 | 480091 | 4742 | 170866 |

## Coverage by partition

| Partition | Tokens | Top-1 | Top-k | Ambiguous | Unresolved |
|---|---:|---:|---:|---:|---:|
| `evaluation` | 265701 | 124335 | 196256 | 2031 | 69242 |
| `source` | 1047643 | 490522 | 771817 | 7471 | 274965 |

## Coverage by source and partition

| Source/partition | Tokens | Top-1 | Top-k | Ambiguous | Unresolved |
|---|---:|---:|---:|---:|---:|
| `ponomar-elizabeth-bible-2026-08-09:evaluation` | 134910 | 63057 | 99563 | 1055 | 35245 |
| `ponomar-elizabeth-bible-2026-08-09:source` | 526947 | 246233 | 388419 | 3705 | 138096 |
| `wikisource-church-slavonic-bible-2026-08-09:evaluation` | 130791 | 61278 | 96693 | 976 | 33997 |
| `wikisource-church-slavonic-bible-2026-08-09:source` | 520696 | 244289 | 383398 | 3766 | 136869 |

## Gap categories by source

| Source | Category | Tokens |
|---|---|---:|
| `ponomar-elizabeth-bible-2026-08-09` | `unknown-lexeme` | 167424 |
| `ponomar-elizabeth-bible-2026-08-09` | `missing-declension-or-class` | 49 |
| `ponomar-elizabeth-bible-2026-08-09` | `missing-verb-principal-part` | 36 |
| `ponomar-elizabeth-bible-2026-08-09` | `missing-accent-or-orthographic-metadata` | 5832 |
| `ponomar-elizabeth-bible-2026-08-09` | `ambiguity-or-spelling-variant` | 5294 |
| `wikisource-church-slavonic-bible-2026-08-09` | `unknown-lexeme` | 165020 |
| `wikisource-church-slavonic-bible-2026-08-09` | `missing-declension-or-class` | 21 |
| `wikisource-church-slavonic-bible-2026-08-09` | `missing-verb-principal-part` | 36 |
| `wikisource-church-slavonic-bible-2026-08-09` | `missing-accent-or-orthographic-metadata` | 5789 |
| `wikisource-church-slavonic-bible-2026-08-09` | `ambiguity-or-spelling-variant` | 5272 |

## Gap categories by partition

| Partition | Category | Tokens |
|---|---|---:|
| `evaluation` | `unknown-lexeme` | 66807 |
| `evaluation` | `missing-declension-or-class` | 12 |
| `evaluation` | `missing-verb-principal-part` | 18 |
| `evaluation` | `missing-accent-or-orthographic-metadata` | 2405 |
| `evaluation` | `ambiguity-or-spelling-variant` | 2234 |
| `source` | `unknown-lexeme` | 265637 |
| `source` | `missing-declension-or-class` | 58 |
| `source` | `missing-verb-principal-part` | 54 |
| `source` | `missing-accent-or-orthographic-metadata` | 9216 |
| `source` | `ambiguity-or-spelling-variant` | 8332 |

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
| 8 | `missing-accent-or-orthographic-metadata` | `возврати́сѧ` | 348 | 338 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 9 | `ambiguity-or-spelling-variant` | `сотвори́лъ` | 343 | 322 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 10 | `ambiguity-or-spelling-variant` | `ты́сѧщы` | 333 | 311 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 11 | `ambiguity-or-spelling-variant` | `і҆а́кѡвъ` | 297 | 279 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 12 | `ambiguity-or-spelling-variant` | `ѕло̀` | 261 | 255 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 13 | `ambiguity-or-spelling-variant` | `а҆арѡ́нъ` | 253 | 243 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 14 | `unknown-lexeme` | `старѣ́йшины` | 252 | 242 | review the token against target-recension evidence and create or reject a lexical candidate |
| 15 | `ambiguity-or-spelling-variant` | `и҆́мꙋтъ` | 242 | 223 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 16 | `ambiguity-or-spelling-variant` | `го́ре` | 238 | 198 | review whether this is a permitted Synodal spelling variant, abbreviation, or distinct lexeme |
| 17 | `unknown-lexeme` | `непра́вды` | 225 | 215 | review the token against target-recension evidence and create or reject a lexical candidate |
| 18 | `unknown-lexeme` | `ца́рствова` | 224 | 208 | review the token against target-recension evidence and create or reject a lexical candidate |
| 19 | `ambiguity-or-spelling-variant` | `вїно̀` | 221 | 207 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 20 | `unknown-lexeme` | `двє́ри` | 218 | 208 | review the token against target-recension evidence and create or reject a lexical candidate |
| 21 | `unknown-lexeme` | `свѧще́нницы` | 213 | 209 | review the token against target-recension evidence and create or reject a lexical candidate |
| 22 | `ambiguity-or-spelling-variant` | `вѣ́сть` | 213 | 207 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 23 | `unknown-lexeme` | `ꙗ҆ви́сѧ` | 213 | 205 | review the token against target-recension evidence and create or reject a lexical candidate |
| 24 | `unknown-lexeme` | `ѡ҆полчи́шасѧ` | 213 | 197 | review the token against target-recension evidence and create or reject a lexical candidate |
| 25 | `unknown-lexeme` | `нѣ́цыи` | 211 | 203 | review the token against target-recension evidence and create or reject a lexical candidate |
| 26 | `unknown-lexeme` | `предадѐ` | 207 | 203 | review the token against target-recension evidence and create or reject a lexical candidate |
| 27 | `unknown-lexeme` | `воста́ни` | 204 | 183 | review the token against target-recension evidence and create or reject a lexical candidate |
| 28 | `unknown-lexeme` | `ла́ктей` | 200 | 119 | review the token against target-recension evidence and create or reject a lexical candidate |
| 29 | `unknown-lexeme` | `вни́детъ` | 199 | 193 | review the token against target-recension evidence and create or reject a lexical candidate |
| 30 | `unknown-lexeme` | `во́ньже` | 198 | 186 | review the token against target-recension evidence and create or reject a lexical candidate |
| 31 | `unknown-lexeme` | `жє́ртвы` | 193 | 187 | review the token against target-recension evidence and create or reject a lexical candidate |
| 32 | `ambiguity-or-spelling-variant` | `своѝ` | 193 | 187 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 33 | `unknown-lexeme` | `і҆ꙋде́є` | 192 | 192 | review the token against target-recension evidence and create or reject a lexical candidate |
| 34 | `unknown-lexeme` | `кни́зѣ` | 192 | 188 | review the token against target-recension evidence and create or reject a lexical candidate |
| 35 | `unknown-lexeme` | `глаго́лаша` | 192 | 184 | review the token against target-recension evidence and create or reject a lexical candidate |
| 36 | `unknown-lexeme` | `і҆ꙋде́и` | 190 | 188 | review the token against target-recension evidence and create or reject a lexical candidate |
| 37 | `ambiguity-or-spelling-variant` | `и҆́мамъ` | 188 | 174 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 38 | `unknown-lexeme` | `ѹ҆́мретъ` | 186 | 169 | review the token against target-recension evidence and create or reject a lexical candidate |
| 39 | `unknown-lexeme` | `лакѡ́тъ` | 185 | 116 | review the token against target-recension evidence and create or reject a lexical candidate |
| 40 | `unknown-lexeme` | `нечести́выхъ` | 184 | 182 | review the token against target-recension evidence and create or reject a lexical candidate |
| 41 | `missing-accent-or-orthographic-metadata` | `возвѣстѝ` | 182 | 174 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 42 | `unknown-lexeme` | `и҆ноплемє́нницы` | 181 | 167 | review the token against target-recension evidence and create or reject a lexical candidate |
| 43 | `unknown-lexeme` | `слы́шахъ` | 180 | 172 | review the token against target-recension evidence and create or reject a lexical candidate |
| 44 | `unknown-lexeme` | `и҆зведѐ` | 178 | 178 | review the token against target-recension evidence and create or reject a lexical candidate |
| 45 | `unknown-lexeme` | `ᲂу҆́стъ` | 177 | 166 | review the token against target-recension evidence and create or reject a lexical candidate |
| 46 | `unknown-lexeme` | `нача̀` | 176 | 174 | review the token against target-recension evidence and create or reject a lexical candidate |
| 47 | `unknown-lexeme` | `є҆ли́цы` | 173 | 165 | review the token against target-recension evidence and create or reject a lexical candidate |
| 48 | `unknown-lexeme` | `слы́шите` | 172 | 165 | review the token against target-recension evidence and create or reject a lexical candidate |
| 49 | `unknown-lexeme` | `дале́че` | 171 | 169 | review the token against target-recension evidence and create or reject a lexical candidate |
| 50 | `unknown-lexeme` | `премꙋ́дрости` | 170 | 169 | review the token against target-recension evidence and create or reject a lexical candidate |
| 51 | `unknown-lexeme` | `господи́нꙋ` | 170 | 162 | review the token against target-recension evidence and create or reject a lexical candidate |
| 52 | `unknown-lexeme` | `є҆зекі́а` | 170 | 160 | review the token against target-recension evidence and create or reject a lexical candidate |
| 53 | `unknown-lexeme` | `ѻ҆лтарѧ̀` | 169 | 145 | review the token against target-recension evidence and create or reject a lexical candidate |
| 54 | `unknown-lexeme` | `па́ѵелъ` | 166 | 166 | review the token against target-recension evidence and create or reject a lexical candidate |
| 55 | `unknown-lexeme` | `творѧ́й` | 166 | 162 | review the token against target-recension evidence and create or reject a lexical candidate |
| 56 | `unknown-lexeme` | `воцари́сѧ` | 164 | 164 | review the token against target-recension evidence and create or reject a lexical candidate |
| 57 | `unknown-lexeme` | `ѡ҆правда̑нїѧ` | 164 | 158 | review the token against target-recension evidence and create or reject a lexical candidate |
| 58 | `unknown-lexeme` | `Лꙋ́чше` | 163 | 161 | review the token against target-recension evidence and create or reject a lexical candidate |
| 59 | `ambiguity-or-spelling-variant` | `и҆́мате` | 163 | 148 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 60 | `unknown-lexeme` | `слы́ша` | 162 | 156 | review the token against target-recension evidence and create or reject a lexical candidate |
| 61 | `unknown-lexeme` | `є҆лїссе́й` | 158 | 136 | review the token against target-recension evidence and create or reject a lexical candidate |
| 62 | `unknown-lexeme` | `бога́тство` | 157 | 154 | review the token against target-recension evidence and create or reject a lexical candidate |
| 63 | `unknown-lexeme` | `блгⷭ҇ве́нъ` | 156 | 155 | review the token against target-recension evidence and create or reject a lexical candidate |
| 64 | `unknown-lexeme` | `введѐ` | 155 | 153 | review the token against target-recension evidence and create or reject a lexical candidate |
| 65 | `unknown-lexeme` | `со́нмѡмъ` | 154 | 152 | review the token against target-recension evidence and create or reject a lexical candidate |
| 66 | `unknown-lexeme` | `ка́менїемъ` | 154 | 150 | review the token against target-recension evidence and create or reject a lexical candidate |
| 67 | `unknown-lexeme` | `ѻ҆вна̀` | 154 | 148 | review the token against target-recension evidence and create or reject a lexical candidate |
| 68 | `unknown-lexeme` | `слꙋжи́ти` | 153 | 148 | review the token against target-recension evidence and create or reject a lexical candidate |
| 69 | `unknown-lexeme` | `сꙋббѡ́ты` | 153 | 138 | review the token against target-recension evidence and create or reject a lexical candidate |
| 70 | `unknown-lexeme` | `погꙋби́ти` | 152 | 150 | review the token against target-recension evidence and create or reject a lexical candidate |
| 71 | `unknown-lexeme` | `ᲂу҆слы́ша` | 150 | 149 | review the token against target-recension evidence and create or reject a lexical candidate |
| 72 | `ambiguity-or-spelling-variant` | `и҆́маши` | 150 | 143 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 73 | `unknown-lexeme` | `ѻ҆рꙋ́жїе` | 150 | 142 | review the token against target-recension evidence and create or reject a lexical candidate |
| 74 | `unknown-lexeme` | `ѹ҆слы́ша` | 149 | 148 | review the token against target-recension evidence and create or reject a lexical candidate |
| 75 | `unknown-lexeme` | `навꙋходоно́соръ` | 149 | 145 | review the token against target-recension evidence and create or reject a lexical candidate |
| 76 | `unknown-lexeme` | `царе́ва` | 148 | 144 | review the token against target-recension evidence and create or reject a lexical candidate |
| 77 | `unknown-lexeme` | `ѻ҆́ньже` | 147 | 141 | review the token against target-recension evidence and create or reject a lexical candidate |
| 78 | `unknown-lexeme` | `вы́шнѧгѡ` | 146 | 146 | review the token against target-recension evidence and create or reject a lexical candidate |
| 79 | `unknown-lexeme` | `ца́рствовати` | 146 | 144 | review the token against target-recension evidence and create or reject a lexical candidate |
| 80 | `unknown-lexeme` | `ве́лїй` | 145 | 144 | review the token against target-recension evidence and create or reject a lexical candidate |
| 81 | `unknown-lexeme` | `богѡ́въ` | 145 | 137 | review the token against target-recension evidence and create or reject a lexical candidate |
| 82 | `unknown-lexeme` | `кни́гꙋ` | 145 | 133 | review the token against target-recension evidence and create or reject a lexical candidate |
| 83 | `unknown-lexeme` | `клѧ́тсѧ` | 144 | 144 | review the token against target-recension evidence and create or reject a lexical candidate |
| 84 | `unknown-lexeme` | `врагѡ́въ` | 144 | 140 | review the token against target-recension evidence and create or reject a lexical candidate |
| 85 | `unknown-lexeme` | `ѡ҆деснꙋ́ю` | 143 | 141 | review the token against target-recension evidence and create or reject a lexical candidate |
| 86 | `unknown-lexeme` | `поги́бнетъ` | 143 | 139 | review the token against target-recension evidence and create or reject a lexical candidate |
| 87 | `ambiguity-or-spelling-variant` | `по́ли` | 142 | 142 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 88 | `unknown-lexeme` | `возврати́тсѧ` | 142 | 134 | review the token against target-recension evidence and create or reject a lexical candidate |
| 89 | `unknown-lexeme` | `сі́мѡнъ` | 140 | 140 | review the token against target-recension evidence and create or reject a lexical candidate |
| 90 | `unknown-lexeme` | `весе́лїе` | 140 | 138 | review the token against target-recension evidence and create or reject a lexical candidate |
| 91 | `unknown-lexeme` | `лꙋка́вое` | 140 | 136 | review the token against target-recension evidence and create or reject a lexical candidate |
| 92 | `missing-accent-or-orthographic-metadata` | `ско́рби` | 140 | 136 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 93 | `missing-accent-or-orthographic-metadata` | `Ѡ҆брати́сѧ` | 140 | 130 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 94 | `unknown-lexeme` | `две́рїй` | 140 | 127 | review the token against target-recension evidence and create or reject a lexical candidate |
| 95 | `ambiguity-or-spelling-variant` | `нѣ́кїй` | 138 | 138 | review whether this is a permitted Synodal spelling variant, abbreviation, or distinct lexeme |
| 96 | `unknown-lexeme` | `со́нъ` | 138 | 122 | review the token against target-recension evidence and create or reject a lexical candidate |
| 97 | `unknown-lexeme` | `ѻ҆тцы̑` | 138 | 122 | review the token against target-recension evidence and create or reject a lexical candidate |
| 98 | `unknown-lexeme` | `вє́си` | 138 | 103 | review the token against target-recension evidence and create or reject a lexical candidate |
| 99 | `unknown-lexeme` | `тꙋ́къ` | 138 | 100 | review the token against target-recension evidence and create or reject a lexical candidate |
| 100 | `missing-accent-or-orthographic-metadata` | `ѻ҆нꙋ̀` | 137 | 135 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 101 | `unknown-lexeme` | `прѡ́чаѧ` | 136 | 136 | review the token against target-recension evidence and create or reject a lexical candidate |
| 102 | `unknown-lexeme` | `Воста́ша` | 136 | 134 | review the token against target-recension evidence and create or reject a lexical candidate |
| 103 | `missing-accent-or-orthographic-metadata` | `зна́мєнїѧ` | 136 | 134 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 104 | `unknown-lexeme` | `тала̑нтъ` | 136 | 101 | review the token against target-recension evidence and create or reject a lexical candidate |
| 105 | `unknown-lexeme` | `вѣне́цъ` | 134 | 134 | review the token against target-recension evidence and create or reject a lexical candidate |
| 106 | `unknown-lexeme` | `ве́лїимъ` | 132 | 131 | review the token against target-recension evidence and create or reject a lexical candidate |
| 107 | `unknown-lexeme` | `ѿкꙋ́дꙋ` | 131 | 123 | review the token against target-recension evidence and create or reject a lexical candidate |
| 108 | `unknown-lexeme` | `воздвиго́шасѧ` | 130 | 128 | review the token against target-recension evidence and create or reject a lexical candidate |
| 109 | `unknown-lexeme` | `мꙋ́жескъ` | 130 | 127 | review the token against target-recension evidence and create or reject a lexical candidate |
| 110 | `unknown-lexeme` | `ѿстꙋпѝ` | 130 | 126 | review the token against target-recension evidence and create or reject a lexical candidate |
| 111 | `ambiguity-or-spelling-variant` | `а҆арѡ́на` | 130 | 126 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 112 | `unknown-lexeme` | `нечи́стъ` | 130 | 118 | review the token against target-recension evidence and create or reject a lexical candidate |
| 113 | `unknown-lexeme` | `внеза́пꙋ` | 129 | 127 | review the token against target-recension evidence and create or reject a lexical candidate |
| 114 | `unknown-lexeme` | `разꙋмѣ́ти` | 129 | 127 | review the token against target-recension evidence and create or reject a lexical candidate |
| 115 | `unknown-lexeme` | `мꙋкѝ` | 129 | 125 | review the token against target-recension evidence and create or reject a lexical candidate |
| 116 | `ambiguity-or-spelling-variant` | `ты́сѧща` | 129 | 119 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 117 | `unknown-lexeme` | `и҆мѣ́нїѧ` | 128 | 128 | review the token against target-recension evidence and create or reject a lexical candidate |
| 118 | `unknown-lexeme` | `прїѧ́ша` | 128 | 128 | review the token against target-recension evidence and create or reject a lexical candidate |
| 119 | `unknown-lexeme` | `ꙗ҆ви́тсѧ` | 128 | 128 | review the token against target-recension evidence and create or reject a lexical candidate |
| 120 | `unknown-lexeme` | `разгнѣ́васѧ` | 128 | 126 | review the token against target-recension evidence and create or reject a lexical candidate |
| 121 | `unknown-lexeme` | `самарі́и` | 128 | 126 | review the token against target-recension evidence and create or reject a lexical candidate |
| 122 | `unknown-lexeme` | `ско́ты` | 128 | 122 | review the token against target-recension evidence and create or reject a lexical candidate |
| 123 | `unknown-lexeme` | `цр҃ь` | 128 | 117 | review the token against target-recension evidence and create or reject a lexical candidate |
| 124 | `unknown-lexeme` | `воздви́же` | 127 | 127 | review the token against target-recension evidence and create or reject a lexical candidate |
| 125 | `unknown-lexeme` | `пре́йде` | 127 | 121 | review the token against target-recension evidence and create or reject a lexical candidate |
| 126 | `unknown-lexeme` | `поклони́сѧ` | 126 | 126 | review the token against target-recension evidence and create or reject a lexical candidate |
| 127 | `missing-accent-or-orthographic-metadata` | `ты̑ѧ` | 126 | 118 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 128 | `unknown-lexeme` | `дрꙋгі́й` | 125 | 123 | review the token against target-recension evidence and create or reject a lexical candidate |
| 129 | `unknown-lexeme` | `кꙋ́пнѡ` | 125 | 123 | review the token against target-recension evidence and create or reject a lexical candidate |
| 130 | `unknown-lexeme` | `є҆ле́й` | 125 | 116 | review the token against target-recension evidence and create or reject a lexical candidate |
| 131 | `missing-accent-or-orthographic-metadata` | `взѧ́ти` | 124 | 121 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 132 | `unknown-lexeme` | `вы́шнїй` | 124 | 120 | review the token against target-recension evidence and create or reject a lexical candidate |
| 133 | `unknown-lexeme` | `послꙋ́шайте` | 124 | 114 | review the token against target-recension evidence and create or reject a lexical candidate |
| 134 | `unknown-lexeme` | `поко́й` | 123 | 119 | review the token against target-recension evidence and create or reject a lexical candidate |
| 135 | `unknown-lexeme` | `вознесѐ` | 122 | 122 | review the token against target-recension evidence and create or reject a lexical candidate |
| 136 | `unknown-lexeme` | `поги́бе` | 122 | 122 | review the token against target-recension evidence and create or reject a lexical candidate |
| 137 | `unknown-lexeme` | `є҆ле́а` | 122 | 122 | review the token against target-recension evidence and create or reject a lexical candidate |
| 138 | `unknown-lexeme` | `ѡ҆ста́нокъ` | 122 | 110 | review the token against target-recension evidence and create or reject a lexical candidate |
| 139 | `unknown-lexeme` | `старѣ́йшина` | 122 | 76 | review the token against target-recension evidence and create or reject a lexical candidate |
| 140 | `unknown-lexeme` | `про́чїи` | 121 | 121 | review the token against target-recension evidence and create or reject a lexical candidate |
| 141 | `unknown-lexeme` | `і҆еремі́а` | 121 | 119 | review the token against target-recension evidence and create or reject a lexical candidate |
| 142 | `unknown-lexeme` | `слы́шати` | 121 | 118 | review the token against target-recension evidence and create or reject a lexical candidate |
| 143 | `unknown-lexeme` | `созадѝ` | 121 | 115 | review the token against target-recension evidence and create or reject a lexical candidate |
| 144 | `unknown-lexeme` | `всели́сѧ` | 121 | 113 | review the token against target-recension evidence and create or reject a lexical candidate |
| 145 | `unknown-lexeme` | `а҆́може` | 120 | 118 | review the token against target-recension evidence and create or reject a lexical candidate |
| 146 | `unknown-lexeme` | `наведꙋ̀` | 120 | 118 | review the token against target-recension evidence and create or reject a lexical candidate |
| 147 | `unknown-lexeme` | `возда́стъ` | 120 | 112 | review the token against target-recension evidence and create or reject a lexical candidate |
| 148 | `unknown-lexeme` | `живꙋ́щыѧ` | 120 | 112 | review the token against target-recension evidence and create or reject a lexical candidate |
| 149 | `unknown-lexeme` | `возврати́шасѧ` | 118 | 118 | review the token against target-recension evidence and create or reject a lexical candidate |
| 150 | `unknown-lexeme` | `вни́дꙋтъ` | 117 | 117 | review the token against target-recension evidence and create or reject a lexical candidate |
| 151 | `unknown-lexeme` | `поги́бнꙋтъ` | 117 | 117 | review the token against target-recension evidence and create or reject a lexical candidate |
| 152 | `unknown-lexeme` | `со́нма` | 117 | 117 | review the token against target-recension evidence and create or reject a lexical candidate |
| 153 | `unknown-lexeme` | `приложѝ` | 116 | 114 | review the token against target-recension evidence and create or reject a lexical candidate |
| 154 | `unknown-lexeme` | `прїиди́те` | 116 | 114 | review the token against target-recension evidence and create or reject a lexical candidate |
| 155 | `unknown-lexeme` | `повѣ́да` | 116 | 110 | review the token against target-recension evidence and create or reject a lexical candidate |
| 156 | `unknown-lexeme` | `па́схꙋ` | 116 | 106 | review the token against target-recension evidence and create or reject a lexical candidate |
| 157 | `unknown-lexeme` | `наказа́нїе` | 115 | 113 | review the token against target-recension evidence and create or reject a lexical candidate |
| 158 | `ambiguity-or-spelling-variant` | `и҆́мамы` | 115 | 113 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 159 | `unknown-lexeme` | `воста́нетъ` | 114 | 114 | review the token against target-recension evidence and create or reject a lexical candidate |
| 160 | `unknown-lexeme` | `приложи́тъ` | 114 | 114 | review the token against target-recension evidence and create or reject a lexical candidate |
| 161 | `unknown-lexeme` | `вои́стиннꙋ` | 114 | 113 | review the token against target-recension evidence and create or reject a lexical candidate |
| 162 | `unknown-lexeme` | `восто́къ` | 114 | 108 | review the token against target-recension evidence and create or reject a lexical candidate |
| 163 | `unknown-lexeme` | `і҆исꙋ́сꙋ` | 114 | 108 | review the token against target-recension evidence and create or reject a lexical candidate |
| 164 | `unknown-lexeme` | `послѣ̑днѧѧ` | 113 | 113 | review the token against target-recension evidence and create or reject a lexical candidate |
| 165 | `unknown-lexeme` | `а҆ммѡ́нихъ` | 112 | 110 | review the token against target-recension evidence and create or reject a lexical candidate |
| 166 | `unknown-lexeme` | `вавѷлѡ́нскїй` | 112 | 110 | review the token against target-recension evidence and create or reject a lexical candidate |
| 167 | `missing-accent-or-orthographic-metadata` | `вели́кагѡ` | 112 | 110 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 168 | `unknown-lexeme` | `вы́шше` | 110 | 108 | review the token against target-recension evidence and create or reject a lexical candidate |
| 169 | `unknown-lexeme` | `заповѣ́даю` | 110 | 108 | review the token against target-recension evidence and create or reject a lexical candidate |
| 170 | `unknown-lexeme` | `ме́рзость` | 110 | 106 | review the token against target-recension evidence and create or reject a lexical candidate |
| 171 | `unknown-lexeme` | `грѧдѝ` | 110 | 104 | review the token against target-recension evidence and create or reject a lexical candidate |
| 172 | `unknown-lexeme` | `послꙋ́шай` | 110 | 104 | review the token against target-recension evidence and create or reject a lexical candidate |
| 173 | `unknown-lexeme` | `зане́же` | 109 | 109 | review the token against target-recension evidence and create or reject a lexical candidate |
| 174 | `unknown-lexeme` | `саꙋ́лꙋ` | 109 | 109 | review the token against target-recension evidence and create or reject a lexical candidate |
| 175 | `unknown-lexeme` | `колєсни́цы` | 108 | 108 | review the token against target-recension evidence and create or reject a lexical candidate |
| 176 | `unknown-lexeme` | `прибли́жисѧ` | 108 | 108 | review the token against target-recension evidence and create or reject a lexical candidate |
| 177 | `unknown-lexeme` | `высо́кихъ` | 108 | 94 | review the token against target-recension evidence and create or reject a lexical candidate |
| 178 | `unknown-lexeme` | `ѡ҆крє́стнаѧ` | 108 | 70 | review the token against target-recension evidence and create or reject a lexical candidate |
| 179 | `unknown-lexeme` | `повелѣ̑нїѧ` | 107 | 103 | review the token against target-recension evidence and create or reject a lexical candidate |
| 180 | `ambiguity-or-spelling-variant` | `чесѡ̀` | 107 | 101 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 181 | `unknown-lexeme` | `телца̀` | 107 | 96 | review the token against target-recension evidence and create or reject a lexical candidate |
| 182 | `unknown-lexeme` | `пе́рвѣе` | 106 | 106 | review the token against target-recension evidence and create or reject a lexical candidate |
| 183 | `unknown-lexeme` | `по́йдꙋтъ` | 106 | 106 | review the token against target-recension evidence and create or reject a lexical candidate |
| 184 | `unknown-lexeme` | `і҆еремі́и` | 106 | 106 | review the token against target-recension evidence and create or reject a lexical candidate |
| 185 | `unknown-lexeme` | `высотꙋ̀` | 106 | 102 | review the token against target-recension evidence and create or reject a lexical candidate |
| 186 | `unknown-lexeme` | `по́йте` | 106 | 100 | review the token against target-recension evidence and create or reject a lexical candidate |
| 187 | `unknown-lexeme` | `нача́ша` | 106 | 99 | review the token against target-recension evidence and create or reject a lexical candidate |
| 188 | `unknown-lexeme` | `дꙋбра́вы` | 105 | 105 | review the token against target-recension evidence and create or reject a lexical candidate |
| 189 | `unknown-lexeme` | `небє́сныѧ` | 105 | 105 | review the token against target-recension evidence and create or reject a lexical candidate |
| 190 | `unknown-lexeme` | `пребꙋ́детъ` | 105 | 105 | review the token against target-recension evidence and create or reject a lexical candidate |
| 191 | `unknown-lexeme` | `вавѷлѡ́нска` | 105 | 103 | review the token against target-recension evidence and create or reject a lexical candidate |
| 192 | `unknown-lexeme` | `писа́нїе` | 105 | 103 | review the token against target-recension evidence and create or reject a lexical candidate |
| 193 | `unknown-lexeme` | `поклони́шасѧ` | 104 | 104 | review the token against target-recension evidence and create or reject a lexical candidate |
| 194 | `unknown-lexeme` | `пойдꙋ̀` | 104 | 102 | review the token against target-recension evidence and create or reject a lexical candidate |
| 195 | `unknown-lexeme` | `си́льныхъ` | 104 | 102 | review the token against target-recension evidence and create or reject a lexical candidate |
| 196 | `unknown-lexeme` | `рабы̑ни` | 104 | 101 | review the token against target-recension evidence and create or reject a lexical candidate |
| 197 | `unknown-lexeme` | `фарїсе́є` | 104 | 99 | review the token against target-recension evidence and create or reject a lexical candidate |
| 198 | `unknown-lexeme` | `ѻ҆рꙋ́жїемъ` | 104 | 99 | review the token against target-recension evidence and create or reject a lexical candidate |
| 199 | `unknown-lexeme` | `помоли́сѧ` | 104 | 98 | review the token against target-recension evidence and create or reject a lexical candidate |
| 200 | `unknown-lexeme` | `є҆ле́емъ` | 104 | 96 | review the token against target-recension evidence and create or reject a lexical candidate |
| 201 | `unknown-lexeme` | `погꙋби́тъ` | 104 | 93 | review the token against target-recension evidence and create or reject a lexical candidate |
| 202 | `unknown-lexeme` | `сі̑кль` | 104 | 68 | review the token against target-recension evidence and create or reject a lexical candidate |
| 203 | `unknown-lexeme` | `Слы́шавше` | 103 | 102 | review the token against target-recension evidence and create or reject a lexical candidate |
| 204 | `missing-accent-or-orthographic-metadata` | `древа̀` | 103 | 101 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 205 | `unknown-lexeme` | `Житїѐ` | 102 | 99 | review the token against target-recension evidence and create or reject a lexical candidate |
| 206 | `unknown-lexeme` | `послы̀` | 102 | 98 | review the token against target-recension evidence and create or reject a lexical candidate |
| 207 | `unknown-lexeme` | `воню̀` | 101 | 101 | review the token against target-recension evidence and create or reject a lexical candidate |
| 208 | `unknown-lexeme` | `вы́нꙋ` | 101 | 101 | review the token against target-recension evidence and create or reject a lexical candidate |
| 209 | `unknown-lexeme` | `грѧде́тъ` | 101 | 97 | review the token against target-recension evidence and create or reject a lexical candidate |
| 210 | `unknown-lexeme` | `молю́` | 101 | 95 | review the token against target-recension evidence and create or reject a lexical candidate |
| 211 | `unknown-lexeme` | `а҆вессалѡ́мъ` | 101 | 89 | review the token against target-recension evidence and create or reject a lexical candidate |
| 212 | `unknown-lexeme` | `преста̀` | 100 | 99 | review the token against target-recension evidence and create or reject a lexical candidate |
| 213 | `unknown-lexeme` | `свы́ше` | 100 | 99 | review the token against target-recension evidence and create or reject a lexical candidate |
| 214 | `unknown-lexeme` | `кни̑ги` | 100 | 95 | review the token against target-recension evidence and create or reject a lexical candidate |
| 215 | `unknown-lexeme` | `непра́вдꙋ` | 100 | 94 | review the token against target-recension evidence and create or reject a lexical candidate |
| 216 | `unknown-lexeme` | `пе́рвенца` | 100 | 79 | review the token against target-recension evidence and create or reject a lexical candidate |
| 217 | `unknown-lexeme` | `блгⷭ҇вѝ` | 99 | 99 | review the token against target-recension evidence and create or reject a lexical candidate |
| 218 | `unknown-lexeme` | `словесє́мъ` | 99 | 99 | review the token against target-recension evidence and create or reject a lexical candidate |
| 219 | `unknown-lexeme` | `ᲂу҆чн҃цы̀` | 99 | 98 | review the token against target-recension evidence and create or reject a lexical candidate |
| 220 | `unknown-lexeme` | `првⷣныхъ` | 99 | 95 | review the token against target-recension evidence and create or reject a lexical candidate |
| 221 | `unknown-lexeme` | `и҆здале́ча` | 98 | 98 | review the token against target-recension evidence and create or reject a lexical candidate |
| 222 | `unknown-lexeme` | `покры̀` | 98 | 98 | review the token against target-recension evidence and create or reject a lexical candidate |
| 223 | `unknown-lexeme` | `првⷣнъ` | 98 | 96 | review the token against target-recension evidence and create or reject a lexical candidate |
| 224 | `unknown-lexeme` | `си́ленъ` | 98 | 96 | review the token against target-recension evidence and create or reject a lexical candidate |
| 225 | `unknown-lexeme` | `живꙋ́щымъ` | 98 | 94 | review the token against target-recension evidence and create or reject a lexical candidate |
| 226 | `unknown-lexeme` | `і҆ѡсафа́тъ` | 98 | 94 | review the token against target-recension evidence and create or reject a lexical candidate |
| 227 | `unknown-lexeme` | `ѻ҆рꙋ̑жїѧ` | 98 | 94 | review the token against target-recension evidence and create or reject a lexical candidate |
| 228 | `unknown-lexeme` | `Є҆ли́кѡ` | 98 | 92 | review the token against target-recension evidence and create or reject a lexical candidate |
| 229 | `unknown-lexeme` | `воевѡ́ды` | 96 | 96 | review the token against target-recension evidence and create or reject a lexical candidate |
| 230 | `unknown-lexeme` | `колесни́цъ` | 96 | 92 | review the token against target-recension evidence and create or reject a lexical candidate |
| 231 | `unknown-lexeme` | `ѻ҆бои́хъ` | 96 | 89 | review the token against target-recension evidence and create or reject a lexical candidate |
| 232 | `unknown-lexeme` | `снѣдѧ́тъ` | 96 | 84 | review the token against target-recension evidence and create or reject a lexical candidate |
| 233 | `unknown-lexeme` | `по́йде` | 95 | 95 | review the token against target-recension evidence and create or reject a lexical candidate |
| 234 | `unknown-lexeme` | `чрез̾` | 95 | 93 | review the token against target-recension evidence and create or reject a lexical candidate |
| 235 | `unknown-lexeme` | `помо́литсѧ` | 95 | 91 | review the token against target-recension evidence and create or reject a lexical candidate |
| 236 | `unknown-lexeme` | `послꙋ́шаетъ` | 95 | 91 | review the token against target-recension evidence and create or reject a lexical candidate |
| 237 | `unknown-lexeme` | `и҆са́ѵъ` | 95 | 85 | review the token against target-recension evidence and create or reject a lexical candidate |
| 238 | `unknown-lexeme` | `ꙗ҆́тъ` | 94 | 94 | review the token against target-recension evidence and create or reject a lexical candidate |
| 239 | `unknown-lexeme` | `дабы̀` | 94 | 93 | review the token against target-recension evidence and create or reject a lexical candidate |
| 240 | `unknown-lexeme` | `за́повѣдїй` | 94 | 93 | review the token against target-recension evidence and create or reject a lexical candidate |
| 241 | `unknown-lexeme` | `поѧ́стъ` | 94 | 92 | review the token against target-recension evidence and create or reject a lexical candidate |
| 242 | `unknown-lexeme` | `саꙋ́ла` | 94 | 92 | review the token against target-recension evidence and create or reject a lexical candidate |
| 243 | `unknown-lexeme` | `ѡ҆ста́сѧ` | 94 | 92 | review the token against target-recension evidence and create or reject a lexical candidate |
| 244 | `unknown-lexeme` | `сю́дꙋ` | 94 | 46 | review the token against target-recension evidence and create or reject a lexical candidate |
| 245 | `unknown-lexeme` | `Слы́шавъ` | 93 | 93 | review the token against target-recension evidence and create or reject a lexical candidate |
| 246 | `unknown-lexeme` | `возмо́жетъ` | 92 | 92 | review the token against target-recension evidence and create or reject a lexical candidate |
| 247 | `unknown-lexeme` | `си́льнїи` | 92 | 92 | review the token against target-recension evidence and create or reject a lexical candidate |
| 248 | `unknown-lexeme` | `ѧ҆зы́комъ` | 92 | 92 | review the token against target-recension evidence and create or reject a lexical candidate |
| 249 | `unknown-lexeme` | `поѧ́тъ` | 92 | 91 | review the token against target-recension evidence and create or reject a lexical candidate |
| 250 | `unknown-lexeme` | `і҆иꙋ́й` | 92 | 82 | review the token against target-recension evidence and create or reject a lexical candidate |
| 251 | `unknown-lexeme` | `вско́рѣ` | 91 | 91 | review the token against target-recension evidence and create or reject a lexical candidate |
| 252 | `unknown-lexeme` | `млⷭ҇тивъ` | 91 | 91 | review the token against target-recension evidence and create or reject a lexical candidate |
| 253 | `unknown-lexeme` | `погꙋблю̀` | 91 | 91 | review the token against target-recension evidence and create or reject a lexical candidate |
| 254 | `missing-accent-or-orthographic-metadata` | `бо́ги` | 91 | 90 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 255 | `unknown-lexeme` | `ᲂу҆би́` | 91 | 86 | review the token against target-recension evidence and create or reject a lexical candidate |
| 256 | `unknown-lexeme` | `ѹ҆бѝ` | 91 | 86 | review the token against target-recension evidence and create or reject a lexical candidate |
| 257 | `unknown-lexeme` | `сокрѡ́вища` | 91 | 83 | review the token against target-recension evidence and create or reject a lexical candidate |
| 258 | `unknown-lexeme` | `заповѣ́дахъ` | 90 | 90 | review the token against target-recension evidence and create or reject a lexical candidate |
| 259 | `unknown-lexeme` | `живы́й` | 90 | 88 | review the token against target-recension evidence and create or reject a lexical candidate |
| 260 | `unknown-lexeme` | `и҆збра́нныхъ` | 90 | 88 | review the token against target-recension evidence and create or reject a lexical candidate |
| 261 | `unknown-lexeme` | `сотвори́вый` | 90 | 87 | review the token against target-recension evidence and create or reject a lexical candidate |
| 262 | `unknown-lexeme` | `всѧ́чєскаѧ` | 90 | 80 | review the token against target-recension evidence and create or reject a lexical candidate |
| 263 | `unknown-lexeme` | `ст҃ы́ни` | 89 | 88 | review the token against target-recension evidence and create or reject a lexical candidate |
| 264 | `unknown-lexeme` | `мѡа́вли` | 89 | 87 | review the token against target-recension evidence and create or reject a lexical candidate |
| 265 | `unknown-lexeme` | `і҆саа́къ` | 89 | 87 | review the token against target-recension evidence and create or reject a lexical candidate |
| 266 | `unknown-lexeme` | `ко́ль` | 89 | 83 | review the token against target-recension evidence and create or reject a lexical candidate |
| 267 | `unknown-lexeme` | `нача́токъ` | 89 | 83 | review the token against target-recension evidence and create or reject a lexical candidate |
| 268 | `unknown-lexeme` | `кѡры́сти` | 89 | 77 | review the token against target-recension evidence and create or reject a lexical candidate |
| 269 | `missing-accent-or-orthographic-metadata` | `чи́ста` | 88 | 86 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 270 | `unknown-lexeme` | `воста́нꙋтъ` | 88 | 85 | review the token against target-recension evidence and create or reject a lexical candidate |
| 271 | `unknown-lexeme` | `і҆ѡнаѳа́на` | 88 | 84 | review the token against target-recension evidence and create or reject a lexical candidate |
| 272 | `unknown-lexeme` | `ѡ҆братѧ́тсѧ` | 87 | 87 | review the token against target-recension evidence and create or reject a lexical candidate |
| 273 | `missing-accent-or-orthographic-metadata` | `ничесѡ́же` | 87 | 87 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 274 | `unknown-lexeme` | `прїидꙋ̀` | 87 | 85 | review the token against target-recension evidence and create or reject a lexical candidate |
| 275 | `unknown-lexeme` | `і҆ꙋде́ѡмъ` | 87 | 85 | review the token against target-recension evidence and create or reject a lexical candidate |
| 276 | `unknown-lexeme` | `И҆мѣ́ѧй` | 87 | 83 | review the token against target-recension evidence and create or reject a lexical candidate |
| 277 | `unknown-lexeme` | `всꙋ́е` | 87 | 81 | review the token against target-recension evidence and create or reject a lexical candidate |
| 278 | `unknown-lexeme` | `комꙋ́ждо` | 87 | 81 | review the token against target-recension evidence and create or reject a lexical candidate |
| 279 | `unknown-lexeme` | `помышлє́нїѧ` | 86 | 86 | review the token against target-recension evidence and create or reject a lexical candidate |
| 280 | `unknown-lexeme` | `і҆ꙋде́ю` | 86 | 86 | review the token against target-recension evidence and create or reject a lexical candidate |
| 281 | `unknown-lexeme` | `ѡ҆снова̑нїѧ` | 86 | 86 | review the token against target-recension evidence and create or reject a lexical candidate |
| 282 | `unknown-lexeme` | `плѣне́нїе` | 86 | 84 | review the token against target-recension evidence and create or reject a lexical candidate |
| 283 | `unknown-lexeme` | `царє́вы` | 86 | 84 | review the token against target-recension evidence and create or reject a lexical candidate |
| 284 | `unknown-lexeme` | `є҆зекі́и` | 86 | 84 | review the token against target-recension evidence and create or reject a lexical candidate |
| 285 | `unknown-lexeme` | `ѡ҆полчи́сѧ` | 86 | 84 | review the token against target-recension evidence and create or reject a lexical candidate |
| 286 | `unknown-lexeme` | `є҆гѵ́птѧне` | 86 | 82 | review the token against target-recension evidence and create or reject a lexical candidate |
| 287 | `unknown-lexeme` | `высота̀` | 86 | 80 | review the token against target-recension evidence and create or reject a lexical candidate |
| 288 | `unknown-lexeme` | `преда́стъ` | 86 | 80 | review the token against target-recension evidence and create or reject a lexical candidate |
| 289 | `unknown-lexeme` | `чл҃вѣ́ческїй` | 86 | 80 | review the token against target-recension evidence and create or reject a lexical candidate |
| 290 | `unknown-lexeme` | `ᲂу҆́шы` | 86 | 78 | review the token against target-recension evidence and create or reject a lexical candidate |
| 291 | `ambiguity-or-spelling-variant` | `про́клѧтъ` | 86 | 78 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 292 | `unknown-lexeme` | `пе́рвенецъ` | 85 | 85 | review the token against target-recension evidence and create or reject a lexical candidate |
| 293 | `unknown-lexeme` | `сꙋдїи̑` | 85 | 85 | review the token against target-recension evidence and create or reject a lexical candidate |
| 294 | `unknown-lexeme` | `царе́въ` | 85 | 83 | review the token against target-recension evidence and create or reject a lexical candidate |
| 295 | `unknown-lexeme` | `ви́дитъ` | 84 | 84 | review the token against target-recension evidence and create or reject a lexical candidate |
| 296 | `unknown-lexeme` | `саваѡ́ѳъ` | 84 | 84 | review the token against target-recension evidence and create or reject a lexical candidate |
| 297 | `unknown-lexeme` | `ѕвѣ́зды` | 84 | 84 | review the token against target-recension evidence and create or reject a lexical candidate |
| 298 | `missing-accent-or-orthographic-metadata` | `и҆ны̑мъ` | 84 | 84 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 299 | `missing-accent-or-orthographic-metadata` | `и҆ны́хъ` | 84 | 83 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 300 | `missing-accent-or-orthographic-metadata` | `возвѣщꙋ̀` | 84 | 82 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 301 | `ambiguity-or-spelling-variant` | `ѻ҆́ный` | 84 | 82 | review whether this is a permitted Synodal spelling variant, abbreviation, or distinct lexeme |
| 302 | `unknown-lexeme` | `є҆леаза́ръ` | 84 | 80 | review the token against target-recension evidence and create or reject a lexical candidate |
| 303 | `unknown-lexeme` | `постыдѧ́тсѧ` | 84 | 78 | review the token against target-recension evidence and create or reject a lexical candidate |
| 304 | `unknown-lexeme` | `потреби́тъ` | 84 | 78 | review the token against target-recension evidence and create or reject a lexical candidate |
| 305 | `unknown-lexeme` | `поми́лꙋй` | 84 | 77 | review the token against target-recension evidence and create or reject a lexical candidate |
| 306 | `unknown-lexeme` | `снѣ́сте` | 84 | 76 | review the token against target-recension evidence and create or reject a lexical candidate |
| 307 | `unknown-lexeme` | `блгⷣти` | 83 | 83 | review the token against target-recension evidence and create or reject a lexical candidate |
| 308 | `missing-accent-or-orthographic-metadata` | `возми́те` | 83 | 81 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 309 | `unknown-lexeme` | `ѡ҆шꙋ́юю` | 83 | 80 | review the token against target-recension evidence and create or reject a lexical candidate |
| 310 | `unknown-lexeme` | `весе́лїемъ` | 82 | 82 | review the token against target-recension evidence and create or reject a lexical candidate |
| 311 | `unknown-lexeme` | `и҆зы́ти` | 82 | 82 | review the token against target-recension evidence and create or reject a lexical candidate |
| 312 | `unknown-lexeme` | `сокрꙋшѝ` | 82 | 82 | review the token against target-recension evidence and create or reject a lexical candidate |
| 313 | `missing-accent-or-orthographic-metadata` | `бо̀` | 82 | 82 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 314 | `unknown-lexeme` | `галїле́и` | 82 | 80 | review the token against target-recension evidence and create or reject a lexical candidate |
| 315 | `unknown-lexeme` | `даві́дова` | 82 | 80 | review the token against target-recension evidence and create or reject a lexical candidate |
| 316 | `unknown-lexeme` | `повелѣ́нїе` | 82 | 80 | review the token against target-recension evidence and create or reject a lexical candidate |
| 317 | `unknown-lexeme` | `послѣдѝ` | 82 | 79 | review the token against target-recension evidence and create or reject a lexical candidate |
| 318 | `unknown-lexeme` | `а҆вїмеле́хъ` | 82 | 78 | review the token against target-recension evidence and create or reject a lexical candidate |
| 319 | `unknown-lexeme` | `востѡ́къ` | 82 | 78 | review the token against target-recension evidence and create or reject a lexical candidate |
| 320 | `unknown-lexeme` | `ве́лїе` | 81 | 81 | review the token against target-recension evidence and create or reject a lexical candidate |
| 321 | `unknown-lexeme` | `мно́жае` | 81 | 81 | review the token against target-recension evidence and create or reject a lexical candidate |
| 322 | `unknown-lexeme` | `житїѧ̀` | 81 | 77 | review the token against target-recension evidence and create or reject a lexical candidate |
| 323 | `unknown-lexeme` | `мздꙋ̀` | 81 | 77 | review the token against target-recension evidence and create or reject a lexical candidate |
| 324 | `unknown-lexeme` | `пїла́тъ` | 81 | 76 | review the token against target-recension evidence and create or reject a lexical candidate |
| 325 | `unknown-lexeme` | `а҆рхїере́є` | 81 | 74 | review the token against target-recension evidence and create or reject a lexical candidate |
| 326 | `unknown-lexeme` | `і҆а́кѡвль` | 80 | 80 | review the token against target-recension evidence and create or reject a lexical candidate |
| 327 | `unknown-lexeme` | `вни́деши` | 80 | 78 | review the token against target-recension evidence and create or reject a lexical candidate |
| 328 | `unknown-lexeme` | `завѣща̀` | 80 | 78 | review the token against target-recension evidence and create or reject a lexical candidate |
| 329 | `unknown-lexeme` | `кни́жницы` | 80 | 78 | review the token against target-recension evidence and create or reject a lexical candidate |
| 330 | `unknown-lexeme` | `мно́жествѣ` | 80 | 78 | review the token against target-recension evidence and create or reject a lexical candidate |
| 331 | `unknown-lexeme` | `помощѝ` | 80 | 78 | review the token against target-recension evidence and create or reject a lexical candidate |
| 332 | `unknown-lexeme` | `пшени́чны` | 80 | 78 | review the token against target-recension evidence and create or reject a lexical candidate |
| 333 | `missing-accent-or-orthographic-metadata` | `и҆мы́й` | 80 | 78 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 334 | `unknown-lexeme` | `согрѣшѝ` | 80 | 76 | review the token against target-recension evidence and create or reject a lexical candidate |
| 335 | `ambiguity-or-spelling-variant` | `сѧ̀` | 80 | 76 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 336 | `unknown-lexeme` | `ю҆́гꙋ` | 80 | 75 | review the token against target-recension evidence and create or reject a lexical candidate |
| 337 | `unknown-lexeme` | `є҆фре́мъ` | 80 | 74 | review the token against target-recension evidence and create or reject a lexical candidate |
| 338 | `unknown-lexeme` | `і҆еровоа́мъ` | 80 | 74 | review the token against target-recension evidence and create or reject a lexical candidate |
| 339 | `unknown-lexeme` | `зла̑ты` | 80 | 73 | review the token against target-recension evidence and create or reject a lexical candidate |
| 340 | `missing-accent-or-orthographic-metadata` | `возвѣсти́те` | 80 | 71 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 341 | `unknown-lexeme` | `ко́жи` | 80 | 65 | review the token against target-recension evidence and create or reject a lexical candidate |
| 342 | `missing-accent-or-orthographic-metadata` | `плѡ́ти` | 80 | 62 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 343 | `unknown-lexeme` | `дв҃дꙋ` | 79 | 79 | review the token against target-recension evidence and create or reject a lexical candidate |
| 344 | `unknown-lexeme` | `пе́рсть` | 79 | 79 | review the token against target-recension evidence and create or reject a lexical candidate |
| 345 | `unknown-lexeme` | `прⷭ҇нѡ` | 79 | 79 | review the token against target-recension evidence and create or reject a lexical candidate |
| 346 | `unknown-lexeme` | `і҆ꙋдє́й` | 79 | 79 | review the token against target-recension evidence and create or reject a lexical candidate |
| 347 | `unknown-lexeme` | `і҆саа́ка` | 79 | 77 | review the token against target-recension evidence and create or reject a lexical candidate |
| 348 | `unknown-lexeme` | `нача̑лницы` | 79 | 75 | review the token against target-recension evidence and create or reject a lexical candidate |
| 349 | `unknown-lexeme` | `нача̑льницы` | 79 | 75 | review the token against target-recension evidence and create or reject a lexical candidate |
| 350 | `unknown-lexeme` | `є҆ди́ныѧ` | 79 | 72 | review the token against target-recension evidence and create or reject a lexical candidate |
| 351 | `unknown-lexeme` | `де́бри` | 79 | 71 | review the token against target-recension evidence and create or reject a lexical candidate |
| 352 | `unknown-lexeme` | `даві́довѣ` | 78 | 78 | review the token against target-recension evidence and create or reject a lexical candidate |
| 353 | `unknown-lexeme` | `нечести́вїи` | 78 | 78 | review the token against target-recension evidence and create or reject a lexical candidate |
| 354 | `unknown-lexeme` | `посе́мъ` | 78 | 78 | review the token against target-recension evidence and create or reject a lexical candidate |
| 355 | `unknown-lexeme` | `є҆́здра` | 78 | 78 | review the token against target-recension evidence and create or reject a lexical candidate |
| 356 | `unknown-lexeme` | `ѱало́мъ` | 78 | 78 | review the token against target-recension evidence and create or reject a lexical candidate |
| 357 | `unknown-lexeme` | `ѿсю́дꙋ` | 78 | 78 | review the token against target-recension evidence and create or reject a lexical candidate |
| 358 | `unknown-lexeme` | `лжꙋ̀` | 78 | 76 | review the token against target-recension evidence and create or reject a lexical candidate |
| 359 | `unknown-lexeme` | `согрѣши́хомъ` | 78 | 76 | review the token against target-recension evidence and create or reject a lexical candidate |
| 360 | `unknown-lexeme` | `старѣ́йшинъ` | 78 | 76 | review the token against target-recension evidence and create or reject a lexical candidate |
| 361 | `unknown-lexeme` | `воззрѣ́въ` | 78 | 75 | review the token against target-recension evidence and create or reject a lexical candidate |
| 362 | `unknown-lexeme` | `а҆ммѡ̑ни` | 78 | 74 | review the token against target-recension evidence and create or reject a lexical candidate |
| 363 | `unknown-lexeme` | `достоѧ́нїе` | 78 | 74 | review the token against target-recension evidence and create or reject a lexical candidate |
| 364 | `unknown-lexeme` | `тꙋ́не` | 78 | 74 | review the token against target-recension evidence and create or reject a lexical candidate |
| 365 | `unknown-lexeme` | `ᲂу҆го́дно` | 77 | 77 | review the token against target-recension evidence and create or reject a lexical candidate |
| 366 | `unknown-lexeme` | `возлїѧ́нїѧ` | 77 | 73 | review the token against target-recension evidence and create or reject a lexical candidate |
| 367 | `unknown-lexeme` | `вы́ше` | 77 | 73 | review the token against target-recension evidence and create or reject a lexical candidate |
| 368 | `unknown-lexeme` | `возра́дꙋютсѧ` | 77 | 71 | review the token against target-recension evidence and create or reject a lexical candidate |
| 369 | `unknown-lexeme` | `ѹ҆́шы` | 77 | 68 | review the token against target-recension evidence and create or reject a lexical candidate |
| 370 | `unknown-lexeme` | `мно́гꙋ` | 76 | 76 | review the token against target-recension evidence and create or reject a lexical candidate |
| 371 | `unknown-lexeme` | `плѣне́нїѧ` | 76 | 76 | review the token against target-recension evidence and create or reject a lexical candidate |
| 372 | `unknown-lexeme` | `слы́шахомъ` | 76 | 76 | review the token against target-recension evidence and create or reject a lexical candidate |
| 373 | `unknown-lexeme` | `ване́а` | 76 | 74 | review the token against target-recension evidence and create or reject a lexical candidate |
| 374 | `unknown-lexeme` | `вѣ́съ` | 76 | 74 | review the token against target-recension evidence and create or reject a lexical candidate |
| 375 | `unknown-lexeme` | `слы́шаша` | 76 | 74 | review the token against target-recension evidence and create or reject a lexical candidate |
| 376 | `unknown-lexeme` | `по́йдемъ` | 76 | 72 | review the token against target-recension evidence and create or reject a lexical candidate |
| 377 | `unknown-lexeme` | `ст҃ы̑мъ` | 76 | 72 | review the token against target-recension evidence and create or reject a lexical candidate |
| 378 | `unknown-lexeme` | `ꙗ҆́звою` | 76 | 72 | review the token against target-recension evidence and create or reject a lexical candidate |
| 379 | `unknown-lexeme` | `и҆спе́рва` | 76 | 70 | review the token against target-recension evidence and create or reject a lexical candidate |
| 380 | `unknown-lexeme` | `златы́хъ` | 76 | 64 | review the token against target-recension evidence and create or reject a lexical candidate |
| 381 | `unknown-lexeme` | `возглаго́лю` | 75 | 75 | review the token against target-recension evidence and create or reject a lexical candidate |
| 382 | `unknown-lexeme` | `поги́бель` | 75 | 73 | review the token against target-recension evidence and create or reject a lexical candidate |
| 383 | `unknown-lexeme` | `слы́шасте` | 75 | 73 | review the token against target-recension evidence and create or reject a lexical candidate |
| 384 | `unknown-lexeme` | `стра́жꙋ` | 75 | 73 | review the token against target-recension evidence and create or reject a lexical candidate |
| 385 | `unknown-lexeme` | `ѡ҆держа́нїе` | 75 | 71 | review the token against target-recension evidence and create or reject a lexical candidate |
| 386 | `unknown-lexeme` | `бл҃гъ` | 74 | 74 | review the token against target-recension evidence and create or reject a lexical candidate |
| 387 | `unknown-lexeme` | `царе́во` | 74 | 74 | review the token against target-recension evidence and create or reject a lexical candidate |
| 388 | `unknown-lexeme` | `ѻ҆внѡ́въ` | 74 | 74 | review the token against target-recension evidence and create or reject a lexical candidate |
| 389 | `unknown-lexeme` | `воззва̀` | 74 | 73 | review the token against target-recension evidence and create or reject a lexical candidate |
| 390 | `unknown-lexeme` | `даві́довъ` | 74 | 73 | review the token against target-recension evidence and create or reject a lexical candidate |
| 391 | `unknown-lexeme` | `грѧдꙋ́тъ` | 74 | 72 | review the token against target-recension evidence and create or reject a lexical candidate |
| 392 | `unknown-lexeme` | `сребро́мъ` | 74 | 70 | review the token against target-recension evidence and create or reject a lexical candidate |
| 393 | `unknown-lexeme` | `і҆ѡа́съ` | 74 | 70 | review the token against target-recension evidence and create or reject a lexical candidate |
| 394 | `unknown-lexeme` | `ды́мъ` | 74 | 68 | review the token against target-recension evidence and create or reject a lexical candidate |
| 395 | `unknown-lexeme` | `и҆збра́хъ` | 74 | 66 | review the token against target-recension evidence and create or reject a lexical candidate |
| 396 | `unknown-lexeme` | `подо́бїе` | 74 | 66 | review the token against target-recension evidence and create or reject a lexical candidate |
| 397 | `unknown-lexeme` | `прїе́млетъ` | 74 | 57 | review the token against target-recension evidence and create or reject a lexical candidate |
| 398 | `unknown-lexeme` | `глаго́лѧй` | 73 | 73 | review the token against target-recension evidence and create or reject a lexical candidate |
| 399 | `unknown-lexeme` | `мно́жествꙋ` | 73 | 73 | review the token against target-recension evidence and create or reject a lexical candidate |
| 400 | `unknown-lexeme` | `царє́мъ` | 73 | 73 | review the token against target-recension evidence and create or reject a lexical candidate |
| 401 | `unknown-lexeme` | `ѡ҆брати́тсѧ` | 73 | 73 | review the token against target-recension evidence and create or reject a lexical candidate |
| 402 | `unknown-lexeme` | `а҆рхїере́й` | 73 | 71 | review the token against target-recension evidence and create or reject a lexical candidate |
| 403 | `ambiguity-or-spelling-variant` | `ꙗ҆́вѣ` | 73 | 71 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 404 | `unknown-lexeme` | `сꙋббѡ́тꙋ` | 73 | 69 | review the token against target-recension evidence and create or reject a lexical candidate |
| 405 | `unknown-lexeme` | `црⷭ҇тво` | 73 | 60 | review the token against target-recension evidence and create or reject a lexical candidate |
| 406 | `unknown-lexeme` | `восхо́щетъ` | 72 | 72 | review the token against target-recension evidence and create or reject a lexical candidate |
| 407 | `unknown-lexeme` | `всели́шасѧ` | 72 | 72 | review the token against target-recension evidence and create or reject a lexical candidate |
| 408 | `unknown-lexeme` | `и҆спо́лнь` | 72 | 72 | review the token against target-recension evidence and create or reject a lexical candidate |
| 409 | `unknown-lexeme` | `манассі́ина` | 72 | 72 | review the token against target-recension evidence and create or reject a lexical candidate |
| 410 | `unknown-lexeme` | `і҆еровоа́ма` | 72 | 72 | review the token against target-recension evidence and create or reject a lexical candidate |
| 411 | `unknown-lexeme` | `і҆саа́кꙋ` | 72 | 72 | review the token against target-recension evidence and create or reject a lexical candidate |
| 412 | `unknown-lexeme` | `ѡ҆скꙋдѣ́етъ` | 72 | 72 | review the token against target-recension evidence and create or reject a lexical candidate |
| 413 | `unknown-lexeme` | `ѳѷмїа́мъ` | 72 | 72 | review the token against target-recension evidence and create or reject a lexical candidate |
| 414 | `unknown-lexeme` | `Согрѣши́ша` | 72 | 71 | review the token against target-recension evidence and create or reject a lexical candidate |
| 415 | `unknown-lexeme` | `дрꙋгѡ́въ` | 72 | 70 | review the token against target-recension evidence and create or reject a lexical candidate |
| 416 | `unknown-lexeme` | `повелѣ́нїемъ` | 72 | 70 | review the token against target-recension evidence and create or reject a lexical candidate |
| 417 | `unknown-lexeme` | `соберꙋ̀` | 72 | 70 | review the token against target-recension evidence and create or reject a lexical candidate |
| 418 | `unknown-lexeme` | `ѡ҆бита́ющихъ` | 72 | 70 | review the token against target-recension evidence and create or reject a lexical candidate |
| 419 | `unknown-lexeme` | `по́йдетъ` | 72 | 68 | review the token against target-recension evidence and create or reject a lexical candidate |
| 420 | `unknown-lexeme` | `полцѣ̀` | 72 | 68 | review the token against target-recension evidence and create or reject a lexical candidate |
| 421 | `unknown-lexeme` | `ѡ҆чи́ститъ` | 72 | 66 | review the token against target-recension evidence and create or reject a lexical candidate |
| 422 | `unknown-lexeme` | `прозва̀` | 72 | 64 | review the token against target-recension evidence and create or reject a lexical candidate |
| 423 | `unknown-lexeme` | `і҆́ѡвъ` | 72 | 63 | review the token against target-recension evidence and create or reject a lexical candidate |
| 424 | `unknown-lexeme` | `согрѣши́тъ` | 72 | 62 | review the token against target-recension evidence and create or reject a lexical candidate |
| 425 | `unknown-lexeme` | `стѡѧ́ла` | 72 | 49 | review the token against target-recension evidence and create or reject a lexical candidate |
| 426 | `unknown-lexeme` | `возмо́гꙋтъ` | 71 | 71 | review the token against target-recension evidence and create or reject a lexical candidate |
| 427 | `unknown-lexeme` | `слы́ши` | 71 | 71 | review the token against target-recension evidence and create or reject a lexical candidate |
| 428 | `unknown-lexeme` | `сни́детъ` | 71 | 71 | review the token against target-recension evidence and create or reject a lexical candidate |
| 429 | `missing-accent-or-orthographic-metadata` | `разꙋмѣ̀` | 71 | 70 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 430 | `unknown-lexeme` | `возвратѧ́тсѧ` | 71 | 69 | review the token against target-recension evidence and create or reject a lexical candidate |
| 431 | `unknown-lexeme` | `ви́диши` | 70 | 70 | review the token against target-recension evidence and create or reject a lexical candidate |
| 432 | `unknown-lexeme` | `непра́вда` | 70 | 70 | review the token against target-recension evidence and create or reject a lexical candidate |
| 433 | `unknown-lexeme` | `хеврѡ́нъ` | 70 | 70 | review the token against target-recension evidence and create or reject a lexical candidate |
| 434 | `unknown-lexeme` | `і҆и҃левѣ` | 70 | 70 | review the token against target-recension evidence and create or reject a lexical candidate |
| 435 | `unknown-lexeme` | `ѡ҆бита́ти` | 70 | 70 | review the token against target-recension evidence and create or reject a lexical candidate |
| 436 | `unknown-lexeme` | `валаа́мъ` | 70 | 68 | review the token against target-recension evidence and create or reject a lexical candidate |
| 437 | `unknown-lexeme` | `пла́касѧ` | 70 | 68 | review the token against target-recension evidence and create or reject a lexical candidate |
| 438 | `unknown-lexeme` | `повѣ́даша` | 70 | 68 | review the token against target-recension evidence and create or reject a lexical candidate |
| 439 | `unknown-lexeme` | `писа́нїѧ` | 70 | 67 | review the token against target-recension evidence and create or reject a lexical candidate |
| 440 | `unknown-lexeme` | `же́ртвенникъ` | 70 | 66 | review the token against target-recension evidence and create or reject a lexical candidate |
| 441 | `unknown-lexeme` | `премꙋ́дръ` | 70 | 66 | review the token against target-recension evidence and create or reject a lexical candidate |
| 442 | `unknown-lexeme` | `рахи́ль` | 70 | 66 | review the token against target-recension evidence and create or reject a lexical candidate |
| 443 | `missing-accent-or-orthographic-metadata` | `ꙗ҆ды́й` | 70 | 66 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 444 | `ambiguity-or-spelling-variant` | `даѧ́ти` | 70 | 66 | review whether this is a permitted Synodal spelling variant, abbreviation, or distinct lexeme |
| 445 | `unknown-lexeme` | `стражбы̑` | 70 | 60 | review the token against target-recension evidence and create or reject a lexical candidate |
| 446 | `unknown-lexeme` | `и҆ноплеме́нникѡвъ` | 69 | 69 | review the token against target-recension evidence and create or reject a lexical candidate |
| 447 | `unknown-lexeme` | `потреби́ти` | 69 | 69 | review the token against target-recension evidence and create or reject a lexical candidate |
| 448 | `unknown-lexeme` | `блгⷭ҇ве́нїе` | 69 | 67 | review the token against target-recension evidence and create or reject a lexical candidate |
| 449 | `unknown-lexeme` | `проро́цы` | 69 | 67 | review the token against target-recension evidence and create or reject a lexical candidate |
| 450 | `unknown-lexeme` | `скотѡ́въ` | 69 | 67 | review the token against target-recension evidence and create or reject a lexical candidate |
| 451 | `unknown-lexeme` | `спасе́тсѧ` | 69 | 67 | review the token against target-recension evidence and create or reject a lexical candidate |
| 452 | `unknown-lexeme` | `возвращꙋ̀` | 69 | 65 | review the token against target-recension evidence and create or reject a lexical candidate |
| 453 | `unknown-lexeme` | `благово́нїѧ` | 68 | 68 | review the token against target-recension evidence and create or reject a lexical candidate |
| 454 | `unknown-lexeme` | `заха́рїа` | 68 | 68 | review the token against target-recension evidence and create or reject a lexical candidate |
| 455 | `unknown-lexeme` | `и҆зведо́хъ` | 68 | 68 | review the token against target-recension evidence and create or reject a lexical candidate |
| 456 | `unknown-lexeme` | `пришле́цъ` | 68 | 68 | review the token against target-recension evidence and create or reject a lexical candidate |
| 457 | `unknown-lexeme` | `созида́ти` | 68 | 68 | review the token against target-recension evidence and create or reject a lexical candidate |
| 458 | `unknown-lexeme` | `слы́шитъ` | 68 | 67 | review the token against target-recension evidence and create or reject a lexical candidate |
| 459 | `unknown-lexeme` | `вселѧ́тсѧ` | 68 | 66 | review the token against target-recension evidence and create or reject a lexical candidate |
| 460 | `unknown-lexeme` | `повелѣ́нїю` | 68 | 66 | review the token against target-recension evidence and create or reject a lexical candidate |
| 461 | `unknown-lexeme` | `сконча̀` | 68 | 66 | review the token against target-recension evidence and create or reject a lexical candidate |
| 462 | `unknown-lexeme` | `трꙋба́ми` | 68 | 66 | review the token against target-recension evidence and create or reject a lexical candidate |
| 463 | `unknown-lexeme` | `согрѣши́хъ` | 68 | 65 | review the token against target-recension evidence and create or reject a lexical candidate |
| 464 | `unknown-lexeme` | `а҆вени́ръ` | 68 | 64 | review the token against target-recension evidence and create or reject a lexical candidate |
| 465 | `unknown-lexeme` | `со́лнцемъ` | 68 | 64 | review the token against target-recension evidence and create or reject a lexical candidate |
| 466 | `unknown-lexeme` | `снѣ́стъ` | 68 | 60 | review the token against target-recension evidence and create or reject a lexical candidate |
| 467 | `unknown-lexeme` | `безче́стїе` | 67 | 67 | review the token against target-recension evidence and create or reject a lexical candidate |
| 468 | `unknown-lexeme` | `ле́стїю` | 67 | 67 | review the token against target-recension evidence and create or reject a lexical candidate |
| 469 | `unknown-lexeme` | `бѣ́хꙋ` | 67 | 65 | review the token against target-recension evidence and create or reject a lexical candidate |
| 470 | `unknown-lexeme` | `возда́мъ` | 67 | 65 | review the token against target-recension evidence and create or reject a lexical candidate |
| 471 | `unknown-lexeme` | `грѧды́й` | 67 | 65 | review the token against target-recension evidence and create or reject a lexical candidate |
| 472 | `unknown-lexeme` | `стєзѝ` | 67 | 65 | review the token against target-recension evidence and create or reject a lexical candidate |
| 473 | `unknown-lexeme` | `ѻ҆де́ждꙋ` | 67 | 65 | review the token against target-recension evidence and create or reject a lexical candidate |
| 474 | `unknown-lexeme` | `ѿи́метъ` | 67 | 63 | review the token against target-recension evidence and create or reject a lexical candidate |
| 475 | `unknown-lexeme` | `сожжѐ` | 67 | 61 | review the token against target-recension evidence and create or reject a lexical candidate |
| 476 | `unknown-lexeme` | `а҆́гнцєвъ` | 66 | 66 | review the token against target-recension evidence and create or reject a lexical candidate |
| 477 | `unknown-lexeme` | `ᲂу҆би́ти` | 66 | 66 | review the token against target-recension evidence and create or reject a lexical candidate |
| 478 | `unknown-lexeme` | `посла́нїе` | 66 | 66 | review the token against target-recension evidence and create or reject a lexical candidate |
| 479 | `unknown-lexeme` | `совершѝ` | 66 | 66 | review the token against target-recension evidence and create or reject a lexical candidate |
| 480 | `unknown-lexeme` | `царе́вꙋ` | 66 | 66 | review the token against target-recension evidence and create or reject a lexical candidate |
| 481 | `unknown-lexeme` | `ѡ҆брати́шасѧ` | 66 | 66 | review the token against target-recension evidence and create or reject a lexical candidate |
| 482 | `unknown-lexeme` | `ѻ҆́троки` | 66 | 66 | review the token against target-recension evidence and create or reject a lexical candidate |
| 483 | `missing-accent-or-orthographic-metadata` | `свидѣ̑нїѧ` | 66 | 66 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 484 | `ambiguity-or-spelling-variant` | `ѻ҆смы́й` | 66 | 66 | review whether this is a permitted Synodal spelling variant, abbreviation, or distinct lexeme |
| 485 | `unknown-lexeme` | `ᲂу҆́зрѧтъ` | 66 | 65 | review the token against target-recension evidence and create or reject a lexical candidate |
| 486 | `missing-accent-or-orthographic-metadata` | `возвѣсти́ти` | 66 | 65 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 487 | `unknown-lexeme` | `вѣкѡ́въ` | 66 | 64 | review the token against target-recension evidence and create or reject a lexical candidate |
| 488 | `unknown-lexeme` | `гедеѡ́нъ` | 66 | 64 | review the token against target-recension evidence and create or reject a lexical candidate |
| 489 | `unknown-lexeme` | `непоро́ченъ` | 66 | 64 | review the token against target-recension evidence and create or reject a lexical candidate |
| 490 | `unknown-lexeme` | `Погибо́ша` | 66 | 64 | review the token against target-recension evidence and create or reject a lexical candidate |
| 491 | `unknown-lexeme` | `сто́гнахъ` | 66 | 64 | review the token against target-recension evidence and create or reject a lexical candidate |
| 492 | `unknown-lexeme` | `трапе́зꙋ` | 66 | 64 | review the token against target-recension evidence and create or reject a lexical candidate |
| 493 | `unknown-lexeme` | `бл҃же́ни` | 66 | 63 | review the token against target-recension evidence and create or reject a lexical candidate |
| 494 | `unknown-lexeme` | `нача́тки` | 66 | 62 | review the token against target-recension evidence and create or reject a lexical candidate |
| 495 | `unknown-lexeme` | `ѹ҆чн҃цы̀` | 66 | 58 | review the token against target-recension evidence and create or reject a lexical candidate |
| 496 | `unknown-lexeme` | `а҆ма́нъ` | 66 | 56 | review the token against target-recension evidence and create or reject a lexical candidate |
| 497 | `unknown-lexeme` | `бли́жнѧгѡ` | 66 | 54 | review the token against target-recension evidence and create or reject a lexical candidate |
| 498 | `unknown-lexeme` | `пристꙋпи́ша` | 65 | 65 | review the token against target-recension evidence and create or reject a lexical candidate |
| 499 | `unknown-lexeme` | `і҆а́кѡвлѧ` | 65 | 65 | review the token against target-recension evidence and create or reject a lexical candidate |
| 500 | `unknown-lexeme` | `ѡ҆быдо́ша` | 65 | 65 | review the token against target-recension evidence and create or reject a lexical candidate |
