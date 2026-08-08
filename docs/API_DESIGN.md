# Public API redesign

## Decision

The dictionary-backed facade adopts Ruthenian's ordinary call shape—lemma plus
direct grammatical dimensions—without adopting Ruthenian's unconditional
`String` result or fallback totality. Old Church Slavonic keeps ordered source
variants, competing analyses, provenance, and typed failures.

## Before and after

| Current root API | Redesigned root API |
|---|---|
| `noun(lemma, NounCell)` | `noun(lemma, case, number)` |
| `adjective(lemma, AdjectiveCell)` | `adjective(...)` for long and `short_adjective(...)` for short |
| `finite_verb(lemma, FiniteVerbCell)` | `verb`, `imperfect`, `aorist`, or direct-dimension `finite_verb` |
| `imperative(lemma, ImperativeCell)` | `imperative(lemma, person, number)` |
| `l_participle(lemma, LParticipleCell)` | `l_participle(lemma, gender, number)` |
| `noun_paradigm(id)` | `noun_paradigm(lemma)` and `noun_paradigm_by_id(id)` |
| public `variants: Vec<_>` | nonempty `FormSet` with `primary_text()` and ordered iterators |
| blanket core re-export | curated root plus `advanced` and `trace` namespaces |

The root adds resolved `Noun`, `Adjective`, `Verb`, and `Participle` handles for
repeated calls. Each handle stores stable dictionary identity, while all class,
stem, formation, and source facts remain owned by the generated registry and the
canonical resolvers.

## Namespace boundary

The facade root owns common grammar types, structured results, lookup, direct
lemma functions, resolved handles, and lemma-oriented paradigms. `advanced`
owns cell structs, by-ID operations, explicit rule lexemes and `*_with` calls,
dictionary metadata, and raw feature-key access. `trace` owns rule and evidence
diagnostics.

The rule-only `old-church-slavonic-core` crate remains independently usable. Its
callers must continue to supply lexical class and principal-part metadata where
the citation alone cannot determine them.

## Pre-redesign public inventory

Before implementation, the facade root combined four different audiences. Its
dictionary-facing functions were:

- `lookup(lemma, part_of_speech)`;
- raw `dictionary_form_by_id`, `form_by_id`, `dictionary_paradigm_by_id`,
  `closed_class`, and `closed_class_by_id`;
- `noun`, `adjective`, `finite_verb`, `imperative`, `l_participle`, and
  `participle`, each taking its corresponding cell struct;
- the matching `*_by_id` operations and ID-oriented noun, adjective, finite,
  imperative, l-participle, and declined-participle paradigms;
- citation/non-finite `adjective_comparatives`, `infinitive`, `supine`,
  `verbal_noun`, and `participle_citation`, plus their by-ID forms;
- productive `noun_with`, `adjective_with`, `finite_verb_with`,
  `imperative_with`, `infinitive_with`, `supine_with`, `l_participle_with`, and
  `participle_with`; and
- dictionary-metadata evaluation operations for finite verbs, imperatives,
  l-participles, and declined participles.

It also publicly exposed `DictionaryVerbMetadata` and every normalized metadata
analysis/policy type. Finally, `pub use old_church_slavonic_core::*` copied all
core root names into the facade: every grammar enum and cell, `FormSet` and its
evidence/error types, rule IDs and predicted forms, and the public `adjective`,
`grammar`, `noun`, `orthography`, `pronoun`, `result`, `trace`, and `verb` modules.
That blanket export is the source of many names that had no ordinary
dictionary-backed meaning at the facade root.

The pure core's intentional public inventory remains:

- common grammar and lexical-class enums and typed cells;
- structured result, error, warning, evidence, and trace types;
- `noun::{NounLexeme, decline}`;
- `adjective::{AdjectiveLexeme, decline, decline_stem}`;
- `verb::{VerbLexeme, VerbLexemeBuilder, finite, imperative, infinitive, supine,
  l_participle, participle}`;
- lossless `orthography` validation/key operations; and
- the `pronoun` namespace documenting that closed-class forms remain facade data.

The core still exposes those modules because its audience is explicitly
rule-based. Its root grammar/result/trace conveniences are now explicit re-export
lists rather than wildcard exports.

## Recorded baseline

Before the redesign on 2026-08-08, `cargo fmt --all -- --check`, strict workspace
clippy, all workspace/all-target/all-feature tests, and `cargo xtask check-all`
passed. The executable suite contained 68 tests. Registry and accuracy guards
reported 3,081 lexemes and 137,406 ordered forms, current generated/report
artifacts, runtime code with no I/O or network access, current attribution, and
passing examples. These counts and reports are the no-regression baseline for the
API-only change.

## Implemented root signatures

The ordinary one-cell functions are now:

```text
noun(lemma, case, number)
adjective(lemma, case, number, gender, animacy)
short_adjective(lemma, case, number, gender, animacy)
verb(lemma, person, number)
imperfect(lemma, person, number)
aorist(lemma, person, number)
finite_verb(lemma, tense, person, number)
imperative(lemma, person, number)
l_participle(lemma, gender, number)
infinitive(lemma)
supine(lemma)
verbal_noun(lemma)
comparative(lemma)
```

Named participle functions return a resolved `Participle`, not a citation string.
Lemma paradigms use the unsuffixed names; every one has a stable dictionary
identity counterpart under `advanced::by_id::*_paradigm_by_id`. Generic form selectors are
`advanced::{noun_form, adjective_form, finite_verb_form, imperative_form,
l_participle_form, participle_form}`.

## Deliberate differences from Ruthenian

- Every inflection remains `Result<FormSet, InflectionError>`, not `String`.
- Unsupported and historically invalid cells remain errors, not plausible
  substitutes.
- `FiniteTense` remains a real OCS paradigm dimension.
- Participles retain their verbal metadata instead of becoming untyped adjective
  strings.
- Closed-class and numeric generation remain dictionary-backed until independently
  specified productive APIs exist.

Flattening would lose real information. For example, the first-singular aorist of
`бꙑти` has source-ordered `бѣхъ` and `бꙑхъ`, and a declined past-active participle
of `благословити` retains two metadata-backed analyses. Ambiguous `блѧдь`, missing
principal parts, unsupported sigmatic formations, and historically invalid cells
must also remain distinguishable.

## One-generation-path rule

Direct functions construct internal cells and delegate to the canonical by-ID
resolver after one lemma resolution. Handle methods call that same resolver.
Paradigms enumerate those same handle/by-ID methods and retain every cell outcome,
including typed failures. No ergonomic wrapper owns endings, stem selection,
variant merging, or source precedence.
