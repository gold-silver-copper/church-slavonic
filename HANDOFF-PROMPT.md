# Handoff: the church-slavonic program as of 2026-09-01

Read this first in a fresh session. It is the map of what exists, what
is true, and what the next wave should be — so work starts from verdicts
and recorded numbers, never from re-derivation. The diary (NOTES.md) and
the executed prompts (V1.1-PROMPT.md, SYNTAX-PROMPT.md,
SYNTAX2-PROMPT.md, and vertograd's PHASE*-PROMPT.md) hold the full
detail; every one carries a dated execution postscript.

## The two repositories

- `~/Desktop/code/church-slavonic` — the language library (this repo,
  pushed to github.com/gold-silver-copper/church-slavonic). Workspace:
  `church-slavonic` (public API + generated tables),
  `church-slavonic-core` (rules), `church-slavonic-syntax` (the
  treebank crate, two waves old), `extractor`, `xtask`. Version 1.1.0.
- `~/Desktop/code/vertograd` — the monastery game, the crate's first
  consumer (local only, no remote). Ten phases done; consumes 1.1.0 via
  `[patch.crates-io]`. Its audits are the crate's defect intake — that
  loop has run twice and works.

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
20.0% analyzed + 27.1% closed-class = 47.1% mechanical; 23.7%
ambiguous; 29.0% verbatim; 0.2% apparatus. Genesis 1 is complete in the
committed hand overlay (`data/treebank-hand/b00.sexp`) at its 79.4%
ceiling, lint-clean. The titlo layer (`data/titlo.tsv`, 21 rows)
reproduces 80.9% of its admitted families' print tokens. Commands:
`cargo xtask build-treebank` (~65s, gitignored `treebank/`),
`check-treebank` (coverage table + invariant), `scripts/fetch-bible.sh`
(pinned source, sha256-checked, offline-soft).

## The next wave, in priority order (the wave-2 verdict list)

1. **The v1.x crate program — Synodal pronouns.** The headline,
   verified: Synodal npron is EMPTY (zero `syn:` rows; the rule returns
   the empty string) — yet и҆́же/ꙗ҆́же/є҆́же, ве́сь/всѧ́къ, сво́й are
   among the commonest words of the language. Add the enclitic personal
   forms (ѧ҆̀, є҆го̀, и҆̀хъ, мѝ, тѝ …) the pronoun table doesn't
   carry. Data + rules + extractor work, in the proper layers, gated by
   accuracy as ever. The treebank's ambiguous/verbatim deltas are the
   consumer proof.
2. **The Bible-as-source design.** The Elizabethan print is already
   sha256-pinned; admitting it as an extractor source (witness-class,
   like data/witnesses.tsv was in v1.1) would arbitrate every recorded
   convention divergence at once: ї-before-vowel (тве́рдїю, но́щїю —
   the pinned sources spell і in 2,388 cells and ї in zero), the
   ѡ-plural (во́ды vs вѡды̀), бы́ти's «бѣ̀». Design question, not a
   patch: decide what "accuracy against the Bible" means before
   writing code.
3. **Cheap syntax-side coverage**: more titlo rows (блгⷭ҇вѝ →
   благослови́ти; the спⷭ҇ family's other lemmas; nominative-stem rows
   like ѻ҆ц҃ъ), one tsv line each, verified by the titlo table's
   empirical check.
4. **The ambiguous band (23.7%)** — syntactic disambiguation. Still
   deliberately UNDESIGNED. It is the largest remaining slice; when
   taken up, it deserves its own prompt with its own invariant (a
   disambiguation that cannot be refuted is a guess). Do not smuggle it
   into another wave.
5. **The game**, when returned to: phase 10 shipped (34-book library,
   named feasts, chronicle breadth, cell-naming miss feedback,
   DUE_CAP=8, azbuka primer). The natural next vertograd move is a
   play-audit on the current crate, which doubles as intake for wave 1
   of the v1.x program. One old nit remains queued: Polyakov civil-«я»
   headwords are dropped at intake.

## Known honest edges (do not "fix" casually)

- The Bible JSON's Psalter is a 22-verse fragment — Ps 90 does not
  exist in it; chapter-level work uses Genesis/Luke.
- Three titlo lemmas are reconstructed citation forms the print never
  spells (і҆зра́иль, і҆зра́илевъ, і҆исꙋ́съ), marked in the tsv.
- Six deliberate ambiguities stand in the Genesis 1 hand overlay with
  reasons written in the file — a false analysis is worse than none.
- бꙋ́детъ is a schema gap (the 38-cell finite schema has no future
  block); the grave→acute enclitic accent shift («Землѧ́ же») is out
  of scope by design. Both recorded in NOTES.md part-3 verdicts.

The working style that produced all of this: one prompt per wave, parts
committed separately, every claim measured and written down, defects
fixed in their proper layer, and the unattested never invented.
