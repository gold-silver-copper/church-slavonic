# Prompt: church-slavonic 2.0 — the lexicon-first rewrite

Run in `~/Desktop/code/church-slavonic`. This prompt SUPERSEDES
HANDOFF-PROMPT.md, V1.3-PROMPT.md and the standing rules they carry. Do
not read them for rules; read them only when a part below sends you to a
specific number or file. The long-form design is the artifact
"Church Slavonic v2 Design" (2026-09-04); everything a session needs from
it is restated here so this file is sufficient on its own.

The occasion: a root-cause review (2026-09-04, at e2f24f9) found that
every recurring cost of the 1.x program — waves that take days, defects
that are about the machinery rather than the language, tables that
regenerate and keys that renumber on every rule change, an accuracy gate
that reads 100.00% by construction — follows from ONE design decision:
the tables are defined as the sources minus the rule engine. This prompt
replaces that design. Backward compatibility with 1.x is not a goal.
Version 2.0.0 is the finish line.

## The diagnosis (verified 2026-09-04; trust it over re-derivation)

- **[verified] The tables are a diff, and the diff is computed three
  times.** `extractor::extract::finalize` (3,824-line file; ~280-line
  function) runs the rule, subtracts, infers fact cells, re-subtracts,
  shadows `_n` rows against the bare row, and re-infers per candidate;
  the runtime facade replays the same ladder (own cell, bare cell, facts
  own-else-bare, rule) through `core::resolution`; `check-registry`
  audits it a third time. Every 1.1 and 1.2 defect (стрищѝ, дои́ти,
  ѻ҆гꙋре́цъ, всякую, га́ды/гадѡ́въ) is a defect of this machinery.
- **[verified] The rule engine has no lexical knowledge.** `noun_class`
  picks a declension from the lemma's last letters. So the commonest
  words are the "exceptions": `syn:ра́бъ` stores 19 cells plus rows
  `_2` and `_3`; `syn:гра́дъ` has three rows. 3,701 of 8,313 noun rows
  are `_n` variants; 3,893 store one cell.
- **[verified] The lexical facts exist in the sources and are thrown
  away.** Polyakov tags every lexeme with gender (`m`/`f`/`n`), animacy
  (`anim`/`inan`), aspect (`pf`/`ipf`), transitivity, and a paradigm
  class from `flexslav.htm` (181 codes over 31,225 S/A/V lexemes; N1t
  3,418, N2i 3,100, A1t\* 3,094, V11a 2,915 …). The pipeline parses the
  class into `polyakov::Entry::class` and uses it only for a printed
  agreement count.
- **[verified] Source noise is stored as truth.** 1,033 OCS noun rows
  store a nominative singular that differs from the key
  (`ocs:алъканиѥ` cell 0 = «алъкаиѥ», a Wiktionary typo).
- **[recorded] The stored mass is lexical.** The v0.9 close (NOTES,
  memory) judged the remaining rows "irreducibly lexical at current
  source coverage". A lexicon is the data structure for lexical facts;
  1.x has nowhere to put one.
- **[verified] The metric cannot fail.** 100.00%/0 on every source is
  true by construction. The informative numbers are held-out UD PROIEL
  dev+test recall (nouns 92.04%, adjectives 83.82%, verbs 85.58%,
  pronouns 99.25%, npron 93.21%) and the Bible treebank (analyzed 21.5%,
  closed-class 27.1%, ambiguous 31.0%, verbatim 20.2% of 631,946
  tokens). These become the gates.
- **Not a problem:** build time (release build of the library 5.3 s),
  runtime speed, crate size.

## The design (the contract; every part implements a piece of it)

A form is produced by four independent stages, each owning one kind of
knowledge:

1. **Lexeme** — from the lexicon (or the guesser for OOV input): id,
   lemma, POS, gender, animacy, letter CLASS, STRESS paradigm, stem
   alternations the class cannot derive, explicit per-cell overrides,
   variants, provenance.
2. **Letters** — the class table gives, per cell, an ending and a stem
   selector (fleeting vowel dropped, velar palatalised, iotated present
   stem, …). Output: the recension's canonical letters, no marks, no
   wide letters unless lexical, plus one bit: `number_mark` (the print
   distinguishes this cell from a look-alike singular; Alypy §6).
3. **Stress** — the stress paradigm gives, per cell, a vowel index or
   "the ending" (falling back to the last stem vowel when the ending has
   no vowel). Output: `Option<u8>` (None for OCS and titlo lemmas).
4. **Typography** — `Form::print(recension)`: one pure function. Order:
   wide ѡ/є or kamora on a number-marked cell; oxia inside, varia on a
   final vowel; the monosyllable's varia; psili on an initial vowel;
   initial ѻ/є; ї for an unstressed non-initial і before a vowel or й
   (the v1.3 finding — it reads the stress, so stress is placed first).
   OCS printing drops the stress and maps the alphabet.

There is NO fallback ladder. A lexeme is complete by construction. The
only string a consumer sees is the output of stage 4, and the analyzer
reads it back to stage 1 plus the cell.

```rust
pub struct Form { letters: String, stress: Option<u8>, number_mark: bool }
impl Form { pub fn print(&self, r: Recension) -> String; pub fn key(&self) -> String }
```

### The lexicon

Committed, human-readable, the single source of truth. Import writes to
it, people edit it, the library embeds it with `include_str!`. Nothing
generates it.

```
lexicon/syn/{nouns,adjectives,verbs,pronouns,closed}.tsv
lexicon/ocs/{nouns,adjectives,verbs,pronouns}.tsv
lexicon/quarantine.tsv      # source entries judged noise, WITH the reason
lexicon/classes/{noun,adj,verb,pronoun}.toml   # the class tables (stage 2)
lexicon/stress.toml                            # the stress paradigms (stage 3)
```

Columns, tab-separated, `-` for empty, `#` comments allowed, one lexeme
per line, sorted by id:

```
id  lemma  pos  gender  anim  class  stress  stems  overrides  variants  src  note
рабъ.n     ра́бъ    n  m  anim  N1t   b   -        -                         gen.pl=рабѡ́въ          P:N1t;A:§12   -
ѻтецъ.n    ѻ҆те́цъ  n  m  anim  N1c*  b   obl=ѻтц  voc.sg=ѻ҆́тче              -                     P:N1c*;W:Lk15:12  -
сынъ.n     сы́нъ    n  m  anim  N1t   c   -        dat.sg=сы́нови;nom.pl=сы́нове  gen.pl=сынѡ́въ    P:N1t;A:§14   u-stem relics
сꙑнъ.n.2   сꙑнъ    n  m  anim  u     -   -        -                         -                     K:сꙑнъ#2      -
```

- **id** = the lemma in canonical letters with marks stripped, `.pos`,
  and `.n` ONLY for a true homograph (different gender, class or gloss).
  Assigned once at import; re-import matches an existing entry by
  lemma+pos+class before it creates an id; ids are never reassigned or
  renumbered. A consumer may persist them.
- **class** names a row of `lexicon/classes/*.toml`; **stress** names a
  paradigm of `lexicon/stress.toml` or spells one inline (grammar below).
- **stems** = named stems the class cannot derive from the lemma
  (`obl=`, `pres=`, `aor=`, `pap=` …), in canonical letters.
- **overrides** = `cell=printform;…` — full print forms for cells where
  class+stress are WRONG. Each is a claim the eval checks.
- **variants** = `cell=printform|printform;…` — additional attested
  forms for a cell, indexed by the analyzer, never returned by
  `inflect`. Spelling variants, source disagreements, minority stresses
  live HERE, on the lexeme, never as another lexeme.
- **src** = provenance tokens: `P:<class>` Polyakov, `A:§<n>` Alypy,
  `R:` ru.wiktionary, `K:` Kaikki, `U:` UD train, `W:<book><ch>:<v>` a
  witnessed Bible line, `H:` a hand edit (must be accompanied by `W:` or
  a note). Import never touches a column of an entry carrying `H:`.

Cell names: nouns `nom|gen|dat|acc|ins|loc|voc . sg|du|pl`; adjectives
`pos|comp . m|f|n . sg|du|pl . case` with `short.`/`long.` prefix where
the class has both series; verbs `pres|impf|aor|impv . 1|2|3 . sg|du|pl`,
`inf`, `lpart . m|f|n . sg|du|pl`, participles
`pap|pdp|ppp|pdpp . short|long . gender . number . case` (present/past ×
active/passive); pronouns per the personal matrix, `refl.case`,
`clit.<cell>`. A verb class declares WHICH cells it has (бы́ти declares a
future block; the 1.x global arity of 558 is gone).

Stress column grammar (Synodal only; `-` for OCS):

```
a0 | a1 | a2      fixed on stem vowel N
b                 on the ending wherever it has a vowel, else last stem vowel
c, d, …           named mobile paradigms, DEFINED BY PART 1's clustering, in stress.toml
<name>{cell=N|e;…}   a named paradigm with per-cell exceptions (N = vowel index, e = ending)
{cell=…}          purely per-cell
```

The mark kind (oxia/varia/kamora) is never stored; stage 4 decides it.

### Standing rules (absolute, from Part 0 onward)

- Rules and data are INDEPENDENT. A change to a class table, the stress
  module or `print` never edits a lexicon line; a lexicon edit never
  requires a regeneration. If you find yourself writing code that
  subtracts a source against a rule to decide what to store, stop: that
  is the 1.x design. The lexicon stores lexical facts (class, stress,
  stems, overrides), not diffs.
- No sense numbering by sort. No `_n` keys. No fact cells inside form
  rows. No per-source equality relations in the library (the library has
  ONE equality: `Form::key`). Transliteration conventions are absorbed
  at IMPORT by inverting `print`, once.
- Curation is allowed and recorded (`H:` + `W:`/note). The 1.x rule that
  values enter only through the extractor is retired.
- After every part: `cargo test --workspace` green, zero warnings,
  `cargo xtask eval` run and its three numbers pasted into the part's
  CHANGELOG entry. The gates below are on THOSE numbers. There is no
  100%/0 gate.
- Commit per part. CHANGELOG.md entry per part with the eval numbers.
  `docs/DESIGN.md` is the design record (Part 0 writes it); NOTES.md
  receives a short dated entry only for a DECISION or a REFUTATION, not
  a diary. HANDOFF-PROMPT.md is rewritten once, at Part 5.
- The Bible round-trip invariant stays: render(tree) equals the pinned
  print byte-for-byte for every verse; ambiguity is recorded, never
  guessed.

## Part 0 — Freeze, scaffold, baselines

1. Tag `v1.2.0-final` at the current HEAD. Commit V1.3-PROMPT.md as
   superseded (leave it; add one line at its top saying so).
2. `git mv crates legacy`. Rename the legacy packages
   (`church-slavonic` → `church-slavonic-legacy`, and the same suffix for
   `-core`, `-syntax`, `extractor`, `xtask`); fix their internal path
   deps; workspace members become `legacy/*` plus the new `crates/*`.
   The legacy tree must keep building and `cargo run -p
   xtask-legacy -- accuracy` / `check-treebank` must keep working until
   Part 5 — they are the baseline instruments.
3. Create `crates/church-slavonic` (the library; dependency
   `unicode-normalization` only) and `crates/church-slavonic-tools`
   (import, eval, treebank; serde/serde_json allowed) with a `cargo
   xtask` alias in `.cargo/config.toml` pointing at the tools binary.
   Module skeleton for the library: `grammar`, `orthography`, `form`,
   `lexicon`, `paradigm::{noun,adj,verb,pronoun}`, `stress`, `inflect`,
   `analyze`, `guess`.
4. Port UNCHANGED: `legacy/church-slavonic-core/src/orthography.rs`
   (Unit model, `realise`, `comparison_key`, `stress`, `units`/`join`)
   into `orthography`; `grammar.rs` enums into `grammar` plus the typed
   cell structs (`NounCell { case, number }`, `AdjCell`, `VerbCell`,
   `ParticipleCell`, `PronounCell`, each with `fn name() -> String` and
   `fn parse(&str)` matching the cell grammar above). Port the SOURCE
   PARSERS from the legacy extractor (`polyakov.rs`, `alypy.rs`,
   `kaikki.rs`, `ruwiktionary.rs`, `treebank.rs` loaders) into the tools
   crate as `sources::*` — parsers only, not `extract.rs`, `assign.rs`,
   `cells.rs`, `checks.rs`, `bootstrap.rs`. Port the syntax crate's
   `sexpr.rs`, `node.rs`, `lint.rs`, `bible.rs`, `closed.rs`, `titlo.rs`
   into `tools::treebank` with the leaf grammar extended to carry a
   lexeme id (`(n землѧ.n :case acc :num sg)`); the old lemma-keyed
   leaves stay parseable until Part 2 re-lifts.
5. Write `docs/DESIGN.md` from the design section above (expand, do not
   paraphrase away the rules). Write `cargo xtask eval` as a skeleton
   that prints the three numbers as `n/a` and, in a `--legacy` mode,
   prints the 1.2 baselines by calling the legacy harness. Record the
   baselines in CHANGELOG under "2.0.0 Part 0".

Gate: both trees build; `cargo xtask eval --legacy` prints the numbers
of the diagnosis; docs/DESIGN.md committed.

## Part 1 — Synodal nouns (the design proved or refuted here)

Nouns first because they have the richest lexical structure, the
noisiest 1.x history and the clearest source classes.

1. **Class tables.** `lexicon/classes/noun.toml`: one entry per Polyakov
   noun code (`N1t`, `N1t*`, `N1j`, `N1k`, `N1c*`, `N1a`, `N1i`, `N1e`,
   `N1in`, `N2*`, `N3*`, `N4*`, …, read from
   `references/downloads/polyakov/flexslav.htm`), each listing per cell
   an ending and a stem selector, per recension where OCS differs. Seed
   the endings from the legacy `Row` consts in
   `legacy/church-slavonic-core/src/noun.rs` and from Alypy's tables;
   the legend's own exemplars (`раб-ъ`, `осел-ъ` with the fleeting `2ъ`,
   `отроц-ѣ`) are the unit tests. Stem selectors are a small closed enum
   in `paradigm::noun`, not free code.
2. **Importer.** `cargo xtask import polyakov --pos noun` → a diff
   against `lexicon/syn/nouns.tsv`. Per entry: class = Polyakov's code
   (identity map); gender/animacy from the tags (`m`/`f`/`n`, `anim`/
   `inan`; `m/f`, `anim/inan` and `pl` (plurale tantum) get a `note`);
   invert `print` on every form (civil «я» → ѧ/ꙗ by position, one acute
   → a stress index, ї → і, erok/titlo spellings skipped as in 1.2);
   fit the stress paradigm; whatever class+stress reproduce is dropped;
   the rest becomes `overrides` (the primary by Polyakov's count) or
   `variants`. Then `import alypy --pos noun` and `import ruwiktionary
   --pos noun` merge INTO existing entries (matching lemma+pos+class),
   adding `src` tokens and variants; a print-exact Alypy form beats a
   transliterated primary in the same cell (1.x invariant 5, now an
   import rule). Suspects → `quarantine.tsv` with a reason: nominative ≠
   lemma, no stress in an accented source, a paradigm that fits no class
   above a threshold you choose and record.
3. **Stress inventory — measured, not assumed.** Over the imported
   forms, per class, cluster the stress-position vector across cells;
   name the recurring shapes (`a<N>`, `b`, then `c`, `d` … as found) in
   `lexicon/stress.toml` with a one-line gloss and the count of lexemes
   each covers; the residue is written per-cell in the stress column.
   Record the table (shape → lexemes covered) in NOTES as a DECISION.
   The v0.8 rejection of mobile tokens was measured under the ending
   heuristic; this measurement is under the class prior. Report both
   numbers side by side.
4. **Library.** `Lexicon::synodal()` parses the tsv on first use;
   `lexeme.inflect(NounCell)` runs stages 2–4; `lexeme.paradigm()`
   iterates every cell; `Lexicon::find(lemma, Pos)` is accent-tolerant.
   `guess(lemma, Pos::Noun)` is the legacy `noun_class` heuristic
   producing a provisional `Lexeme { provenance: Guessed, .. }`.
5. **Lexicon consistency test** (a unit test, not a metric): every
   override and variant is reproduced when asked for; every Polyakov
   noun form is reproduced, a variant, or quarantined. It prints the
   count of unaccounted forms and fails only if the count grows past
   the committed number.

Gate: the engine reproduces ≥ 99% of Polyakov's noun forms with
`overrides` on < 5% of entries; `syn:ра́бъ` is ONE line with class
`N1t`, stress `b`, and at most one variant; the stress inventory table is
in NOTES; `guess` leave-one-out accuracy on nouns is measured and
recorded (first number of its kind — no threshold yet).

## Part 2 — The analyzer, the eval, the treebank

1. **Analyzer.** `Lexicon::analyze(surface) -> Vec<Analysis { lexeme,
   cell, exact: bool, variant: bool }>`: index = every lexeme × every
   cell (variants included) → `(Form::key, id, cell, exact print)`,
   sorted; built lazily on first call; input folded by `Form::key`;
   ranked exact-print first, primaries before variants. Ambiguity is
   returned, never resolved. Target: index build < 1 s in release for
   the noun lexicon; a query is a binary search.
2. **`cargo xtask eval`** prints three numbers, each of which can go
   down: (a) held-out recall on UD PROIEL dev+test (and Syntacticus,
   reported separately) — the share of annotated tokens whose form the
   lexicon+engine produce for the annotated lemma and cell; (b) Bible
   coverage — the share of the pinned Bible's tokens `analyze` resolves,
   split unambiguous / ambiguous / none, computed with the treebank
   loader; (c) guesser accuracy — hide each lexeme in turn, `guess` it
   from its lemma, compare paradigms. Per POS, per recension.
3. **Treebank.** Rewire `build-treebank` to `analyze`; leaves carry the
   lexeme id; `check-treebank` re-renders every verse and prints the
   coverage table. Re-lift the Genesis 1 hand overlay onto ids. The
   invariant holds at zero mismatches.

Gate: noun Bible coverage ≥ the 1.2 treebank's noun share (compute it
from the legacy treebank in Part 0 and record it); `build-treebank` < 10
s; zero mismatches; the three eval numbers printed for nouns.

## Part 3 — Adjectives, verbs, participles, pronouns (Synodal)

1. **Adjectives**: classes from the `A*` codes (short/long series are
   two blocks of ONE lexeme, not two lemmas as in 1.x — the class says
   which series exist); comparative and superlative as cells; the
   possessive adjectives' nominal cells as a class, not a fact.
2. **Verbs**: classes from the `V*` codes; stem selectors for the
   present, aorist, imperfect, l-participle and the four participle
   stems (the 1.x per-verb participle-stem cells become `stems=`
   entries only where the class cannot derive them); each class
   declares its cell set; `бы́ти` gets its own class with the future
   and imperfect blocks (бꙋ́детъ, бѣ̀ — 2,170 and 855 Bible tokens).
3. **Pronouns**: the personal matrix, the reflexive, the clitics and
   the closed non-personal lexicon become ~60 lexicon lines with
   overrides, from Alypy §47/§48, Polyakov APRO/SPRO and the 1.2
   witnesses (`data/witnesses.tsv` rows convert to `W:` provenance;
   the file is then deleted).
4. **Closed classes**: Polyakov's ADV, ADVPRO, CONJ, PR, PART, INTJ,
   NUM entries (≈2,600) become uninflected lexemes in
   `lexicon/syn/closed.tsv`, so the analyzer resolves them and the
   treebank's `closed.rs` table shrinks to what no source lists.
5. Importers for each POS as in Part 1; the consistency test extended.

Gate: every 1.2 accuracy source (Polyakov, Alypy, ru.wiktionary,
witnesses) is reproduced, a variant, or quarantined, with the counts;
held-out recall per POS ≥ the 1.2 baseline; Bible coverage ≥ the 1.2
treebank's 48.6% mechanical with verbatim ≤ 20.2%; zero mismatches.

## Part 4 — Old Church Slavonic

1. Importers for Kaikki (`kaikki.jsonl`) and the UD PROIEL train split
   into `lexicon/ocs/*.tsv`; class chosen as the inventory row whose
   table reproduces the most cells (record ties as a note); no stress.
   The Kaikki typo class (nominative ≠ lemma; 1,033 rows in 1.x) goes
   to quarantine by default with reason `kaikki-nom-mismatch`.
2. OCS printing in `Form::print` (drop stress; ꙑ/оу/ѫ/ѥ/ꙗ alphabet;
   the legacy `realise_ocs`).
3. The OCS ending differences live in the class tables' `ocs` column,
   ported from the legacy `Row.ocs` consts and its `regular_rules_golden`
   test (which becomes a unit test of the class tables).

Gate: UD dev+test recall ≥ the 1.2 baseline for every POS (nouns 92.04,
adjectives 83.82, verbs 85.58, pronouns 99.25, npron 93.21); Syntacticus
reported.

## Part 5 — Cutover and 2.0.0

1. Delete `legacy/` entirely, `V1.*-PROMPT.md`, `SYNTAX*-PROMPT.md`,
   `FINISH-PROMPT.md`, `ROGUELIKE-PROMPT.md`, `deprecated/`,
   `experiments/`, `data/intermediate`, `data/witnesses.tsv`,
   `data/titlo.tsv` (titlo lemmas are lexicon entries with a titlo
   lemma and `stems=`), and the `.cargo` alias's legacy paths.
2. README rewritten: what it is, the four stages in one paragraph, the
   lexicon format, the three eval numbers as the only tables, the API,
   sources and licences, how to import. No accuracy table.
3. HANDOFF-PROMPT.md rewritten once for the 2.x program (what exists,
   the numbers, the open designs: the ambiguous band, adverb
   derivation, sentence-level accent).
4. Version 2.0.0 for `church-slavonic`; publish `church-slavonic-tools`
   only if it is useful outside the repo (default: do not).
5. In `~/Desktop/code/vertograd`: migrate to the typed API and ids
   (`lexicon.get("рабъ.n").inflect(NounCell{..}).print(Synodal)`), use
   `analyze` where the game reverse-looked-up forms, re-paste any
   content string that changed from the crate's real output, GLOSSARY.md
   updated, `./scripts/headless-test.sh` green. A string the crate now
   spells differently from a pinned source page is a lexicon finding
   (fix the lexicon line with provenance), never a workaround.

Gate: the workspace is two crates; `cargo xtask eval` is the README's
tables; the game's suite is green; tag `v2.0.0`.

## What NOT to do (each was tried in 1.x and is why we are here)

- Do not store forms the engine already produces. Do not store forms
  the engine does NOT produce either, except as `overrides`/`variants`
  on a lexeme line. The lexicon holds lexical facts.
- Do not derive a lexeme's identity from a sort of its forms. Ids are
  assigned once.
- Do not add a "fact cell", a "resolution ladder", a "shadow" pass, or
  a second equality relation. If a source disagrees with the print, the
  print wins at import and the source's form is a variant.
- Do not gate on self-consistency. Gate on held-out recall, Bible
  coverage, guesser accuracy.
- Do not re-derive the diagnosis numbers; they are dated above.
- Do not smuggle in syntactic disambiguation, adverb derivation, or
  enclitic accent shifts; each is a later design with its own metric.

## Known honest edges (carry them, do not "fix" casually)

- Loanword ї before a consonant (кївѡ́тъ, вїно̀) is lexical: the lexeme's
  letters carry it when a source spells it; `print` never invents it.
- The -ѡ adverbs are closed-class lexemes, not neuter adjectives.
- The grave→acute shift before an enclitic («Землѧ́ же») is a token
  sequence rule; out of scope.
- The Psalter in the pinned Bible JSON is a 22-verse fragment.
- Three titlo lemmas are reconstructed citation forms (і҆зра́иль,
  і҆зра́илевъ, і҆исꙋ́съ); mark them in `note`.

The finish line: a fresh session can open `lexicon/syn/nouns.tsv`, read
one line, and predict every form the library will print; a rule change
is a diff to a toml file and a change in three eval numbers; a data
change is a diff to a tsv line with its provenance; the game and the
treebank consume ids that do not move; and the next wave, whatever it
is, is a lexicon diff or a class-table diff, measured by numbers that
can go down.

## Execution postscript (2026-09-05)

Executed in one session, 2026-09-04/05, Parts 0–5 in order; each part's
commit, numbers and deviations are in CHANGELOG.md under 2.0.0 and in
NOTES.md. Deviations from the plan: the Part 1 noun gate (99% / 5%) is
recorded as unmet at 94.7% (source noise, lexeme preference); the OCS
class tables were seeded from Kaikki's own paradigms rather than ported
from the legacy ending consts; `data/witnesses.tsv` lived until Part 5
because the legacy baseline instrument read it; the held-out recall is
measured under the 1.2 harness's manuscript fold so the baselines
compare. Every other gate was met.

