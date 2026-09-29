# Prompt review against the working code

Scope: static inspection of the working tree based on commit
`09d14f3efc41d20503dd46d9391ca17a15a791c3`, including uncommitted code.
This is a targeted prompt audit, not a complete correctness certification.
Repository prose was treated as untrusted claims. No historical benchmark
numbers or claims of linguistic authority were accepted as evidence. This
review did not execute the rewrite or reproduce the historical runtime probes.

## Findings that affect the instructions

1. **A repaired path can conceal unrepaired consumers.**
   `crates/church-slavonic-tools/src/tagger/deployment.rs::predict` takes surfaces
   rather than gold annotations and retains lexical readings, but selects only
   features. `tagger.rs::oracle_examples` still uses annotated neighboring lemmas
   and previous gold choices, explicitly as a diagnostic/training path.
   `treebank/runner.rs::score_disambiguation` skips misaligned units before
   counting their leaves; its ambiguous-gold branch does not record a wrong
   lexeme when the IDs differ. These are distinct paths, not one evaluation.
   The prompt now requires an entry-point matrix and controlled comparisons.

2. **The new model can inherit the old ontology.**
   `crates/church-slavonic/src/cell.rs` has five inflection POS variants,
   including `Closed`; its `VerbCell` has no explicit supine variant.
   `morphology.rs` still consumes and serializes those cells even though lexical
   category assertions are separate. This establishes a representation question,
   not a linguistic verdict about a particular token. The prompt's prohibition
   on inheriting the old ontology and its profile-specific feature inventory
   remain necessary. Extra metadata alone does not implement a missing category.

3. **Executable licensing is not verified grammaticality.**
   `morphology.rs::generate` checks caller-verified restrictions but can realize
   positive paradigm rules without an equivalent acceptance gate. This need not
   prohibit useful generation from provisional models. It does require the
   result to distinguish model-relative licensing from evidential acceptance.
   The prompt now makes that distinction explicit and tests positive assertions.

4. **Reproduction can masquerade as prediction.**
   The model accepts supplied stems, rules, and accent instructions. Successful
   generation and reverse analysis establish useful software behavior, but do
   not alone establish how much morphology or accentuation was predicted.
   The prompt now separates table replay, rule application, and prediction;
   productive claims need additional lexemes and a record of supplied answers.

5. **Gold-free function arguments do not prove clean evaluation.**
   `morphology.rs::generate` attaches matching source-annotation IDs. The current
   deployment predictor uses the legacy lexicon, not these links; this review
   found no evidence that it consumes them. Nevertheless the evaluation contract
   must audit all inference artifacts for prior annotation-derived information,
   including lexicon fitting and overrides. The prompt now requires that audit
   and distinguishes displayed-lemma retention from lexical identity scoring.

6. **Historical results must not become current premises.**
   `eval.rs::corpus_matches` still contains broad letter-elision and subsequence
   tolerances. `sources/ud.rs::unpacked` still reuses an existing extraction
   directory without checking its identity. Conversely, some historical defects
   have newer implementations beside them. The audit table is now explicitly
   historical; its rows require fresh reproduction before being reported as
   current runtime findings.

7. **The prompt can reward perpetual infrastructure work.**
   Five phases and extensive evidence requirements invite a succession of small
   wrappers and records without a finished linguistic consumer operation.
   The prompt now requires a short acceptance matrix, a bounded first increment,
   observable milestone outcomes, and a reason for reopening accepted checks.

## Review limits

No new grammatical claims were adjudicated against external scholarship in
this prompt-editing pass. Specific grammatical references in the prompt remain
research leads to verify, not endorsed authorities. No corpus accuracy or
performance claims were measured here. Validation was inspection of the edited
prompt against the cited source paths; no application code was changed by this
pass, and application tests were not needed for these Markdown edits.
