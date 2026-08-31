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

## Phase 2 — new-source pass and the data ceiling (2026-08-30)

Candidates examined for a sixth pinned table source; none qualified:
- **GORAZD (gorazd.org)** — digitized SJS/Cejtlin OCS lexicon: dictionary entries with citations, not paradigm tables; no bulk-download artifact or redistribution license published. Rejected (not machine-extractable paradigms; license unclear).
- **UniMorph `chu`** — 4,302 triples over **9 lemmas**, derived from English Wiktionary (already a pinned source via Kaikki). Rejected (redundant and tiny).
- **DIACU (2025)** — diachronic Church Slavonic text collection: plain text, no morphological annotation. Rejected (cannot attest cells).
- **Ponomar / Slavonic Computing Initiative** — GPLv3 liturgical suite; its Slavonic data are wordlists/hyphenation, not tagged paradigms. Rejected (license + shape).
- **Sobolevsky and other pre-1929 grammars** — public domain but print-only; no pinnable machine-readable artifact exists. Rejected (would require an OCR/keying project, out of scope).
- **Russian National Corpus Church Slavonic subcorpus** — not redistributable; no bulk artifact. Rejected.

With no qualifying source, the post-ingestion census re-run does not apply. **The data ceiling is reached**: the recorded verdict stands — the remaining stored mass (per-row ending choice, spelling variance, mobile accent) is irreducibly lexical at current source coverage, and the accent (v0.8.0) and letters-differ (v0.9.0) censuses are mined out. Mining stops here.

Deferred beyond 1.0 (would need new data or new scope, both recorded): Synodal non-personal pronouns (Alypy §§46–55 + 98 Polyakov forms), ru.wiktionary periphrastic perfect rows, Syntacticus l-participles (PROIEL XML marks them differently).


## Research diary moved from the changelog (findings by release)

### 0.9.0
- The scoping ceiling (1,020 rows "entirely explained" by animacy)
  dissolved on contact: 1,513 of the 1,552 nominative-shaped-accusative
  rows store exactly ONE such cell, where any mechanism — derivation or a
  fact cell — is a storage wash. The planned `inan` fact cell (arity 23)
  was therefore NOT added, per this wave's own condition: an arity bump
  nothing needs is dead weight.
- Selection bias claimed two designs this wave, both caught by refresh
  deltas: the possessive adjectives' genitive-shaped accusative flip (640
  stored gen-shaped cells are the rule's MISSES; 463 nominative-shaped
  possessive accusatives were silently covered and broke — blanket flip
  +1,295 rows, possessive-only flip still +38 cells) and both mandated
  rule flips (locative plural `-ахъ`: +30 noun rows; long locative
  singular `-омъ`: +28 adjective rows — the stored `-скомъ`/`-ской`
  forms are outnumbered by the covered `-ѣмъ` forms they would break).
  Stored-form tallies see only what the rule misses; every ending-choice
  hypothesis must be judged by a full refresh, never by counting misses.
- The possessives' nominal-cell class (`а҆́велемъ` InstSg) measured at 4
  stored cells — below any implementation threshold.
- The re-run censuses are essentially unchanged (letters-differ: noun
  2,991 / adj 3,167 / verb 1,857 ending-class rows; the accent censuses
  as in 0.8.0). The remaining mass is per-row ending choice and variant
  noise — irreducibly lexical at current source coverage. No recorded
  next lever clears the bar this wave's failures set; the honest next
  step is new source data, not new mechanism.

### 0.8.0
- Mobile-stress scheme tokens — the planned second stage — were measured
  and dropped: clustering the 1,951 mobile rows' attested stress shapes
  found the commonest scheme recurring on just 15 rows, and most rows
  (adj 432/574, verb 618/850) not expressible as a stem/ending partition
  at all, so no closed scheme vocabulary earns a const table. Mobile
  paradigms stay stored.
- The adoption coverage gate stays at >=2 covered cells: lowering it to 1
  adopted 52 more tokens but cost 0.02 noun bare accuracy for ~550 bytes —
  a measured wash that regresses.
- Two experiments were tried and reverted: computing the distinguishing
  markers from within-paradigm ending collisions (breaks `сынѡ́въ` —
  GenPl `-овъ` widens with no singular collision), and widening anywhere
  unconditionally (breaks `безпꙋ̑тіѧ` — a pre-stress `о`/`е` under a
  non-final stress does not widen).
- The remaining census (rows without a token, by dominant blocker):
  letters-differ 8,668 rows (a letter-paradigm problem, not an accent
  one — the next lever), too-sparse 7,681, mobile 1,951, wide 434,
  kamora 99. Token/Polyakov-class agreement is 68/99 across 26 classes
  (85/128 across 29 in 0.7.0 — fewer, more convention-faithful tokens).

## Phases 3–4 — process retired, 1.0.0 shipped (2026-08-30)

- Changelog rewritten consumer-facing; the "Findings, stated plainly" diary moved here (below). README regenerated from actual `accuracy` and `speedmark` runs (26,282 dev+test slots, 153,765 Syntacticus slots, 200,906 Polyakov analyses mapped; table hits 5–10M calls/s, rule fallback 50–150k/s; bundled tables ~5 MB).
- Refresh determinism spot-checked: two consecutive refreshes byte-identical.
- Versions: both crates 0.9.0 → 1.0.0 lockstep; docs build clean; `publish --dry-run` clean for core (facade dry-run blocks only on core not yet being on crates.io, as expected).
