# Prompt: make church-slavonic shaped exactly like gold-silver-copper/english

You are working in the church-slavonic workspace. Read the `english`
repository first — not a summary of it, the actual files: the root
`README.md`, `crates/english/src/lib.rs`, `crates/english-core/src/*.rs`,
`crates/extractor/src/{lib,pipeline,assign,bootstrap}.rs`,
`crates/xtask/src/main.rs`, `.github/workflows/ci.yml`, and one generated
table (`crates/english/generated/adverb_phf.rs`). Then read this repo's
current tree. This prompt asks you to make the second look like the first,
and it supersedes every earlier plan and prompt in `docs/`.

## What english is, in one paragraph — the target, verbatim in spirit

Four crates. `english` (one 24 KB `lib.rs`: table-first, rule-fallback,
case restoration, deterministic `_n` sense keys) + `english-core` (~30 KB
of suffix-list rules with no data dependency) + `extractor` (dump → filter
→ extract → number by a pure sort → emit PHF tables) + `xtask` (8 KB,
exactly three commands: `refresh-data`, `check-registry`, `accuracy`).
About 1 MB of generated tables that are *the whole artifact*: a refresh
regenerates everything from the source alone. No lockfile, no review
ledger, no override channel, no provenance records, no archive, no
holdout, no gap worklist, no immutability gate, no `docs/` directory —
the README says so explicitly and is the only documentation. CI is three
jobs: tests, `check-registry`, fmt+clippy. Experiments live in
`experiments/`, excluded from the workspace.

## What is wrong here, stated plainly so it is not repeated

This repo inverted every one of those choices and then built machinery to
manage the consequences: 11 crates, a 30k-line xtask with dozens of
commands, ~25 documents, 60+ curated TSVs, review ledgers, an immutable
archive, guard witnesses, completion inventories, and an oracle gate whose
worklist has 53,879 rows that a person or agent must "admit" one class at
a time. english has *zero* admissions: an attested form the rules do not
predict is simply extracted into the table. The bottleneck we built is the
curation layer english does not have.

## The rules of the rewrite (each is an english property; none is optional)

1. **The sources are the truth; the tables are the artifact.** A refresh
   regenerates every table from the pinned sources alone, deterministically.
   No file under `data/` is hand-edited. If a form is attested and the rules
   do not predict it, it is in the table — automatically, with no review.
2. **Rules are small suffix/ending lists, measured by accuracy**, exactly
   like `IRREGULAR_SUFFIXES`: a rule earns its place by moving the accuracy
   number; over-generalising rules are kept when the measured trade-off
   says so and commented that way.
3. **Three xtask commands.** `refresh-data`, `check-registry` (dump-free:
   keys unique and well-formed, no empty columns, rule/table layering
   holds), `accuracy` (with the sources; prints the README table). Nothing
   else. No ledgers, no gates over gates.
4. **Deterministic `_n` sense keys by a pure sort of emitted forms.** No
   identity table, no lockfile. Keys may renumber on refresh; the README
   says so.
5. **One README.** No `docs/`. What english documents in its README, we
   document in ours: architecture, accuracy table, install snippet, sense
   keys, data provenance and licence, disclaimers.
6. **CI = tests + check-registry + lint.** Nothing that needs multi-GB
   sources, no scheduled audits, no publish dry-runs, no witnesses.
7. **Experiments are excluded from the workspace**, in `experiments/`.
   Anything that is not the four crates goes there or is deleted.
8. **Process rule going forward:** no new command, ledger, gate, document,
   or crate may be added without deleting one. The tree's silhouette is a
   deliverable.

## Target tree

```
church-slavonic/
  README.md                       # the only document
  Cargo.toml                      # 4 members; experiments excluded
  .github/workflows/ci.yml        # test / check-registry / lint
  crates/
    church-slavonic/              # facade: ChurchSlavonic::noun/verb/adj/pronoun...
      src/lib.rs                  # ONE file, english's shape and size class
      generated/{noun,verb,adj,pronoun}_phf.rs   # ~1–3 MB total, the artifact
      tests/                      # a handful: paradigm matrices, rule guards, sense stability
      examples/                   # assert-based demos CI runs
    church-slavonic-core/         # rules only, no data, no deps beyond unicode-normalization
      src/{lib,grammar,noun,adj,verb,pronoun,orthography,sense_key,utils}.rs
    extractor/                    # sources → tables; owns editorial policy
    xtask/                        # refresh-data / check-registry / accuracy
  data/intermediate/              # gitignored filtered sources
  experiments/                    # analyzer, dictionary, anything else — excluded
```

## What the language forces us to add — and nothing more

english has one source and no orthographic axis; Church Slavonic has two
recensions and several sources. Keep the additions minimal and english-
shaped:

- **Recension is a grammar enum passed by reference like `&Number`.**
  `Recension::{OldChurchSlavonic, Synodal}` lives in `grammar.rs`; every
  facade call takes it (`ChurchSlavonic::noun("градъ", &Case::Genitive,
  &Number::Singular, &Recension::Synodal)`). Realisation (jers, nasals,
  accents, titla) is a rule module in core, applied on output the way
  english restores case. No scoped handles, no profiles, no identity
  layer: a lemma is a string; homographs and cross-recension doublets are
  `_n` keys assigned by the sort, per the english rule.
- **Sources.** Exactly the labelled full-form sources, each a pinned
  download with a checksum in the README table: the Wiktionary/Kaikki OCS
  dump (inflection tables, as english uses), the Alypy grammar tables, and
  — if and only if it is a labelled lemma→forms resource — the Ponomar
  2016 Church Slavonic dictionary/frequency artifacts. A raw text corpus
  (the Bible) is **not** a source for tables: english does not extract
  from prose, and neither do we. Accented Synodal coverage is whatever the
  labelled sources give; the README accuracy table reports it honestly,
  the way english reports Wiktionary recall — it is not a gap to be burned
  down by hand.
- **Accuracy = recall of attested source slots through any key, per POS
  per recension**, plus bare-key primary correctness, printed by
  `cargo xtask accuracy` as the README table. Those two tables are the
  entire progress metric of the project.

## What is deleted (all of it, in one commit each, docs to git history)

- Crates: `old-church-slavonic-core`, `synodal-church-slavonic-core`,
  `synodal-church-slavonic`, `synodal-church-slavonic-dictionary`,
  `church-slavonic-orthography` (folded into core as a rule module),
  `church-slavonic-dictionary` (moved to `experiments/analyzer` with the
  synodal analyzer, excluded from the workspace), the old extractor.
  Published names get one last release whose README points here, per the
  existing deprecation precedent; then they leave the tree.
- xtask: every command except the three. That includes the gold gate and
  its oracles, the identity/coherence gate, the projection study, the
  burn-down tooling, the completion inventories, the lexical-union
  ledger, the archive, the witnesses, the pilot accuracy replays (their
  job is now `accuracy`), and `check-structure`.
- Data: every curated TSV under `data/`, `reports/`, `data/unified`,
  `data/morphology`; the generated registries and residue tables (the new
  extractor regenerates the real tables from the sources).
- Documents: all of `docs/` and the root prompt files. The README is
  written fresh from english's README as a template.
- Dependencies: down to `phf` + `unicode-normalization` in the shipped
  crates.

## What is kept, and where it goes

- The merged rule kernel (the five POS modules, the divergence registry's
  *content*) becomes `church-slavonic-core`'s rule modules — trimmed to
  english's style: tables of endings, a recension condition where the
  registry proved one, no traces, no evidence types, no typed defects, no
  `FormSet`. A cell the rules cannot produce is simply absent and the
  table serves it.
- The orthography engine's folds and projection rules become
  `core::orthography` (used by the extractor to normalise sources and by
  the facade to realise output).
- The extractor keeps the source parsers (Kaikki tables, the Alypy table
  extraction) as its `extract` stage; `assign` numbers senses by the sort;
  `bootstrap` emits PHF.
- The two oracles' *data* (what english would call the filtered dump)
  live in `data/intermediate/` as regenerable caches, gitignored.

## Execution order (each step lands green: `cargo test --workspace`,
`check-registry`, lint; `accuracy` reported at every step)

1. **Freeze the current tree with a tag** (`v1.1.0-alpha` exists) and
   write the new README skeleton from english's, with an empty accuracy
   table — so the target is visible before deletion.
2. **Build `church-slavonic-core` in english's shape** from the merged
   kernel: rules only, `grammar.rs` with the minimal enums (`Case`,
   `Number`, `Gender`, `Person`, `Tense`, `Form`, `Degree`, `Recension`),
   `sense_key.rs` shared with the extractor. Golden tests like
   `regular_rules_golden.rs`.
3. **Build the extractor** in english's four stages over the labelled
   sources; `refresh-data` emits the PHF tables; `check-registry` audits
   them; `accuracy` scores rules+tables against the sources per POS per
   recension. Commit the first tables and the first accuracy table.
4. **Build the facade** as one `lib.rs`: table-first, rule-fallback,
   case restoration, `_n` keys, `&Recension` on every call; tests in
   english's set (paradigm matrix for the copula, pronoun matrix, rule
   regression guards, sense-key stability).
5. **Delete everything else** per the list above; move the analyzer to
   `experiments/`; publish the deprecation releases; rewrite CI to three
   jobs.
6. **Publish** `church-slavonic` and `church-slavonic-core` at the next
   breaking version with the README accuracy table filled in. Done means:
   the tree matches the target silhouette, xtask has three commands,
   `cargo test --workspace` and `check-registry` are green, and the README
   is the only document.

## Report back

The tree listing with file sizes (english's `git ls-tree` silhouette is the
comparison), the two accuracy tables, the dependency list of the shipped
crates, and the line count of `xtask/src/main.rs`.
