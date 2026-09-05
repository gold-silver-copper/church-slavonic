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

## v1.1 consumer-defect program — 2026-08-31

Part 0 (ledger): the vertograd consumer's [avoid] list diagnosed against
the committed tables. Three surprises: «вожжѝ» and «пожа́тъ» are ATTESTED
cells (the audit was wrong, not the crate — the -ѧти aorist in -ѧ́тъ and
the assimilated imperative are real print, with «пожа̀»/«возжзѝ» as _n
variants); «стрищѝ» and «дои́ти» have rows whose attested present blocks
prove their stems, yet unattested cells fall to the blind rule; and
«кѷпарі́съ» was a lookup miss — the key spells «кѵпарі́съ» and the fold
did not treat the kendema. Ledger: crates/church-slavonic/tests/
consumer_defects.rs, born ignored, un-ignored per part.

Part 1 (izhitsa fold): `ѷ ~ ѵ` folded in the Synodal lookup key only —
zero table keys spell `ѷ`, 439 spell `ѵ`, and `comparison_key` already
folded the pair; lookup now agrees. Invariant 4 added to the lib.rs
contract. Gates: accuracy 100/0 everywhere, check-registry OK.

Part 2 (the fact mechanism, three linked repairs + one blast-radius fix):
- `infer_verb_override` donors now strip END-STRESSED endings
  («стриже́ши», «дои́ши») — end-stressed verbs could never hypothesize.
- The override inference and the accusative-shape subtraction both run
  AGAIN post-merge: Polyakov's single-cell observations meant a row's
  cells never met inside one observation, so neither pass could see them
  together (the стрищѝ/дои́ти/а҆ве́ль class of misses).
- `override_stems` repairs what the infinitive hid: a Second-class fact
  on an -ити lemma re-derives the stems as a true i-verb (дои́ти is not
  до + и҆тѝ); a present-stem fact on a -щи lemma restores the
  neutralized velar to the aorist/l-stems (стриг-ти vs пек-ти). New
  `l_participle_from_stems` + the runtime's l_participle entering the
  engine on class/stem facts, not only accent. Unaccented derived stems
  thread the lemma's accent like the plain rule path.
- The shape fact reads EVERY attested accusative (sources 3/10/17, src ==
  cell skipped): a nominative-shaped plural teaches the singular
  (ѻ҆гꙋре́цъ). The extractor's re-store pass reads its sources LIVE so
  restoring one accusative immediately derives the rest — the stale
  snapshot had re-stored mutually-derivable pairs and tripped
  rule_table_sync (118 → 0).
- The derivation corrected the AUDITOR twice more: the imperative of
  дои́ти is «доѝ» (the «напоѝ» print type), not the Russian «до́й»; and
  «возжещѝ»'s three imperative spellings are all attested — the guard
  asserts the set, since sense numbers may renumber (documented).
- Gates: accuracy 100.00%/0 on every pinned source; check-registry OK;
  held-out delta: OCS dev+test verbs 7414 → 7413 (−1 slot, recorded);
  all other held-out rows unchanged.

Part 3 (the default, measured): the held-out corpora are all OCS — whose
rule already answers the nominative shape — so flipping the SYNODAL
masculine default cannot move held-out recall at all; the arbitrating
statistic is Polyakov itself: masculine nouns are 72.9% animate by lemma
(5227/7174, confirming the design note) and 53.3% by token. The animate
default STANDS. Consequence recorded in the ledger: «прꙋ́дъ» and
«ко́локолъ» have no attested accusative in any pinned text and keep the
default; the consumer's prompts already avoid them.

Part 4 (the witness source): `data/witnesses.tsv` — curated single cells
from the running print, each citing a verbatim line of the
vertograd-fetched texts; ingested as `Source::Witness` through the same
gather/finalize/audit as every source (its own 100.00%/0 accuracy row),
verified by the new `cargo xtask check-witnesses` (offline-soft).
Admitted: the inanimate accusative of «ѡ҆́блакъ» («вше́дше во ѡ҆́блакъ»,
Lk 9:34) and «ꙗ҆́блонь»'s nominative (Joel 1:12) + feminine instrumental
«ꙗ҆́блонею» (Song 8:5). NO accusative of «ꙗ҆́блонь» exists in the pinned
print — the expectation was NOT invented; the cell stays rule-served and
the ledger documents it (Polyakov's own citation form is «я́блоня»,
which the intake currently drops for its civil «я» — noted as a future
ingestion nit). Ledger: 7/7 green, zero ignored.

Part 5 (close): version 1.1.0; consumer-facing CHANGELOG per class;
README accuracy tables regenerated from the real run (the witnessed-print
row now among the 100.00%/0 sources). The consumer proof: vertograd
consumes 1.1.0 via [patch.crates-io], its 30 tests and full headless
suite pass unchanged, its PHASE9 [avoid] list carries a dated arbitration
postscript («пожа́тъ»/«вожжѝ» were attested — the audit corrected;
«доѝ»/«стри́глъ» healed upstream; «смоло̀» stays unattested and
unchronicled). Remaining known nit for a future wave: Polyakov headwords
spelled with civil «я» («я́блоня») are dropped at intake — mapping them
to «ꙗ» would admit the ja-stem paradigm this program could only witness
piecemeal.

## 2026-09-01 — the syntax crate (SYNTAX-PROMPT.md, parts 0–3)

A new consumer lives in the workspace: `church-slavonic-syntax`, whose
one standing rule is the round-trip invariant — render(tree) equals the
pinned print byte-for-byte, no other definition of correct. Parts 0–2:
the sexpr reader (positions in errors, print/parse round-trip), the
ordered Node model with verbatim (w) leaves, the (cap …) wrapper, the
punctuation glue rule, the closed-class table (every entry counted
verbatim in the print), exact-output rendering through the public API
(Genesis 1:1 lifts and matches the print — «бг҃ъ» verbatim-with-reason,
titlo), and the linter (only reliable rules; subject checks opt-in via
the subj head).

Part 3, the baseline measurement: `ChurchSlavonic::lemmas` (new public
enumeration; base keys only, `_n` senses excluded) feeds the inverse
index — 235,400 distinct Synodal surfaces from 3,868 noun + 4,054
adjective + 4,307 verb lemmas (~500k generator calls, ~4s). The
auto-lift walked ALL 77 books / 34,470 verses (~10s end to end,
`cargo xtask build-treebank`), and the check re-renders every stored
tree against the print: **zero mismatches**. First-run coverage over
631,946 tokens:

  analyzed 101,222 (16.0%) · closed-class 164,350 (26.0%) ·
  ambiguous 95,392 (15.1%) · verbatim 269,965 (42.7%) · apparatus 1,017

Two print oddities the invariant caught at build (both now handled by
structure, not special cases): a FREE-STANDING period («а҆ссѷрі́йскъ .»,
4 Kings 17:3 — a lone punctuation token keeps its own space, so it stays
a verbatim leaf) and a «(,*…» opening cluster (Proverbs 15:33 — the
lift verifies its own reconstruction under the glue rule and falls back
to whole-token verbatim when the split cannot rebuild the token).

The burn-down reading: 42% of scripture lifts mechanically today; the
15.1% ambiguous band (acc=nom and friends) awaits a syntactic
disambiguation design that is deliberately NOT this wave; the 42.7%
verbatim band is the crate-vocabulary frontier (titlo abbreviations,
proper names, pronouns and participles the index does not yet generate,
and genuinely missing lemmas — the part-4 harvest samples it).

## 2026-09-01 — syntax part 4: Genesis 1 hand-lifted, defects harvested

Ps 90 was the prompt's candidate, but this JSON's Psalter is a known
22-verse fragment (one "chapter", mid-psalm text) — recorded here; the
hand-lift chapter became Genesis 1. Verses 1–8 are hand-lifted with
structure (cl/subj/np/pp) in the COMMITTED overlay
`data/treebank-hand/b00.sexp` (human work, unlike the derived treebank);
check-treebank prefers hand entries and reports them as their own
ceiling row: 120 tokens — 50 analyzed (41.7%) + 42 closed-class = 76.7%
lifted, ZERO ambiguous (the annotator resolves what the index may not
guess), 28 verbatim each with a reason. All eight verses render
byte-for-byte against the print, first try.

Pipeline improvements the lift exposed (syntax-crate side, done now):
«во», «верхꙋ̀», «посредѣ̀», «а҆́ки» enter the closed table (counted);
the infinitive joins the inverse index (the crate always offered it).
Rebuilt whole-Bible baseline: 17.1% analyzed + 27.1% closed = 44.2%
mechanical, 15.1% ambiguous, 40.5% verbatim.

THE HARVEST — crate-facing, classified, NOT fixed in this wave; each
wrong-form entry still needs the arbitration check against the pinned
tables before the next v1.x program treats it as a defect:

- **Wrong-form candidates** (crate output ≠ print for the same cell):
  «тве́рдїю» print vs «тве́рдію» crate (ins sg of тве́рдь — the ї-
  before-vowel spelling; same class as the «преѡбраже́ніе» observation
  from vertograd phase 10); «во́ды» print (nom/acc pl of вода̀, Gen 1)
  vs «вѡды̀» crate (the ѡ-plural disambiguation convention AND the
  accent differ); «вторы́й» print vs the crate's second-ordinal form.
- **Missing forms, systematic** (a class, not a lemma): бы́ти's
  imperfect «бѣ̀» (crate says «бѧ́ше») and future «бꙋ́детъ» — the
  Bible's two commonest verb tokens after «речѐ»; the grave→acute
  alternation before an enclitic («Землѧ́ же» against nominative
  «землѧ̀») — a context rule no cell can carry.
- **Missing lemmas**: «неꙋстро́енъ»; ordinals beyond є҆ди́нъ
  (вторы́й, тре́тїй, четве́ртый, пѧ́тый, шесты́й all verbatim in Gen 1).
- **Genuinely outside crate scope** (orthographic/abbreviation layer):
  titlo abbreviations (бг҃ъ, дх҃ъ, бж҃їй, гдⷭ҇ь…) — by far the largest
  verbatim class; the single-char uk «ѹ҆́тро» against the crate's
  «оу҆́тро» digraph (a lookup-folding question, the ѷ→ѵ precedent).
- **Index scope, not crate defects** (next syntax wave): pronouns and
  non-personal pronouns (ꙗ҆́же, є҆гѡ̀, и҆́хъ, себѣ̀ …) and present
  participles (разлꙋча́ющи, сѣ́ющее, пресмыка́ющемꙋсѧ) — the crate's
  pronoun/npron/participle APIs exist and are simply not yet inverted.

## 2026-09-01 — syntax part 5: the close

check-treebank wired into xtask (offline-soft only on the absent
source); full workspace suites green; `cargo xtask accuracy` closed
where it opened — 100.00% / gap 0 on every pinned table source (this
wave changed no tables; the one crate-side addition is the read-only
`ChurchSlavonic::lemmas` enumeration); check-registry and
check-witnesses pass; zero warnings. README carries the syntax crate
section: invariant first, the real coverage table, the escape-hatch
philosophy in two sentences.

The burn-down shape for future waves: (1) invert the pronoun, npron and
participle APIs into the index — pure syntax-crate work, likely the
largest single coverage gain; (2) run the part-4 harvest through the
crate's arbitration discipline and open a v1.x wave for what survives
(ї-before-vowel, the ѡ-plural convention, бы́ти's бѣ̀/бꙋ́детъ, the
ordinals); (3) a titlo-abbreviation lookup layer (гдⷭ҇ь and kin are the
single largest verbatim class); (4) syntactic disambiguation of the
15.1% ambiguous band — a separate design, deliberately not smuggled
into this one.

## 2026-09-01 — syntax wave 2, part 3: the harvest arbitrated

Verdicts, each checked against the committed tables (the arbitration
discipline of v1.1 — sometimes the audit is wrong):

- **«тве́рдїю» vs «тве́рдію»** — CRATE RIGHT AGAINST ITS SOURCES. The
  тве́рдь row attests only dat pl; the ins sg is rule-served, and the
  pinned Synodal sources NEVER spell ї before a vowel (0 cells of
  їю/їе/їѧ/їи against 2,388 of і-spellings). The Bible print's
  ї-before-vowel is a REAL convention the sources don't carry; adopting
  it means admitting the pinned Bible as a source (it is already
  sha256-pinned) and deciding the realise-layer question — a v1.x
  DESIGN item, not a bug fix.
- **«во́ды» vs «вѡды̀»** — same class, two questions. The вода̀ row
  attests exactly one cell («во́дꙋ», root-stressed acc sg); the plural
  is rule-served (ѡ-convention, end stress). (a) The ѡ-plural spelling:
  print says во́ды — Bible-as-source question, same as above. (b) The
  accent: the attested «во́дꙋ» root stress is NOT propagated to the
  plural nominative — a candidate for the v1.1 fact mechanism
  (accusative-shape/accent facts), v1.x intake.
- **бы́ти's «бѣ̀»** — MISSING FORM, unattested in any pinned source
  (the row is rich — бѣ́ста dual aorist, participles — but the
  imperfect 3sg is rule-served as «бѧ́ше»). The Bible uses бѣ̀
  constantly; Bible-as-source would attest it directly. v1.x intake.
- **бы́ти's «бꙋ́детъ»** — SCHEMA GAP: the 38-cell finite schema has no
  future block at all. Out of scope by the current design; recorded as
  a schema-level v1.x design question, not a data row.
- **«неꙋстро́енъ»** — NOT MISSING: `syn:неꙋстро́енъ_2` exists; the
  defect was in THIS PROGRAM's lemmas() enumeration, which skipped
  every `_n` key and made sole-`_n` lemmas invisible — the enumeration
  analogue of v1.1's ко́локолъ_2 lookup finding. FIXED HERE (the
  part-3 exception: a defect in the consumer's own layer): lemmas()
  now lists a base whose only row is sense-numbered. ~2,000 lemmas
  surfaced (nouns 3,868→4,905, adjectives 4,054→4,860, verbs
  4,307→4,466); treebank verbatim 32.0% → 29.0% from this fix alone.
- **The ordinals (вторы́й …)** — MISSING LEMMAS, confirmed (no rows,
  base or sensed). v1.x intake.
- **«Землѧ́ же» (grave→acute before an enclitic)** — OUT OF SCOPE BY
  DESIGN: a context rule over token sequences; no cell can carry it.
  If ever modeled, it belongs to the renderer/orthography layer with
  the Bible as its source. Deferred with reasons.
- **Synodal npron is empty** (part-1 finding, restated for the intake):
  zero syn: rows and an empty-string rule. The relative and possessive
  pronouns (и҆́же/ꙗ҆́же/є҆́же families) are among the commonest tokens
  of the language. v1.x intake, likely the single highest-value item.

## 2026-09-01 — syntax wave 2, part 4: Genesis 1 complete

All 31 verses stand in the committed hand overlay, byte-exact and — new
gate — LINT-CLEAN: check-treebank now lints every hand entry (hand trees
claim structure, so their claims are checked; auto-lifted trees are flat
and stay unlinted). Ceiling row: 617 tokens — 281 analyzed (45.5%) + 209
closed-class = 79.4% lifted, 6 deliberately ambiguous, 121 verbatim, 0
unaccounted.

The deliberate ambiguities are annotation honesty, recorded in the file:
«во дни̑» reads accusative plural but the crate offers only dual cells
for the spelling; «два̀ свѣти̑ла вели̑каѧ» sets the print's plural -аѧ
agreement against a dual numeral; «дѡбра̀» is a short plural adjective
the crate can only read as noun cells of добро̀ — a false analysis is
worse than none. The verbatim band is the intake, itemized: the
ве́сь/всѧ́къ npron family (part-3's headline gap), enclitic pronouns
(ѧ҆̀, є҆го̀, и҆̀хъ, себѣ̀ — cliticized forms the pronoun table does not
carry), бы́ти's future, the ordinals, and titlo families not yet
admitted (блгⷭ҇вѝ = благословѝ; гл҃ѧ against the crate's participle
spelling).

## 2026-09-01 — syntax wave 2, part 5: the close

Final state, whole Bible, zero round-trip mismatches, hand overlay
lint-clean: analyzed 20.0% + closed 27.1% = 47.1% mechanical (wave
start: 44.2%), ambiguous 23.7%, verbatim 29.0% (wave start: 40.5%),
apparatus 0.2%. Genesis 1 complete at its 79.4% ceiling. All standing
gates green; accuracy closed at 100.00%/0 untouched (this wave's
crate-side changes are the read-only lemmas() enumeration — extended to
npron and fixed for sole-_n lemmas — and nothing else).

What remains of the verbatim band, by class — the next map:
1. SYNODAL NPRON (и҆́же/ве́сь/всѧ́къ/сво́й families) + the enclitic
   personal forms (ѧ҆̀, є҆го̀, и҆̀хъ, мѝ, тѝ, сѧ̀…) — the v1.x
   program's headline: new table rows and pronoun cells, fed by the
   part-3 verdict list.
2. More titlo rows (блгⷭ҇вѝ's благослови́ти, the спⷭ҇ family's other
   lemmas, ѻ҆ц҃ъ's nominative) — syntax-side, one tsv row each.
3. The Bible-as-source design question (ї-before-vowel, the ѡ-plural,
   бѣ̀): the print is already sha256-pinned; admitting it as an
   extractor source would arbitrate every convention divergence at
   once. A v1.x design, not a patch.
4. The ambiguous band (23.7% and now the largest non-mechanical slice):
   syntactic disambiguation, still deliberately its own future design.

## 2026-09-01 — v1.2 (Synodal pronouns), part 0: design decisions

Executing V1.2-PROMPT.md. Three schema questions and two precedence
rules, decided before code, each with the measurement that will judge
it (part 4's per-family verbatim table over the whole Bible; the
baseline: 184,241 verbatim tokens, of which the pronoun families hold
≈45,000 — possessives 16,352, се́й/то́й/и҆́нъ/са́мъ 7,337, ве́сь/всѧ́къ
5,937, и҆́же 5,391, third-person short forms 4,735, clitics 2,898,
кто̀/что̀ 2,466, себѐ 1,240).

1. **The series axis — decided as recommended.** The 54-cell npron row
   is the SHORT (pronominal) series; an APRO entry's `plen` forms enter
   the ADJECTIVE table under the long citation form, exactly as
   `adjective_series_lemmas` already splits an adjective entry (the
   adjective schema has no series axis either: `-ъ` and `-ый` are two
   lemmas there). Polyakov pre-expands `plen/brev` into both analyses,
   so a cell the two series share (єди́ному) attests both lemmas. The
   npron key is the short citation form (всѧ́къ, ѻ҆́въ, є҆ди́нъ); the
   long one (всѧ́кїй, ѻ҆́вый, є҆ди́ный) is the adjective's, derived by
   the same headword/legend rule. Alypy §48.4 prints ѻ҆́въ/ѻ҆́вый side by
   side and feeds both tables from one grid.
2. **The reflexive — decided as recommended.** Six cells appended to the
   personal matrix (90..96, by case; the nominative is blank by rule) and
   a facade `reflexive(case, recension)`. `Person` stays as it is (the
   verb schema shares it). Sources: Alypy §47's third column (себѐ,
   with its clitic alternatives), Polyakov's себѐ (SPRO, 6,674 tokens),
   UD `Reflex=Yes` and PROIEL `Pk` — the 1,069 + 5,775 reflexive skips
   the recall harness has been dropping since Phase 0.
3. **The clitic cells — decided, with one refinement.** The enclitic
   personal forms become addressable cells instead of rank-10 variants:
   first and second person × number × {dat, acc} (96..108), the
   reflexive's {dat, acc} (108..110), AND the third person's accusative
   clitics × gender × number (110..119) — the refinement: Alypy §47
   prints the third person's short accusatives as alternatives too
   («є҆го̀, и҆̀»; «ѧ҆̀, и҆̀хъ»; nominative «ѻ҆́нъ (и҆̀)»), and the Bible
   uses «ѧ҆̀» 853 times as the enclitic plural/dual accusative beside the
   full «и҆̀хъ» (1,221). Arity 90 → 119. Facades: `clitic(person, number,
   gender, case, recension) -> Option<&str>` (None where the language
   has no clitic: every first/second-person dat/acc and every
   third-person accusative has one; nothing else does) and
   `reflexive_clitic(case, recension)`. The rule owns the closed
   inventory in both recensions (OCS ми/мѧ/ти/тѧ/си/сѧ/нꙑ/вꙑ/на/ва and
   the anaphor's и/ѭ/ѥ/ꙗ/ѩ; Synodal мѝ/мѧ̀/тѝ/тѧ̀/сѝ/сѧ̀/ны̀/вы̀ and
   и҆̀/ю҆̀/є҆̀/ѧ҆̀); the tables store only what the sources attest
   against it. Routing: Polyakov's `clit` tag → the clitic cell; an
   Alypy alternative, and a Polyakov third-person form, that equals the
   rule's clitic prediction for its cell (compared through
   `comparison_key`, so civil «я» reaches it) → the clitic cell. The
   OCS treebank mappers keep attesting the full cells (UD does not tag
   clisis; the OCS primary accusative IS the clitic form, мѧ).
4. **Precedence: the print outranks the transliteration.** A source
   property, `Source::letters_exact`: Alypy and the witness file are
   print-exact; Polyakov and ru.wiktionary are civil transliterations
   that cannot encode two distinctions the print makes — ꙗ against ѧ
   (civil «я»; `orthography.rs` realises initial я → ꙗ, right for
   ꙗ҆́же and wrong for ѧ҆̀) and the oxia against the varia on a
   monosyllable's only vowel (the print's «и҆̀хъ» accusative and «и҆̀мъ»
   dative against the genitive «и҆́хъ»; the Bible carries exactly these
   two words with a non-final varia, 1,711 + 1,220 tokens, and nothing
   else). Rule: when a print-exact observation merges into a
   transliterated one, a form that differs from the cell's primary ONLY
   within those two classes takes the primary slot and the
   transliterated spelling stays as a variant; a witness row (a quoted,
   verified line of running print) takes the primary slot
   unconditionally. Nothing attested is deleted. `realise` must keep an
   explicit varia on a monosyllable (today it normalises it to the
   oxia, so the print's «и҆̀хъ» could not even be stored). Recorded in
   the lib.rs lookup invariants as number 5.
5. **`+`-headwords.** A Polyakov headword `X+же` is the print's one word
   `Xже` (то́йже 14,975 tokens, что́же 100): admitted by joining, declined
   by the же-wrap the OCS rule already has. Every other `+` headword
   (и́+на, и́+въ, что́+либо, ничто́же+въ) stays rejected as before.

Also recorded for part 1, verified against the committed row: the
`syn:personal` primary spells «ꙗ҆̀» at cells 45/63/81/87 (Polyakov's
«я́», tag-bundled over `sg,f,nom|pl,m,acc|pl,f,nom/acc|pl,n,acc|du,acc`),
«и҆́хъ» at the locative cells 53/71/89 and at the feminine plural
NOMINATIVE 66 (Polyakov's `pl,gen/loc|pl,m,acc|pl,f,nom/acc` bundle);
Alypy §47 prints «ѧ҆̀» / «(н)и́хъ» / «ѻ҆нѣ̀» / «и҆̀мъ» there. Under
decision 4, Alypy's forms arbitrate every one of those cells; the
Bible witnesses pin the two the grammar leaves open (ѻ҆на̀ as the
feminine singular nominative against the bundle's ꙗ҆̀; the plural
accusative «и҆̀хъ»).

## 2026-09-01 — v1.2 part 1: the Synodal personal row arbitrated against the print

The row that existed is now true. Three mechanisms, each in its own layer,
and twelve witness rows:

- **Orthography**: `realise` keeps an explicit varia on a monosyllable's
  only vowel (the print's «и҆̀хъ»/«и҆̀мъ» against «и҆́хъ»; the Bible carries
  exactly those two words with a non-final varia, 1,711 + 1,220 tokens),
  and `transliteration_equivalent` names the two classes a civil
  transliteration cannot encode (ꙗ/ѧ, that varia).
- **Precedence as a source property** (`Source::letters_exact`,
  `Observation::precise`): a print-exact observation merging into a
  transliterated one takes the primary slot where it differs only within
  those classes; a witness row is primary unconditionally
  (`merge_as_primary`, into a lexeme with exactly one observation — a
  homograph set keeps the v1.1 separate observation). `witnesses.tsv`
  now accepts `pronoun` and `npron` with symbolic cells
  (`cells::parse_cell`: `3.f.sg.dat`, `m.pl.gen`, `pl.acc`).
  Lookup invariant 5 in lib.rs.
- **The bare key of the shared pronoun row** goes to the row of PRIMARY
  readings (`Candidate::primary`, pronoun only): the lexicographic sort
  had handed `syn:personal` to the shortest variant row («ны̀» as the
  plural nominative, «тебѐ» as the genitive, «и҆̀» as the nominative) and
  the print-primaries to `personal_10`. Tried for every part of speech
  first: OCS/Synodal bare-primary rates jumped (verbs OCS 92.43% →
  99.98%) but the treebank refuted it at Gen 1:21 — Polyakov's counts
  are per FORM, so at a tag-bundled cell (`pl,nom/acc`) the nominative's
  frequency wears the accusative's tag and «га́ды» outranked the animate
  «гадѡ́въ». Reverted to the pronoun row; the lemma-keyed rows' true
  per-cell primary is a Bible-as-source question, recorded.
- **Two source-reading defects found on the way**: (1) Polyakov's
  third-person headword is the anaphor «и҆̀», whose nominative the
  language does not use — its `nom/acc` bundles had put «и҆̀», «ꙗ҆̀»,
  «є҆̀» and «и҆́хъ» into the NOMINATIVE cells; the anaphor's nominative
  analyses are now skipped (12, counted). (2) Polyakov transcribes the
  erok and consonant-borne abbreviation marks with a kamora on a
  consonant («нас̑», «нбс̑нѣй»); 3,724 such spellings were attested as
  forms and, once the primary sort reached them, «нбс̑нѣй» became
  небе́сный's bare locative (the treebank refused it at Gen 1:14). They
  are abbreviations, not forms: skipped, counted; 366 noun, 449
  adjective and 337 verb rows that existed only for them are gone.

Witnesses (all verified, `check-witnesses` 15/15): мы̀ (Gen 19:13),
тебє̀ (Gen 3:10), ѻ҆́нъ (Gen 15:10), ѻ҆на̀ (Lev 20:17), є҆́й (Gen 16:8),
є҆́ю (Gen 9:1), ѧ҆̀ dual (Gen 1:17), и҆̀мъ (Gen 9:1), и҆̀хъ (Gen 6:13),
ни́хъ ×3 (Gen 24:3, Gen 30:34, Eph 2:10). The primary row now differs
from the rule in exactly twelve cells (не́ю ×3, и҆̀мъ ×3, и҆̀хъ ×2,
ни́ми ×3, ѧ҆̀ at the neuter plural accusative — part 3 moves that one
to its clitic cell); every transliterated spelling and the anaphor's
short forms remain reachable as `_n` variants (ledger test
`tests/synodal_pronouns.rs`, exact outputs, verse per cell).

Gates: accuracy 100.00% / gap 0 on every source, the new "Synodal
(witnessed print)" pronoun row 12/12; check-registry, check-witnesses,
all suites, zero warnings. Bare-primary rates: Polyakov pronouns 100% →
78.87% (the 15 demotions ARE the arbitration — the corpus's primaries
were the transliteration's), Alypy pronouns 68.89% → 76.67%; the erok
cleanup lifted Polyakov nouns 96.43% → 96.86%, adjectives 95.95% →
96.71%, verbs 97.08% → 97.20%. Treebank rebuilt: zero mismatches over
34,470 verses; from this part alone verbatim 29.0% → 27.8%, ambiguous
23.7% → 24.7%, analyzed 20.0% → 20.2% (и҆̀хъ, и҆̀мъ, ѧ҆̀ now lift).

## 2026-09-01 — v1.2 part 2: Synodal non-personal pronouns — rule, sources, rows

Synodal npron is no longer empty. The rule (`core::npron_syn`, dispatched
by recension from `npron`) spells the print's closed lexicon cell by
cell from Alypy §47 (то́й, мо́й) and §48 (кто̀/что̀, кі́й, на́шъ, ѻ҆́въ):
literal tables for то́й/се́й/ве́сь/кі́й/кто̀/что̀, ending tables for the
ending-stressed possessives (мо́й, тво́й, сво́й, чі́й), the stem-stressed
soft (на́шъ, ва́шъ) and hard (ѻ҆́въ, ѻ҆́нъ, є҆ди́нъ, всѧ́къ, толи́къ,
є҆ли́къ, коли́къ, и҆́нъ, са́мъ; velar softening всѧ́цѣмъ/є҆ли́цы) classes,
the relative и҆́же as the third-person row + же with the print's plural
varia (и҆̀же, ꙗ҆̀же, и҆̀хже, и҆̀мже against и҆́же, ꙗ҆́же, є҆́же, ю҆́же —
the Bible: ꙗ҆̀же 1,214 / ꙗ҆́же 392, и҆̀же 1,078 / и҆́же 1,438), and the
ни-/нѣ́- prefixes and же/жде/ждо enclitics by strip-decline-rewrap
(никто́же, нѣ́кій, то́йже, кі́йждо). `realise` treats a solid enclitic's
host as the word for the monosyllable-varia rule; the personal rule now
says «и҆̀мъ»/«и҆̀хъ» itself (pron:plural-varia) and «ѧ҆̀» for the neuter
plural accusative (§47 prints no other).

Sources admitted: Polyakov APRO (75 declinable lemmas; `brev` → the npron
row under the short citation form, `plen` → the ADJECTIVE row under the
long one, by `adjective_series_lemmas` — a pronominal class cites its
short form even in -ой, so мо́й no longer legends a «мъ»), the SPRO
кто̀/что̀ family (singular-only; classes PNkto/PNcto), the `X+же`
headwords joined (то́йже 14,975 tokens; every other `+` headword still
rejected), Alypy §47.2 (то́й), §47.1's мо́й columns, §48.0/1/2/4 — a
`Paradigm` now names its column range where a grid prints two paradigms
side by side (ѻ҆́въ | ѻ҆́вый feeds both tables). Left out: §48.3 (the
толи́цы fragment), ADVPRO (adverbs), себѐ (part 3).

Two source-reading rules, both measured, both in the extractor:
- **A transliteration's letters are the rule's where it cannot encode
  them** (`attested_matches`): a Polyakov form that differs from the
  prediction only by ꙗ/ѧ or the monosyllable varia is the rule's form
  and is not stored (its «ꙗ҆́же» reproduces the print's «ꙗ҆̀же»); the
  accuracy harness scores each source under what it can encode
  (`Source::letters_exact`), so the civil «ꙗ҆̀» variants of part 1 are
  gone from the personal row too — they were never distinct forms.
  Exactness is per attested form (`Observation::exact`), so a grammar's
  «ѧ҆̀» beside a dictionary's «ꙗ҆̀» keeps its letters.
- **The number mark of a pronoun cell is the rule's**
  (`pronoun_attested_matches`, Pronoun and NPron only): the dictionary's
  tag bundles (`sg,f,nom|pl,m,acc|pl,f,nom/acc`) put the singular's
  spelling «всѧ̀», «сіѧ̀», «на́ша», «є҆ди́на» into the plural cells the
  print marks with the kamora («всѧ̑» 1,791 tokens against «всѧ̀» 100);
  a transliterated form differing only in the stress MARK on the same
  vowel is the rule's form. Scoped to the pronoun rows on purpose: the
  noun/adjective ^-cells were validated under exact matching and stay so.

Witnesses (+8, `check-witnesses` 23/23): the relative's plural cells the
bundles mis-attest (и҆̀же Gen 14:5, и҆̀мже Ex 6:26 / Ex 35:26, ни́хже ×3
Gen 19:29 / 24:37 / 41:47, ꙗ҆̀же Gen 1:21) and всѧ̑ (Gen 1:31).

Numbers. 158 rows, 53 bare rows, 60 lemmas, 1444 stored cells (585 in bare rows). The possessives
мо́й/тво́й/сво́й have NO bare row: the rule reproduces every corpus
primary. Accuracy 100.00% / gap 0 on every source including the new rows
— Non-personal pronouns Synodal (Alypy) 252/252, (Polyakov) 1,651/1,651,
(witnessed print) 8/8. The rule's honest measure, the bare-primary table:
Polyakov npron 93.03% (115 of 1,651 attested cells demoted to a variant
or answered otherwise), Alypy npron 85.32%; the personal row's Polyakov
bare rate rose 78.87% → 91.55% (the ꙗ҆̀ artefacts are no longer counted
as demotions). Skips now counted: adjective long series without an
attested masculine nominative 121 (APRO `plen` forms of PA1 lemmas whose
long nominative the entry never spells — са́мъ's са́мый is its own entry),
"pronoun: outside the personal matrix" 20 (себѐ and the noun-like
и҆́мѧрекъ). Treebank: zero mismatches unchanged (the lift is part 4).
Registry: `npron_syn`'s test shows сь/се́й, тъ/то́й, вьсь/ве́сь,
къто/кто̀, -ѥго/-егѡ̀ and the relative's plural varia cell by cell.

## 2026-09-01 — v1.2 part 3: the reflexive and the clitic cells

The personal matrix grew from 90 to 119 cells: the reflexive at 90..96
(by case; the nominative blank), the clitics at 96..119 (first and second
person `number × {dat, acc}`, the third person's accusatives `gender ×
number`, the reflexive's dative and accusative). `schema::reflexive_cell`,
`clitic_cell`, `reflexive_clitic_cell` and the decoder
`pronoun_features` own the geometry; `Person` is untouched. The core rule
spells the closed inventory in both recensions (OCS ми/мѧ/ти/тѧ/на/ва/
нꙑ/вꙑ/си/сѧ and the anaphor's и/ѭ/ѥ/ꙗ/ѩ, where the clitic IS the primary
accusative; Synodal мѝ/мѧ̀/тѝ/тѧ̀/ны̀/вы̀/сѝ/сѧ̀ and и҆̀/ю҆̀/є҆̀/ѧ҆̀; the
reflexive себє̀/себѣ̀/себѐ/собо́ю — the genitive in є like тебє̀, the
Bible's 111 against the accusative's 237). Facades: `reflexive`,
`clitic` (`None` where the language has no clitic), `reflexive_clitic`,
each with a `_sense` twin for the variant keys — the accuracy harness
found the first version blind to them (Polyakov's Russianism «себѧ̀»
unreachable) and the twins are the fix, in the API's own pattern.

Sources: Polyakov's `clit` tag routes to the clitic cell (мѧ̀ 9,226 + 3,849,
мѝ, тѝ, тѧ̀, ны, вы, сѧ, си); the anaphor entry's forms that spell the
rule's clitic (through `comparison_key`, so civil «я» reaches «ѧ҆̀»)
route there too, and where the clitic IS the full form (ю҆̀, є҆̀, the dual
and neuter-plural ѧ҆̀) they attest both cells — the first version left
the full cells to the prepositional variants (ню̀, нѐ, нѧ̀) and the table
said so. Alypy §47's alternatives («мнѣ̀, мѝ», «є҆го̀, и҆̀», «ѧ҆̀, и҆̀хъ»,
the nominative row's «(и҆̀)») route by the same rule; its third column
is the reflexive, recognised by its forms. The dictionary writes an
enclitic unstressed («ны», «ся»): on the pronoun rows a monosyllable's
mark presence is the rule's (`number_mark_equivalent`; two witnesses
pin it, ны̀ Gen 47:25 and сѧ̀ Ex 28:43). Tried as a general
transliteration class first and refuted twice — the accent-pattern
inference and the post-assignment re-store pass re-materialised the
unaccented «рабъ» into ра́бъ's bare row (Gen 9:25), and once those two
passes judged forms under the same per-cell exactness, the strip
still reshuffled сꙋ́дъ's lexicographic bare key (Gen 18:25) — so it is
scoped to the pronoun rows, and the noun/adjective/verb tables are
byte-identical to part 2's. UD `Reflex=Yes` (minus `Poss=Yes`, the
possessive свои) and PROIEL `Pk` now map to the reflexive cells.

The `syn:personal` primary row now differs from the rule in six cells
only (не́ю ×3 at the dual genitive, ни́ми ×3 at the plural instrumental —
the corpus's prepositional primaries); the ѧ҆̀ cell of part 1 is the
rule's. Gates: accuracy 100.00% / gap 0 on every source (the variant
gaps the harness raised at first — reflexive variants unreachable —
closed by the `_sense` facades); check-registry; check-witnesses 25/25;
all suites; zero warnings; treebank unchanged at zero mismatches.
Corpus recall now COUNTS the reflexives it used to drop: pronouns UD
dev+test 4,918/4,960 → 5,984/6,029 (99.25%; the denominator +1,069),
Syntacticus 26,764/27,025 → 32,525/32,800 (99.16%; +5,775) — recall on
the old cells did not fall. Bare-primary: Polyakov pronouns 91.55% →
93.41%, Alypy 76.67% → 84.21%.

## 2026-09-01 — v1.2 part 4: the consumer proof — the treebank

The syntax crate inverts the new cells: every Synodal non-personal
pronoun lemma × 54 cells (`lemmas()` now lists 60), the reflexive's
five cases, and every clitic cell, into the surface → analyses index;
two leaves join the tree language — `(pers … :clit yes)` for a cell's
enclitic form and `(refl :case dat)` / `(refl :case acc :clit yes)` for
себѐ — with exact-output tests pasted from the crate, and the old "syn
npron refuses to render" test replaced by the rendering one. The
round-trip invariant held at zero mismatches over all 34,470 verses
after the rebuild (inverse index 839,106 surfaces).

Whole Bible, before → after (part 1's build → part 4's):
analyzed 20.2% → 21.2%, closed 27.1%, ambiguous 24.7% → 31.2%,
verbatim 27.8% → 20.3% (176,852 → 129,239 tokens). Per family
(verbatim / ambiguous):

| family | before | after |
|---|---|---|
| possessives (мо́й, тво́й, сво́й, на́шъ, ва́шъ) | 16,518 / 0 | 27 / 13,845 |
| се́й, то́й, и҆́нъ, са́мъ | 6,467 / 1,764 | 1,451 / 4,321 |
| ве́сь, всѧ́къ | 7,228 / 419 | 64 / 5,536 |
| и҆́же | 5,391 / 0 | 642 / 6,797 |
| third-person short forms | 715 / 16,137 | 618 / 17,368 |
| clitics (мѝ, тѧ̀, ны̀, сѧ̀ …) | 2,898 / 1,667 | 784 / 1,813 |
| кто̀, что̀ and compounds | 2,466 / 0 | 470 / 1,996 |
| себѐ | 1,240 / 0 | 15 / 854 |

The families moved from verbatim into ANALYZED and AMBIGUOUS — the
possessives' 13,845 ambiguous tokens are the honest homographs of the
paradigm (моѧ̑ is nominative and accusative of three genders; the
disambiguation design is still deliberately its own future). The
residue is itemised in part 5.

One more assignment finding on the way: the new adjective row всѧ́кій
(the APRO long series) took its bare key from a row carrying Polyakov's
single unaccented «всякую», so «всѧ́кꙋю» in Gen 1:29 rendered without
its stress. A stored Synodal form with no stress mark under an accented
lemma is a transliteration's dropped accent, never the print's: the
bare-key sort now orders clean rows before noisy ones
(`Candidate::noise`, after soft/primary, before the form signature).
Polyakov bare-primary rates: nouns 96.86% → 96.91%, adjectives 96.67% →
96.74%, verbs 97.20% → 97.29%; accuracy 100.00%/0 on every source.

Genesis 1 (the hand overlay): every pronoun leaf lifted — the ве́сь/всѧ́къ
family (всѧ́кꙋ, всѧ̑, всѣ́ми, все́ю, всеѧ̀, всѣ̑мъ, всѧ́комꙋ; всѧ́кꙋю /
всѧ́кое as the adjective всѧ́кій, the part-0 decision), the relative
(ꙗ҆́же ×4, є҆мꙋ́же ×2, ꙗ҆̀же as the plural nominative of 1:22 and the
plural accusative of 1:21, и҆́же), сво́й, на́шъ, є҆ли́къ, the
third-person accusatives (и҆̀хъ ×2 full; the dual «ѧ҆̀» of 1:17 as the
full cell — the two lights; the «ѧ҆̀» of 1:22 as the clitic, masculine
plural for the mixed creatures), є҆́ю, and себѣ̀ as (refl :case loc).
Ceiling 79.4% → 85.1% lifted (316 analyzed + 209 closed of 617; 86
verbatim, 6 deliberate ambiguities unchanged). Lint-clean.

## 2026-09-01 — v1.2 part 5: the close

Version 1.2.0 (`church-slavonic`, `church-slavonic-core`); CHANGELOG
entry per part; README tables regenerated from the final `cargo xtask
accuracy` run (29 recall rows at 100.00% / 0) and the final
`check-treebank`. Two last crate-side items from the residue: (1)
`lemmas(NonPersonalPronoun)` now chains the rule's closed lexicon
(`ChurchSlavonicCore::npron_lexicon`) — a lemma the rule serves entirely
has no table row and was invisible to the lift (никто́же 174 tokens,
нѣ́кто, кто́же); (2) a witness for the relative's masculine accusative
«є҆го́же» (Gen 22:2; 503 tokens) against the dictionary's `nom/acc`
bundle. Treebank rebuilt: zero mismatches.

Final table, whole Bible (631,946 tokens): analyzed 21.5% (wave start
20.0%), closed 27.1%, ambiguous 31.0% (23.7%), verbatim 20.2% (29.0%;
127,492 tokens), apparatus 0.2%. Per family, verbatim before → after:
possessives 16,518 → 27; се́й/то́й/и҆́нъ/са́мъ 6,467 → 1,451; ве́сь/всѧ́къ
7,228 → 64; и҆́же 5,391 → 135; third-person short forms 715 → 618;
clitics 2,898 → 784; кто̀/что̀ 2,466 → 235; себѐ 1,240 → 15. Genesis 1
at 85.1% lifted (79.4%).

What of the pronoun band is STILL verbatim, by class — the next map:
1. **The prepositional н- forms** (ни́мъ 730, немꙋ̀ 524, ни̑мъ 353, негѡ̀
   328, нѧ̀ 130, ню̀ 98, нѧ́же 70, ню́же 62): attested, stored as `_n`
   variants of the personal row and of и҆́же, and the lift inverts the
   primary row only. A syntax-side design: either invert the variant
   keys (`(pers … :key personal_3)`) or give the prepositional form a
   feature of its own; the print is unambiguous about it (always after a
   preposition).
2. **The enclitic accent contexts** (тѧ 177, мѧ 229, ми 161, ти 53, вы́
   80, ты́ 180, что́ 74, кто́ 61, Ѻ҆ни́ 83, Ѻ҆на́ 43): the print writes a
   clitic unstressed after a proparoxytone and a host with the oxia
   before же/бо/ли — the «Землѧ́ же» class, out of scope by design (a
   context rule over token sequences, recorded since syntax wave 2).
3. **The print's ї before a vowel** (сїѧ̑ 1,080, сїѐ 675, сїю̀ 175, сїѧ̀
   175, всѧ́кїѧ 70, всѧ́кїй 28): the crate's canonical Synodal typography
   folds ї to і because the pinned sources never spell ї before a vowel;
   séй's whole feminine/neuter is in this class. The Bible-as-source
   design (HANDOFF item 2) is the only clean resolution.
4. **Capitalised sentence-initial forms** whose lowercase is ambiguous
   or prepositional (Сїѧ̑ 159, Ѻ҆ни́, Вси́ 14): the `cap` path lifts a
   capital only when the lowercase lifts.
5. **Residual bundle artefacts** the witnesses did not reach (ѻ҆́но 58 —
   the demonstrative ѻ҆́нъ's neuter with Polyakov's second entry's
   stress; всѧ́цѣй 45 — the print's dative/locative of всѧ́кїй against
   the rule's всѧ́кой; вси́ 34 before an enclitic).
Outside the pronoun band, the biggest verbatim tokens are the intake for
the next waves: бꙋ́детъ 2,170 / бꙋ́дꙋтъ 660 / бѣ̀ 855 (бы́ти's future and
imperfect — the schema gap), the titlo families (гдⷭ҇а 994, гдⷭ҇ꙋ 567,
нн҃ѣ 834, бж҃їѧ 283), the closed-class words not yet in the table (и҆лѝ
1,027, ѹ҆̀бо 761, да́же 730, та́мѡ 671, нижѐ 607, поне́же 492, ѕѣлѡ̀ 452,
па́че 451, до́ндеже 434 — one attested row each), the numerals (два̀,
трѝ, пѧ́ть, се́дмь, два́десѧть), and the nouns with the print's plural
marks (лю́дїе 661, мꙋ́жїе 376).

The disambiguation of the ambiguous band — now 31.0% and the largest
non-mechanical slice, the possessives' 13,845 homographs its newest
members — stays deliberately its own future design.

## 2026-09-04 — 2.0 Part 1: Synodal nouns — decisions and findings

- **Stress inventory (decision).** Census over the fitted Polyakov noun
  lexemes: a 12,082 · b 619 · a{pl=E} 20 · a{ins.pl=E} 18 ·
  a{gen.pl=E;dat.pl=E} 18 · b{voc.sg=S} 16 · a{gen.pl=E} 11 · the rest
  ≤ 10 each (172 specs). Named paradigms: a, b, c (S;pl=E), d (E;pl=S);
  everything else inline per lexeme. Among b-lexemes with an attested
  vocative, 75 stress the ending and 20 the stem — the vocative
  retraction (ра́бе) is a minority and stays per lexeme. Consistent with
  the v0.8 rejection of mobile tokens; the class prior did not change
  the verdict.
- **Kamora against widening under final stress (finding).** Over
  Polyakov's noun forms: 197 forms carry the kamora on a final stressed
  vowel with a narrow о/е in the stem (брозды̑, вдовы̑, борбы̑) against 508
  with a widened stem letter (брѡзды́, вдѡвы́, вєрхи́; lexical ѡ included
  in that count) — the same lexeme often has both. `Form::print` keeps
  the widening (the v0.8 convention); the kamora readings are stored per
  lexeme. The Bible will arbitrate in Part 2.
- **Bundled tags (finding).** A Polyakov form tagged `pl,gen/acc` attests
  the accusative only weakly: the print's animate plural accusative is
  the nominative-shaped one as often as the genitive-shaped one
  (а҆́ггелы, а҆рхіепі́скопы against рабѡ́въ). The importer lets a form tagged
  for the cell alone outrank a bundled one and accepts any alternative
  for a bundled-only cell — the 1.x «га́ды/гадѡ́въ» problem, closed at
  import.
- **Polyakov's codes are loose (finding).** ѻ҆се́лъ, со́нъ, ле́въ, ле́нъ are
  coded N1t; the legend's N1t* exemplars are осел-ъ, сон-ъ. The importer
  tries the fleeting and velar twins and keeps the best fit; a
  monosyllable drops its only vowel (днѝ, сна̀), a dropped vowel after a
  vowel leaves й (бойцы̀), and ле́въ : льва̀ needs `stems=1=льв`.
- **The print's second plural series (finding).** -ахъ/-амъ/-ами, -ове,
  -ови, the zero genitive plural on the full stem (ѻ҆тє́цъ, ѕлодѣ́латєль)
  and the -ѧхъ/-ѧмъ/-ѧми of the -іе neuters are alternatives in most
  classes; the majority alternative per (class, cell) is the primary,
  by census (`== alternative preference`).
- **Number marks (finding).** The legend omits the mark on cells the
  print marks: every masculine ins.pl (рабы̑, а҆́ггєлы) and the neuters'
  nom.pl (бдѣ̑ніѧ, беззакѡ́ніѧ). Measured per (class, cell) and set by
  `--fix-marks`; N1k nom.pl is split 98/120 and stays per lexeme.
- **Gate shortfall (decision).** 94.87% primary / 96.81% reachable /
  true exceptions on 7.36% of lexemes against the written 99% / 5%. The
  residue is source noise and lexeme-level preference (samples in the
  CHANGELOG entry); no mechanism is missing. Part 2 proceeds; the gate
  is recorded as unmet.

## 2026-09-04 — 2.0 Part 2: the treebank re-lift policy (decision) and a 1.x overlay error (finding)

- **Re-lift policy.** Until every part of speech lives in the 2.0
  lexicon, `build-treebank` re-lifts stored trees in place rather than
  from scratch: a verbatim or 1.x-noun leaf is re-analyzed, an ambiguous
  1.x leaf keeps its count (a noun-only analyzer cannot rule out the
  verb reading the 1.x index saw). Every replacement must render the
  token back byte-for-byte on its own before it enters the tree. The
  full re-lift from the 2.0 analyzer alone is Part 3's, when the legacy
  dependency goes.
- **A false analysis in the hand overlay.** Genesis 1:29's two «є҆́же»
  were `(n є҆́жъ :case voc :num sg)` in the 1.x overlay — the vocative
  of the hedgehog, which happened to print as the relative pronoun. The
  round-trip invariant cannot see a homograph; the id leaf exposed it
  (єжъ.n is N1s, whose vocative is є҆́жꙋ). Corrected to the relative.
  The lesson stands from v1.1: a homographic render is not an analysis.

## 2026-09-05 — 2.0 Part 3: decisions and findings

- **Enclitics are a stem entry, not a rewrap (decision).** `stems=encl=сѧ`
  (verbs), `encl=же|жде|ждо|либо|надесѧть` (pronouns, compound
  adjectives): the class works on the lemma without the enclitic and the
  print writes it solid after every ending, dropping the jer before it.
  One mechanism replaces 1.x's strip-decline-rewrap and its reflexive
  special case; the number mark skips the enclitic's vowels
  (`Form::mark_skip`), which the 1.x renderer got wrong for є҆гѡ́же.
- **The print's own accent choices are data (finding).** Two words carry
  a non-final varia (и҆̀хъ, и҆̀мъ against и҆́хъ, и҆́мъ) and the relative
  keeps it under же (ꙗ҆̀же, и҆̀мже); сво́й's plural takes the kamora
  (своѧ̑) where the rule would widen (свѡѧ̀). A form model that decides
  the mark kind alone cannot store these prints; `varia` and `kamora`
  flags set by `from_print` make every attested print round-trip, and a
  class-built form still takes the rule.
- **Polyakov's closed classes are homographs (finding).** и҆ is a CONJ
  entry and a PART entry, не likewise; indexing them as two lexemes made
  every function word ambiguous (closed 27.1% → 4.0%, ambiguous 64.4%).
  The lifter treats closed-class readings of one surface as one
  function word, `(f и҆)` by its surface; the leaf claims no lexeme.
- **Headword versus citation form (decision).** 216 Polyakov headwords
  spell the citation form otherwise than the attested forms (тьма̀ /
  тма̀, ѻ҆бразъ / ѡ҆́бразъ). The attested print is the lemma and the id;
  the headword is a note. Quarantining them (Part 1) lost the treebank's
  тма̀.
- **Participle columns the legend does not print (finding).** The short
  masculine accusative is the genitive-shaped -ща (330/374), the short
  plural nominative -ще (1,084 overrides before), the long neuter
  nominative Polyakov's -ѧй tagged m/n (918), the past active long
  nominative both -ивый and -ившїй, the past passive short plural
  obliques keep the double н (-нныхъ). Explicit columns cut verb
  overrides from 48.8% to 24.7% of lexemes.
- **Alypy's dual `-ѣ` (finding).** The 1.x loader expanded «пи́ш-е-та, -ѣ»
  to пи́шеѣ (the last hyphen segment replaced whole) and stored it as an
  attested form; 1.x's 100.00% on Alypy included it. The ending rule now
  replaces the last segment only when it is no longer than the ending.
- **Pronoun classes are literal tables (decision).** The third person,
  the relative, кто̀/что̀ and the athematic verbs are classes whose cells
  are literal endings on a (possibly empty) stem: a closed paradigm is
  data, and the fit still derives its stress and its exceptions.
- **Ambiguity is where the work is (finding).** 39.1% of Bible tokens
  have several exact readings — nom/acc/voc of every noun, the genders
  of every adjective form, du/pl homographs. Disambiguation by context
  is a later, separate design; the treebank records the count.
- **witnesses.tsv (deviation).** Converted to `W:` provenance (2 noun
  rows quarantined: lemmas the lexicon lacks); the file stays until
  Part 5 because the legacy accuracy harness — the baseline instrument
  Part 0 requires until then — reads it.

## 2026-09-05 — 2.0 Part 4: decisions and findings

- **OCS classes come from Kaikki, not from the legacy consts (decision).**
  The prompt suggested an `ocs` column ported from the 1.x ending arrays;
  Kaikki's 2,826 full paradigms with their own stem-class tags are a
  better source: a class is a shape the data attests, its row the
  majority ending per cell, and the fit reports how well each entry sits
  in it. The tables are per recension; the Synodal inventory is not
  touched.
- **The measurement must be the 1.2 measurement (finding).** With a plain
  key the OCS lexicon scored 46/22/10% on nouns/adjectives/verbs; ported
  the harness's fold (jers, шт, ѣ~е, contractions) and its abbreviation
  rule — the commonest tokens (г҃ь, б҃ъ, іс҃ъ) are written under a titlo —
  and the same lexicon scores 95/89/86%. The numbers are comparable now;
  they were not before.
- **Ties are the class-choice problem (finding).** A UD lemma attested in
  one cell fits fifty classes equally; the first row of the table won and
  въсхотѣти landed in an -ати class. The exemplar's shared ending decides
  a tie, classes seeded from present-less entries (`?` in the name) come
  last, and a class must produce the lemma from the citation cell before
  it is tried.
- **The third person is one lexeme in both recensions (decision).** и/ѥго
  (OCS) and ѻ҆́нъ/є҆гѡ̀ (print) are PP3 lines on the empty stem; the relative
  is PPize with `encl=же`; Kaikki's form-of headwords (ими, ѩ, ѭ) are its
  typo class and quarantined.
- **Residue to chase later, not now.** Guessed verbs cannot know an iotated
  present stem (люблѭ from любити) — a class-level `iot` derivation would
  give it; the Kaikki tables print no past participles, so the derived
  ones (stems 7/8/11) are a rule, not data; 14,059 dev+test slots are
  skipped by the loader (subjunctives, supines, ambiguous cases) as in 1.2.

## 2026-09-05 — 2.0 Part 5: the cutover — decisions and findings

- **The consumer is the last audit (finding).** Migrating the game found
  four lexicon defects the harness had not: the stress paradigm of a verb
  whose participles outnumber its finite cells (дои́ти), a Polyakov class
  missing from the inventory (Viti: и҆тѝ and 41 compounds), the gender of
  что̀, and the plural varia of the third person lost to an import that
  predated the `varia` flag. Each is fixed in the lexicon or the fit, none
  in the game.
- **Re-paste, do not argue (decision).** Where the game's pinned strings
  disagreed with the crate, the print decided (ѻ҆́вцꙋ 14:1, the ї before a
  vowel) or the only pinned attestation did (Polyakov's хартіѧ̀, ѳи́та,
  ꙗ҆ицѐ); the 1.x strings had come from the 1.x rule echoing the game's
  own accent. The letter name ѳи́та rests on one Polyakov attestation and
  is recorded as such.
- **The titlo layer belongs to the lexicon (decision).** `lexicon/titlo.tsv`
  and `church_slavonic::titlo` let a consumer decline a titlo-written
  lemma through its full lexeme; the treebank's `(abbr …)` wrapper is one
  consumer, the game another.
- **The legacy instruments are gone with their baselines recorded.** Every
  1.2 number the gates compared against is in the CHANGELOG's Part 0
  entry; nothing needs the deleted tree.

## 2026-09-05 — 2.1 Part 0: the stem census (findings)

- **Stored stems are mostly not lexical (finding).** Of the 1,442 OCS verb
  lines with a present stem, 370 are a regular derivation of the
  infinitive stem (theme-vowel drop, iotation, -ова- → -ꙋ), 491 store a
  whole attested present form (the seeding took one form's prefix when
  only one present form was attested), and most of the 581 "suppletive"
  stems are short prefixes of the same kind (блазнити 2=бла). The true
  suppletion is a few dozen lines. The Synodal lexicon's 636 stored
  stems are the participle-stem inference of Part 3 reading whole forms
  off cells with an empty ending (`9=алчꙋщꙋ`), plus about a hundred
  genuine present suppletions (взѧ́ти → возм) and a hundred stem-1
  corrections (би́ти → бі).
- **The guesser number is the honest baseline.** A guessed OCS verb
  reproduces 22.7% of its present cells today, a Synodal one 46.5%;
  Part 1 is measured against these.

## 2026-09-05 — 2.1 Part 1: present stems by derivation — decisions and findings

- **A class is a Leskien type, and the type declares the stem of every
  cell (decision).** The first pass matched types by predicting the first
  and third person singular but read each cell's ending against the first
  stem that was a prefix of the form; members whose iotation changed no
  letter (гонити: гон = гон) then voted `2-иши` where любити voted
  `1-иши`, and the class carried both as alternatives. The type now says
  which stem a cell is built on (class IV: first person and imperfect on
  the iotated stem, the rest on the plain one; class III -j-: everything
  on the iotated stem; velars: 2 for the first person and third plural,
  3 for the other persons, 4 for the imperative), and the data supplies
  only the ending. Where the data disagrees with the declared stem the
  other stems are tried and the ending recorded against whichever fits —
  the aorist of a velar verb (влѣче) reads `3-е`.
- **The spelling rule after a husher is the crate's, not the table's
  (decision).** Kaikki writes прошѫ, пишетъ, рождѫ, хождаахъ and люблѭ,
  глаголѥтъ, гонꙗахъ: the iotated vowel is plain after ж ч ш щ ц жд. The
  count over every Kaikki form is a few thousand to a few hundred in the
  rule's favour (жд+ѭ 108 forms from three lemmas' participles, ж+ѭ 2;
  nouns: пищѩ 20, кръвоточицѩ 12). Applied in `Ext` and at the ending in
  OCS; the Synodal rule (ѧ/ѣ → а after a husher) stays a derivation's
  only, and applying it at the ending cost 139 Bible tokens before it was
  taken back. The rule reached nouns and adjectives too: OCS noun recall
  94.87 → 95.48, adjectives 89.35 → 89.31 (one token), Kaikki noun cells
  99.1 → 98.6% (their ѩ after ц/щ became variants).
- **The hidden consonant is a stem, the infinitive is the base
  (decision).** The first seeding put the dental of грѧсти into stem 1
  (`1=ext:д`), so no cell of the class produced the infinitive and the
  citation filter rejected every such class; every -сти verb then landed
  in the one class whose seed happened to be a с-stem. Now stem 1 is the
  bare base (грѧ), the infinitive is `1-сти`, and stem 2 restores the
  consonant. The same for velars (рещи: `2=ext:к`, `3=pal1`, `4=pal2`).
  с-stems (нести, пасти) are the plain consonant class `V:I:C`.
- **Kaikki's tables are template output with junk in the non-present
  cells (finding).** коснѫти's aorist and l-participle are given as
  косехъ, кослъ; клѧти's as кльнхъ, кльнлъ; грѧсти's l-participle as
  грѧдлъ. The class cells stay data-driven for those blocks, so the tables
  reproduce the junk as they did in 2.0 (the Kaikki number is a
  reproduction number, not a correctness one); the UD attestations
  (коснѫ сѧ, клѧтъ) enter as variants. A later part could declare the
  aorist and l-participle by type as the present now is.
- **The residue is real suppletion, 56 lines named in the census
  (finding).** възьмати → въземл, бьрати → бер, жьдати → жид, пѣти → по,
  трьти → тьр, мрѣти → мьр, сърѣсти → сърѧщ, лити → лѣ, смиꙗти → смѣ,
  клеветати (Kaikki's клевещѣтъ), роути (Kaikki's ров). Wiktionary's
  cluster iotation (блазнити → блажнѭ, съмотрити → съмощрѫ, оклоснити →
  оклошнѫ) is not modelled: Synodal has соблазню̀, and `iotate` serves
  both recensions.
- **Kaikki iotates the present participle of the -вратити family
  (finding).** вращѧщ- for вратити where the language has вратѧ, вратѧщ-
  (просѧ, ходѧщ- elsewhere in the same source); seven lexemes, about 350
  cells, left as variants rather than an alternative on every class IV
  verb.
- **A 2.0 defect (finding, fixed).** The participle blocks' delegation
  read the Synodal adjective table whatever the recension, so an OCS
  lexeme had no past participle at all; found by the exemplar test's
  любленъ, fixed in `collect`, and part of the verb recall rise.
- **The UD fit's tie-break was the table's first row (finding, fixed).**
  For a lexeme the seeding did not place, `best_fit` preferred `classes[0]`
  as if it were the seeded class; with the classes sorted by name that
  was `V:I:C`, and 922 UD verbs took it. The seeded class is now an
  explicit argument, the OCS verb table is written largest class first
  so a tie goes to the commonest class of the lemma's shape, an exemplar
  is the member with the class's commonest ending, and residue classes
  are excluded for an unplaced lexeme.
- **Polyakov writes ї before a vowel inside a stem (finding).** би́ти:
  бію́, бія́хъ, біе́нъ beside би́ти, би́ша, би́лъ. The letters layer keeps і as
  a letter (`iota`), so бити's imperfect and passive participle are
  class-level (`2-ѧхъ`, `14=ext:ен:iota`); the remaining 90 `1=бі`-type
  lines are lexemes where the present stem is the only і-stem Polyakov
  attests and the class's stem 1 still serves a form with і.
- **Synodal residue (finding).** 358 lines: the long past passive's нн/н
  is Polyakov's own variation (благовѣща́нный against V11a's н;
  оу҆ра́неный against V21n's нн), genuine present suppletion (взѧ́ти →
  возм, зва́ти → зов, и҆ма́ти → емл, вергнꙋти → верж in the -щи class of its
  doublet возврещи́), and the -нꙋти/-щи doublets whose stems 1/2/3/7 are
  those of the other infinitive. Not moved into the classes.

## 2026-09-05 — 2.1 Part 2: syncretism by underspecification — decisions and findings

- **The set is canonical, sorted by the cell's order (decision).** A
  `CellSet` is sorted and deduplicated; its first cell is what a leaf
  renders through and what `:alt` indexes. The name factors a product
  (`nom|acc|voc.sg`) and lists the rest in cell order, which is case-major
  for nouns (`nom.pl|gen.sg|acc.pl`). `parse` prefers the listed reading
  when every `|`-piece is a whole cell, so `name` checks its factored
  form reads back as itself and lists otherwise — the pronoun's bare
  `dat` (the reflexive) is the case that bites (`3.m.sg.gen|3.m.sg.dat`).
- **A leaf writes a product set as features and any other set as `:cell`
  (decision).** `:case nom|acc :num sg` over-claims nothing only when the
  set is the product of its feature values; жены-type sets (gen.sg,
  nom.pl, acc.pl) are not, and they are common, so the leaf names the
  set outright. Two spellings, one meaning; the reader accepts both
  everywhere.
- **The titlo hides more than the accent (finding).** The first census
  found 3,337 leaves "incomplete" — every one under an `(abbr …)`: дх҃ъ
  abbreviates дꙋ́хъ (nom.sg) and дꙋ̑хъ (gen.pl, acc.pl) alike, so the
  token's set is larger than the full print's. The check now asks the
  titlo index for abbreviated tokens; the leaves were right.
- **Five parts syncretism, one part homonymy (finding).** Of the 40.2%
  the 2.0 treebank called ambiguous, 34.0% is one lexeme in several
  cells and 6.0% is several lexemes. The analyzer's own count over the
  Bible agrees (32.6% against 5.0%). The commonest sets are the
  masculine and neuter singular (nom|acc, nom|acc|voc, gen|acc of the
  animates) and the long adjective's m|n oblique cells; the largest are
  the pronouns' (всѧ̑ names sixteen cells).
- **The hand overlay narrows 179 of 283 leaves (finding).** Every hand
  cell is inside the lexicon's set (`narrow-hand`: 0 findings); the hand
  chose among nom|acc|voc.sg 493 times in Genesis alone. That choice is
  what the constraint-based disambiguation of `docs/OPEN-DESIGNS.md` 1b
  has to reproduce; Genesis 1 is its gold.

## 2026-09-05 — 2.1 Part 3: the close

- Version 2.1.0, tag `v2.1.0`; docs regenerated from the final numbers.
  No lexicon or class change in this part; the game's 35 tests and the
  headless run pass against 2.1.0 unchanged.
- **What 2.1 did not do, on purpose.** The non-present verb cells of the
  OCS classes are still the data's majority (Kaikki's косехъ, кослъ
  reproduced as before); Wiktionary's cluster iotation (блажнѭ) is not
  modelled; the Synodal нн/н long participle stays a lexeme fact; a
  non-product set on a leaf is `:cell`, not a wider feature product.
  Each is written down in `docs/OPEN-DESIGNS.md` or `HANDOFF-PROMPT.md`
  with its number.

## 2026-09-05 — 2.2 Part 0: the censuses (findings)

- **The OCS non-present cells disagree with the type exactly where
  Kaikki's template is wrong.** The type prediction (aorist: sigmatic on
  a vowel stem, -ох- on a consonant stem with the palatalised velar before
  е, -нѫ- kept in class II; the imperfect -ѣа-/-аа-/-ꙗа- by type; the
  l-participle on the infinitive stem) agrees with every vowel-stem class
  in all three blocks and disagrees with every consonant-stem class in
  the aorist and l-participle: class II (косехъ, кослъ), the velars
  (стрѣжехъ, влѣчехъ), the dentals (грѧдехъ, гнестхъ), the nasals
  (кльнхъ, кльнлъ), and V:I:C's imperfect (понесахъ). 501 OCS verb lines
  carry UD variants in these blocks (674 aorist forms): the corpus has
  the right forms and the tables do not.
- **Two thirds of the adverbs are an adjective's cell.** 1,435 of the
  2,313 closed adverbs are printed exactly as an adjective already in the
  lexicon prints its neuter short nominative (with the wide ѡ in 857) or
  its short locative; 811 have no adjective (the primary adverbs and the
  compounds), 67 differ in accent or letters.
- **The preposition frames from the treebank are the grammar's**, with
  the syncretic sets as a separate count that a disambiguator will
  resolve (въ: loc 2,488 and acc 1,490 unambiguous, acc|nom|voc 1,875 the
  masculine inanimate's set).
- **Solid enclitics are a small, closed set of hosts.** 524 tokens
  analyse only as host + enclitic once the host's final varia is read as
  an oxia or its jer is restored: и҆̀хже (270), ѻ҆́ньже (147), во́ньже,
  на́ньже, за́ньже, Землѧ́же. The 3,954 "neither" are ѹ҆̀бо (766, a
  spelling of ꙋ҆́бо with the ligature ѹ) and ordinary words ending in the
  letters of -сѧ/-ти that the lexicon lacks; -же/-бо/-ли leave 95/774/82,
  and негѡ́же (63) is него + же, a form the personal pronoun prints only
  after a preposition.
- **Homonymy is mostly across parts of speech.** 19,402 of 37,647 `:amb`
  tokens read as several parts of speech; the commonest surfaces are
  lexicon duplicates or near-duplicates the census exposes (гдⷭ҇ь 3,579,
  а҆́зъ 1,939, ва́мъ 1,358, мнѣ̀ 1,260): a lexicon cleaning precedes any
  disambiguator.
- **The stress exception lists are not paradigms.** 1,870 Synodal lines
  carry a list, in 1,099 shapes; the twelve commonest shapes per part of
  speech absorb 145 of 395 noun lists, 37 of 242 adjective lists, 242 of
  1,202 verb lists. The noun shapes are paradigms (a{pl=E}, a{gen.pl=E},
  b{voc.sg=S}, b{acc.sg=S} the рꙋка̀ retraction); the verb shapes are
  Polyakov's per-lexeme noise (b{pres.3.sg=S} on 43 reflexives). Part 6
  will name the noun and adjective paradigms and leave the verb lists.

## 2026-09-05 — 2.2 Part 1: the non-present verb cells by type — decisions and findings

- **The type's cell is the primary and Kaikki's majority is not an
  alternative (decision).** Keeping косехъ beside коснѫхъ "for
  reproduction" would put a form the language never had into the
  analyzer; the 1,105 Kaikki cells that disagree with the type are
  counted in the CHANGELOG as what they are. The Kaikki number is a
  reproduction number and it went down; the corpus numbers went up.
- **The root aorist of class II is an alternative on a palatalised stem
  (decision).** въздвиже and въздвигошꙙ in the UD data are the root
  aorist beside въздвигнѫ; the class names `13=pal1` and offers
  `1-нѫ|13-е`, `1-нѫхъ|1-охъ`. The census predictor reports the class's
  primary.
- **What the UD aorist variants are now (finding).** Of the 644 left,
  the classes reproduce the form under the manuscript fold in the
  recall (the number rose); the variants are spellings the fold does not
  cover — ѣ for ꙗ (ѣвихъ), ъи for ꙑ (бъихъ), ꙙ for ѧ, ръ for рь
  (въвръгохъ) — plus бꙑти's athematic aorist in an aje class and вести's
  old sigmatic вѣсѧ. They are not paradigm gaps.
- **A ї-stem cell is one whose ending opens with a vowel or й
  (finding).** Polyakov writes бі́й, бі́йте, бїѧ́хъ, бїю́щїй and би́хъ,
  би́лъ, би́ти: the letters layer's і is the print's rule for и before a
  vowel or й, so the class can say it (`2-й`), and the 19 lines that
  stored `1=бі` — and printed their infinitive as вбітѝ, a 2.1 defect
  the consistency test did not see because it checks nouns' nominatives
  only — are gone. A citation-cell check for verbs belongs in that test.

