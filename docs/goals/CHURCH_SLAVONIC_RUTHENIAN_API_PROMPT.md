# Make the Old Church Slavonic API as approachable as the Ruthenian API

Work in the existing `church-slavonic` Rust workspace. Redesign the published
`old-church-slavonic` facade so its ordinary API has the clarity and directness
of `../ruthenian/crates/ruthenian-core`, while preserving the linguistic honesty
that the Old Church Slavonic project gets from dictionary evidence, ordered
variants, explicit lexical metadata, provenance, and typed failure.

Backwards compatibility and breaking SemVer are not concerns. Prefer one coherent
API over aliases that permanently expose both old and new designs. Do not publish
new crate versions, create releases, or push changes unless separately requested.

The desired experience is:

```rust
use old_church_slavonic::{
    noun, verb, Case, Number, Person,
};

let dual_dative = noun("обѣдъ", Case::Dative, Number::Dual)?;
assert_eq!(dual_dative.primary_text(), "обѣдома");

let first_singular = verb(
    "благословити",
    Person::First,
    Number::Singular,
)?;
assert_eq!(first_singular.primary_text(), "благословлѭ");
# Ok::<(), old_church_slavonic::InflectionError>(())
```

Repeated use should be similarly compact:

```rust
use old_church_slavonic::{Case, Noun, Number, Person, Verb};

let meal = Noun::new("обѣдъ")?;
assert_eq!(
    meal.form(Case::Dative, Number::Dual)?.primary_text(),
    "обѣдома",
);

let bless = Verb::new("благословити")?;
assert_eq!(
    bless.present(Person::First, Number::Singular)?
        .primary_text(),
    "благословлѭ",
);
# Ok::<(), old_church_slavonic::InflectionError>(())
```

Do **not** imitate Ruthenian by returning an unconditional `String` or a sentinel
such as `?`. Ruthenian is a rule-only specified language in which every permitted
input is deliberately total. Old Church Slavonic is an attested historical
language whose evidence can contain homographs, multiple variants, competing
analyses, incomplete principal parts, unsupported formations, and genuinely
invalid historical cells. Those distinctions must remain visible.

The objective is Ruthenian-like ergonomics, not Ruthenian-like information loss.

---

## 1. Audit both public APIs before editing

Read the complete relevant API and design material, including at least:

- `../ruthenian/DIRECTION.md`;
- `../ruthenian/crates/ruthenian-core/{README.md,src/lib.rs}`;
- Ruthenian's `grammar.rs`, `noun.rs`, `adjective.rs`, `verb.rs`, `pronoun.rs`,
  `numeral.rs`, `fallback.rs`, public examples, conformance tests, and guards;
- the root and crate READMEs in this workspace;
- `crates/old-church-slavonic/src/{lib,lookup,metadata,paradigm}.rs`;
- all modules in `crates/old-church-slavonic-core/src/`;
- `crates/old-church-slavonic/tests/public_api.rs`;
- `docs/{ARCHITECTURE,MORPHOLOGY_SPEC,ORTHOGRAPHY,DATA_PIPELINE,GUARDS}.md`;
  and
- current accuracy, extraction, corpus, and verb-metadata reports.

Inventory every currently public item in both publishable crates. Record which OCS
items are ordinary caller-facing morphology, which are dictionary-resolution
operations, and which exist for extraction, evaluation, explicit metadata, or
debugging. Identify public names that exist only because
`pub use old_church_slavonic_core::*` exposes the entire core at the facade root.

Write a short design note before implementation containing:

1. the current public signatures;
2. the intended root signatures;
3. items moving to an advanced namespace;
4. deliberate semantic differences from Ruthenian; and
5. examples of source variants or errors that would be lost by returning `String`.

Run the full baseline checks and record their results. The redesign must not change
dictionary forms, variant ordering, lookup ambiguity, metadata provenance,
productive morphology, or accuracy metrics merely to simplify call syntax.

---

## 2. Apply the transferable Ruthenian API principles

Use these Ruthenian principles where they fit Old Church Slavonic:

1. A normal call is a lemma followed by the grammatical dimensions of the
   requested form.
2. Common grammar enums are the interface; callers should not need to construct a
   cell struct for a one-off request.
3. A selector that chooses a distinct paradigm should normally become a named
   function or object method.
4. Repeated calls can bind lexical identity once in a lightweight object.
5. A paradigm enumerator must call the same cell resolver as the one-cell API.
6. The crate root should present a curated ordinary API, not every implementation
   and evaluation type.
7. Every public function needs a useful rustdoc example.

Do not transfer these Ruthenian decisions:

- unconditional totality;
- `String` as the only result type;
- a plausible fallback for a historically nonexistent cell;
- automatic class inference where OCS lexical evidence does not justify it;
- flattening several attested variants to one string; or
- treating a participle citation as an ordinary adjective stem when its oblique
  formation requires independent verbal metadata.

Old Church Slavonic's common `Case`, `Number`, `Gender`, `Animacy`, and `Person`
types already align well with Ruthenian. Retain real additional dimensions such as
`FiniteTense`: OCS has multiple synthetic finite past systems, so it is not the
single-value pseudo-dimension that Ruthenian removed.

---

## 3. Replace ordinary cell-struct calls with direct typed arguments

Make the ordinary facade root expose signatures equivalent to the following.
Names may change only when a documented OCS linguistic distinction makes the
shown name misleading.

```rust
pub type InflectionResult = Result<FormSet, InflectionError>;

pub fn noun(
    lemma: &str,
    case: Case,
    number: Number,
) -> InflectionResult;

/// Long/definite adjective.
pub fn adjective(
    lemma: &str,
    case: Case,
    number: Number,
    gender: Gender,
    animacy: Animacy,
) -> InflectionResult;

/// Short/indefinite adjective.
pub fn short_adjective(
    lemma: &str,
    case: Case,
    number: Number,
    gender: Gender,
    animacy: Animacy,
) -> InflectionResult;

/// Present indicative, matching Ruthenian's ordinary `verb` entry point.
pub fn verb(
    lemma: &str,
    person: Person,
    number: Number,
) -> InflectionResult;

pub fn imperfect(
    lemma: &str,
    person: Person,
    number: Number,
) -> InflectionResult;

pub fn aorist(
    lemma: &str,
    person: Person,
    number: Number,
) -> InflectionResult;

pub fn finite_verb(
    lemma: &str,
    tense: FiniteTense,
    person: Person,
    number: Number,
) -> InflectionResult;

pub fn imperative(
    lemma: &str,
    person: Person,
    number: Number,
) -> InflectionResult;

pub fn l_participle(
    lemma: &str,
    gender: Gender,
    number: Number,
) -> InflectionResult;

pub fn infinitive(lemma: &str) -> InflectionResult;
pub fn supine(lemma: &str) -> InflectionResult;
pub fn verbal_noun(lemma: &str) -> InflectionResult;
pub fn comparative(lemma: &str) -> InflectionResult;
```

Each convenience function must construct the internal typed cell and immediately
delegate to the one authoritative resolver. Do not create a second generation
path. For example, `verb` delegates to `finite_verb` with `FiniteTense::Present`;
`imperfect` and `aorist` do the corresponding thing.

Use `adjective` for the long paradigm and `short_adjective` for the short paradigm,
following Ruthenian's function-selection rule. Move the generic form-selecting
operation to an advanced API such as `advanced::adjective_form` if it still has a
real consumer.

Keep cell structures available for paradigms, generic tooling, and advanced calls,
but ordinary callers must not need `NounCell`, `AdjectiveCell`, `FiniteVerbCell`,
`ImperativeCell`, or `LParticipleCell` for these common requests.

---

## 4. Add resolved `Noun`, `Adjective`, and `Verb` handles

Add facade-level lexical handles analogous to Ruthenian's `Noun` and `Adjective`,
but make construction honest about dictionary resolution.

```rust
pub struct Noun { /* resolved immutable identity */ }

impl Noun {
    pub fn new(lemma: &str) -> Result<Self, InflectionError>;
    pub fn from_id(id: &str) -> Result<Self, InflectionError>;
    pub fn lemma(&self) -> &str;
    pub fn id(&self) -> &str;
    pub fn form(&self, case: Case, number: Number) -> InflectionResult;
    pub fn paradigm(&self) -> NounParadigm;
}
```

Provide parallel objects:

- `Adjective::{new,from_id,lemma,id,long,short,comparative,paradigm}`;
- `Verb::{new,from_id,lemma,id,present,imperfect,aorist,finite,imperative,
  infinitive,supine,verbal_noun,l_participle,participle,finite_paradigm}`.

Construction by lemma resolves exactly one dictionary lexeme of the required part
of speech. It must return `UnknownLemma` or `AmbiguousLexeme` rather than storing a
guess. `from_id` validates that the ID exists and has the correct part of speech.

Store only stable resolved identity needed to avoid repeated ambiguous lookup. Do
not duplicate mutable class, stem, formation, or source facts into these handles;
those must continue to come from the authoritative generated registry and metadata
resolver. Handles must be cheap to clone and require no runtime file or network I/O.

The methods must call the same by-ID resolver as the free functions. Add guards
proving that free-function, handle-method, by-ID, and paradigm results remain equal
for the same lexeme and cell.

Keep explicit productive caller metadata separate. `NounLexeme`,
`AdjectiveLexeme`, `VerbLexeme`, their builders, and `*_with` operations belong in
the advanced rule API; they are not substitutes for resolved dictionary handles.

---

## 5. Give participles Ruthenian-like names without discarding OCS metadata

Replace ordinary `ParticipleKind` selection with named operations:

- `present_active_participle`;
- `present_passive_participle`;
- `past_active_participle`; and
- `past_passive_participle`.

Unlike Ruthenian, do not return a bare `String` and tell callers to pass it through
ordinary adjective declension. OCS participle analyses may have multiple citations,
distinct oblique stems, formation-specific alternations, and source-backed
metadata. Preserve that state in a resolved `Participle` handle or an equally typed
design:

```rust
let participle = verb.past_active_participle()?;

let short = participle.short(
    Case::Genitive,
    Number::Singular,
    Gender::Masculine,
    Animacy::Inanimate,
)?;

let long = participle.long(
    Case::Genitive,
    Number::Singular,
    Gender::Masculine,
    Animacy::Inanimate,
)?;
```

The handle should expose `kind`, `citation`, `short`, `long`, and `paradigm`.
Named constructors may return several ordered analyses inside `FormSet`, but must
not silently pick one formation. Keep the generic `ParticipleCell` resolver in the
advanced namespace.

If a handle cannot be introduced cleanly in this change, named functions may take
the agreement dimensions directly. Do not create eight unrelated long/short/kind
functions unless the object design is demonstrably worse.

---

## 6. Make successful forms nonempty and easy to consume

Preserve `FormSet` or rename it to a clearer name such as `Forms`, but enforce the
invariant that every successful result contains at least one variant. The current
public `Vec<FormVariant>` permits an empty successful set and forces
`primary_source_order()` to return `Option` even though the resolver should never
produce one.

Prefer a representation such as:

```rust
pub struct FormSet {
    primary: FormVariant,
    alternatives: Vec<FormVariant>,
    // source, warnings, trace, analyses, and other existing evidence
}
```

Expose read-only conveniences:

```rust
impl FormSet {
    /// The first variant in deterministic source order; not a claim of
    /// linguistic superiority.
    pub fn primary(&self) -> &FormVariant;
    pub fn primary_text(&self) -> &str;
    pub fn variants(&self) -> impl ExactSizeIterator<Item = &FormVariant>;
    pub fn texts(&self) -> impl ExactSizeIterator<Item = &str>;
    pub fn into_primary_text(self) -> String;
}
```

Retain access to romanization, source, warnings, evidence, traces, and analyses.
Document that `primary` means deterministic source order only. Do not implement an
implicit conversion, `Deref<Target = str>`, or a `Display` implementation that
quietly hides alternatives.

Preserve the current distinction between:

- exact dictionary forms;
- dictionary-metadata generation;
- explicit caller metadata;
- OOV prediction;
- curated override;
- multiple morphological analyses; and
- unsupported or historically invalid requests.

Add a test using the two attested aorists of `бꙑти` and another using a declined
participle with multiple metadata analyses. Both must retain their complete ordered
variants through every new convenience layer.

---

## 7. Make paradigms lemma-oriented and consistent

The current facade uses a lemma for one-cell calls but a lexeme ID for functions
named `noun_paradigm`, `adjective_paradigm`, and the verb paradigms. Correct that
surprise:

```rust
pub fn noun_paradigm(lemma: &str) -> Result<NounParadigm, InflectionError>;
pub fn noun_paradigm_by_id(id: &str) -> Result<NounParadigm, InflectionError>;
```

Apply the same convention to adjectives, finite verbs, imperatives,
l-participles, and declined participles.

Add Ruthenian-like present-only `verb_paradigm(lemma)` if it is useful for the
ordinary nine person-number cells. Keep a clearly named `finite_verb_paradigm`
for all present, imperfect, and aorist cells.

Every paradigm type must consistently provide:

- `lemma()` and `id()`;
- `get(...)` using direct grammatical dimensions where ergonomic;
- `iter()`;
- `IntoIterator` for borrowed and owned values when useful; and
- deterministic documented cell order.

Per-cell errors must remain visible. Do not omit unsupported cells to make a
paradigm appear total. A paradigm constructor may succeed after resolving the
lexeme while individual `CellOutcome` values contain typed failures.

Prove that every paradigm cell invokes the same public resolver as a direct call.
There must be one generation path.

---

## 8. Curate the facade root and move specialist APIs

Remove the blanket facade export:

```rust
pub use old_church_slavonic_core::*;
```

Replace it with explicit exports. The facade root should contain primarily:

- `Case`, `Number`, `Gender`, `Animacy`, `Person`, and `FiniteTense`;
- `PartOfSpeech` where lookup needs it;
- the simple free functions;
- `Noun`, `Adjective`, `Verb`, and `Participle`;
- `FormSet`, `FormVariant`, `InflectionError`, and the common warning/source
  types;
- lemma-oriented paradigms and lookup; and
- a small optional `prelude` if it materially improves examples.

Move specialist operations behind intentional modules, for example:

```text
old_church_slavonic::advanced::cells
old_church_slavonic::advanced::metadata
old_church_slavonic::advanced::rules
old_church_slavonic::advanced::by_id
old_church_slavonic::advanced::raw_features
old_church_slavonic::trace
```

The exact module tree may differ, but the root must no longer mix a beginner's
`noun` call with extraction/evaluation types such as metadata formation policies,
normalized metadata fields, generic string feature keys, and leakage-controlled
evaluation entry points.

Keep `old-church-slavonic-core` usable for explicit rule-based callers. Give its
exports the same intentional organization, but do not pretend an OCS citation form
contains lexical class facts that actually require caller metadata.

Avoid public functions whose names differ only by unexplained suffixes. Use these
conventions consistently:

- no suffix: normal lemma API;
- `_by_id`: stable dictionary identity API;
- `_with`: explicit caller-supplied lexical metadata;
- `_from_dictionary_metadata`: specialist audited/evaluation API, advanced only.

---

## 9. Do not fake missing Ruthenian features

Ruthenian has compact typed entry points for personal pronouns and arbitrary
numeric cardinal/ordinal generation. OCS currently relies heavily on dictionary
closed-class cells and does not have equivalent productive coverage.

Do not add a superficially matching `numeral(u64, ...)` or feature-only pronoun API
unless the actual morphology and lexical mapping are independently implemented and
tested. It is acceptable for the first redesigned API to leave the existing
closed-class resolver in `advanced` and document this remaining difference.

Typed convenience functions for pronouns, reflexives, determiners, or numerals may
be added only when they map losslessly to current dictionary identities and cells.
Do not use `Option<Gender>` and `Option<Person>` throughout the ordinary root merely
to wrap the current generic `ClosedClassCell`.

Similarly, a separate validated orthography/transliteration crate like
`ruthenian-orthography` is a possible later project, not a prerequisite for this
API redesign. Do not introduce lossy Cyrillic/Glagolitic transliteration to claim
surface similarity.

---

## 10. Tests and guards

Add compile-and-run public API tests demonstrating at least:

1. one-line noun, long adjective, short adjective, present verb, imperfect,
   aorist, imperative, l-participle, infinitive, and supine calls;
2. `Noun`, `Adjective`, and `Verb` construction and repeated use;
3. lemma and by-ID calls producing identical results;
4. handle methods producing identical results to free functions;
5. every paradigm cell producing the same result as the corresponding direct
   call;
6. deterministic source-ordered alternatives surviving `primary_text()` access;
7. ambiguity remaining `AmbiguousLexeme`;
8. an unknown lemma remaining typed rather than becoming `?` or a guess;
9. missing metadata, unsupported formation, and historically invalid cells
   remaining distinct;
10. explicit `*_with` rule generation remaining independent of the dictionary;
11. hostile, empty, oversized, non-NFC, mixed-script, and Glagolitic input remaining
    panic-free and typed; and
12. ordinary examples importing only curated root items.

Add structural guards that fail if:

- the facade restores a blanket core re-export;
- an ordinary function again requires a cell struct;
- a successful `FormSet` can be empty;
- a convenience function bypasses the canonical resolver;
- a paradigm drops failed cells;
- a new root function lacks a rustdoc example;
- dictionary variant ordering changes; or
- the runtime crate gains file, network, JSON, TSV, XML, or Lua access.

Validate each new guard with an injected witness: apply the named mutation, observe
the guard fail for the intended reason, then revert the mutation. Document the
witness in `docs/GUARDS.md`.

Do not compare OCS and Ruthenian surface forms. API-shape tests may use Ruthenian as
a design reference, but all linguistic assertions must use pinned OCS dictionary,
grammar, and corpus evidence already admitted by this repository.

---

## 11. Documentation and migration

Rewrite the facade README and crate-level rustdoc around the first ten lines a user
should write. Lead with simple calls, then resolved objects, then variants and
errors, and only afterward explain advanced metadata and provenance.

Include a concise table mapping the former API to the new API, for example:

| Former call | New ordinary call |
|---|---|
| `noun(lemma, NounCell { case, number })` | `noun(lemma, case, number)` |
| `adjective(lemma, AdjectiveCell { form: Long, ... })` | `adjective(lemma, ...)` |
| `adjective(lemma, AdjectiveCell { form: Short, ... })` | `short_adjective(lemma, ...)` |
| `finite_verb(lemma, Present cell)` | `verb(lemma, person, number)` |
| `finite_verb(lemma, Imperfect cell)` | `imperfect(lemma, person, number)` |
| `noun_paradigm(id)` | `noun_paradigm(lemma)` or `noun_paradigm_by_id(id)` |
| `forms.primary_source_order().unwrap().text` | `forms.primary_text()` |

Document why OCS intentionally continues returning `Result<FormSet, ...>` instead
of Ruthenian's `String`. Show:

- an exact table form;
- two source-ordered variants;
- a metadata-generated form with provenance;
- an ambiguous lemma resolved with `lookup` and `from_id`;
- an unsupported historical cell; and
- an explicit OOV `*_with` call.

Update architecture and morphology documents for the simple/advanced boundary.
Remove stale references to old signatures from every README, example, test, and
report explanation. Do not retain deprecated wrappers solely for compatibility.

---

## 12. Verification and completion criteria

Run, at minimum:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
cargo xtask check-all
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
CARGO_NET_OFFLINE=true cargo package -p old-church-slavonic-core --allow-dirty
CARGO_NET_OFFLINE=true cargo package -p old-church-slavonic --allow-dirty
git diff --check
```

Inspect both package archives. They must include the required licenses,
attribution, README, and generated runtime registry while excluding extractor
inputs, evaluation corpora, detailed corpus observations, and unrelated reports.

The task is complete only when:

- the ordinary calls read like the Ruthenian API while returning honest OCS
  structured results;
- common calls take direct grammatical dimensions rather than cell structs;
- present `verb`, named adjective paradigms, and named verb systems exist;
- resolved `Noun`, `Adjective`, and `Verb` handles work by lemma and ID;
- lemma-oriented paradigm functions and explicit `_by_id` alternatives exist;
- successful form sets are nonempty and have a convenient documented primary
  accessor;
- variants, analyses, romanization, sources, warnings, traces, and errors remain
  intact;
- the facade root is curated and specialist types are deliberately namespaced;
- no morphology, variant order, provenance, accuracy report, or runtime boundary
  regresses;
- all checks, docs, package verification, and injected guard witnesses pass; and
- the final report identifies deliberate differences that remain because OCS is
  attested and dictionary-backed rather than a total rule-only specified language.

Finish with a concise implementation report containing the old/new public API
map, files changed, tests and guards added, commands run, package contents checked,
and any intentionally deferred Ruthenian-like features such as productive numeric
generation or a separate orthography crate.
