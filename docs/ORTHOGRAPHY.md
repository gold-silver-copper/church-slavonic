# Orthography and Unicode contract

## Scope

The project represents canonical Old Church Slavonic spellings found in its pinned
source. It does not modernize them into a later Church Slavonic recension.

Every source spelling has three distinct roles:

1. **raw source form** — retained in the normalized registry;
2. **canonical display form** — NFC, otherwise lossless;
3. **lookup key** — NFC plus Unicode lowercase.

Explicit aliases connect the page title, the canonical head form, and source-listed
alternatives to a lexeme. Thus the page title `царь` can find a displayed head form
such as `цар҄ь` without deleting the combining palatalization mark from display.

## What lookup does not erase

Lookup never strips or substitutes:

- `ъ` or `ь`;
- `ѣ`, `ѧ`, `ѫ`, `ꙑ`, or other historical letters;
- palatalization marks, titla, breathing marks, or accents;
- Glagolitic letters;
- manuscript abbreviations.

Cyrillic and Glagolitic are never implicitly transliterated during lookup or
inflection. Source romanization remains separate metadata. A caller may explicitly
apply the normalized Glagolitic profile described below after morphology.

## Explicit normalized Glagolitic realization

`realize_glagolitic` composes after any single-word morphology result. The
`Jagic1879NormalizedOcs` profile follows the classical Glagolitic/Cyrillic table
reproduced in Unicode TN41 revision 1, Appendix A. It converts the complete word,
never a Glagolitic stem plus Cyrillic endings, and returns `TransliteratedForm`
with rule `OCS-GLAG-JAGIC-01`, direction, fidelity, ordered loss records, and a
trace. `transliterate_glagolitic_to_cyrillic` supplies the reverse normalized
operation and matches the two yer-plus-i sequences for `ꙑ` and `ы` before their
component letters.

`realize_glagolitic_variants` maps every `FormSet` variant one-to-one in source
order; it never chooses the primary spelling or discards alternatives. The caller
keeps the original `FormSet` for dictionary or productive provenance, while each
orthographic result records its own fidelity and losses.

The shared alphabet, including both yers, yat, uk, all four yuses, fita, izhitsa,
and the two yeri sequences, is reversible. The scripts do not encode every
Cyrillic distinction one-to-one. The following paths therefore require
`TransliterationLossPolicy::Report`; `Reject` returns
`InflectionError::UnrepresentableOrthography` at the first such scalar:

- Cyrillic presentation variants such as `є`, modern `у`, round or ornamental
  `о`, broad omega, closed yuses, zemlya, dzelo, and Polivanova's `ї`
  allograph of initial izhe;
- Polivanova's natural-to-secondary normalization of Cyrillic iotated `ꙗ` to
  Glagolitic a and `ѥ` to Glagolitic e, because early Glagolitic has no matching
  iotated-vowel letters; and
- Cyrillic xi, psi, and ot, which expand to Glagolitic letter sequences.

Existing Glagolitic input is validated and returned unchanged. That status says
only that caller input was preserved; source attestation still belongs to the
dictionary result. The locked exact `ⱁⰽⱁ` paradigm therefore keeps all of its
source spellings ahead of normalized realization.

This profile is not diplomatic transcription. Later Croatian Glagolitic letters,
manuscript abbreviations, superscripts, breathing marks, and the two rare
colliding Glagolitic letters identified by TN41 are rejected or, where the
sources define a normalized fold, explicitly loss-reported. The OCS
palatalization mark U+0484 used with Glagolitic `l`, `n`, and `r` in Polivanova
§132 and neutral Unicode combining marks such as U+0301 are preserved. Phrase results
remain typed multi-token objects; realize each word token explicitly rather than
passing whitespace through this word-level API.

Unicode composites `й` and `ѷ` are decomposed onto their mapped base letters and
recomposed on the reverse path, so their code-point spellings remain reversible.

## Validation

`Lemma::parse` is the shared boundary for every ordinary lemma-based call. It:

- rejects empty input, whitespace, control characters, and input longer than 4,096
  Unicode scalar values;
- NFC-normalizes the spelling without stripping historical characters or marks;
- rejects a combining mark that has no preceding lemma letter;
- rejects punctuation and markup rather than treating them as part of a word;
- accepts a single Cyrillic or Glagolitic script and reports it through
  `Lemma::script()`; and
- rejects Latin, mixed Cyrillic–Glagolitic, Cyrillic–Latin, and
  Glagolitic–Latin input.

Validation yields `InflectionError::InvalidLemma { input, reason }`, preserving
the original rejected spelling. Dictionary misses after successful validation
yield `UnknownLemma { lemma, part_of_speech }` instead. A prevalidated `Lemma` can
be passed to ordinary calls as `&lemma`; those calls deliberately re-enter the
same validation path rather than growing parallel behavior.

The lower-level `canonical_display` and `lookup_key` helpers remain available in
the rule-only core for registry tooling. They normalize losslessly but are not a
claim that arbitrary alphabetic input is an OCS dictionary lemma.

Raw exact metrics compare stored display strings byte-for-byte. The separately named
normalized metric applies the same NFC-plus-lowercase function as lookup and does not
strip a historical letter or combining mark. The reports never combine those two
denominators.
