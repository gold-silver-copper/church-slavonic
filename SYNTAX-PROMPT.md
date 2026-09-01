# Prompt: the syntax crate — trees that round-trip the Bible

Run in `~/Desktop/code/church-slavonic`. This repo's standing rules hold:
after EVERY part, `cargo xtask accuracy` shows 100.00% / variant gap 0 on
every pinned table source, `cargo xtask check-registry` and
`check-witnesses` pass, all suites green, zero warnings; no code path
hardcodes an output string for a specific lemma — attested values enter
only through the extractor from a pinned source; dated NOTES.md diary
entry per wave; commit per part. The new crate adds its own standing rule,
stated now and never weakened:

> **The round-trip invariant.** For every verse that has a tree,
> `render(tree)` equals the pinned print byte-for-byte — every accent,
> titlo, variant marker and punctuation mark. There is no other
> definition of "correct".

The headline is NOT "generates proper sentences". Free generation is what
the invariant EARNS: a renderer that reproduces 34,470 attested verses
can be trusted with a new tree. A generator without a round-trip target
emits plausible-but-wrong Church Slavonic and nobody ever knows.

## What is being built

A new workspace member `crates/church-slavonic-syntax`, a consumer of
`church-slavonic` exactly as vertograd is: it owns NO morphology. It
owns: an S-expression parser (hand-rolled, zero deps — match the repo's
instinct; sexprs are a small parser), a `Node` tree model, `render()`,
`lint()`, an inverse index for auto-lifting, and the treebank pipeline.
The Bible trees themselves are DATA, not crate contents: derived from the
pinned text, regenerable, license-honest (gitignored like vertograd's
fetched sources; the pipeline and the check are what's committed).

The tree language, fixed points:

- **Ordered children.** Church Slavonic word order is free and
  meaningful; the tree RECORDS order, it never derives it. Rendering is
  a left-to-right walk plus a spacing/punctuation rule.
- **Analyzed leaves** carry lemma + explicit features and inflect
  through the existing public API (`ChurchSlavonic::noun/adj/verb/
  l_participle/participle/pronoun`):
  `(n гдⷭ҇ь :case nom :num sg)`,
  `(v рещѝ :t aor :p 3 :num sg)`,
  `(adj вели́кїй :case acc :num sg :g m)`.
- **Features are explicit and CHECKED, never inferred.** No "the
  adjective inherits case from its head". For an annotation of attested
  text the tree must be refutable: `lint()` flags adj≠noun disagreement
  within an NP, verb≠subject person/number, preposition-case mismatch.
  Explicit-plus-checked beats inferred, because inference hides
  annotation errors.
- **Closed-class table**: prepositions, conjunctions, particles, the
  enclitics (же, бо, ли, ꙋ҆бо, …) — hand-written, uninflected, so the
  linter can tell a function word from an unanalyzed one.
- **The verbatim leaf — the load-bearing decision.** `(w "гдⷭ҇ь")` is a
  witnessed surface form, rendered as-is; it may carry belief
  annotations (`:lemma`, `:case`) that the renderer ignores and the
  linter may use. Every verse can be treed TODAY (tokenize, wrap all in
  `w`, round-trip holds trivially); progress is LIFTING `w` nodes to
  analyzed nodes, which succeeds only when the crate's output matches
  the attested surface exactly. Analyzed-token coverage is the program's
  burn-down number, and a lift that fails where the crate SHOULD agree
  is a consumer-found crate defect — it feeds the same wave machinery
  as the vertograd audit, in the crate's proper layers, never patched
  in the syntax crate.
- Punctuation, verse numbers, footnote/variant apparatus are verbatim
  tokens or serializer concerns. Never grammar.

The source is the one already found, validated and PINNED (vertograd
PHASE4-PROMPT.md — reuse verbatim, do not re-research): repo
`asdf-a11/ChurchSlavonicBibleInUtf8`, file `CSlElizabeth-CS.json`,
sha256 `de40ffb4457c2d61f1330eff631496091ad69046efa08781326cdf733e28dc1e`
(12,763,661 bytes), 77 books / 34,470 verses. Known pitfalls, already
paid for: `꙾…꙾` variant markers and bracketed footnote numbers `[26]`
inside verse text (Luke 15:12); the round-trip target is the verse AS
PRINTED, so the tokenizer must carry the apparatus through, not strip
it. A `scripts/fetch-bible.sh` (or xtask subcommand) downloads to a
gitignored dir, verifies the sha256, and everything degrades soft
offline.

## Part 0 — plumbing proved with zero linguistics

The sexpr parser (atoms, strings, keywords, nested lists; precise error
positions), the `Node` model, `render()` for verbatim-and-punctuation
trees only, and the tokenizer. Gate: tokenize one pinned Gospel chapter,
wrap every token in `(w …)`, render, and assert byte-equality with the
print — one test per known pitfall verse (Luke 15:12 included). Parser
round-trips its own output (`parse(print(tree)) == tree`). Commit before
any morphology.

## Part 1 — the renderer meets the crate

Analyzed leaves render through the public API; the closed-class table
enters (each member either crate-verified or a verbatim witness — the
vertograd rule); the spacing rule handles punctuation tokens. Exact-
output tests: a handful of hand-written trees whose rendered sentences
are asserted against pasted crate output, plus one real half-verse
hand-lifted and byte-checked against the print. Recension is a render
parameter (Synodal now; OCS costs nothing to leave open).

## Part 2 — the linter

`lint(tree)` returns findings, never panics: NP-internal disagreement,
verb↔subject disagreement, preposition-case mismatch (table-driven,
starting small and honest — only rules that are actually reliable),
unknown atom heads, an analyzed leaf whose rendered form fails to match
a sibling `:expect` annotation. Unit tests per finding kind, plus one
clean tree asserting zero findings.

## Part 3 — the inverse index and auto-lift

Invert the generator: for every lemma the crate knows, generate its
paradigm into a surface→(lemma, features) index (vertograd's
`known_form_parses` is the sketch; this one is exhaustive over the
crate's lemma inventory and lives on the syntax-crate side). Then the
pipeline: for each verse, tokenize → wrap verbatim → lift every token
whose surface matches the index UNAMBIGUOUSLY; ambiguous matches (acc=
nom and friends) are recorded as ambiguous, kept verbatim, never
guessed. One `.sexp` file per book; `cargo xtask check-treebank`
re-renders every tree against the pinned text (the invariant, enforced
in bulk) and prints the coverage table: per book, tokens total /
analyzed / ambiguous / verbatim. No silent caps — if a book is skipped
or truncated, the table says so. Run it over at least the Gospels in
this part; the whole Bible if runtime allows (report the timing either
way). Record the first-run coverage numbers in NOTES.md — this is the
program's baseline, the analogue of the first gold-gap measurement.

## Part 4 — one chapter lifted to the ceiling, defects harvested

Pick one short, well-loved chapter (Ps 90 is a fine candidate — the
game's Sixth Hour already carries it) and hand-lift it as far as honesty
allows: every token either analyzed, closed-class, apparatus, or
verbatim-with-reason. Deliverables: the chapter's coverage row at its
ceiling; a dated NOTES.md list of every crate gap the lifting exposed,
each classified (missing lemma / missing form / wrong form / genuinely
outside scope — e.g. imperfects or duals the tables don't carry), with
wrong-form entries verified against the pinned tables before being
called defects (the v1.1 arbitration lesson: sometimes the audit is
wrong). Do NOT fix crate defects in this wave — harvest them; they are
the next v1.x program's intake.

## Part 5 — verification and close

- `check-treebank` wired into the repo's standard verification path,
  offline-soft.
- Full-workspace suites green, `cargo xtask accuracy` untouched at
  100.00% / gap 0 (this wave changes no tables), zero warnings.
- README gains the syntax crate section: the invariant stated first,
  the coverage table reproduced from the real run, the escape-hatch
  philosophy in two sentences.
- NOTES.md diary close: baseline coverage, the defect harvest, and the
  burn-down shape for future waves (lift coverage up; each wave's
  failed lifts feed a crate wave; disambiguation of acc=nom by
  syntactic context is a LATER, separate design — do not smuggle it in
  here).

The finish line: `cargo xtask check-treebank` walks scripture and
reports, book by book, how much of the Bible the machine truly
understands — and every verse it touches comes back letter-perfect,
because the tree was never allowed to say more than the print attests.

---

## Execution postscript (2026-09-01)

All six parts ran the same day, one commit each (parts 0–5), every
standing gate green throughout. Deviations and findings of record:

- **Part 0**: the round-trip target is the verse text TRIMMED of the
  JSON arrangement's cosmetic leading space (33,259 of 34,470 verses
  carry one); the source holds zero interior double spaces, so
  single-space token joining is byte-exact. The gate ran over Genesis 1
  plus all of Luke — 1,181 verses.
- **Part 1**: a `(cap …)` wrapper entered the tree language —
  sentence-initial capitals otherwise force every first word to
  verbatim. Genesis 1:1 lifted and matched the print first try.
- **Part 3**: the lemma inventory needed a small public API on the main
  crate (`ChurchSlavonic::lemmas`, read-only, base keys only) — code,
  not tables; accuracy closed at 100.00%/0 untouched. The invariant
  caught two print oddities on its first bulk run: the free-standing
  period of 4 Kings 17:3 (a lone punctuation token keeps its own
  space) and the «(,*…» cluster of Proverbs 15:33 — answered by
  structure (the lift verifies its own reconstruction and falls back
  to whole-token verbatim), not by special cases. Whole-Bible build
  ~10 s; zero round-trip mismatches across 34,470 stored trees.
- **Part 4**: Ps 90 was unavailable — this JSON's Psalter is a known
  22-verse fragment — so the hand-lift chapter is Genesis 1 (verses
  1–8, committed in `data/treebank-hand/`, 76.7% lifted, zero
  ambiguous). The harvest is in NOTES.md, classified; wrong-form
  candidates still owe the arbitration check before any v1.x wave
  treats them as defects.
- **Baseline** (after the part-4 pipeline gains): 17.1% analyzed +
  27.1% closed-class = 44.2% of scripture lifts mechanically; 15.1%
  ambiguous, recorded and never guessed; 40.5% verbatim — the
  frontier, led by titlo abbreviations and the not-yet-inverted
  pronoun/participle APIs.
