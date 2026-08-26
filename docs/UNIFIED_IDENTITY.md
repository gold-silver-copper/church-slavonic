# Unified identity layer (merge phase 2)

Design and contract for the shared lexeme-identity layer of the OCS/Synodal
merge. Contract: `docs/UNIFIED_LANGUAGE_PROMPT.md`, execution plan step 2,
entered after the phase-1 decision point passed on the numbers in
`reports/projection-study.md` (commit 9afdab7).

## 1. The abstract identity key — a keying convention, not a form

An abstract lexeme is keyed by:

```
<pos>:<projection-normal form>[_<homograph ordinal>]
```

- `<pos>` is the shared part-of-speech code in the OCS short registry
  (`noun`, `adj`, `verb`, `pron`, `num`, `det`).
- `<projection-normal form>` is the **accentless-folded projection-normal
  form of the lemma**: the comparison key of the projection study's declared
  rules (`crates/xtask/src/projection_study.rs`, `study_key` + `project`).
  Both citation surfaces reduce to it — the Synodal citation by the pure
  symmetric folds (accent strip, uk, omega, i-variants, ja, izhitsa), the
  OCS citation by the folds plus the generative correspondence rules (yery,
  yus reflexes, jer reflexes, uk digraph, zelo). Example: OCS `агньць` and
  Synodal `а҆́гнецъ` share the key `noun:агнецъ`.
- `_<homograph ordinal>` (`_2`, `_3`, …) disambiguates two abstract lexemes
  colliding on the same `<pos>:<form>`. Per the prompt's identity-stability
  rule the ordinal is assigned **once per abstract lexeme** by one
  deterministic sort — ascending OCS extracted-lexeme id (which embeds the
  content signature) — and is projected unchanged to both recensions.
  There is never a per-recension re-sort, so one identity cannot carry
  different keys in different recensions. This mirrors the `church-slavonic`
  facade's deterministic homograph scheme without reusing its per-recension
  sort.

This is explicitly a **keying convention**, not a reconstructed
proto-form (prompt, "Out of scope"). The projection-normal form is not
claimed to be a historical spelling; it is the canonical spelling of the
equivalence class the correspondence rules define, and it is fully
recomputable from either citation surface plus the committed rules.

## 2. What an identity entry carries

One row of `data/unified/identity.tsv` per abstract lexeme:

| column | meaning |
|---|---|
| `abstract_key` | the identity key above |
| `pos` | shared pos code |
| `ocs_lexeme_id` | native id in `data/extracted/lexemes.tsv` |
| `ocs_citation` | OCS citation surface (Wiktionary lemma, as printed) |
| `ocs_lemma_key` | the `church-slavonic` facade lemma key (facade homograph suffix included); empty when the facade does not serve the lexeme |
| `synodal_lexeme_id` | native Synodal registry id (usable with the `Inflector`) |
| `synodal_citation` | Synodal citation surface (registered lemma) |

Per-recension citation surfaces attach as stored strings, never derived at
lookup time: where the correspondence rules are ambiguous (jer reflexes,
big-yus reflex) the stored surface IS the resolution, reviewed once at
admission. Public IDs stay resolvable in both directions: nothing about
either family's native ID changes, and the registry indexes both — an
existing `LexemeId` or facade lemma key resolves to its abstract entry, and
the abstract key resolves back to both native handles.

## 3. Admission policy for the initial table

The initial table is generated from the projection study's 638
registered-lexeme matches (the high-confidence tier) by
`cargo xtask unified-identity`, restricted to the **unambiguous 1:1** core:

- exactly one pos-compatible (candidate key, registered Synodal lexeme)
  pairing for the OCS lexeme, and
- no other OCS lexeme pairing with the same Synodal lexeme.

That yields **599 identity entries**. The remainder of the 638, plus the
study's 592 oracle-type-only matches, land in
`data/unified/identity-candidates.tsv` (defect-candidates precedent: a
review queue, never an identity claim), tagged:

| kind | count | meaning |
|---|---:|---|
| `ambiguous` | 7 | 2+ pos-compatible registered pairings |
| `ambiguous-many-to-one` | 26 | several OCS lexemes project onto one Synodal lexeme |
| `registered-pos-mismatch` | 6 | registered match, incompatible part of speech |
| `oracle-type` | 592 | lemma surface attested in Synodal evidence, no registered lexeme |
| `lexical-union-proposal` | 957 | (phase 6) a Synodal lexical-union queue claim whose cross-recension identity is unconfirmed; provenance is the ledger claim id |
| `lexical-union-homograph` | 43 | (phase 6) a queue claim blocked by a cross-source homograph |

Since merge phase 6 (`docs/UNIFIED_DATA.md` §4) every candidate row carries
a `provenance` column: `projection-study` for the four original kinds, the
Synodal lexical-union ledger claim id for the two ingested kinds.

Promotion from candidates to the identity table is a reviewed edit followed
by `cargo xtask unified-identity` — the generator is the single writer of
all three artifacts, and `--check` holds them byte-stable in CI.

## 4. The runtime registry — lexicons as views

`church_slavonic_core::identity::IdentityRegistry` (the kernel owns the
type; it ships no data) parses the committed table and serves:

- `resolve(abstract_key) -> &IdentityEntry`
- `ocs_lemma_key(abstract_key)` — plug into the `church-slavonic` facade
  (`noun`, `adjective`, `present`, …);
- `synodal_lexeme_id(abstract_key)` — wrap in `LexemeId` for the Synodal
  `Inflector` (`form_by_id`);
- `by_ocs_lexeme_id`, `by_synodal_lexeme_id` — reverse lookups from the
  native IDs.

This is the minimal honest phase-2 shape: the two families' lexicons become
*consumable as views* through these resolvers without rewriting either
family's internals. Phase 4/5 (kernel and facade merges) will grow the
identity layer into the primary lemma space; this phase only establishes it
and gates it.

## 5. The projection-coherence gate

Per the prompt's gates section. For every identity entry, each recension's
attested cells replay through the projection rules against the other side:

- **OCS side**: every attested OCS cell (`data/extracted/forms.tsv`)
  projects to its candidate keys; a cell matches when some candidate is
  attested in the Synodal evidence (gold token oracle types, Alypy paradigm
  surfaces, `exact_forms.tsv` printed surfaces) — the study's accent-blind
  semantics, per the accent asymmetry.
- **Synodal side**: every printed surface bound to the entry's Synodal
  lexeme in `exact_forms.tsv`; a cell matches when its folded key is
  reachable from some projected OCS attested cell of the same entry.

`data/unified/coherence-baseline.tsv` commits the per-entry counts — full
enumeration over the identity table, no subsetting and no ratchet
arithmetic (the synodal-gold precedent). `cargo xtask unified-identity
--check`, wired into `check-structure` and `ci.yml`, regenerates and:

1. fails as **projection-coherence REGRESSION** when any entry's matched
   counts fall below the committed baseline (or an entry disappears);
2. otherwise fails as **staleness** when any of the three artifacts differs
   byte-for-byte from the committed file.

Improvements therefore land as reviewed diffs of the baseline, and
regressions cannot land silently. Initial baseline: OCS side
**10152 / 24135** attested cells matched, Synodal side **1255 / 1892**
attested cells matched, over the 599 entries.

## 6. Non-goals of this phase

- No change to either family's inflection kernels, dictionaries, or public
  IDs (phase 4/5).
- No cross-recension provenance in `analyze_text` yet — the identity table
  enables it, phase 3+ implements it under the provenance-ranking rule.
- Candidates are not identities: nothing in `identity-candidates.tsv` is
  consumed at runtime.
