; Genesis 1:1-8, hand-lifted 2026-09-01 (SYNTAX-PROMPT.md part 4).
; Every analyzed leaf renders through the crate and the whole verse is
; byte-checked against the pinned print by cargo xtask check-treebank.
; Verbatim leaves carry their reason in the harvest (NOTES.md):
;   бг҃ъ/дх҃ъ/бж҃їй — titlo abbreviations; бѣ̀/бꙋ́детъ — быти forms the
;   crate does not conjugate (imperfect бѣ̀, future бꙋ́детъ);
;   Землѧ́ — grave→acute before the enclitic же; ѹ҆́тро — single-char
;   uk ѹ where the crate spells the оу digraph; тве́рдїю — print ї
;   against crate і; ꙗ҆́же — lifted in v1.2 part 4 (the relative и҆́же);
;   неꙋстро́ена — lemma missing; разлꙋча́ющи — present participle,
;   not yet indexed; вторы́й — print form differs from the crate's.
(verse 1 1 (s (cl
  (pp (cap (f въ)) (n нача́ло :case loc :num sg))
  (v сотвори́ти :t aor :p 3 :num sg)
  (subj (np (w "бг҃ъ" :lemma бо́гъ :case nom)))
  (np (n не́бо :case acc :num sg) (f и҆) (n землѧ̀ :case acc :num sg))
  (p "."))))
(verse 1 2 (s
  (cl (subj (np (w "Землѧ́" :lemma землѧ̀ :case nom))) (f же)
    (w "бѣ̀" :lemma бы́ти)
    (adj неви́димь :case nom :num sg :g f) (f и҆) (w "неꙋстро́ена") (p ","))
  (f и҆)
  (cl (subj (np (n тма̀ :case nom :num sg)))
    (pp (f верхꙋ̀) (n бе́здна :case gen :num sg)) (p ","))
  (f и҆)
  (cl (subj (np (w "дх҃ъ" :lemma дꙋ́хъ :case nom) (w "бж҃їй" :lemma бо́жїй)))
    (v носи́тисѧ :t impf :p 3 :num sg)
    (pp (f верхꙋ̀) (n вода̀ :case gen :num sg)) (p "."))))
(verse 1 3 (s
  (cl (cap (f и҆)) (v рещѝ :t aor :p 3 :num sg)
    (subj (np (w "бг҃ъ" :lemma бо́гъ :case nom))) (p ":")
    (cl (f да) (w "бꙋ́детъ" :lemma бы́ти) (n свѣ́тъ :case nom :num sg)) (p "."))
  (cl (cap (f и҆)) (v бы́ти :t aor :p 3 :num sg)
    (subj (np (n свѣ́тъ :case nom :num sg))) (p "."))))
(verse 1 4 (s
  (cl (cap (f и҆)) (v ви́дѣти :t aor :p 3 :num sg)
    (subj (np (w "бг҃ъ" :lemma бо́гъ :case nom)))
    (np (n свѣ́тъ :case acc :num sg)) (p ",")
    (f ꙗ҆́кѡ) (n добро̀ :case nom :num sg) (p ","))
  (f и҆)
  (cl (v разлꙋчи́ти :t aor :p 3 :num sg)
    (subj (np (w "бг҃ъ" :lemma бо́гъ :case nom)))
    (pp (f междꙋ̀) (n свѣ́тъ :case ins :num sg))
    (f и҆)
    (pp (f междꙋ̀) (n тма̀ :case ins :num sg)) (p "."))))
(verse 1 5 (s
  (cl (cap (f и҆)) (v нарещѝ :t aor :p 3 :num sg)
    (subj (np (w "бг҃ъ" :lemma бо́гъ :case nom)))
    (np (n свѣ́тъ :case acc :num sg)) (np (n де́нь :case acc :num sg)) (p ",")
    (f а҆) (np (n тма̀ :case acc :num sg))
    (v нарещѝ :t aor :p 3 :num sg) (np (n но́щь :case acc :num sg)) (p "."))
  (cl (cap (f и҆)) (v бы́ти :t aor :p 3 :num sg)
    (subj (np (n ве́черъ :case nom :num sg))) (p ","))
  (cl (f и҆) (v бы́ти :t aor :p 3 :num sg)
    (subj (np (w "ѹ҆́тро" :lemma оу҆́тро :case nom))) (p ","))
  (np (n де́нь :case nom :num sg) (adj є҆ди́нъ :case nom :num sg :g m)) (p ".")))
(verse 1 6 (s
  (cl (cap (f и҆)) (v рещѝ :t aor :p 3 :num sg)
    (subj (np (w "бг҃ъ" :lemma бо́гъ :case nom))) (p ":")
    (cl (f да) (w "бꙋ́детъ" :lemma бы́ти) (n тве́рдь :case nom :num sg)
      (pp (f посредѣ̀) (n вода̀ :case gen :num sg))) (p ",")
    (f и҆)
    (cl (f да) (w "бꙋ́детъ" :lemma бы́ти) (w "разлꙋча́ющи" :lemma разлꙋча́ти)
      (pp (f посредѣ̀) (n вода̀ :case gen :num sg)
        (f и҆) (n вода̀ :case gen :num sg))) (p "."))
  (cl (cap (f и҆)) (v бы́ти :t aor :p 3 :num sg) (f та́кѡ) (p "."))))
(verse 1 7 (s
  (cl (cap (f и҆)) (v сотвори́ти :t aor :p 3 :num sg)
    (subj (np (w "бг҃ъ" :lemma бо́гъ :case nom)))
    (np (n тве́рдь :case acc :num sg)) (p ","))
  (f и҆)
  (cl (v разлꙋчи́ти :t aor :p 3 :num sg)
    (subj (np (w "бг҃ъ" :lemma бо́гъ :case nom)))
    (pp (f междꙋ̀) (n вода̀ :case ins :num sg)) (p ",")
    (cl (pn и҆́же :case nom :num sg :g f) (w "бѣ̀" :lemma бы́ти)
      (pp (f под̾) (w "тве́рдїю" :lemma тве́рдь :case ins))) (p ",")
    (f и҆)
    (pp (f междꙋ̀) (n вода̀ :case ins :num sg)) (p ",")
    (cl (pn и҆́же :case nom :num sg :g f) (w "бѣ̀" :lemma бы́ти)
      (pp (f над̾) (w "тве́рдїю" :lemma тве́рдь :case ins))) (p "."))))
(verse 1 8 (s
  (cl (cap (f и҆)) (v нарещѝ :t aor :p 3 :num sg)
    (subj (np (w "бг҃ъ" :lemma бо́гъ :case nom)))
    (np (n тве́рдь :case acc :num sg)) (np (n не́бо :case acc :num sg)) (p "."))
  (cl (cap (f и҆)) (v ви́дѣти :t aor :p 3 :num sg)
    (subj (np (w "бг҃ъ" :lemma бо́гъ :case nom))) (p ",")
    (f ꙗ҆́кѡ) (n добро̀ :case nom :num sg) (p "."))
  (cl (cap (f и҆)) (v бы́ти :t aor :p 3 :num sg)
    (subj (np (n ве́черъ :case nom :num sg))) (p ","))
  (cl (f и҆) (v бы́ти :t aor :p 3 :num sg)
    (subj (np (w "ѹ҆́тро" :lemma оу҆́тро :case nom))) (p ","))
  (np (n де́нь :case nom :num sg) (w "вторы́й")) (p ".")))
; Genesis 1:9-31, hand-lifted 2026-09-01 (SYNTAX2 part 4): the auto-lift
; (pronouns, participles, the titlo layer and the sole-_n enumeration fix
; all live) resolved by the annotator. Deliberately left ambiguous, with
; reasons: дни̑ («во дни̑» reads accusative PLURAL but the crate offers
; only dual cells for this spelling); свѣти̑ла and вели̑каѧ after «два̀»
; (the print's -аѧ plural agreement against a dual numeral is a real
; grammatical question, not a coin to flip); дѡбра̀ (the predicate is a
; short PLURAL adjective — the only crate readings are noun cells of
; добро̀, and a false analysis is worse than none).
; v1.2 part 4 (2026-09-01): the pronoun leaves lifted — the ве́сь/всѧ́къ
; family as (pn …) with the long series as the adjective всѧ́кій (the
; part-0 decision), the relative и҆́же with its plural varia (ꙗ҆̀же
; 1:21/1:22, є҆мꙋ́же 1:11/1:12), the third-person accusatives as the
; full «и҆̀хъ» (1:27, 1:28) and the dual «ѧ҆̀» (1:17, the two lights)
; against the clitic «ѧ҆̀» of 1:22 (:clit yes; masculine plural for the
; mixed creatures — the print's form is one for every gender), the
; reflexive «себѣ̀» as (refl :case loc) after въ, є҆ли̑ка as the pronoun
; є҆ли́къ. What remains verbatim is the recorded frontier: the ordinals,
; бы́ти's imperfect бѣ̀ and future бꙋ́детъ / бꙋ́дꙋтъ, the print's ї before
; a vowel (тве́рдїю, но́щїю, подо́бїю, бы́лїе — the Bible-as-source
; question), the single-character uk (ѹ҆́тро), the plural-marked nouns
; the crate spells otherwise (во́ды, га́ды, ѕвѣ̑ри, скоты̀, собра̑нїѧ), and
; titlo families not yet admitted (блгⷭ҇вѝ, гл҃ѧ's participle spelling).
(verse 1 9 (s (cap (f и҆)) (v рещѝ :t aor :p 3 :num sg) (n бг҃ъ :case nom :num sg) (p ":") (f да) (v собра́тисѧ :t pres :p 3 :num sg) (n вода̀ :case nom :num sg) (p ",") (pn и҆́же :case nom :num sg :g f) (f под̾) (n не́бо :case ins :num sg) (p ",") (f въ) (w "собра́нїе") (w "є҆ди́но") (p ",") (f и҆) (f да) (v ꙗ҆ви́тисѧ :t pres :p 3 :num sg) (n сꙋ́ша :case nom :num sg) (p ".") (cap (f и҆)) (v бы́ти :t aor :p 3 :num sg) (f та́кѡ) (p ".") (cap (f и҆)) (v собра́тисѧ :t aor :p 3 :num sg) (n вода̀ :case nom :num sg) (p ",") (pn и҆́же :case nom :num sg :g f) (f под̾) (n не́бо :case ins :num sg) (p ",") (f въ) (w "собра̑нїѧ") (pn сво́й :case acc :num pl :g n) (p ",") (f и҆) (v ꙗ҆ви́тисѧ :t aor :p 3 :num sg) (n сꙋ́ша :case nom :num sg) (p ".")))
(verse 1 10 (s (cap (f и҆)) (v нарещѝ :t aor :p 3 :num sg) (n бг҃ъ :case nom :num sg) (n сꙋ́ша :case acc :num sg) (n землѧ̀ :case acc :num sg) (p ",") (f и҆) (w "собра̑нїѧ") (n вода̀ :case gen :num pl) (v нарещѝ :t aor :p 3 :num sg) (n мо́ре :case acc :num pl) (p ".") (cap (f и҆)) (v ви́дѣти :t aor :p 3 :num sg) (n бг҃ъ :case nom :num sg) (p ",") (f ꙗ҆́кѡ) (n добро̀ :case nom :num sg) (p ".")))
(verse 1 11 (s (cap (f и҆)) (v рещѝ :t aor :p 3 :num sg) (n бг҃ъ :case nom :num sg) (p ":") (f да) (v прорасти́ти :t pres :p 3 :num sg) (n землѧ̀ :case nom :num sg) (w "бы́лїе") (adj травны́й :case acc :num sg :g n) (p ",") (part сѣ́ѧти :t pres :voice act :series long :case acc :num sg :g n) (n сѣ́мѧ :case acc :num sg) (f по) (n ро́дъ :case dat :num sg) (f и҆) (f по) (w "подо́бїю") (p ",") (f и҆) (n дре́во :case acc :num sg) (adj плодови́тый :case acc :num sg :g n) (part твори́ти :t pres :voice act :series long :case acc :num sg :g n) (n пло́дъ :case acc :num sg) (p ",") (pn и҆́же :case dat :num sg :g n) (n сѣ́мѧ :case nom :num sg) (pers :p 3 :num sg :case gen :g n) (f въ) (pers :p 3 :num sg :case loc :g n) (p ",") (f по) (n ро́дъ :case dat :num sg) (f на) (n землѧ̀ :case loc :num sg) (p ".") (cap (f и҆)) (v бы́ти :t aor :p 3 :num sg) (f та́кѡ) (p ".")))
(verse 1 12 (s (cap (f и҆)) (v и҆знестѝ :t aor :p 3 :num sg) (n землѧ̀ :case nom :num sg) (w "бы́лїе") (adj травны́й :case acc :num sg :g n) (p ",") (part сѣ́ѧти :t pres :voice act :series long :case acc :num sg :g n) (n сѣ́мѧ :case acc :num sg) (f по) (n ро́дъ :case dat :num sg) (f и҆) (f по) (w "подо́бїю") (p ",") (f и҆) (n дре́во :case acc :num sg) (adj плодови́тый :case acc :num sg :g n) (part твори́ти :t pres :voice act :series long :case acc :num sg :g n) (n пло́дъ :case acc :num sg) (p ",") (pn и҆́же :case dat :num sg :g n) (n сѣ́мѧ :case nom :num sg) (pers :p 3 :num sg :case gen :g n) (f въ) (pers :p 3 :num sg :case loc :g n) (p ",") (f по) (n ро́дъ :case dat :num sg) (f на) (n землѧ̀ :case loc :num sg) (p ".") (cap (f и҆)) (v ви́дѣти :t aor :p 3 :num sg) (n бг҃ъ :case nom :num sg) (p ",") (f ꙗ҆́кѡ) (n добро̀ :case nom :num sg) (p ".")))
(verse 1 13 (s (cap (f и҆)) (v бы́ти :t aor :p 3 :num sg) (n ве́черъ :case nom :num sg) (p ",") (f и҆) (v бы́ти :t aor :p 3 :num sg) (w "ѹ҆́тро") (p ",") (n де́нь :case nom :num sg) (w "тре́тїй") (p ".")))
(verse 1 14 (s (cap (f и҆)) (v рещѝ :t aor :p 3 :num sg) (n бг҃ъ :case nom :num sg) (p ":") (f да) (w "бꙋ́дꙋтъ") (n свѣти́ло :case nom :num pl) (f на) (n тве́рдь :case loc :num sg) (adj небе́сный :case loc :num sg :g f) (p ",") (v ѡ҆свѣща́ти :t pres :p 3 :num sg :form inf) (n землѧ̀ :case acc :num sg) (f и҆) (v разлꙋча́ти :t pres :p 3 :num sg :form inf) (f междꙋ̀) (n де́нь :case ins :num sg) (f и҆) (f междꙋ̀) (w "но́щїю") (p ":") (f и҆) (f да) (w "бꙋ́дꙋтъ") (f въ) (w "зна́мєнїѧ") (f и҆) (f во) (n вре́мѧ :case acc :num pl) (p ",") (f и҆) (f во) (w "дни̑" :amb 2) (f и҆) (f въ) (n лѣ́то :case acc :num pl) (p ",")))
(verse 1 15 (s (f и҆) (f да) (w "бꙋ́дꙋтъ") (f въ) (w "просвѣще́нїе") (f на) (n тве́рдь :case loc :num sg) (adj небе́сный :case loc :num sg :g f) (p ",") (f ꙗ҆́кѡ) (v свѣти́ти :t pres :p 3 :num sg :form inf) (f по) (n землѧ̀ :case loc :num sg) (p ".") (cap (f и҆)) (v бы́ти :t aor :p 3 :num sg) (f та́кѡ) (p ".")))
(verse 1 16 (s (cap (f и҆)) (v сотвори́ти :t aor :p 3 :num sg) (n бг҃ъ :case nom :num sg) (w "два̀") (w "свѣти̑ла" :amb 6) (w "вели̑каѧ" :amb 6) (p ":") (n свѣти́ло :case acc :num sg) (adj вели́кій :case acc :num sg :g n) (f въ) (n нача́ло :case acc :num pl) (n де́нь :case gen :num sg) (p ",") (f и҆) (n свѣти́ло :case acc :num sg) (w "ме́ншее") (f въ) (n нача́ло :case acc :num pl) (n но́щь :case gen :num sg) (p ",") (f и҆) (w "ѕвѣ́зды") (p ":")))
(verse 1 17 (s (f и҆) (v положи́ти :t aor :p 3 :num sg) (pers :p 3 :num du :case acc :g n) (n бг҃ъ :case nom :num sg) (f на) (n тве́рдь :case loc :num sg) (adj небе́сный :case loc :num sg :g f) (p ",") (f ꙗ҆́кѡ) (v свѣти́ти :t pres :p 3 :num sg :form inf) (f на) (n землѧ̀ :case acc :num sg) (p ",")))
(verse 1 18 (s (f и҆) (v владѣ́ти :t pres :p 3 :num sg :form inf) (n де́нь :case ins :num sg) (f и҆) (w "но́щїю") (p ",") (f и҆) (v разлꙋча́ти :t pres :p 3 :num sg :form inf) (f междꙋ̀) (n свѣ́тъ :case ins :num sg) (f и҆) (f междꙋ̀) (w "тмо́ю") (p ".") (cap (f и҆)) (v ви́дѣти :t aor :p 3 :num sg) (n бг҃ъ :case nom :num sg) (p ",") (f ꙗ҆́кѡ) (n добро̀ :case nom :num sg) (p ".")))
(verse 1 19 (s (cap (f и҆)) (v бы́ти :t aor :p 3 :num sg) (n ве́черъ :case nom :num sg) (p ",") (f и҆) (v бы́ти :t aor :p 3 :num sg) (w "ѹ҆́тро") (p ",") (n де́нь :case nom :num sg) (w "четве́ртый") (p ".")))
(verse 1 20 (s (cap (f и҆)) (v рещѝ :t aor :p 3 :num sg) (n бг҃ъ :case nom :num sg) (p ":") (f да) (v и҆звестѝ :t pres :p 3 :num pl) (w "во́ды") (w "га́ды") (n дꙋша̀ :case gen :num pl) (adj жи́въ :case gen :num pl :g f) (p ",") (f и҆) (n пти́ца :case nom :num pl) (part лета́ти :t pres :voice act :series long :case nom :num pl :g f) (f по) (n землѧ̀ :case loc :num sg) (p ",") (f по) (n тве́рдь :case loc :num sg) (adj небе́сный :case loc :num sg :g f) (p ".") (cap (f и҆)) (v бы́ти :t aor :p 3 :num sg) (f та́кѡ) (p ".")))
(verse 1 21 (s (cap (f и҆)) (v сотвори́ти :t aor :p 3 :num sg) (n бг҃ъ :case nom :num sg) (n ки́тъ :case acc :num pl) (w "вели̑кїѧ") (p ",") (f и҆) (pn всѧ́къ :case acc :num sg :g f) (n дꙋша̀ :case acc :num sg) (adj живо́тенъ :case gen :num pl :g m) (n га́дъ :case acc :num pl) (p ",") (pn и҆́же :case acc :num pl :g f) (v и҆звестѝ :t aor :p 3 :num pl) (w "во́ды") (f по) (n ро́дъ :case dat :num pl) (pers :p 3 :num pl :case gen :g m) (p ",") (f и҆) (pn всѧ́къ :case acc :num sg :g f) (n пти́ца :case acc :num sg) (adj перна́тъ :case acc :num sg :g f) (f по) (n ро́дъ :case dat :num sg) (p ".") (cap (f и҆)) (v ви́дѣти :t aor :p 3 :num sg) (n бг҃ъ :case nom :num sg) (p ",") (f ꙗ҆́кѡ) (w "дѡбра̀" :amb 6) (p ".")))
(verse 1 22 (s (cap (f и҆)) (w "блгⷭ҇вѝ") (pers :p 3 :num pl :case acc :g m :clit yes) (n бг҃ъ :case nom :num sg) (p ",") (w "гл҃ѧ") (p ":") (w "расти́тесѧ") (f и҆) (w "мно́житесѧ") (p ",") (f и҆) (v напо́лнити :t pres :p 2 :num pl :form imp) (w "во́ды") (p ",") (pn и҆́же :case nom :num pl :g f) (f въ) (n мо́ре :case loc :num pl) (p ",") (f и҆) (n пти́ца :case nom :num pl) (f да) (w "ѹ҆мно́жатсѧ") (f на) (n землѧ̀ :case loc :num sg) (p ".")))
(verse 1 23 (s (cap (f и҆)) (v бы́ти :t aor :p 3 :num sg) (n ве́черъ :case nom :num sg) (p ",") (f и҆) (v бы́ти :t aor :p 3 :num sg) (w "ѹ҆́тро") (p ",") (n де́нь :case nom :num sg) (w "пѧ́тый") (p ".")))
(verse 1 24 (s (cap (f и҆)) (v рещѝ :t aor :p 3 :num sg) (n бг҃ъ :case nom :num sg) (p ":") (f да) (v и҆звестѝ :t pres :p 3 :num sg) (n землѧ̀ :case nom :num sg) (n дꙋша̀ :case acc :num sg) (adj жи́въ :case acc :num sg :g f) (f по) (n ро́дъ :case dat :num sg) (p ",") (adj четвероно́гій :case acc :num pl :g n) (f и҆) (w "га́ды") (p ",") (f и҆) (w "ѕвѣ̑ри") (n землѧ̀ :case gen :num sg) (f по) (n ро́дъ :case dat :num sg) (p ".") (cap (f и҆)) (v бы́ти :t aor :p 3 :num sg) (f та́кѡ) (p ".")))
(verse 1 25 (s (cap (f и҆)) (v сотвори́ти :t aor :p 3 :num sg) (n бг҃ъ :case nom :num sg) (w "ѕвѣ̑ри") (n землѧ̀ :case gen :num sg) (f по) (n ро́дъ :case dat :num sg) (p ",") (f и҆) (w "скоты̀") (f по) (n ро́дъ :case dat :num sg) (pers :p 3 :num pl :case gen :g m) (p ",") (f и҆) (pn ве́сь :case acc :num pl :g m) (w "га́ды") (n землѧ̀ :case gen :num sg) (f по) (n ро́дъ :case dat :num sg) (pers :p 3 :num pl :case gen :g m) (p ".") (cap (f и҆)) (v ви́дѣти :t aor :p 3 :num sg) (n бг҃ъ :case nom :num sg) (p ",") (f ꙗ҆́кѡ) (w "дѡбра̀" :amb 6) (p ".")))
(verse 1 26 (s (cap (f и҆)) (v рещѝ :t aor :p 3 :num sg) (n бг҃ъ :case nom :num sg) (p ":") (v сотвори́ти :t pres :p 1 :num pl) (n человѣ́къ :case acc :num sg) (f по) (n ѻ҆́бразъ :case dat :num sg) (pn на́шъ :case dat :num sg :g m) (f и҆) (f по) (w "подо́бїю") (p ",") (f и҆) (f да) (v ѡ҆блада́ти :t pres :p 3 :num sg) (n ры́ба :case ins :num pl) (adj морскі́й :case ins :num pl :g f) (p ",") (f и҆) (n пти́ца :case ins :num pl) (adj небе́сный :case ins :num pl :g f) (p ",") (p "(") (f и҆) (w "ѕвѣрьмѝ") (p ")") (f и҆) (n ско́тъ :case ins :num pl) (p ",") (f и҆) (pn ве́сь :case ins :num sg :g f) (n землѧ̀ :case ins :num sg) (p ",") (f и҆) (pn ве́сь :case ins :num pl :g m) (n га́дъ :case ins :num pl) (w "пресмыка́ющимисѧ") (f по) (n землѧ̀ :case loc :num sg) (p ".")))
(verse 1 27 (s (cap (f и҆)) (v сотвори́ти :t aor :p 3 :num sg) (n бг҃ъ :case nom :num sg) (n человѣ́къ :case acc :num sg) (p ",") (f по) (n ѻ҆́бразъ :case dat :num sg) (w "бж҃їю") (v сотвори́ти :t aor :p 3 :num sg) (pers :p 3 :num sg :case acc :g m) (p ":") (n мꙋ́жъ :case acc :num sg) (f и҆) (n жена̀ :case acc :num sg) (v сотвори́ти :t aor :p 3 :num sg) (pers :p 3 :num pl :case acc :g m) (p ".")))
(verse 1 28 (s (cap (f и҆)) (w "блгⷭ҇вѝ") (pers :p 3 :num pl :case acc :g m) (n бг҃ъ :case nom :num sg) (p ",") (w "гл҃ѧ") (p ":") (w "расти́тесѧ") (f и҆) (w "мно́житесѧ") (p ",") (f и҆) (v напо́лнити :t pres :p 2 :num pl :form imp) (n землѧ̀ :case acc :num sg) (p ",") (f и҆) (v госпо́дствовати :t pres :p 2 :num pl :form imp) (pers :p 3 :num sg :case ins :g f) (p ",") (f и҆) (v ѡ҆блада́ти :t pres :p 2 :num pl :form imp) (n ры́ба :case ins :num pl) (adj морскі́й :case ins :num pl :g f) (p ",") (p "(") (f и҆) (w "ѕвѣрьмѝ") (p ")") (f и҆) (n пти́ца :case ins :num pl) (adj небе́сный :case ins :num pl :g f) (p ",") (f и҆) (pn ве́сь :case ins :num pl :g m) (n ско́тъ :case ins :num pl) (p ",") (f и҆) (pn ве́сь :case ins :num sg :g f) (n землѧ̀ :case ins :num sg) (p ",") (f и҆) (pn ве́сь :case ins :num pl :g m) (w "га́дами") (w "пресмыка́ющимисѧ") (f по) (n землѧ̀ :case loc :num sg) (p ".")))
(verse 1 29 (s (cap (f и҆)) (v рещѝ :t aor :p 3 :num sg) (n бг҃ъ :case nom :num sg) (p ":") (f сѐ) (p ",") (v да́ти :t aor :p 1 :num sg) (pers :p 2 :num pl :case dat) (adj всѧ́кій :case acc :num sg :g f) (n трава̀ :case acc :num sg) (w "сѣ́меннꙋю") (part сѣ́ѧти :t pres :voice act :series long :case acc :num sg :g f) (n сѣ́мѧ :case acc :num sg) (p ",") (n є҆́жъ :case voc :num sg) (v бы́ти :t pres :p 3 :num sg) (f верхꙋ̀) (n землѧ̀ :case gen :num sg) (pn ве́сь :case gen :num sg :g f) (p ":") (f и҆) (adj всѧ́кій :case acc :num sg :g n) (n дре́во :case acc :num sg) (p ",") (n є҆́жъ :case voc :num sg) (v и҆мѣ́ти :t pres :p 3 :num sg) (f въ) (refl :case loc) (n пло́дъ :case acc :num sg) (n сѣ́мѧ :case gen :num sg) (w "сѣ́меннагѡ") (p ",") (pers :p 2 :num pl :case dat) (w "бꙋ́детъ") (f въ) (n снѣ́дь :case acc :num sg) (p ":")))
(verse 1 30 (s (f и҆) (pn ве́сь :case dat :num pl :g m) (w "ѕвѣрє́мъ") (adj зе́менъ :case dat :num pl :g m) (p ",") (f и҆) (pn ве́сь :case dat :num pl :g f) (w "пти́цамъ") (adj небе́сный :case dat :num pl :g f) (p ",") (f и҆) (pn всѧ́къ :case dat :num sg :g m) (n га́дъ :case dat :num sg) (w "пресмыка́ющемꙋсѧ") (f по) (n землѧ̀ :case loc :num sg) (p ",") (pn и҆́же :case nom :num sg :g m) (v и҆мѣ́ти :t pres :p 3 :num sg) (f въ) (refl :case loc) (n дꙋша̀ :case acc :num sg) (n живо́тъ :case gen :num sg) (p ",") (f и҆) (pn всѧ́къ :case acc :num sg :g f) (n трава̀ :case acc :num sg) (adj зеле́нъ :case acc :num sg :g f) (f въ) (n снѣ́дь :case acc :num sg) (p ".") (cap (f и҆)) (v бы́ти :t aor :p 3 :num sg) (f та́кѡ) (p ".")))
(verse 1 31 (s (cap (f и҆)) (v ви́дѣти :t aor :p 3 :num sg) (n бг҃ъ :case nom :num sg) (pn ве́сь :case acc :num pl :g n) (p ",") (pn є҆ли́къ :case acc :num pl :g n) (v сотвори́ти :t aor :p 3 :num sg) (p ":") (f и҆) (f сѐ) (w "дѡбра̀" :amb 6) (w "ѕѣлѡ̀") (p ".") (cap (f и҆)) (v бы́ти :t aor :p 3 :num sg) (n ве́черъ :case nom :num sg) (p ",") (f и҆) (v бы́ти :t aor :p 3 :num sg) (w "ѹ҆́тро") (p ",") (n де́нь :case nom :num sg) (w "шесты́й") (p ".")))
