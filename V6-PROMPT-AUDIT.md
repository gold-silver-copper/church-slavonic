# Adversarial code and prompt review

Reviewed 2026-09-06. Scope: current working-tree source for morphology,
contextual analysis, publication, generation evaluation, and representative
finite-paradigm tests; V5 and V6 as proposals. Repository prose and historical
results were not accepted as authority. This is a targeted static review,
not a full correctness certification. No implementation changes or application
tests were performed in this review.

## Judgment

V6 is the better starting point, but it still lets an implementation accumulate
teaching examples and evidence infrastructure without replacing ordinary
consumers. Its completion language also risks making unspecified expert review
an indefinite obligation. The revised prompt makes consumer migration and
reusable morphology explicit, and requires a finite acceptance contract with
separate engineering and scholarly-validation statuses.

## Findings checked against current executable source

Paths below are relative to the repository root. These are static findings,
not newly reproduced failures or numerical accuracy claims.

| Evidence | Implication for the rewrite prompt |
|---|---|
| `crates/church-slavonic/src/sentence/mod.rs::disambiguate` mutates the existing tree through `rules::disambiguate`; `contextual_trace` separately lifts the witness afresh. `sentence/rules.rs::apply_rules` narrows candidates using preposition, adjacency, and subject assumptions. | A separate trace API does not replace ordinary contextual decisions. Require preserved candidates and source-addressed premises in the declared consumer. The trace alone does not demonstrate that each exclusion is linguistically wrong. |
| `crates/church-slavonic/src/sentence/trace.rs::Event` contains a rule, child index, context snapshot, and proposed node. | An execution snapshot is not a represented syntactic attachment or independently validated premise. |
| `crates/church-slavonic/src/morphology/stem.rs::apply` performs an explicit final replacement on a supplied stem. `crates/church-slavonic-tools/tests/finite_paradigms.rs` loads supplied model data and compares two 19-row fixtures through generation, analysis, and reload. | These support a legitimate bounded rule-execution contract. They do not by themselves demonstrate unsupplied stem prediction or productive coverage of additional lexemes. Count these achievements separately. |
| `crates/church-slavonic/src/morphology.rs::analyze_budgeted` iterates lexemes, inventories cells, and calls generation while charging a work budget. | Resource limits are useful, but production-scale latency remains a measurement question. Require realistic request and population benchmarks before choosing an index or declaring readiness. No latency defect was benchmarked here. |
| `crates/church-slavonic-tools/src/import/mod.rs::write_outcome_at` calls `lexicon_stage.commit()` then `quarantine_stage.commit()`. `import/publication.rs::commit` persists one file. | Both outputs are staged first, but interruption or failure between commits can expose a mixed pair. Specify publication units and recovery; per-file atomicity is insufficient for a claim of joint atomicity. |
| `crates/church-slavonic/src/morphology.rs::engine_sha256` hashes a selected list of embedded files. | Qualify implementation identity. This function alone does not identify the complete build and dependency closure. No stale-artifact exploit was reproduced. |

## Linguistic checks outside repository documents

Krause and Slocum's [OCS lesson 9, §44](https://lrc.la.utexas.edu/eieol/ocsol/90)
describes agreement numerals and nominal numerals differently, including
singular feminine inflection for five through ten with a genitive plural
complement. This supports separating lexical classification, inflection,
semantic quantity, and construction. It does not dictate universal Rust types
or establish the same treatment for every Synodal source.

Their [OCS lesson 1, §§4–4.2](https://lrc.la.utexas.edu/eieol/ocsol/10)
distinguishes simple and compound tenses, describes future uses of present
forms, and warns that extracting stems from infinitives can be obscured by
historical changes. This supports auditing the inherited `FiniteTense::Future`
classification and distinguishing supplied allomorphs from productive rules.
It does not alone prove the enum wrong for every supported profile. The page's
literal paradigms and prose-described variants also have different scopes.

These university-hosted teaching accounts were opened during this review.
They are scholarship consulted outside the repository, not independent blind
validation of fixtures already derived from them or expert approval of the
library. Consequential profile-specific rules still need appropriate sources.

## Suspicions rejected or narrowed

- Default generation evaluation is not the old relaxed-recall path:
  `crates/church-slavonic-tools/src/eval.rs::run` calls `generation::evaluate`.
  Its report explicitly conditions results on mapped annotated lemmas and cells.
  This is not unknown-surface analysis accuracy, but that distinction is stated.
- `VerbCell::Supine` exists. An audit claiming the category is absent is stale;
  representation alone does not establish complete coverage.
- `voc-drop`, `bare-voc`, and `bare-loc` are retired from the inspected rule
  implementation. Their historical failures are not current findings.
- Generation and lexical acceptance machinery already exists, including
  `morphology/lexical_acceptance.rs::with_lexical_acceptance`. Caller acceptance
  is a policy decision, not scholarly certification. The next prompt should
  not prescribe rebuilding machinery merely because an old audit omitted it.
- Existing model rules now include stem replacement. Describing realization
  as only supplied-stem concatenation is incomplete.

## Prompt changes and verification

Updated V6 in place to require explicit ordinary-consumer migration; progress
measured separately for lexemes, cells, generalization, and integration; a
reusable-family increment before further isolated examples; conditioned stem
operations and profile-specific tense analysis; publication-unit recovery;
realistic performance targets; and an initial decision about the status of
external expert review.

Retained source preservation, explicit matching policies, ambiguity retention,
independent evaluation, migration accounting, and a finite scope. The prompt
still requires an initial acceptance table because the repository cannot
establish the user's desired coverage merely by asserting it in prose.

Verification: inspected the cited symbols and source branches; checked the two
edited Markdown files for trailing whitespace and final newlines. No historical
test logs were counted as verification of this review. The rewrite was not
executed, and existing implementation changes were preserved.
