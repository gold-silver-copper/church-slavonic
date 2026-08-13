# Synodal v0.9 inflection-engine improvement execution prompt

You are working in the Church Slavonic Rust workspace.

Goal: improve the Synodal Russian Church Slavonic inflection engine as a
linguistic engine. Do not optimize corpus top-k coverage, add forms merely
because they are frequent, or treat corpus matches as morphological evidence.

Start by reading:

- `AGENTS.md`
- `docs/SYNODAL_V08_INFLECTION_ENGINE_AUDIT.md`
- `docs/SYNODAL_MORPHOLOGY.md`
- `docs/SYNODAL_ORTHOGRAPHY.md`
- `data/synodal/engine_capabilities.tsv`
- `crates/synodal-church-slavonic-core/`
- `crates/synodal-church-slavonic/`
- `crates/synodal-church-slavonic-dictionary/`

Preserve the existing architecture:

- The core crate is a pure, filesystem-free morphology and Unicode engine.
- The facade handles lexical resolution, specifications, provenance, irregular
  overrides, and accents.
- Generated forms must remain distinguishable from attestations.
- Unsupported or insufficiently evidenced behavior must return typed errors.
- Old Church Slavonic evidence cannot directly establish a Synodal surface
  form.
- Runtime crates must remain deterministic, offline, no-default-feature
  compatible, and WASM compatible.

Implement a coherent v0.9 improvement centered on the following priorities.

## 1. Reusable accent paradigms

Expand the accent system beyond its single `мꙋдръ` example.

- Audit the pinned sources for complete, reusable accent behavior.
- Add source-backed fixed-stem, fixed-ending, or genuinely mobile patterns
  where the evidence supports them.
- Permit several cell-scoped accent rules within one lexical paradigm.
- Preserve acute, grave, kamora, breathing, combining-mark order, and exact-cell
  precedence.
- Never infer stress from spelling alone.
- Liturgical rendering must continue to return
  `OrthographicMetadataRequired` when no applicable accent evidence exists.
- Add complete positive and negative tests for every admitted accent pattern.

## 2. Noun morphology

Extend productive noun support beyond the seven currently implemented regular
classes.

Prioritize source-backed behavior for:

- mixed and consonantal declensions;
- heteroclitic or stem-extending nouns;
- lexical stem alternants;
- number-restricted and defective nouns;
- reviewed velar and sibilant alternations;
- documented ending variants in Alypy §§34–44.

Model these with closed linguistic types, explicit principal parts, stem
alternants, restrictions, and ordered variants. Do not hide irregular behavior
in string heuristics or untyped maps.

## 3. Lexical metadata and irregular paradigms

Make more registered lemmas usable by the productive engine.

- Identify lexemes that have sufficient source evidence for a productive
  class, gender, animacy, principal parts, alternants, or accent paradigm.
- Upgrade only those lexemes with independently reviewed metadata.
- Generalize the existing “exact override plus licensed regular background”
  design demonstrated by `сынъ`.
- Preserve suppletion, defectiveness, ambiguity, variant ordering, evidence,
  and provenance.
- Do not bulk-convert `LexicalForm` entries based on suffix guessing or corpus
  frequency.

## 4. Verb usability

Keep independent verb principal parts; do not derive every system from one
generic stem.

Improve:

- typed principal-part construction;
- reusable builders or templates for genuinely regular source-backed series;
- diagnostics explaining which principal part is missing;
- irregular and defective verb paradigms;
- paradigm APIs covering all represented finite and participial systems.

Do not implement short superlatives, supines, verbal nouns, new futures, or
other currently unsupported systems unless a complete target-recension
input/output contract and source-backed paradigm can be established.

## 5. API ergonomics

Improve ordinary usage without weakening the linguistic contracts.

Consider:

- a public injectable `Lexicon` or `LexemeProvider` abstraction;
- convenient constructors for common typed specifications;
- batch inflection and complete-paradigm APIs;
- stable machine-readable diagnostics;
- clear access to successful, failed, irregular, and ambiguous paradigm cells.

Avoid duplicate generation paths. Registered lexemes and explicit
specifications must delegate to the same productive kernel after their identity
and override layers.

## Testing and evidence requirements

- Every new productive rule needs a stable rule ID, target recension, exact
  citation, explicit input contract, valid inventory, invalid inventory,
  alternation rules, accent contract, golden examples, boundary cases, and
  typed failure behavior.
- Add complete paradigm goldens from source-backed representative lexemes.
- Test missing and contradictory metadata, invalid cells, variant ordering,
  exact/irregular/productive precedence, hostile Unicode, combining-mark order,
  and panic freedom.
- Add tests using previously unregistered lemmas with explicit metadata so
  success does not depend on dictionary memorization.
- Corpus coverage may be rerun only as a regression signal. Do not select or
  justify rules using coverage movement.
- Do not add mass exact-form tables to simulate productive morphology.

## Documentation and audit

- Update `docs/SYNODAL_MORPHOLOGY.md`.
- Update `data/synodal/engine_capabilities.tsv`.
- Extend the deterministic engine audit to describe the new capabilities and
  remaining blockers.
- Keep historical v0.4–v0.7 audits immutable.
- Update public crate READMEs and examples where behavior or ergonomics changes.
- Ensure generated files are byte-current.

## Verification

Run targeted tests while implementing, then complete:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test -p synodal-church-slavonic-core --all-features
cargo test -p synodal-church-slavonic --all-features
cargo test -p synodal-church-slavonic-dictionary --all-features
cargo test --workspace --all-targets --all-features
cargo test --workspace --doc
cargo xtask synodal-engine-audit --check
cargo xtask synodal-check
cargo xtask check-all
```

Also verify native no-default-feature builds, `wasm32-unknown-unknown` builds,
package dry-runs, deterministic generated artifacts, and a clean generated
tree.

Before finishing, review the complete diff separately for linguistic
overgeneralization, unsupported claims, incorrect variant ordering, provenance
mistakes, API duplication, panics, and missing negative tests. Fix all confirmed
problems and rerun affected checks.

Deliver a concise report covering:

- productive morphology added;
- accent paradigms added;
- lexical and irregular metadata added;
- API changes;
- deliberately unsupported behavior;
- sources and rule IDs;
- tests and verification results;
- remaining linguistic risks.

Do not commit, push, publish crates, create a branch, or open a pull request
unless separately requested.
