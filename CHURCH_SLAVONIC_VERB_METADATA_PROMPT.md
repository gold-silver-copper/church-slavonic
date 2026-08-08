# Build end-to-end Old Church Slavonic verb metadata and generation

Work in the existing `church-slavonic` Rust workspace. Improve the published
`old-church-slavonic` inflector so a normal lemma-based API can use independently
audited lexical principal parts and productive rules when an exact dictionary cell
is absent.

This is an Old Church Slavonic (`cu`) project. Do not silently substitute later
Russian, Bulgarian, Serbian, Croatian, or other Church Slavonic recension forms.
Preserve source spelling and manuscript/editorial variation as evidence.

The governing rule is honesty: exact dictionary evidence wins; a traceable
metadata-backed prediction comes next; `MissingLexicalMetadata`, ambiguity, or
`UnsupportedCell` is preferable to a plausible invented form.

The main outcome should be that calls such as:

```rust
finite_verb("нести", imperfect_cell)
```

can return an exact table form when present, otherwise construct a typed lexical
analysis from source-backed principal parts and invoke the production core. Users
should not have to reconstruct `VerbLexeme` manually for known dictionary lemmas.

Do not add heuristic suffix guessing merely to increase coverage. Every accepted
metadata derivation, formation selection, and generated variant must have a stated
authority, provenance, and leakage-controlled evaluation.

---

## 1. Audit the current implementation and regenerate the baseline

Before changing code, read at least:

- `CHURCH_SLAVONIC_VERB_EXPANSION_PROMPT.md` and
  `reports/verb-expansion.md`;
- `docs/MORPHOLOGY_SPEC.md`, `docs/ORTHOGRAPHY.md`,
  `docs/ARCHITECTURE.md`, `docs/DATA_PIPELINE.md`, `docs/GUARDS.md`, and
  `docs/CORPUS_EVALUATION.md`;
- `crates/old-church-slavonic-core/src/{grammar,verb,adjective,result,trace}.rs`;
- the public table-first facade in `crates/old-church-slavonic/src/`;
- the generated registry schema and lookup implementation;
- verb extraction in `crates/old-church-slavonic-extractor/src/extract.rs`;
- dictionary OOV evaluation in `crates/xtask/src/main.rs` and corpus evaluation in
  `crates/xtask/src/corpus.rs`;
- `data/SOURCES.toml`, `data/evaluation-sources.json`, and current reports; and
- all core, public-API, extractor, registry, no-panic, guard-witness, package, and
  report-freshness tests.

Regenerate all relevant reports from the committed inputs and record the real
starting metrics. Do not copy the following values blindly. At the time this prompt
was written, the project contained 3,081 lexemes, 134,761 public cells, and 137,406
ordered dictionary variants. Useful current signals included:

- held dictionary imperfect: 384/384;
- held dictionary imperative: 157/188;
- native new-aorist lookup: 1,682/2,643;
- native imperfect lookup: 376/1,725;
- native lemma-disjoint final lookup: 341/623; and
- 153,310 declined-participle source rows still rejected because row-local
  tense/voice identity is unavailable.

Those figures use different denominators and answer different questions. Preserve
that separation. Dictionary agreement is not independent proof of a productive
rule, and the native score currently uses oracle metadata from another token of the
same lemma.

Write a baseline report before tuning. Identify how many known verb lemmas can
construct each required metadata field, how many public requests remain table-only,
and how many otherwise-generatable cells fail for missing metadata.

---

## 2. Keep source authorities and licensing boundaries distinct

Use these authorities for different purposes:

1. **Grammar defines productive morphology.** Start with University of Texas
   *Old Church Slavonic Online* and cite exact relevant sections. Add another
   scholarly grammar only when its edition and bibliographic identity are explicit.
2. **Pinned English Wiktionary via Kaikki/Wiktextract defines the dictionary
   snapshot.** Use the exact dump and hash in `data/SOURCES.toml`. Audit actual
   page/template revisions for every new extraction or principal-part mapping.
3. **UD OCS PROIEL and native Syntacticus PROIEL/TOROT provide attested-token
   evaluation.** Use only commits and file hashes in
   `data/evaluation-sources.json`.

Do not compile or redistribute UD, PROIEL, TOROT, Syntacticus, or derived token
listings. They are CC BY-NC-SA 4.0 external evaluation inputs. Commit only
non-reconstructive aggregates; keep token details and mismatch excerpts gitignored.

CCMH is not a current input. Use it only after auditing an exact artifact, file-level
license, schema, encoding, and hash. Do not claim a CCMH test merely from its
catalogue description.

No build, unit test, package, or normal `cargo xtask check-all` execution may access
the network or depend on an uncommitted corpus checkout.

---

## 3. Add a typed lexical principal-part model

Introduce a typed representation such as `VerbPrincipalParts`,
`DictionaryVerbMetadata`, or an equivalent design. Names are flexible; independent
dimensions and provenance are not.

The model must represent:

- present class and ordinary present stem;
- first-singular present allomorph;
- lexical aspect without using it to choose a past formation;
- imperfect stem, formation, and documented variant policy;
- one or more aorist analyses, each with its own stem and formation;
- imperative stem and i/yat-series selection;
- independent stems and formations for all four non-l participles;
- l-participle/infinitive-system metadata where relevant;
- cell-specific irregular or suppletive overrides;
- multiple attested analyses without silently selecting one; and
- provenance for every field and override.

At minimum, provenance must distinguish:

- exact dictionary table cell;
- dictionary principal part derived from other safe cells;
- independently curated grammatical override;
- explicit caller metadata;
- corpus observation used only for evaluation; and
- productive rule output.

Do not store morphology as magic feature strings inside the core. Generated registry
serialization may use stable codes, but validate them into enums before generation.
An invalid or unknown generated code must fail a registry check.

If multiple aorist or participle formations are attested, retain separate analyses
with deterministic source ordering and provenance. Do not merge them into an
untraceable bag of surface strings. If `FormSet` cannot express the required
analysis provenance safely, evolve the public result model.

Add validated constructors or builders so impossible combinations are rejected near
metadata construction instead of deep inside generation. Backwards compatibility is
not a constraint, but the resulting API must remain clear and typed.

---

## 4. Derive metadata only from audited diagnostic evidence

Define a written derivation contract for every field before implementing it. Each
contract must state:

- which source cells may supply the field;
- what suffix or alternation is removed or retained;
- which class/formation declarations are prerequisites;
- which ambiguous spellings or shapes cause rejection;
- which other cells cross-check the result;
- whether variants produce multiple analyses; and
- the stable provenance stored with the field.

Apply these constraints:

- Never infer a first-singular allomorph through broad automatic consonant mutation.
- Never choose an aorist from lexical aspect.
- Never choose an imperative series solely because it is common.
- Never use a participle citation to derive itself during evaluation.
- Never use a target cell, duplicate spelling of that cell, or equivalent
  person-number cell as held-out metadata.
- Never treat `error-unrecognized-form`, a missing table heading, or a flattened
  declined-participle row as a principal part.
- Never learn an override from the final lemma or document holdout.

Require cross-cell consistency when more than one safe diagnostic cell exists. If
two cells imply incompatible stems or formations, retain explicit alternatives when
both are defensible or reject the automatic analysis as ambiguous.

Store generated metadata in deterministic, reviewable normalized data, preferably a
dedicated `data/extracted/verb_metadata.tsv` or similarly explicit schema. Generate
static Rust from it; the runtime crate must not read TSV, JSON, XML, or the network.

Add semantic validation for metadata references, enum codes, provenance, duplicate
analyses, empty stems, non-NFC text, invalid overrides, and orphaned lexeme IDs. Add
injected guard witnesses proving each important validation fails.

---

## 5. Connect metadata to the public table-first facade

Preserve and make explicit this resolution order:

1. resolve lemma/POS ambiguity;
2. return the exact source table cell when it exists;
3. otherwise load source-backed metadata for that exact lexeme ID;
4. apply a cell-specific override when one is explicitly registered;
5. otherwise call the production core with each typed metadata analysis;
6. return all defensible ordered analyses/variants with provenance; or
7. return a precise typed error.

Known dictionary forms must never be displaced, reordered, or relabeled as rule
predictions. Existing by-ID and lemma APIs must use the same resolver.

Metadata-backed facade results must use `FormSource::DictionaryMetadataRule` or a
more precise replacement, include the productive rule ID, and retain an ordered
trace separating lexical evidence selection, stem/formation construction,
phonological or orthographic seams, and final ending/agreement.

Do not catch `MissingLexicalMetadata` and silently fall back to a frequent class.
Distinguish at least:

- no dictionary lexeme;
- ambiguous lemma;
- missing principal part;
- contradictory lexical analyses;
- represented but unsupported formation; and
- historically invalid cell.

Keep `*_with(&VerbLexeme, ...)` as the explicit-caller path. It must not secretly
consult the dictionary. Add public examples showing a table result, dictionary
metadata rule, caller metadata rule, ambiguity, and unsupported behavior.

---

## 6. Improve the highest-value verb gaps

The metadata layer is the primary deliverable. Use it to improve these formation
gaps only where grammatical audit and held evidence justify them.

### 6.1 Imperfect variants

Analyze native imperfect mismatches by formation, cell, manuscript, and spelling
pattern. Separate contraction, source orthography, suppletion, wrong principal-part
selection, and genuinely unsupported formation.

Add uncontracted or contracted variants only when a cited rule specifies their
distribution or lexical metadata explicitly declares them. Preserve deterministic
variant order and diplomatic spelling. Do not introduce lossy normalization to hide
disagreement.

### 6.2 Sigmatic aorists

The current enum represents `Sigmatic`, but generation deliberately returns
`UnsupportedCell`. Specify independently supported sigmatic subtype(s), stem input,
loss/alternation behavior, endings, variants, and conflicts.

Do not implement one generic `-s-` suffix. Primary, secondary, root, or other
sigmatic behaviors must remain separate when their stem formation differs. A lexeme
with multiple attested aorists must expose multiple provenance-bearing analyses.

If the audit cannot justify a subtype, keep it explicitly unsupported and document
the precise missing evidence.

### 6.3 Past active participles

Add the primary transformed i-stem `-ьш-` formation and explicitly model supported
final-j deletion and `ov -> u` behavior before `-въш-`. Require an appropriate
principal-part base when automatic derivation is not independently valid.

Evaluate `-ъш-`, `-ьш-`, and `-въш-` separately. Do not use the target participle
citation as its own formation selector.

### 6.4 Irregular and suppletive verbs

Add a small audited override mechanism and prove it with high-value verbs such as
`бꙑти`, `дати`, `ѣсти`, `вѣдѣти`, `хотѣти`, and selected motion verbs. Each override
must cite an exact dictionary cell or grammatical source. Do not force these lexemes
through productive classes merely to increase coverage.

### 6.5 Declined participle extraction

The 153,310 rejected declined rows are a future opportunity, not a target. Admit
them only if a pinned positional/table-block parser can validate complete
present-active, present-passive, past-active, and past-passive shapes atomically.

Fixture-test every accepted template revision and block shape. An unknown sentinel,
row count, heading order, or structural change must reject the entire block. Do not
label rows from sequence unless the complete enclosing structure is verified.

It is acceptable to leave all declined rows rejected in this milestone if the audit
cannot meet that standard.

---

## 7. Add leakage-controlled end-to-end evaluation

Keep the existing questions separate:

1. exact public dictionary-cell recall;
2. productive core behavior with declared/oracle principal parts; and
3. lemma-disjoint OOV behavior.

Add a fourth, primary question:

4. **End-to-end dictionary-metadata generation:** given only a lemma/lexeme ID and
   requested cell, can the public facade resolve safe registry metadata and return
   an attested target without consulting that target?

For this evaluation:

- remove the target cell and every equivalent duplicate before constructing metadata;
- rebuild or filter metadata as if those cells did not exist;
- call the production public facade, not evaluator-local endings;
- report metadata construction separately from form generation;
- keep exact-table hits out of the fallback-rule numerator;
- partition by normalized lemma before learned policies or override tuning;
- freeze current final lemma and document partitions;
- keep development and final results separate; and
- never tune from the final holdout without transparently resetting the baseline.

Report a stage funnel containing at least:

- source verb lexemes/tokens;
- compatible requested cells;
- unambiguous lexemes;
- metadata records found and validated;
- generation attempts and returned forms;
- diplomatic top-1 and any correctness;
- project-lookup top-1 and any correctness; and
- every skip/failure reason.

Slice results by generation path, present class, formation, complete grammatical
cell, metadata source-cell policy, regular versus override analysis, lemma frequency,
lemma partition, manuscript/document partition, and dictionary versus independent
corpus evidence.

For real-text end-to-end scoring, registry metadata must come from the pinned
dictionary or curated overrides, not another token in the same corpus. Keep the
native oracle score as a diagnostic and label it clearly; do not rename it
end-to-end accuracy.

Set non-regression thresholds only after recording the clean baseline and reviewing
denominator changes. Guard both metadata availability and conditional accuracy so a
system cannot improve merely by refusing difficult cells.

Token-level details must remain local and gitignored. Committed reports may contain
aggregate counts, source IDs, hashes, formation names, and skip reasons, but no
reconstructive corpus excerpts.

---

## 8. Required tests

Add or expand tests for:

### Metadata and extraction

- one safe fixture for every admitted principal-part derivation;
- reordered and duplicated source tags;
- conflicting diagnostic cells;
- target-cell and equivalent-cell exclusion;
- unknown class/formation codes;
- multiple attested analyses with stable order;
- invalid, non-NFC, and empty stems;
- provenance round-trip through normalized data and generated Rust;
- content-derived ID changes and alias rewrites; and
- atomic regeneration failure preserving prior outputs.

### Productive core

- full person-number goldens for every newly supported finite formation;
- every historical imperative cell and rejection of non-cells;
- `-ъш-`, `-ьш-`, and `-въш-` participle witnesses where implemented;
- stem-loss and palatalization seams at positive and negative boundaries;
- aspect never changing aorist selection;
- missing metadata naming the exact required field;
- represented unsupported formations returning `UnsupportedCell` immediately; and
- hostile Unicode/input tests proving no panic.

### Public facade

- exact table cell still wins;
- missing table cell uses dictionary metadata;
- by-ID and lemma APIs agree;
- explicit `*_with` remains dictionary-independent;
- irregular override provenance;
- multiple analyses/variants remain ordered;
- ambiguous lemma remains typed;
- no fallback on an unknown lexeme; and
- full paradigms agree with individual cell getters.

### Evaluation and guards

- a synthetic leakage witness that scores correctly only if the target is used;
- corpus mappers rejecting incomplete, contradictory, negative, passive, or
  historically invalid bundles as applicable;
- frozen partition fixtures;
- denominator accounting invariants;
- report freshness;
- package/runtime no-I/O boundary; and
- injected metadata failures detected and reverted by `guard-witnesses`.

Use metamorphic tests where stronger than isolated examples, but retain source-cited
goldens for every linguistic seam.

---

## 9. Documentation and reports

Update documentation in the same change as behavior:

- `docs/MORPHOLOGY_SPEC.md`: every formation, stable rule ID, input contract,
  examples, citations, conflicts, and unsupported boundaries;
- `docs/ARCHITECTURE.md`: table -> metadata -> override/core resolution flow;
- `docs/DATA_PIPELINE.md`: metadata extraction, normalized schema, generation, and
  corpus-detail licensing boundary;
- `docs/CORPUS_EVALUATION.md`: end-to-end score, leakage controls, partitions,
  denominators, and commands;
- `README.md` and crate READMEs: ordinary lemma-based verb behavior and errors;
- `ATTRIBUTION.md`: any new source or redistributed material; and
- a report such as `reports/verb-metadata-expansion.md`.

The implementation report must include exact baseline/final metrics, metadata
coverage by field and formation, end-to-end dictionary and independent-corpus
results, source/template witnesses, extraction counts, deliberate behavior breaks,
review findings and rationale, and remaining unsupported/oracle-dependent areas.

Do not describe oracle metadata as automatic prediction. Do not combine dictionary
and corpus accuracy into one headline percentage.

---

## 10. Non-goals

Do not add:

- morphological analysis or free-text lemmatization;
- phrase-level perfect, pluperfect, future, conditional, passive, or reflexive
  realization;
- automatic clitic placement;
- later-recension normalization;
- reconstructed accents not present in an authority;
- lossy orthographic folding presented as correctness;
- live downloads during build or test; or
- relaxed extraction guards merely to admit more rows.

`сѧ` remains a separate token. Periphrastic `да` forms are not synthetic imperative
cells. Corpus absence is not evidence that a generated paradigm cell is invalid.

---

## 11. Completion gate

Before declaring completion:

1. Inspect the complete change set against the merge base, including staged,
   unstaged, generated, and untracked files.
2. Run targeted checks while implementing, then repository-wide formatting, clippy,
   tests, registry, reports, examples, packages, and runtime-boundary checks.
3. Run `cargo xtask guard-witnesses` and confirm every injected failure is detected
   and reverted.
4. Regenerate dictionary accuracy and extraction reports from the pinned snapshot.
5. Run pinned UD and native Syntacticus evaluation locally, verify every source hash,
   and refresh only aggregate committed reports.
6. Perform a fresh full-diff review for linguistic correctness, leakage, regressions,
   unsafe edge cases, licensing mistakes, and missing tests.
7. Validate every finding, fix all confirmed P0/P1 defects, rerun affected checks,
   and repeat review if a fix changes behavior.
8. Verify published-package contents retain attribution and contain no external
   corpus material.
9. Report exact commands/results, remaining risks, and unsupported formations.

At minimum, final verification should include repository equivalents of:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
cargo xtask check-all
cargo xtask guard-witnesses
cargo xtask accuracy
cargo xtask accuracy-corpus \
  --ud /path/to/UD_Old_Church_Slavonic-PROIEL \
  --syntacticus /path/to/syntacticus-treebank-data
cargo package -p old-church-slavonic-core --allow-dirty
cargo package -p old-church-slavonic --allow-dirty
git diff --check
```

Do not publish, tag, push, or open a pull request unless the user explicitly asks
for that external action.
