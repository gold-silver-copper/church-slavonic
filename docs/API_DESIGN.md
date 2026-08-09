# Public API design

## Decision

The dictionary-backed facade adopts Ruthenian's discoverable
lemma-plus-grammar call shape, validated lemma boundary, resolved lexical handles,
curated prelude, and single generation path. It does not adopt Ruthenian's
unconditional string results, sentinel failure values, or fallback totality.

Old Church Slavonic needs structured results because the pinned sources contain
ordered spelling variants, romanizations, competing lexical analyses, incomplete
principal parts, and genuine historical paradigm gaps.

## Ordinary root

The root API groups calls by the grammatical system they actually request:

```text
noun(lemma, case, number)
long_adjective(lemma, case, number, gender, animacy)
short_adjective(lemma, case, number, gender, animacy)
determiner(lemma, case, number, gender)
pronoun(lemma, case, number)
personal_pronoun(lemma, case, number, person)
gendered_pronoun(lemma, case, number, gender)
numeral(lemma, case, number)
gendered_numeral(lemma, case, number, gender)
present(lemma, person, number)
imperfect(lemma, person, number)
aorist(lemma, person, number)
finite(lemma, tense, person, number)
imperative(lemma, person, number)
l_participle(lemma, gender, number)
infinitive(lemma)
supine(lemma)
verbal_noun(lemma)
comparative_citation(lemma)
```

Named functions bind each of the four declined participle systems. Lemma-oriented
paradigm functions use the corresponding names: `present_paradigm`,
`finite_paradigm`, and the system-specific nominal and closed-class paradigms.

The breaking renames deliberately remove ambiguous aliases:

| Removed | Replacement |
|---|---|
| `verb` | `present` |
| `adjective` | `long_adjective` |
| `finite_verb` | `finite` |
| `verb_paradigm` | `present_paradigm` |
| `finite_verb_paradigm` | `finite_paradigm` |
| `comparative` | `comparative_citation` |
| fallible handle `new` constructors | `resolve` |

The stable-ID namespace follows the same vocabulary:
`finite_by_id`, `present_paradigm_by_id`, `finite_paradigm_by_id`, and
`comparative_citation_by_id`. The former ambiguous by-ID spellings were removed.

## Validated lemma boundary

`Lemma::parse` NFC-normalizes a word, preserves historically meaningful letters
and marks, detects Cyrillic or Glagolitic, and rejects empty, control-bearing,
whitespace-bearing, markup/punctuation-bearing, leading-combining, Latin, and
mixed-script input. It exposes `as_str()` and `script()` without promising a
Cyrillic–Glagolitic transliteration.

Ordinary functions keep the ergonomic `&str` signature. Their shared lookup path
constructs a `Lemma` before normalization or dictionary access, and a validated
`Lemma` can be passed through its `Deref<Target = str>` implementation. This avoids
generic call signatures while giving raw and prevalidated values the same rules.

## Resolved identities

`Noun`, `Adjective`, `Verb`, `Determiner`, `Pronoun`, and `Numeral` share:

- `resolve(lemma)` for one unambiguous dictionary identity;
- `from_id(id)` for a stable source identity;
- `lemma()` and `id()` accessors;
- direct grammatical methods; and
- typed paradigm builders.

`Participle` binds one verb identity and one `ParticipleKind`. Pronoun and numeral
handles expose separate case-number-only, person-indexed, and gender-indexed methods.
This separation follows the audited feature inventory and avoids a catch-all
request made mostly of unrelated `Option` fields.

## Results and errors

All ordinary one-cell calls return `Result<FormSet, InflectionError>`. A `FormSet`
is structurally nonempty and offers:

- `primary()` / `primary_text()` for documented source-first access;
- `variants()` / `texts()` to retain all alternatives;
- `unique_text()` to require exactly one variant;
- `select(VariantPolicy)` for explicit deterministic selection; and
- source, warning, trace, and analysis accessors.

There is intentionally no `Display` implementation that would discard variants.

Contextual errors retain the rejected input or identity. `UnknownLemma` includes
the requested part of speech. `UnsupportedCell` and
`HistoricallyInvalidCell` include the stable lexeme ID and a typed
`RequestedCell`. Invalid input, ambiguity, metadata failures, formation failures,
historical invalidity, and unsupported-but-conceptually-valid requests stay
separate.

## Paradigms and one generation path

Fixed paradigms expose `form(...) -> Result<&FormSet, ParadigmLookupError>` rather
than `Option<&Result<...>>`. `NotRepresented` means that a specialized paradigm
does not contain the requested grammar; `Failed(InflectionError)` means the row
is represented and retained a generation failure.

Every paradigm also supports `iter`, `successes`, `failures`, and `into_rows`.
Builders enumerate grammar `ALL` inventories and call the canonical by-ID one-cell
resolver for each row. Free functions and handle methods delegate to that same
resolver. No wrapper owns endings, stem selection, source precedence, or variant
merging.

## Namespace boundary

The root contains common grammar types, validated lemmas, structured results,
ordinary functions, resolved handles, and typed paradigms. `prelude` contains that
ordinary surface but excludes specialist internals.

- `advanced::cells`: generic cell structures for tools;
- `advanced::by_id`: stable dictionary-ID operations;
- `advanced::rules`: explicit caller metadata and productive rules;
- `advanced::metadata`: audited dictionary principal parts and evaluation entry
  points;
- `advanced::raw_features`: normalized feature keys for extraction and
  diagnostics; and
- `trace`: evidence and rule diagnostics.

No ordinary or game-facing call requires a key such as `decl:pron:gen:sg`.

## Closed-class inventory audit

The pinned generated registry contains 29 pronoun lexemes, eight numeral lexemes,
and one determiner lexeme. Its coherent feature shapes are:

- case-number-only case-number tables, including the reflexive `сѧ`;
- person-indexed case-number tables, including `азъ`;
- gender-indexed case-number tables, including `онъ`;
- case-number-only numeral tables, including `девѧть`; and
- gender-indexed agreeing numeral/determiner tables, including `прьвъ` and `кꙑи`.

The public functions mirror those shapes. A typed paradigm enumerates the relevant
grammatical product and retains `UnsupportedCell` for rows the source does not
provide. The audit does not justify productive arbitrary numerals, a generalized
pronoun declension engine, or treating every pronoun lexeme as all three shapes.
Those remain explicit source-data limitations.

## Deliberate differences from Ruthenian

- A form is `Result<FormSet, InflectionError>`, not an unconditional `String`.
- Multiple source spellings are preserved rather than flattened.
- Unsupported cells and historical gaps fail rather than borrowing a neighboring
  form or returning a sentinel.
- `FiniteTense` remains an explicit OCS dimension.
- Verb formations and principal parts remain independent lexical facts.
- Participles retain their verbal formation and competing analyses before using
  the adjective agreement engine.
- Closed classes and numerals remain exact-table APIs until independent productive
  descriptions and evidence exist.
- Cyrillic and Glagolitic are detected but not automatically transliterated.

These differences preserve the project's source-backed, fail-closed contract.
