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
