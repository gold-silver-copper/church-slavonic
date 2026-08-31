# Program notes (research diary)

## Phase 0 baseline — 2026-08-30, at 6cc200a (v0.9.0)

- `check-registry`: OK.
- `accuracy`: every table source 100.00%, variant gap 0 (nouns/adj/verbs/pronouns, OCS + Synodal, all splits).
- Held-out corpus recall (never table cells, per references/TERMS.md):
  - UD PROIEL dev+test: nouns 8116/8818 (92.04%, gap 702); adj 2134/2546 (83.82%, gap 412); verbs 7294/8529 (85.52%, gap 1235); pronouns 4918/4960 (99.15%, gap 42).
  - Syntacticus 2023-04-28: nouns 44305/48825 (90.74%); adj 11698/13901 (84.15%); verbs 38709/45179 (85.68%); pronouns 26764/27025 (99.03%).
- Relevant skip counters (dev+test): pronoun outside the personal matrix 1,293; pronoun reflexive 1,069; verb l-participle 133; verb future 97. Syntacticus: reflexive 5,775; participle without a tense 1,004; future 570.

Phase 1 targets: non-personal pronouns, l-participle (the two schema gaps README names).

## Phase 1 — schema scope closed (2026-08-30)

- **L-participle** (resultative): verb arity 549 → 558, nominative-only gender/number block at 549 (`schema::l_participle_cell`). Rule: infinitive stem + `л` + gender/number ending; dentals drop (`вести` : `велъ`); a `-сти` infinitive reads as a dental stem (the `нести` type is tabled); reflexive `-сѧ` outside; Synodal accent via the lemma re-stress. Sources mapped: UD train PartRes (45 slots), Polyakov `partcp,perf` (4,082 slots — was skipped), kaikki `l-participle` form-of pages, OCS wiktionary (11). ru.wiktionary perfect rows stay skipped: periphrastic two-word phrases. Verb rows 7,640 → 8,370.
- **Non-personal pronouns**: new POS (`Pos::NPron`), arity 54 (`(gender*3+number)*6+case`), lemma-keyed `npron_phf.rs`, rule = pronominal declension in `core::npron` (hard `тъ`-type, soft `сь`/`мои`/`нашь`, mixed `вьсь`, anaphoric `иже` = `и`-series + `же`, singular-only `къто`/`чьто`, `ни-`/`нѣ-` compounds by strip-decline-rewrap). Sources: kaikki lemma tables + form-of, UD train, both treebank recall mappers (UD `PronType!=Prs`, PROIEL `Pd/Pi/Pr/Px/Ps/Pt/Py`; `Rcp`/`Pc` reciprocals stay skipped). 54 rows stored. Synodal npron has no rule/sources yet (Alypy §§46-55 and Polyakov npron entries stay skipped — 98 + a section; candidate for a later wave).
- **Finalize fix**: same-signature candidates now UNION their raw cells on merge (was first-wins), so the bare-shadow pass can re-materialise every attested rule-equal form a stored bare cell shadows (`еиже` under `иже`'s stored `ѥїиже`). Variant gap back to 0 everywhere.
- Gates: every table source 100.00%/0; check-registry OK; 14 test suites + doctests green.
- Held-out recall after Phase 1 (baseline → now): verbs dev+test 85.52% → 85.59% (denominator +133 l-participle slots); NEW npron dev+test 93.21% (1208/1296), Syntacticus 94.32% (17765/18835); nouns/adj/personal pronouns unchanged.
