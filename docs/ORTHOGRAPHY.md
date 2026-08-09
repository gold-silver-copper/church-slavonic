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

Cyrillic and Glagolitic are not automatically transliterated into one another. Source
romanization is metadata and remains distinct from any future algorithmic
romanization.

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
