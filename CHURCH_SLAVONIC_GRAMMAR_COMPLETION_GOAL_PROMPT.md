/goal Make this Church Slavonic Rust workspace source-bounded and grammar-complete for Old Church Slavonic and Synodal Russian Church Slavonic. Continue autonomously through research, design, implementation, data work, documentation, testing, review, and correction. Do not stop merely because one release-sized slice is complete. Stop only when the completion contract and every final verification condition below are satisfied, or when an essential external source or decision is genuinely inaccessible after all safe alternatives have been exhausted.

# Objective

Build an inflection engine that can generate every grammatically licensed form in
the supported Old Church Slavonic and Synodal Russian Church Slavonic systems.
Support arbitrary lexemes when the caller supplies the complete typed lexical
metadata required by the grammar, and support every inflectable lexeme found in
the locked source union through productive metadata, an explicit irregular
paradigm, or an explicit classification such as defective or indeclinable.

Potentially unattested forms are allowed when they follow a source-backed
productive rule. Label them as predictions and never represent them as attested.
Preserve the distinction among normative, attested, reconstructed, inherited,
variant, analogical, predictive, disputed, and unsupported evidence.

“Every possible word” is not a claim that a finite bundled dictionary contains
an open lexicon or that a bare spelling uniquely determines its paradigm. It
means all of the following:

1. Every inflectional system and every grammatically licensed feature cell in
   the declared recension profiles is represented and implemented.
2. A caller can inflect any lexeme belonging to those systems by supplying a
   complete, validated typed specification containing all non-derivable stems,
   alternants, accent information, restrictions, and lexical identity.
3. Every inflectable lexeme in the locked union of reviewed data sources is
   classified as productively supported, closed irregular, defective,
   indeclinable, ambiguous, disputed, or out of scope, with evidence.
4. The engine distinguishes a historically invalid cell, a category absent from
   a recension, an indeclinable or non-inflectional item, missing caller
   metadata, ambiguity, and a missing implementation. It never collapses these
   states into one generic “unsupported” result.
5. Unattested but rule-licensed output is marked predictive, while an attested
   or normative form retains exact provenance.

Backwards compatibility and breaking semantic-version changes are not concerns.
Prefer a coherent, correct model to compatibility shims.

# Working method

Create a fresh `agent/...` branch from the latest `main`. Read the repository’s
`AGENTS.md` and all nested instructions before changing anything. Inspect the
complete workspace, current status, source manifests, data pipeline, generated
artifacts, public APIs, tests, reports, and architecture. Preserve unrelated
user changes. Record the baseline commands and results before implementation.

At minimum, read and reconcile:

- `docs/MORPHOLOGY_SPEC.md` and the Old Church Slavonic morphology,
  orthography, data-pipeline, architecture, source, and guard documentation;
- `docs/SYNODAL_MORPHOLOGY.md`, `docs/SYNODAL_ORTHOGRAPHY.md`, and every
  Synodal engine audit;
- `docs/SOURCES.md`, `data/SOURCES.toml`, `references/SOURCES.toml`, and all
  attribution or licensing files;
- the Old Church Slavonic and Synodal core, facade, dictionary, extractor,
  generated registry, and `xtask` code;
- all morphology prompts and completion/audit reports, especially
  `SYNODAL_V10_PRODUCTIVE_MORPHOLOGY_AND_LEXICON_PROMPT.md` and its resulting
  audit;
- the capability matrices, lexeme metadata, principal parts, restrictions,
  accent paradigms, exact forms, evidence tables, and rejected-row reports;
- all public-API, rule, extraction, generation, Unicode, no-panic,
  no-default-feature, and WebAssembly tests.

Do not treat old reports as proof of current behavior. Regenerate or independently
verify their claims.

Work in coherent checkpoints. For each checkpoint:

1. select the highest-impact incomplete grammatical system or source frontier;
2. research and document its source contract;
3. update the machine-readable grammar inventory;
4. implement the smallest coherent productive model;
5. add exact or irregular lexical data only where productivity cannot apply;
6. add exhaustive and adversarial tests;
7. regenerate deterministic artifacts and reports;
8. run targeted checks, then the affected broader checks;
9. review the complete checkpoint diff and correct confirmed findings;
10. commit the coherent checkpoint locally with an informative message;
11. update the persistent progress log and immediately continue to the next
    incomplete matrix entry without waiting for user prompting.

Do not publish, push, or open a pull request unless the user has separately
authorized it. Local checkpoint commits are permitted by this goal.

# Define completeness before expanding implementation

Create a versioned, machine-readable completion matrix that is the authoritative
inventory of the supported grammatical universe. Add a maintainer command:

```text
cargo xtask morphology-completeness --check
```

The command must independently validate the inventory, implementation registry,
tests, source citations, lexical classifications, and generated artifacts. It
must fail if any category or legal cell is omitted, incompletely specified, or
assigned a non-final state.

Model independent axes explicitly rather than encoding them in prose. Include,
where applicable:

- recension and orthographic profile;
- part of speech and morphological family;
- declension or conjugation class and subclass;
- gender, animacy, person, number including dual, case including vocative,
  definiteness, degree, tense, mood, voice, aspect, finiteness, participle kind,
  short/long form, and agreement features;
- lexical number restrictions, defectiveness, suppletion, stem alternations,
  principal parts, accent paradigm, clitic or auxiliary behavior, and ordered
  variants;
- synthetic versus analytic realization;
- abstract morphological output, canonical orthographic output, and accented
  output;
- exact, irregular, productive, or composed implementation path;
- provenance, stable rule ID, citation, evidence status, and validation fixture.

Each matrix entry must end in exactly one of these final states:

- `productive-complete`;
- `closed-exact-complete`;
- `irregular-exact-complete`;
- `historically-invalid`;
- `absent-from-recension`;
- `not-inflectional`.

At completion there may be no entries labeled `unknown`, `partial`,
`unsupported`, `source-review-open`, `implementation-missing`, or equivalent.
A genuinely productive class may not remain exact-only. A final “invalid,”
“absent,” or “not inflectional” classification must cite evidence and must not
hide an implementation gap.

Generate human-readable capability and gap reports from the same inventory. Do
not maintain a second hand-edited truth source.

# Grammatical scope

Inventory and implement every source-licensed inflectional category for both
recensions, without assuming that a category in one exists in the other.
Include at least:

- nouns: all declensions, subtypes, stem extensions, hard/soft and mixed
  behavior, gender, animacy, singular/dual/plural, case, vocative, count and
  collective behavior, plural-only and singular-only nouns, indeclinables,
  defectives, mobile vowels, alternations, variants, and irregulars;
- adjectives: long and short forms, all agreement cells, possessive and other
  source-defined types, comparison, comparative and superlative behavior,
  irregular comparison, accent, variants, and defectiveness;
- pronouns and determiners: personal, reflexive, possessive, demonstrative,
  relative, interrogative, indefinite, negative, adjectival, substantival,
  clitic, suppletive, and other source-defined paradigms;
- numerals: cardinal, ordinal, collective, fractional, distributive, compound,
  agreement-governing, substantival, adjectival, and irregular types wherever
  the recension recognizes them;
- verbs: every conjugation and lexical class; present and future systems;
  imperfect, every licensed aorist type, perfect and pluperfect constructions;
  imperative, conditional/subjunctive and other moods; active, middle/reflexive,
  and passive constructions; infinitive, supine where licensed, l-participle;
  all tense/voice participles; short and declined participles; aspect and
  defectiveness; dual; principal-part alternations; suppletive, root, athematic,
  irregular, and impersonal verbs; ordered orthographic and morphological
  variants;
- participles and verbal adjectives: formation and complete adjectival
  declension, agreement, short/long realization, tense/voice identity,
  negation or reflexive behavior where grammatically part of the form, and
  independent provenance through generation;
- productive verbal nouns or other deverbal grammatical formations only where
  the target grammar treats their formation as a defined morphological system;
- analytic grammatical forms: typed multi-token results with auxiliaries,
  participles, particles, word-order or agreement constraints, and component
  provenance. Do not force phrase-valued forms through a single-token API;
- invariant parts of speech: classify them explicitly as `not-inflectional` or
  indeclinable so their presence cannot be mistaken for missing coverage.

Derivational morphology and unrestricted word formation are outside the claim
unless a reviewed target grammar makes a formation part of a closed grammatical
paradigm. Document this boundary precisely.

# Known open seams to resolve

Treat the following as starting hypotheses from the current repository, verify
them against current code and sources, and close every confirmed gap.

For Old Church Slavonic, investigate and resolve at least:

- primary and secondary sigmatic aorists currently represented but not fully
  productive;
- contracted and uncontracted imperfect policies that remain exact-only;
- imperative formation and ordered variants;
- irregular, root, athematic, and suppletive verbs;
- incomplete participial stem seams and all four participle kinds;
- the large rejected set of declined-participle rows whose local evidence does
  not preserve tense or voice;
- exact forms that should become typed productive rules;
- analytic verb phrases outside the existing word inflector;
- complete pronoun, determiner, numeral, comparison, accent, and irregular noun
  inventories;
- every legal dual and vocative cell;
- ambiguity created by manuscript, editorial, dialectal, or orthographic
  variants.

For Synodal Russian Church Slavonic, investigate and resolve at least:

- short superlatives: either implement the target-recension contract or prove
  and encode that the category is not licensed;
- the open source review for the supine;
- productive verbal-noun behavior;
- simple future and underspecified finite-past behavior that is exact-only;
- broader pronoun, determiner, and numeral paradigms;
- remaining noun and adjective families, including independently evidenced
  `любовь` alternants;
- remaining accent paradigms and explicit handling of unknown lexical stress;
- abbreviations, titla, numeral notation, wide-letter and initial-letter
  alternations, breathing, combining-mark order, and other orthographic seams;
- irregular and suppletive verbs;
- wider analytic constructions and their agreement rules;
- lexical restrictions, number inventories, ordered normative variants, and
  defectiveness currently hidden in exact data.

Search the repository for every occurrence of `TODO`, `FIXME`, `unsupported`,
`partial`, `unknown`, `source review`, rejected categories, exact-only capability
states, wildcard fallbacks, guessed metadata, and error variants. Classify each
one as an implementation gap, a source gap, an intentionally invalid request, or
unrelated work. The completion checker must account for all morphology-related
items.

# Research and source-frontier contract

The phrase “all known sources” must be implemented as a reproducible,
source-bounded research process, not an unverifiable claim of omniscience.
Create a source-frontier ledger containing for every considered source:

- stable bibliographic identity, title, author/editor, edition or revision,
  publication date, publisher or institution, and stable URL or catalog ID;
- retrieval date, pinned revision/version, local filename where applicable, and
  cryptographic hash;
- language, historical period, recension, orthography, and geographic scope;
- source type and epistemic role: normative grammar, descriptive grammar,
  critical edition, dictionary, manuscript witness, corpus, teaching grammar,
  encoding standard, or secondary discussion;
- authority tier and justification;
- license, redistribution constraints, and attribution requirements;
- derivation or data lineage, so mirrors and corpora derived from the same
  dictionary or edition are not counted as independent confirmation;
- reviewed sections, extracted grammatical claims, affected rule IDs, conflicts,
  decisions, and remaining access limitations.

Search in English, Russian, Church Slavonic, German, French, and other relevant
scholarly languages. Follow bibliographies backward, search institutional
catalogs and scholarly repositories, and record excluded or inaccessible
sources. Include at minimum, after verifying editions and relevance:

- the grammar and grammatical dictionaries in Polivanova et al., *Old Church
  Slavic*, designed around the oldest Old Church Slavonic manuscripts:
  <https://books.fupress.com/catalogue/old-church-slavic/8465>;
- the LMU *Lexicon of Old Church Slavonic Verbs* (LOVe), including its SJS/LIV
  lineage and its aorist/present-stem and aspect metadata:
  <https://www.punco.slavistik.lmu.de/love.php>;
- the University of Texas *Old Church Slavonic Online* grammatical lessons:
  <https://lrc.la.utexas.edu/eieol_toc/ocsol>;
- the canonical Old Church Slavonic PROIEL treebank and its documented limits:
  <https://universaldependencies.org/treebanks/cu_proiel/index.html>;
- native PROIEL/TOROT/Syntacticus data when conversion loses a needed
  distinction: <https://github.com/syntacticus/syntacticus-treebank-data>;
- Corpus Cyrillo-Methodianum Helsingiense and other lawfully accessible primary
  OCS editions or corpora;
- Alypy/Gamanovich as the principal currently pinned Synodal grammatical anchor:
  <https://www.ponomar.net/files/gama2/toc.html>;
- the Russian National Corpus Church Slavonic collection, while treating its
  partly automatic annotation and limited homonym disambiguation as evaluation
  evidence rather than a grammar oracle:
  <https://ruscorpora.ru/en/corpus/orthlib>;
- the Church Slavonic Bible and other stable target-recension liturgical or
  scriptural editions already named in the repository;
- GORAZD, D’yachenko, Ponomar resources, and every source already present in the
  repository manifests;
- Unicode Technical Note 41 and current Unicode data for Church Slavonic
  encoding, typography, collation, and numeral behavior:
  <https://www.unicode.org/notes/tn41/>.

Do not assume this seed list is exhaustive. Expand it through citation chaining
and catalog discovery. Perform at least two documented source-discovery passes
after the seed inventory is implemented. Source-bounded convergence is reached
only when two consecutive, independently recorded passes find no newly eligible,
accessible source that changes the grammatical inventory, rule contract,
conflict analysis, or validation set.

Use this default evidence hierarchy, documenting justified exceptions:

1. target-recension normative grammars and critical editions;
2. manuscript- or edition-grounded historical dictionaries;
3. independent target-recension primary texts;
4. manually annotated corpora;
5. inherited or comparative evidence explicitly labeled as such;
6. automatically annotated corpora, crowd-edited sources, OCR, generated tables,
   and mirrors, used as candidate or evaluation evidence only.

Grammar sources define productive rules. Lexical sources define identities,
principal parts, restrictions, and exceptions. Corpora attest actual surface
forms and expose counterexamples, but corpus absence never proves that a legal
form is impossible. A generated dictionary table is not independent validation
of a rule reverse-engineered from that table.

Retain contradictions instead of silently selecting the convenient form. Encode
ordered variants where the source licenses both; otherwise require an explicit
profile or return a structured ambiguity/dispute. Never transfer an Old Church
Slavonic surface form directly into Synodal Russian Church Slavonic, or vice
versa, merely because the systems are historically related.

Respect licenses. Do not compile restricted corpora into runtime crates, commit
prohibited text, or reproduce copyrighted sources beyond permitted quotation.
Keep external evaluation inputs outside normal builds, pin their versions and
hashes, and commit only lawful fixtures, aggregate metrics, provenance, and
reproducible import instructions.

# Productive model and arbitrary-lexeme API

Build a grammatical model, not a suffix collection and not an ever-growing exact
form dump. Independent grammatical dimensions must be independent closed Rust
types. Every productive rule must have:

- a stable rule ID and target recension/profile;
- a precise source citation and epistemic status;
- a closed input contract;
- independently supplied lexical principal parts wherever they cannot be safely
  derived;
- a complete valid-cell inventory and explicit invalid/absent cells;
- deterministic ordered variants;
- an accent and orthography contract;
- provenance-preserving traces;
- exhaustive representative goldens, boundary tests, contradictory-metadata
  tests, and typed failure behavior.

Provide strict typed specifications for arbitrary nouns, adjectives, pronouns,
determiners, numerals, verbs, participles, and any other inflecting classes. The
strict API must never guess lexical identity or class from spelling. It must
validate contradictory or incomplete metadata and report all missing fields
precisely.

An optional exploratory analyzer may return multiple ranked analyses for a bare
form, but it must expose ambiguity, evidence, and assumptions. It must never feed
a silent guess into the strict generator. The public API must make the difference
between strict generation and exploratory analysis unmistakable.

Retain or improve an injectable lexicon/provider abstraction so applications can
supply an open lexicon without rebuilding runtime crates. Generated static data
and caller-provided data must use the same validated specification and productive
kernel. Provider composition, identity conflicts, and precedence must be stable,
typed, deterministic, and documented.

Use this resolution principle unless a more precise documented design supersedes
it:

```text
normative exact cell > lexical irregular override > licensed productive rule
```

Do not let a broad override mask a more specific exact cell. Preserve all
evidence and trace steps across resolution. Do not duplicate generation logic in
the dictionary, extractor, facade, or provider layer.

For every inflectable lexeme in the locked source union, record one of:

- productive specification, including every required principal part and
  restriction;
- closed irregular or suppletive paradigm;
- explicitly defective paradigm;
- explicitly indeclinable item;
- evidence-backed ambiguity or source dispute, with all licensed analyses;
- excluded item with a machine-readable reason showing it is outside the target
  recension or inflectional scope.

No known lexeme may remain silently rejected, guessed, or classified only by a
surface suffix.

# Orthography, accent, and Unicode

Keep abstract morphology separate from surface realization. Support documented
profiles for at least:

1. abstract morphological segments/features;
2. canonical target-recension orthography;
3. fully accented/marked output when complete lexical accent metadata exists;
4. any explicitly supported diplomatic or normalized profile.

Do not invent stress for an unaccented lexeme. Unknown lexical accent may remain
an explicit `missing metadata` condition for the fully accented profile, but it
must not block abstract or unaccented canonical morphology when those are
otherwise determined. This is a lexical-input boundary, not an incomplete
grammar state.

Specify normalization, combining-mark order, breathing/accent interaction,
titla, abbreviations, numerals, initial/wide-letter selection, casing, yer and
other alternations, token boundaries, and variant ordering. Test composed and
decomposed Unicode, hostile input, and canonically equivalent strings. Preserve
source spelling where a diplomatic profile requires it; never erase meaningful
variants through normalization.

# Analytic forms

Represent analytic grammatical forms as typed structured results, not as opaque
strings. A result must identify its components, agreement/control relations,
allowed word-order variants, auxiliary or particle lemmas, surface tokens,
orthographic profile, and provenance for every component and composition rule.

Keep the single-word inflector focused on word forms, but expose a coherent
paradigm API capable of returning both synthetic and analytic realizations.
Document when multiple tokens are obligatory, optional, or competing variants.

# Data, generation, and architecture constraints

Preserve these invariants unless a documented full-diff review establishes a
strictly better architecture:

- core crates are pure, deterministic, panic-free, filesystem-free, and
  network-free morphology/orthography/Unicode engines;
- facade crates own lexical identity, providers, provenance, exact and irregular
  overrides, accent metadata, and resolution policy;
- dictionaries remain semantic lookup and analysis layers, not second
  morphology engines;
- extractors and `xtask` own offline ingestion, validation, generation,
  evaluation, and reports;
- runtime crates perform no implicit file, database, environment, locale, clock,
  randomness, or network access;
- no `unsafe`, production `unwrap`/`expect`, silent lossy conversion,
  nondeterministic iteration, or unbounded recursion is introduced;
- native no-default-feature and `wasm32-unknown-unknown` compatibility remain
  first-class;
- generated artifacts are byte-deterministic and reconstructible offline from
  pinned lawful inputs;
- all foreign keys, identities, rule IDs, recensions, feature codes, evidence
  IDs, citations, principal parts, accent scopes, duplicate rows, overlaps, and
  conflicts fail closed during generation;
- attestations, exact normative forms, irregular overrides, inherited forms,
  predictions, and disputes never lose their provenance;
- table size is not used to imitate productivity. Convert repeated exact
  paradigms to productive metadata only after a source-backed class contract is
  established.

Public paradigm and batch APIs must preserve request order, variant order, all
successes and failures, typed error codes, evidence, and trace information.
Complete-paradigm requests must visibly include invalid, absent, defective,
ambiguous, and missing-metadata cells rather than dropping them.

# Validation strategy

Use multiple independent validation layers:

- exhaustive generated tests across every legal matrix cell and every
  productive rule/class;
- complete gold paradigms chosen directly from grammatical sources;
- held-out lexemes not used to derive or tune the rule;
- lexical irregular and defectiveness fixtures;
- exact-before-irregular-before-productive precedence tests;
- cross-recension isolation tests that prohibit fallback or evidence leakage;
- property tests for syncretism, agreement, dual inventories, animacy,
  principal-part requirements, round trips, normalization, deterministic
  ordering, and trace completeness;
- adversarial tests for malformed Unicode, contradictory metadata, duplicate
  evidence, ambiguous identity, and panic freedom;
- corpus evaluation against independently sourced forms, reporting coverage,
  disagreements, ambiguity, and provenance without redefining grammar by token
  frequency;
- deterministic regeneration and offline reconstruction tests;
- public-API and serialization round trips, including provider composition;
- native, feature-combination, no-default-feature, documentation, and WebAssembly
  builds.

For every productive rule, validation must include at least one complete source
golden and one held-out lexeme if an independent example exists. If the historical
record lacks an attested example for a valid cell, test a labeled predictive form
derived from the rule and state that absence of attestation is not confirmation.

Corpus matches are supporting evidence. Corpus mismatches are counterexamples to
investigate. Corpus absence is not a failure. Keep evaluation datasets and
training/derivation sources separate enough to prevent circular validation.

# Documentation and auditability

Keep documentation synchronized with implementation. Update the relevant
morphology, orthography, data-pipeline, architecture, source, attribution, README,
and capability documents whenever a contract changes.

Maintain a persistent progress document generated from or cross-checked against
the machine-readable inventory. It must show:

- total and final-state counts by recension, part of speech, system, and rule;
- every remaining non-final entry while work is in progress;
- lexeme-union classification counts and unresolved conflicts;
- source frontier, last two discovery passes, and access/license limitations;
- implemented checkpoints and verification results;
- corpus and held-out evaluation summaries;
- exact tables converted to productive metadata;
- current known risks and next work item.

Generate final recension-specific audits and one cross-recension completion
audit. Every numeric statement must be reproducible by a checked-in command.

# Mandatory final verification

Before declaring completion, run and pass at least:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
cargo test --workspace --doc
cargo doc --workspace --no-deps
cargo xtask morphology-completeness --check
cargo xtask synodal-engine-audit --check
cargo xtask synodal-check
cargo xtask check-all
```

Also run every relevant repository-specific generator, extractor fixture,
linguistic evaluation, package check, public-API test, feature matrix,
no-default-feature native build, `wasm32-unknown-unknown` build, documentation
test, publish dry run, offline reconstruction, and source-manifest validation.
If a named command has legitimately been replaced, document and run its current
equivalent; do not silently skip it.

Run deterministic generation twice from clean temporary output locations and
prove byte-identical results. Run `git diff --check`. Inspect the complete diff
against the merge base, including staged, unstaged, generated, deleted, and
untracked files. Scan production code for new `unsafe`, panic paths,
`unwrap`/`expect`, implicit I/O boundaries, nondeterministic iteration, and
cross-recension fallback.

Have a fresh independent reviewer that did not implement the work review the
entire intended change set for linguistic correctness, source circularity,
regressions, unsafe edge cases, API coherence, determinism, license compliance,
and missing tests. Validate each finding against current code. Fix every
confirmed P0/P1 issue and every in-scope lower-severity issue; document rejected
findings with evidence. Rerun affected checks and repeat independent review until
no confirmed P0/P1 finding remains.

If a pull request is later authorized, follow `AGENTS.md`’s PR completion gate,
inspect all required CI checks and unresolved actionable review threads, and do
not claim completion while required checks are pending or failing.

# Completion contract

Do not stop until all of these are true:

1. The machine-readable inventory enumerates every source-licensed inflectional
   system and feature cell for both declared recensions.
2. Every inventory entry has one permitted final state, citation, rule or
   rationale, validation reference, and deterministic implementation path.
3. No legal grammatical form returns a generic unsupported or
   implementation-missing result when its strict specification is complete.
4. Every productive class works for arbitrary caller-supplied lexemes with
   complete typed metadata, not only for registered examples.
5. Every inflectable lexeme in the locked source union has a final
   evidence-backed classification and no source row is silently discarded.
6. Exact data is reserved for genuinely closed, normative, irregular,
   suppletive, or lexeme-specific behavior rather than substituting for a
   productive rule.
7. All noun, adjective, comparison, pronoun, determiner, numeral, verb,
   participle, and other licensed paradigms are complete, including dual,
   vocative, variants, restrictions, and defectiveness where applicable.
8. Every licensed analytic construction has a typed structured realization and
   complete component provenance.
9. Old Church Slavonic and Synodal Russian Church Slavonic remain independently
   modeled, sourced, tested, and unable to fall back into one another.
10. Abstract, canonical orthographic, and fully accented output contracts are
    complete; missing lexical accent is explicit metadata, never guessed.
11. Every predicted unattested form is labeled predictive and cannot be
    mistaken for an attestation.
12. All ambiguities and source conflicts are preserved as structured evidence or
    resolved with a documented authority rule.
13. Two consecutive documented source-discovery passes find no new eligible,
    accessible source that changes the inventory, rule contracts, conflict
    decisions, or validation set.
14. All source versions, hashes, lineage, licenses, and attribution requirements
    are recorded and reproducible.
15. Exhaustive cell tests, complete goldens, held-out lexemes, properties,
    adversarial tests, corpus evaluation, and public-API tests pass.
16. Runtime code remains deterministic, offline, panic-free, no-default-feature
    compatible, and WebAssembly compatible, with no new unsafe or implicit I/O.
17. Generated data and reports reproduce byte-for-byte in two clean runs from
    pinned inputs.
18. Every mandatory final verification command and repository-specific check
    passes.
19. A full independent review finds no unresolved confirmed P0/P1 issue, and all
    other in-scope findings are fixed or explicitly justified.
20. The final audits contain no `unknown`, `partial`, `unsupported`,
    `source-review-open`, `implementation-missing`, unexplained exact-only
    productive class, unclassified known lexeme, or unaccounted
    morphology-related TODO.

Do not weaken, rename, or delete a completion criterion merely to make the check
pass. Do not declare success based on corpus percentage, test count, release
number, elapsed time, context length, or a large exact-form table.

# Genuine blockers

If an essential grammar, edition, dictionary, corpus, license determination, or
external service is inaccessible or requires payment/permission, exhaust lawful
open sources, catalogs, mirrors, interlibrary metadata, repository evidence, and
alternative validation paths first. Continue all work not dependent on the
blocker. Do not fabricate evidence and do not mark the affected matrix entry
complete.

Only stop as blocked when the exact inaccessible item is necessary to choose
between materially different grammatical contracts and no safe source-backed
implementation is possible. Report the precise source/decision, why it is
essential, every attempted alternative, completed unaffected work, the exact
matrix entries still non-final, and the minimal user action needed.

# Final handoff

When and only when the completion contract is satisfied, provide a concise final
report containing:

- the grammatical universe and recension profiles completed;
- matrix and lexeme-classification totals by final state;
- major productive systems and irregular inventories added;
- source frontier and the evidence that discovery converged;
- unattested predictive-form policy and remaining inherent lexical-input limits;
- verification commands with results;
- deterministic regeneration and corpus/held-out evaluation results;
- independent-review scope, findings fixed, and findings rejected with rationale;
- branch and commit summary;
- CI/review state if a PR was separately authorized;
- any residual risk that does not contradict the completion contract.

Until then, keep implementing, validating, documenting, reviewing, and advancing
to the next non-final entry automatically.
