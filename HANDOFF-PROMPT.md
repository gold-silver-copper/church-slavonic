# Handoff: the church-slavonic program as of 2026-09-01 (after v1.2)

Read this first in a fresh session. It is the map of what exists, what
is true, and what the next wave should be — so work starts from verdicts
and recorded numbers, never from re-derivation. The diary (NOTES.md) and
the executed prompts (V1.1-PROMPT.md, SYNTAX-PROMPT.md,
SYNTAX2-PROMPT.md, V1.2-PROMPT.md, and vertograd's PHASE*-PROMPT.md)
hold the full
detail; every one carries a dated execution postscript.

## The two repositories

- `~/Desktop/code/church-slavonic` — the language library (this repo,
  pushed to github.com/gold-silver-copper/church-slavonic). Workspace:
  `church-slavonic` (public API + generated tables),
  `church-slavonic-core` (rules), `church-slavonic-syntax` (the
  treebank crate, two waves old), `extractor`, `xtask`. Version 1.2.0.
- `~/Desktop/code/vertograd` — the monastery game, the crate's first
  consumer (local only, no remote). Ten phases done; consumes 1.2.0 via
  `[patch.crates-io]` (since the v1.2 close, with exact-output tests for
  the pronoun forms its content uses). Its audits are the crate's defect
  intake — that loop has run three times and works.

## Standing rules (absolute, both repos)

- Crate: after every part — `cargo xtask accuracy` 100.00% / gap 0 on
  every pinned table source; `check-registry`, `check-witnesses` green;
  zero warnings; no code path hardcodes an output for a specific lemma
  (attested values enter only through the extractor from pinned
  sources); dated NOTES.md entry per wave; commit per part.
- Syntax crate: **the round-trip invariant** — render(tree) equals the
  pinned print byte-for-byte; `cargo xtask check-treebank` must report
  zero mismatches over all 34,470 verses. There is no other definition
  of correct. Ambiguity is recorded (`:amb`), never guessed.
- Game: exact-output test before any lemma enters content (print the
  crate's real output and paste it); every visible string in
  GLOSSARY.md; zero non-Church-Slavonic on screen;
  `./scripts/headless-test.sh` green after every part; every scripted
  mode must be listed in `automated()` or it opens a visible window.
- Arbitration discipline (learned twice, paid for): when a consumer's
  expectation and the crate's output disagree, check the crate against
  its pinned sources BEFORE deciding which side is wrong. Sometimes the
  audit is wrong («пожа́тъ», «вожжѝ», «неꙋстро́енъ»).

## Where the numbers stand

Treebank (whole Bible, 631,946 tokens, zero round-trip mismatches):
21.5% analyzed + 27.1% closed-class = 48.6% mechanical; 31.0%
ambiguous; 20.2% verbatim; 0.2% apparatus (v1.2 took verbatim from
29.0%: the possessives 16,518 → 27 tokens, ве́сь/всѧ́къ 7,228 → 64, и҆́же
5,391 → 135). Genesis 1 is complete in the committed hand overlay
(`data/treebank-hand/b00.sexp`) at its 85.1% ceiling, lint-clean. The
titlo layer (`data/titlo.tsv`, 21 rows) reproduces 80.9% of its
admitted families' print tokens. Accuracy: 29 recall rows at 100.00% /
0, 26 witness rows verified. Commands: `cargo xtask build-treebank`
(~3 min, gitignored `treebank/`), `check-treebank` (coverage table +
invariant), `scripts/fetch-bible.sh` (pinned source, sha256-checked,
offline-soft).

## Done since the wave-2 verdict list

- **v1.2, the Synodal pronoun program** (V1.2-PROMPT.md, seven
  commits): the Synodal npron rule + 158 rows (`core::npron_syn`), the
  reflexive and clitic cells (pronoun arity 119; `reflexive`, `clitic`,
  `reflexive_clitic` + `_sense` twins), the personal row arbitrated
  against the print, lookup invariant 5 (the print outranks the
  transliteration: `Source::letters_exact`, per-form exactness, the
  ꙗ/ѧ and monosyllable-varia folds; on pronoun rows the number mark and
  an enclitic's mark presence too), witnesses.tsv with `pronoun`/`npron`
  symbolic cells, the erok cleanup (3,762 spellings), `+же` headwords,
  the closed lexicon enumerable. Three refutations by the treebank are
  recorded in NOTES (primary-first for all POS; the mark-presence fold
  for nouns; an accentless row holding a bare key). The residue by
  class is NOTES.md's v1.2 part-5 entry.

## The next wave, in priority order

1. **The Bible-as-source design.** The Elizabethan print is already
   sha256-pinned; admitting it as an extractor source (witness-class,
   like data/witnesses.tsv was in v1.1) would arbitrate every recorded
   convention divergence at once: ї-before-vowel (тве́рдїю, но́щїю —
   the pinned sources spell і in 2,388 cells and ї in zero; since v1.2
   this class carries се́й's whole feminine/neuter: сїѧ̑ 1,080, сїѐ 675),
   the ѡ-plural (во́ды vs вѡды̀), бы́ти's «бѣ̀», and the per-cell primary
   of a lemma-keyed row (Polyakov's counts are per form). Design
   question, not a patch: decide what "accuracy against the Bible" means
   before writing code.
2. **Cheap syntax-side coverage**: the closed-class words the table
   lacks (и҆лѝ 1,027, ѹ҆̀бо 761, да́же 730, та́мѡ 671, нижѐ 607, поне́же
   492 — one attested row each), more titlo rows (гдⷭ҇а 994, нн҃ѣ 834,
   блгⷭ҇вѝ; the спⷭ҇ family), and the pronouns' prepositional н- forms
   (ни́мъ 730, немꙋ̀ 524: stored variants the lift does not invert —
   invert the variant keys or feature them).
3. **бы́ти's future and imperfect** (бꙋ́детъ 2,170, бꙋ́дꙋтъ 660, бѣ̀
   855): the 38-cell finite schema has no future block — a schema
   design, recorded since syntax wave 2.
4. **The ambiguous band (31.0%)** — syntactic disambiguation. Still
   deliberately UNDESIGNED and now the largest slice by far (the
   possessives' 13,845 homographs are its newest members); when taken
   up, it deserves its own prompt with its own invariant (a
   disambiguation that cannot be refuted is a guess). Do not smuggle it
   into another wave.
5. **The game**, when returned to: phase 10 shipped (34-book library,
   named feasts, chronicle breadth, cell-naming miss feedback,
   DUE_CAP=8, azbuka primer). The natural next vertograd move is a
   play-audit on the current crate. One old nit remains queued:
   Polyakov civil-«я» headwords are dropped at intake.

## Known honest edges (do not "fix" casually)

- The Bible JSON's Psalter is a 22-verse fragment — Ps 90 does not
  exist in it; chapter-level work uses Genesis/Luke.
- Three titlo lemmas are reconstructed citation forms the print never
  spells (і҆зра́иль, і҆зра́илевъ, і҆исꙋ́съ), marked in the tsv.
- Six deliberate ambiguities stand in the Genesis 1 hand overlay with
  reasons written in the file — a false analysis is worse than none.
- бꙋ́детъ is a schema gap (the 38-cell finite schema has no future
  block); the grave→acute enclitic accent shift («Землѧ́ же», and the
  unstressed clitic after a proparoxytone: «закленꙋ́ тѧ») is out of
  scope by design. Both recorded in NOTES.md (syntax wave 2 part 3,
  v1.2 part 5).
- The bare key of a lemma-keyed row is the lexicographic sort's choice
  among the attested rows (clean before noisy since v1.2); a per-cell
  primary needs the Bible as a source. Primary-first for every part of
  speech was tried and refuted (NOTES, v1.2 part 1) — do not retry it
  without per-cell counts.
- A transliterated source's «я» is ꙗ or ѧ only by the rule; the print
  (Alypy, a witness) decides where they differ (lookup invariant 5).

The working style that produced all of this: one prompt per wave, parts
committed separately, every claim measured and written down, defects
fixed in their proper layer, and the unattested never invented.
