# Prompt: syntax wave 2 — the coverage burn-down

Run in `~/Desktop/code/church-slavonic`. Standing rules hold, unchanged:
after EVERY part, `cargo xtask accuracy` shows 100.00% / gap 0 on every
pinned table source, `check-registry` and `check-witnesses` pass, all
suites green, zero warnings; dated NOTES.md diary entry; commit per
part. The syntax crate's own rule is absolute and inherited from wave 1:

> **The round-trip invariant.** render(tree) equals the pinned print
> byte-for-byte. `cargo xtask check-treebank` must report zero
> mismatches over all 34,470 verses after every part.

Wave 1's baseline (NOTES.md, 2026-09-01): 631,946 tokens — 17.1%
analyzed + 27.1% closed-class = 44.2% mechanical, 15.1% ambiguous,
40.5% verbatim. This wave attacks the verbatim band from its two
largest known holdings — the not-yet-inverted crate APIs and the titlo
abbreviations — and arbitrates the part-4 harvest into a clean v1.x
intake. It does NOT touch the ambiguous band: syntactic disambiguation
stays a separate future design, and nothing here may quietly guess a
reading.

Every coverage claim in this wave is MEASURED: report the whole-Bible
table after each part that changes the index, so each lever's real gain
is on record separately — never lump them.

## Part 1 — invert the rest of the crate

The crate's pronoun, non-personal-pronoun and participle APIs exist and
are simply not in the index. Bring them in, each with its tree leaf:

- **Non-personal pronouns** (`ChurchSlavonic::npron`): extend the
  `PartOfSpeech`/`lemmas` enumeration to the npron table (same
  base-key-only rule; tables untouched — this is read-only API growth,
  wave-1 precedent). Leaf: `(np* лемма :case … :num … :g …)` — pick a
  head that cannot collide with the `np` group (e.g. `pn`). Index all
  gender×number×case cells.
- **Personal pronouns** (`ChurchSlavonic::pronoun`): a closed paradigm,
  no lemma table — enumerate person×number×gender×case directly. Leaf:
  `(pers :p … :num … :case … :g …)`. The pronoun answers vocative with
  nominative — do not index a fake vocative distinction.
- **Participles** (`ChurchSlavonic::participle`): derived from the verb
  inventory. Full cross-product is 4,307 lemmas × tense × voice ×
  series × 7 × 3 × 3 ≈ millions of calls — measure the build time
  first with active-voice only, and report before deciding whether
  passive joins this wave or the next; if anything is deferred, the
  NOTES entry says so in numbers (no silent caps). Leaf:
  `(part лемма :t … :voice … :series … :case … :num … :g …)`.
- **Adjective degrees**: the index only knows the positive; add
  comparative (and superlative if the API's output proves distinct)
  cells to the existing adjective inversion.

Each leaf extends `Node`, the sexpr reader/printer round-trip, the
renderer, `tree_coverage`, and gets exact-output tests with pasted
crate output. Rebuild the treebank and record the coverage table after
EACH of the four additions — four measured deltas, not one.

Wave-1 sanity check to repeat here: ambiguity between a new reading and
an existing one (a participle homographic with an adjective, a pronoun
with a noun) must fall to `:amb` verbatim, never to whichever index
insertion ran last. Add a test asserting one such collision is recorded
as ambiguous.

## Part 2 — the titlo layer (the largest verbatim class)

Sacred abbreviations (гдⷭ҇ь, бг҃ъ, бж҃їй, дх҃ъ, сн҃ъ …) inflect: the
abbreviated STEM is constant while the ending follows the full word's
paradigm, and the print leaves the abbreviated form unaccented. That
makes the class generatable, not listable:

- A COMMITTED pinned table `data/titlo.tsv`: one row per abbreviation —
  abbreviated stem, full lemma, part of speech — plus the row's
  standalone attestation count in the pinned print (the closed-class
  discipline: nothing enters on memory). Seed it from the print itself:
  survey the distinct titlo-bearing tokens (anything containing U+0483
  or the superscript-letter combining range), sort by frequency, and
  admit the top of the list — nouns and adjectives first; record in
  NOTES how much of the titlo token mass the admitted rows cover and
  how much remains.
- Generation: for each row, take the full lemma's paradigm through the
  public API, split off the ending against the full stem, attach it to
  the abbreviated stem, STRIP THE ACCENTS, and index the results as
  analyses that carry the abbreviation. Tree form: the existing
  analyzed leaves grow an optional `:abbr <stem>` feature —
  `(n госпо́дь :case voc :num sg :abbr гдⷭ҇)` renders «гдⷭ҇и». Explicit
  and refutable, like every other feature; render, sexpr round-trip,
  lint and coverage all treat an `:abbr` leaf as analyzed.
- VERIFY the mechanism empirically before shipping it: for every
  admitted row, count how many of that abbreviation's distinct print
  tokens the generated cells reproduce exactly, and report the table in
  NOTES. A row whose generated forms disagree with the print is
  evidence about the elision rule or the paradigm — investigate, and
  if unresolved, drop the row and record why. The invariant stays the
  arbiter: a wrong guess cannot survive check-treebank anyway.
- Rebuild; record the delta. Expectation from wave 1: this is the
  single largest verbatim holding — say what it actually turned out
  to be.

## Part 3 — arbitrate the harvest (the v1.x intake, cleanly separated)

Run wave 1's part-4 wrong-form candidates through the arbitration
discipline — check each against the PINNED tables and sources (the
extractor data is on disk), exactly as v1.1 arbitrated the vertograd
audit. For each: verdict (crate right / print convention differs /
crate defect / undecidable without a new source), evidence, and — for
confirmed defects only — a v1.2 intake entry in NOTES.md:

- «тве́рдїю» (print) vs «тве́рдію» (crate): the ї-before-vowel
  question. Check what the pinned sources themselves spell in exactly
  this position before judging.
- «во́ды» nom/acc pl (print) vs «вѡды̀» (crate): the ѡ-plural
  disambiguation convention AND the accent — two separable questions;
  arbitrate them separately.
- бы́ти's «бѣ̀» and «бꙋ́детъ», the ordinals (вторы́й …),
  «неꙋстро́енъ», the grave→acute alternation before enclitics
  («Землѧ́ же»): classify each as defect, missing coverage, or
  out-of-scope-by-design, with the reasoning written down.

Fix NOTHING in the crate in this wave. The deliverable is the verdict
list — the next v1.x program starts from it instead of re-deriving.
(One exception, wave-1 precedent: a defect that turns out to live in
the SYNTAX crate — an indexing or rendering mistake — is fixed here,
in its proper layer.)

## Part 4 — Genesis 1 to the whole-chapter ceiling

With pronouns, participles and the titlo layer live, extend the
committed hand overlay from Genesis 1:1–8 to all 31 verses. Every token
analyzed, closed-class, apparatus, or verbatim-with-reason; ceiling row
reported; zero lint findings — and wire `lint` into `check-treebank`
for HAND entries (auto-lifted trees are flat and uninteresting to lint;
hand trees claim structure, so their claims get checked). Any new
crate gap the remaining 23 verses expose joins the part-3 verdict list
under the same discipline.

## Part 5 — verification and close

- `check-treebank` zero mismatches over all 34,470 verses; the hand
  overlay lints clean; full workspace suites, accuracy 100.00%/0
  untouched, check-registry, check-witnesses, zero warnings, and the
  wave-1 tests all still green (the new leaves must not have bent the
  old ones).
- README: coverage table regenerated from the real final run; one
  sentence on the titlo layer joining the accuracy story.
- NOTES.md diary close: the four measured deltas of part 1, the titlo
  coverage table of part 2, the verdict list of part 3, the Genesis
  ceiling of part 4, and what remains of the verbatim band by class —
  the next wave's map. The ambiguous band's disambiguation design
  stays explicitly deferred.

The finish line: the coverage table shows how far the mechanical frontier
moved — measured lever by lever — Genesis 1 stands complete at its
ceiling, and the next crate program's intake is a verdict list, not a
pile of suspicions.

---

## Execution postscript (2026-09-01)

Five parts, five commits, every gate green throughout; the invariant
held at zero mismatches over all 34,470 verses after every part.
Findings and deviations of record:

- **Part 1**: the npron leg contributed ZERO — Synodal npron is empty
  in the crate (no syn: rows, an empty-string rule); recorded as the
  intake's highest-value item, the leaf refusing to render empty. The
  pronoun leg was the lever (~29k tokens); participles landed mostly in
  :amb (homographic with adjectives — honest); passive was measured
  separately (−0.6pt verbatim, ~65s build) and admitted. Degrees were
  marginal and kept.
- **Part 2**: the titlo layer reproduced 80.9% of the admitted
  families' token mass (68.6% of all 27,986 abbreviation tokens); weak
  rows (бж҃'s rule-spelling drift, спⷭ҇'s multi-lemma family) kept for
  the record, three reconstructed citation forms marked in the tsv.
  Полѧkov attests some abbreviated headwords directly (бг҃ъ is a table
  lemma), so the auto-lift sees both readings — :amb, as designed.
- **Part 3**: the arbitration flipped one verdict — «неꙋстро́енъ» was
  never missing; THIS program's lemmas() enumeration hid every
  sole-`_n` lemma (the ко́локолъ_2 analogue). Fixed in its proper
  layer: ~2,000 lemmas surfaced, verbatim −3.0pt from that alone.
  тве́рдїю and во́ды are convention divergence (the sources never spell
  ї before a vowel: 0 vs 2,388 cells) — the Bible-as-source design
  question is the clean resolution, deferred to v1.x.
- **Part 4**: Genesis 1 complete, 79.4% ceiling, lint wired into
  check-treebank for hand entries; six deliberate ambiguities kept
  with reasons (a false analysis is worse than none).
- **Net**: verbatim 40.5% → 29.0%; mechanical 44.2% → 47.1%; ambiguous
  15.1% → 23.7% — the disambiguation design's measured future workload.
