# Synodal v0.10 productive morphology and lexicon execution prompt

You are working in the Church Slavonic Rust workspace.

Goal: improve the Synodal Russian Church Slavonic inflection engine as a
productive, source-accountable linguistic system. Extend the strongest parts of
the v0.9 architecture: closed morphological types, independently supplied
principal parts, exact-first resolution, reusable accent paradigms, explicit
defectiveness, stable diagnostics, and one shared productive kernel.

Do not optimize top-k corpus coverage. Do not choose rules because they recover
frequent tokens, bulk-promote exact forms, infer lexical identity from spelling,
or treat Old Church Slavonic or corpus frequency as proof of a Synodal paradigm.
Corpus evaluation may be used only as a final regression signal.

Before changing code, read:

- `AGENTS.md`;
- `SYNODAL_V09_INFLECTION_ENGINE_IMPROVEMENT_PROMPT.md`;
- `docs/SYNODAL_V09_INFLECTION_ENGINE_AUDIT.md`;
- `docs/SYNODAL_MORPHOLOGY.md`;
- `docs/SYNODAL_ORTHOGRAPHY.md`;
- `data/synodal/engine_capabilities.tsv`;
- `data/synodal/lexemes.tsv`;
- `data/synodal/principal_parts.tsv`;
- `data/synodal/noun_restrictions.tsv`;
- `data/synodal/accent_paradigms.tsv`;
- the Synodal core, facade, dictionary, extractor, and `xtask` crates;
- the pinned Alypy source pages relevant to noun §§41–44 and verb §§79–100.

Inspect the complete current diff and repository status before implementation.
Preserve unrelated user changes and do not alter historical v0.4–v0.9 audit
documents.

Preserve these architectural constraints:

- The core crate remains a pure, deterministic, filesystem-free morphology,
  orthography, and Unicode engine.
- The facade owns lexical identity, providers, provenance, exact and irregular
  overrides, accent metadata, and resolution policy.
- The dictionary remains a semantic lookup and analysis layer rather than a
  second morphology engine.
- Registered lexemes and caller-supplied specifications must converge on the
  same productive kernel after identity and override resolution.
- Attestations, normative exact tables, irregular overrides, inherited
  predictions, and productive forms remain distinguishable.
- Missing, contradictory, defective, unsupported, and insufficiently evidenced
  behavior returns typed errors and remains visible in complete paradigms.
- Runtime crates remain offline, deterministic, no-default-feature compatible,
  and compatible with `wasm32-unknown-unknown`.

Implement a coherent v0.10 improvement in the following order.

## 1. Complete additional source-backed noun families

Audit Alypy §§41–44 and admit only families for which the pinned target source
establishes a complete input/output contract. Prioritize:

- fourth-declension neuters with extended `-ат-` stems, such as the documented
  `ѻтроча : ѻтрочат-` pattern;
- fourth-declension feminine extended-stem families represented by examples
  such as `свекры : свекров-` and the independently documented variants of
  `любовь`;
- fourth-declension masculine `-ен-` families such as `камень`, including only
  the collective, alternative, or first-declension behavior that the source
  explicitly licenses;
- additional plural-only third-declension nouns such as `гꙋсли`, `ꙗсли`,
  `перси`, and `мощи`, when stable lexical identity and target-recension
  metadata are independently reviewed;
- other complete paradigms in §§41–44 that materially generalize the engine
  without requiring guessed stems or endings.

Use closed types for every distinct productive contract. Require citation and
oblique stems or other principal parts independently wherever the source shows
stem extension or alternation. Do not derive an extended stem from the lemma by
suffix replacement.

Represent, when source-backed:

- lexical number inventory;
- lexical or cell-relevant animacy constraints;
- independently supplied stem alternants;
- lexeme-specific ending families;
- collective versus ordinary plural behavior;
- ordered normative variants.

Do not hide lexical exceptions in broad string heuristics. Do not generalize a
variant printed for one lexeme to an entire class unless the grammar explicitly
states that it is class-wide. Preserve wide-letter alternations and printed
Church Slavonic orthography only at source-defined seams.

Every new productive rule must have:

- a stable rule ID;
- target recension;
- exact source citation;
- closed input metadata;
- complete valid cell inventory;
- explicit invalid or absent cells;
- ordered variants;
- accent contract;
- complete representative goldens;
- boundary and contradictory-metadata tests;
- typed failure behavior.

## 2. Make reviewed verbs genuinely productive

The existing verb kernel already separates present edges, aorist, imperfect,
imperative, l-participle, and participial principal parts. Improve usefulness by
supplying reviewed lexical metadata and reusable typed construction, not by
deriving every system from the infinitive.

Audit registered verbs and the pinned grammar to select a representative set
with sufficient independent target evidence. Cover distinct conjugations,
stem-final alternations, aspects, aorist and imperfect formations, imperatives,
l-participles, and participles where the evidence is complete.

For admitted verbs:

- store each independently required principal part;
- preserve the first-singular and third-plural present edges;
- represent aspect restrictions and defective systems explicitly;
- use exact or irregular overrides before a licensed productive background;
- retain ordered variants and provenance;
- expose complete `VerbSystem` paradigms, including failures;
- report the precise missing principal parts for every requested system.

Add typed templates only when the source establishes that the same complete
principal-part relationship is reusable across a class. A template must accept
all lexical alternants that cannot be derived safely. It must not become suffix
guessing disguised as a builder.

Do not implement productive supines, verbal nouns, short superlatives, new
futures, or other unsupported systems unless this work discovers and documents
a complete target-recension input/output contract. Otherwise keep them
explicitly unsupported.

Add reusable verb accent paradigms only for complete source-backed behavior.
Never infer stress from an unaccented lemma or from another verb in the same
conjugation.

## 3. Add an injectable lexical-provider API

Design and implement a public provider abstraction so applications can supply
reviewed lexemes without rebuilding the crate. Prefer a small, object-safe or
cleanly generic API with a stable contract, for example a `LexemeProvider` or
`Lexicon` trait, but determine the final interface from the existing resolver
architecture.

The provider must be able to represent the information needed by ordinary
resolution, including:

- stable lexical identity and part of speech;
- productive noun, adjective, and verb metadata;
- number, animacy, defectiveness, and principal-part restrictions;
- exact forms and irregular overrides;
- accent paradigms;
- evidence and target-recension provenance.

Requirements:

- The generated static registry implements or adapts to the same provider
  contract used by caller-supplied providers.
- Provider composition and precedence are deterministic and documented.
- Duplicate or conflicting identities fail closed rather than silently
  shadowing one another.
- Exact/irregular/accent/productive precedence remains unchanged.
- Provider failures use typed errors and stable `ErrorCode` values.
- No provider API gives runtime crates implicit filesystem, network, JSON, TSV,
  or database access. Applications may implement such storage outside the
  runtime crates.
- The provider layer must not duplicate noun or verb generation logic.

Add examples for an in-memory application lexicon, an explicit unregistered
lexeme, provider composition, conflict handling, and exact-first productive
fallback.

## 4. Improve paradigm and batch ergonomics

Build on `VerbSystem`, `Paradigm`, `ParadigmStatus`, and `ErrorCode` without
creating parallel generation paths.

Consider and implement only APIs that materially improve ordinary use:

- batch requests preserving input order and one typed outcome per request;
- complete noun and verb paradigms through an injected provider;
- filters or iterators for successes, failures, irregular cells, ambiguity,
  and individual error codes;
- serialization-friendly owned summaries when the `serde` feature is enabled;
- precise capability and missing-metadata inspection before generation.

Batch operations must not drop failures, reorder variants, erase provenance, or
turn partial success into an undifferentiated error.

## 5. Add linguistic evaluation independent of corpus coverage

Create a small, curated, deterministic engine-evaluation layer based on
linguistic behavior rather than token recovery. It should be reviewable in the
repository and test at least:

- complete held-out paradigms for new noun families;
- complete or deliberately partial verb systems;
- exact-over-irregular-over-productive precedence;
- syncretism and animacy relationships;
- number-restricted and defective cells;
- citation/oblique-stem consistency;
- ordered variants;
- accent scope completeness and overlap rejection;
- combining-mark order and breathing/accent interaction;
- explicit metadata for previously unregistered lemmas;
- panic freedom under hostile Unicode and contradictory metadata.

Do not score this evaluation by corpus frequency. Report pass/fail by linguistic
contract and keep fixtures small enough for direct review.

## Data and extraction integrity

- Upgrade registered lexical rows only when class, principal parts,
  restrictions, accent, identity, and target recension have independent
  reviewed evidence.
- Fail closed if a productive upgrade conflicts with a reviewed lemma, part of
  speech, stable identity, or target recension.
- Validate all foreign keys, evidence IDs, source/target recensions, duplicate
  rows, and closed codes before generating Rust.
- Keep generated registries byte-deterministic.
- Do not mass-convert `LexicalForm` rows or manufacture large exact-form tables
  to imitate productive morphology.
- Old Church Slavonic sources may motivate review but cannot directly establish
  a Synodal surface form.

## Documentation and audit

- Update `docs/SYNODAL_MORPHOLOGY.md` and `docs/SYNODAL_ORTHOGRAPHY.md` where
  contracts change.
- Update the core, facade, and dictionary READMEs with concise executable
  examples.
- Update `data/synodal/README.md` and
  `data/synodal/engine_capabilities.tsv`.
- Generate a new deterministic
  `docs/SYNODAL_V10_PRODUCTIVE_MORPHOLOGY_AND_LEXICON_AUDIT.md`.
- Do not rewrite historical v0.4–v0.9 audits.
- Clearly list deliberately unsupported behavior and the precise evidence or
  design work still needed.
- Treat corpus coverage, if rerun, as an incidental regression signal only.

## Testing requirements

Add positive and negative tests for every admitted rule and API. At minimum,
test:

- complete source-table goldens;
- previously unregistered lemmas using explicit metadata;
- missing and contradictory principal parts;
- invalid lemma/stem/class/gender combinations;
- number and animacy restrictions;
- collective and ordinary plural separation;
- defectiveness and unsupported systems;
- exact, irregular, provider, accent, and productive precedence;
- provider conflicts and deterministic composition;
- batch ordering and retained failures;
- variant ordering and per-variant provenance;
- missing and overlapping accent scopes;
- hostile Unicode and combining-mark order;
- stable error codes;
- absence of production panics.

Do not weaken existing tests or update expected data merely to make a failure
pass. Validate every mismatch against the pinned source or current documented
contract.

## Verification

Run targeted checks while implementing, then complete:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test -p synodal-church-slavonic-core --all-features
cargo test -p synodal-church-slavonic --all-features
cargo test -p synodal-church-slavonic-dictionary --all-features
cargo test -p synodal-church-slavonic-extractor --all-features
cargo test --workspace --all-targets --all-features
cargo test --workspace --doc
cargo xtask synodal-engine-audit --check
cargo xtask synodal-check
cargo xtask check-all
```

Also verify:

- native no-default-feature builds for all three runtime crates;
- `wasm32-unknown-unknown` no-default-feature builds for all three runtime
  crates;
- package-content checks and publish dry-runs;
- byte-stable regeneration on two consecutive runs;
- no generated-tree drift beyond the intended change set;
- no new production `unwrap`, `expect`, `panic!`, `unreachable!`, `todo!`, or
  unsafe code paths.

Before finishing, review the complete diff separately against its merge base.
Check for linguistic overgeneralization, incorrect source interpretation,
unsupported claims, wrong variant order, identity or provenance collisions,
accent gaps, provider precedence bugs, duplicate generation paths, panic paths,
and missing negative tests. Validate each finding against current code and the
pinned sources, fix every confirmed high-severity issue, and rerun all affected
checks.

Deliver a concise report covering:

- productive noun families added;
- productive and irregular verb improvements;
- lexical/provider API design and precedence;
- batch or paradigm ergonomics;
- new linguistic evaluation fixtures;
- lexical metadata and accent paradigms admitted;
- stable rule IDs and source citations;
- deliberately unsupported behavior;
- verification commands and results;
- review findings fixed or rejected with rationale;
- remaining linguistic and API risks.

Do not commit, push, publish crates, create a branch, or open a pull request
unless separately requested.
