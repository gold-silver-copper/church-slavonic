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

The word-level API rejects empty input, whitespace, control characters, and input
longer than 4,096 Unicode scalar values. Non-OCS alphabetic text is not destroyed; it
normally produces `UnknownLemma` unless the caller supplies explicit rule metadata.

Raw exact metrics compare stored display strings byte-for-byte. The separately named
normalized metric applies the same NFC-plus-lowercase function as lookup and does not
strip a historical letter or combining mark. The reports never combine those two
denominators.
