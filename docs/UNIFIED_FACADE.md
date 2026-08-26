# Unified facade and dictionary (merge phase 5)

Design and contract for the recension dimension of the `church-slavonic`
facade and the `church-slavonic-dictionary` crate. Contract:
`docs/UNIFIED_LANGUAGE_PROMPT.md`, execution plan step 5, entered after
phase 4 completed (all five POS kernels merged into `church-slavonic-core`
with the named-divergence registry, commits d4b89e0..c204d4f) on top of the
phase-2 identity layer (`docs/UNIFIED_IDENTITY.md`) and the phase-3
projection module (`church-slavonic-orthography::projection`).

## 1. The decision: a `Recension`-scoped handle, not a parameter

Decided once, applied uniformly: the facade gains
`church_slavonic::recension(Recension) -> RecensionScope`, a `Copy` handle
whose methods mirror the facade's function families
(`scope.noun(lemma, case, number)`, `scope.noun_variants(...)`,
`scope.noun_paradigm(lemma)`, ...). The alternative — threading a
`recension` parameter through every free function
(`noun(lemma, case, number, recension)`) — is rejected for three reasons,
weighed against the actual call-site ergonomics of the existing API:

1. **The ruthenian rule.** Recension selects *realization*: it is a
   profile the whole interaction runs under, not a per-cell linguistic
   dimension like case or number. The facade's own design rule ("an
   output-selecting distinction becomes a separate function; an options
   parameter only when unavoidable") already refuses to widen signatures
   for register-like choices; a recension parameter on every one of the
   ~60 public functions would force every call site to repeat a value
   that is constant across the call site's lifetime. The Synodal family's
   own precedent is the same shape: `Inflector` is a `Copy` profile value
   whose methods take only the linguistic dimensions.
2. **Compatibility is structural, not promised.** The existing OCS-only
   public API must keep working byte-identically. With a scoped handle the
   existing free functions are simply *not touched*: they remain the OCS
   surface, and `recension(Recension::OldChurchSlavonic)` delegates to
   them. A parameter would have re-signatured every function and made
   "byte-identical for existing calls" a migration promise instead of a
   compile-time fact. The untouched existing test suite plus the
   `rewrite-pilot-accuracy` oracle gate prove the invariant.
3. **The recension axis is partial today.** Only two of the seven
   `Recension` values are servable, and the Synodal side serves the
   families the merged kernel + registry can realize (below). A scope
   makes the partiality a property of one type's method set and one error
   variant instead of a per-function caveat.

The free functions are therefore documented as the **OCS compatibility
surface**: `church_slavonic::noun(lemma, case, number)` and
`recension(Recension::OldChurchSlavonic).noun(lemma, case, number)` are the
same call. New code that cares about the recension axis uses the scope;
existing code changes nothing.

### Servable recensions

`Recension::OldChurchSlavonic` and `Recension::SynodalRussian`. Every
method of a scope over any other value returns
`Error::UnsupportedRecension` (the scope constructor itself is total and
infallible, so a recension can be threaded through configuration without a
guard at every construction site).

## 2. Lemma resolution: the identity table is the authority

A scope method's `lemma` argument is resolved in exactly two steps:

1. **Abstract identity key** (`docs/UNIFIED_IDENTITY.md`,
   `<pos>:<projection-normal form>[_<ordinal>]` — syntactically
   distinguishable from any native lemma by the `:` separator, which no
   citation surface contains). Where the identity table
   (`data/unified/identity.tsv`) has an entry whose `pos` matches the
   requested function family, the table is the authority: the key resolves
   to the entry's native handle for the scope's recension — the
   `ocs_lemma_key` (facade homograph key) on the OCS side, the
   `synodal_lexeme_id` (Synodal `Inflector` `LexemeId`) on the Synodal
   side. A key with no matching entry falls through to step 2 (where, not
   being a surface form, it fails as unidentified).
2. **Native key of the scope's recension.** OCS: the facade's own lemma
   keys (deterministic numeric homograph suffixes included), exactly as
   the free functions accept them. Synodal: the registered lemma, resolved
   by the Synodal `Inflector`'s own lookup. Native keys therefore keep
   working for every lexeme not (yet) in the identity table — the table
   grows by reviewed admissions and never gates access to either family's
   native inventory.

**Homograph suffixes.** An abstract key's `_2`/`_3` ordinal is the identity
table's, assigned once per abstract lexeme by the phase-2 deterministic
sort and projected unchanged to both recensions — one identity, one key,
in every recension. The native keys keep their native homograph schemes
(the facade's inventory-sort suffixes on the OCS side, the registry's ids
on the Synodal side); the identity table records the correspondence, and
the scope translates. There is no per-recension re-sorting anywhere.

### Error semantics

Three variants join the facade `Error` (which is `#[non_exhaustive]`):

- `Error::UnsupportedRecension { recension }` — the scope's recension is
  not servable (anything other than the two attested recensions).
- `Error::UnidentifiedLemma { lemma, recension }` — the lemma resolves
  neither through the identity table nor as a native key of the scope's
  recension. This is the typed "not in the identity table" error the
  Synodal route surfaces for un-merged lexemes.
- `Error::NotInRecension { lemma, recension }` — the identity is known
  but carries no citation surface / native handle on the requested side.
  Today every `identity.tsv` entry carries both surfaces, so this arm is
  reserved for the projection-seeded *partial* lexemes the prompt
  anticipates (an OCS-seeded admission whose Synodal accent facts are
  still gap rows, or an OCS-side entry the facade does not serve —
  `ocs_lemma_key` empty). Reserving the variant now fixes the contract
  before partial entries exist.

Past resolution, cell-level failure keeps the existing semantics:
`Error::Underdetermined { lemma }` for a cell the engine cannot commit to
(on the Synodal side this covers registry defects, missing principal
parts, and cells outside the lexeme's licensed inventory), exactly as the
OCS free functions already use it. Paradigm enumeration keeps its
skip-not-error semantics: an underdetermined cell is absent from the
listing.

## 3. The scope surface of this slice

The scope serves the families with a faithful cell mapping onto the merged
kernel's Synodal realization (`GrammarCell`), each with its `_variants`
companion and its paradigm enumeration:

| Scope method | OCS route | Synodal route |
|---|---|---|
| `noun`, `noun_variants`, `noun_paradigm` | free functions | `Noun` handle; the lexeme's own animacy inventory decides the accusative (inanimate convention first, matching the OCS stored-table convention, animate where the lexeme licenses only that) |
| `adjective`, `short_adjective` (+ variants), `adjective_paradigm` | free functions | `AdjectiveCell` (positive degree; long/short as the facade's two functions) |
| `present`, `imperfect`, `aorist`, `imperative` (+ variants) | free functions | `Verb` handle finite/imperative cells |
| `l_participle`, `infinitive` (+ variants) | free functions | `Verb` handle |
| `verb_paradigm` | free function | the mapped `VerbCellKind`s above, enumerated through the single-cell path |

Families **not** on the scope in this slice stay OCS-only free functions:
the supine and verbal noun (no Synodal registry counterpart cell served by
the handles), the participle citation/declension surface, the closed
classes (`pronoun*`, `numeral_form`, `determiner_form` — 17 identity
entries, deferred until the closed-class cell spaces are aligned), the
value-driven numerals, and the `phrases` module. Their recension dimension
lands with later phase-5 slices; the deprecation map tracks them.

Output realization is the recension's: OCS strings are the facade's
attested orthography; Synodal strings are the `Inflector` default profile
(accented, expanded orthography) — recension selects realization,
including accentuation.

The identity table ships in the facade (`include_str!` of
`data/unified/identity.tsv`, parsed once into
`church_slavonic_core::identity::IdentityRegistry` on first use) and is
exposed as `church_slavonic::identity_registry()` so downstream crates
(the dictionary) consume one copy. The deprecation release that publishes
0.3.0 will vendor the table into the crate package; until then the
workspace-relative include is the single source of truth (the
`unified-identity --check` gate holds it byte-stable).

### Dependency graph

`church-slavonic` gains `synodal-church-slavonic` (and through it
`synodal-church-slavonic-core`). This creates no cycle: the synodal crates
depend only downward on `church-slavonic-core` and
`church-slavonic-orthography`, never on `church-slavonic`. The edges are
now:

```
church-slavonic ──> old-church-slavonic-core ──> church-slavonic-core
        │                                              ▲
        └──> synodal-church-slavonic ──> synodal-church-slavonic-core
                                              │        │
                                              │        └──> church-slavonic-orthography
                                              └────────────────────┘ (church-slavonic-core)
church-slavonic-dictionary ──> church-slavonic (+ church-slavonic-orthography)
```

## 4. Dictionary: recension-aware readings and cross-recension senses

`church-slavonic-dictionary` 0.2.0 adds (existing API untouched):

- `lookup_in(query, recension) -> Result<Vec<RecensionSense>, DictionaryError>`.
  A `RecensionSense` wraps a `Sense` with its **provenance recension**
  (the recension of the evidence the sense is attached to — today always
  `OldChurchSlavonic`, the Wiktionary source) and the abstract identity
  key where the sense's lexeme is identified. Under
  `Recension::SynodalRussian` the query is folded to the projection-normal
  comparison key and matched against the identity table's Synodal citation
  surfaces: a sense attached to an OCS lexeme is thereby reachable from
  the Synodal citation surface of the same identity, explicitly marked as
  OCS-provenance evidence. The inverse direction (Synodal-native senses
  reachable from OCS surfaces) is the same mechanism and activates when
  Synodal-provenance senses join the table; the type carries the field
  from day one.
- `lemmatize_in(form, recension) -> Vec<RecensionReading>`. A
  `RecensionReading` is the recension-tagged reading: the lemma under
  which the reading is served (the abstract identity key where the lexeme
  is identified, the native key otherwise), the abstract key when known,
  the `Pos`, the typed `Cell`, and the recension. Under OCS it is the
  existing `lemmatize` index, tagged. Under Synodal it inverts the
  facade's *Synodal scope* paradigm enumeration over the identity table's
  abstract keys (the same single-resolution-path rule as the OCS index),
  with surfaces folded accent-blind by the projection comparison key.

## 5. What does NOT move in this slice: the analyze layer

`synodal-church-slavonic-dictionary`'s `Inflector`-backed analyze
machinery (`analyze_text`, coverage, prediction, families, search) stays
where it is. Its merge into `church-slavonic-dictionary` is the **last**
step of the migration, blocked today by:

- **The accent asymmetry.** `analyze_text`'s provenance ranking and the
  accent-exact token gate are defined over accented Synodal evidence; the
  OCS index is accent-free by nature. Merging the two indexes before the
  cross-recension provenance class
  (same-recension > cross-recension-projected > rule-predicted) is
  implemented in one ranker would either weaken the Synodal gate or
  poison the OCS index with unrankable readings.
- **The gap burn-down state.** The gold gap still carries 53,879 rows
  (~96% `unregistered-lemma`); the identity table covers 599 lexemes. The
  merged analyze layer is only honest when the identity table, not the
  Synodal registry, is the primary lemma space — i.e. after the
  projection-seeded admission waves have moved the bulk of the gap's
  registered mass through review.

Merge plan, in order: (1) grow the identity table through the burn-down
waves; (2) implement the cross-recension provenance class in the Synodal
ranker where it can be gated by `synodal-gold`; (3) lift `analyze_text`
into `church-slavonic-dictionary` keyed by abstract identity with
recension-tagged readings (the `RecensionReading` type introduced here is
the target shape); (4) deprecation-release and delete the
`synodal-church-slavonic*` crate names.

## 6. Versioning

`church-slavonic` 0.3.0 (breaking release per the plan; nothing published
in this slice), `church-slavonic-dictionary` 0.2.0. Workspace manifests
only; the deprecation-release choreography happens at publish time.
