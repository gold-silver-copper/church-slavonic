; Genesis 1:1-8, hand-lifted 2026-09-01 (SYNTAX-PROMPT.md part 4).
; Every analyzed leaf renders through the crate and the whole verse is
; byte-checked against the pinned print by cargo xtask check-treebank.
; Verbatim leaves carry their reason in the harvest (NOTES.md):
;   бг҃ъ/дх҃ъ/бж҃їй — titlo abbreviations; бѣ̀/бꙋ́детъ — быти forms the
;   crate does not conjugate (imperfect бѣ̀, future бꙋ́детъ);
;   Землѧ́ — grave→acute before the enclitic же; ѹ҆́тро — single-char
;   uk ѹ where the crate spells the оу digraph; тве́рдїю — print ї
;   against crate і; ꙗ҆́же — pronouns are not yet indexed;
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
    (cl (w "ꙗ҆́же" :lemma и҆́же) (w "бѣ̀" :lemma бы́ти)
      (pp (f под̾) (w "тве́рдїю" :lemma тве́рдь :case ins))) (p ",")
    (f и҆)
    (pp (f междꙋ̀) (n вода̀ :case ins :num sg)) (p ",")
    (cl (w "ꙗ҆́же" :lemma и҆́же) (w "бѣ̀" :lemma бы́ти)
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
