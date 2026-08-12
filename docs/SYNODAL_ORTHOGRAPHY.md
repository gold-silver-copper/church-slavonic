# Synodal orthography and Unicode contract

## Representations

A lexical record never uses one string for every purpose. It retains:

1. `raw_source`: byte-for-byte source spelling;
2. `expanded`: canonical lexical spelling without abbreviations;
3. `stem`: canonical inflectional representation;
4. `lookup`: normalized search key;
5. `accented`: expanded form with normative accents and breathings; and
6. `printed`: Synodal presentation with positional letters and approved
   abbreviations.

The transformations are explicit and loss-reporting. The input spelling remains
available even when a normalized key is produced.

## Profiles

- `Expanded` returns uncontracted lexical spelling and retains morphologically
  meaningful distinctions.
- `ExpandedAccentless` removes presentation accents through a reported lossy
  transformation. It does not modernize historical letters.
- `SynodalLiturgical` applies sourced accents, breathings, positional-letter
  rules, and semantically licensed abbreviations.

## Validation

Public word inputs are validated before lookup or morphology. A word must be
nonempty, contain Church Slavonic Cyrillic letters and permitted combining marks,
and contain no whitespace, controls, Latin/Greek letters, or private-use code
points. Rendered-text validation is a separate API that can admit sourced
punctuation and spacing.

Stored text uses standard Unicode only. Private-use characters from legacy fonts
are rejected. The font determines glyph appearance, never the stored character.

NFC is a useful baseline but not the whole contract. In particular:

- Church Slavonic breathing U+0486 precedes an acute or grave accent on the same
  base;
- equal canonical-combining-class marks retain their meaningful relative order;
- a combining grapheme joiner is permitted only where the encoded Church
  Slavonic sequence requires it to prevent canonical reordering;
- a combining mark cannot begin a word or follow an incompatible separator;
- titlo and superscript-letter sequences are validated as grapheme clusters; and
- normalization never silently replaces a historical letter with a modern
  Russian one.

These constraints follow UTN #41. Exact allowed mark sequences are implemented as
reviewable tables and covered by hostile-input tests.

## Accent realization

Liturgical generation resolves accent metadata in a fixed order: an exact
reviewed accented/printed cell; an explicit lexical irregular printed override;
a reviewed reusable `AccentParadigm`; otherwise a typed
`OrthographicMetadataRequired` failure. A paradigm is not inferred from corpus
frequency or one surface witness.

The typed model distinguishes a stem vowel counted from the lexical left edge
from an ending vowel counted from the right edge. Multiple disjoint cell/number
scopes can express documented mobility and acute, grave, or kamora selection.
Psili breathing has its own placement rule and is inserted before the stress
mark when both occupy one base. The generated result is validated again through
`SynodalWord`, preserving canonical combining order and rejecting hostile
sequences.

The reviewed `synodal-accent:mudr-fixed-stem` rule applies first-stem-vowel acute
stress to multiple long positive singular forms of `мꙋдръ` under Alypy §57. It
coexists with, and is lower precedence than, the existing exact nominative
accent row.

## Lookup and collation

Lookup normalization is deterministic and separate from printed rendering. It
normalizes canonical-equivalent sequences and optional case, but does not merge
lexically contrastive letters such as yat with `е`, omega with `о`, or decimal
`і` with `и`. Accent-insensitive lookup is an explicit search mode and returns
all compatible lexemes rather than selecting one silently.

Collation uses `CollationProfile::Utn41Revision1`, an inspectable implementation
of UTN #41 §5.1 for validated words. `CollationKey` exposes primary, backward
secondary, case, tertiary, and identical levels, and `CollationStrength` makes
the chosen equivalence explicit.

The primary alphabet follows the pre-reform-Russian-oriented Synodal order.
Graphical/positional `о` variants share a primary weight; `ѻ < о < ѡ` at the
tertiary level. `е` and `є` likewise share primary weight and remain tertiary
distinct. Digraph uk sorts at the `у` primary position and before monograph uk,
while `ѿ` expands as `ѡт`. Acute, grave, kamora, breathing, titlo, and combining
letters receive secondary weights; clusters are scanned backward as required by
the tailoring. Uppercase sorts before lowercase after the secondary level.

This compact implementation accepts the normative Synodal alphabet and reports
an error for other Cyrillic letters instead of silently assigning a misleading
weight. It is intended for words, not arbitrary ICU locale collation of prose,
punctuation, or Typicon symbols. Canonically equivalent input receives the same
key because `SynodalWord` normalizes its stored representation first.

Primary or tertiary equivalence is never used as proof that two spellings
identify one lexeme. Dictionary resolution still uses a separate conservative
lookup key and stable lexeme IDs.

## Positional letters and abbreviation

Initial/medial/final variants (including broad on, omega, uk, and dotted/decimal
`і`) are presentation rules with stable IDs. A rule records the lexical input,
context, output, evidence, and whether reversal is unique.

Contraction requires a resolved lexical identity and semantic sense. Nomina sacra
are never produced by blind substring replacement. If semantic metadata is absent
or several expansions fit a printed form, contraction/expansion returns a typed
ambiguity instead of guessing.

## Numerals

Traditional Cyrillic numerals are a typed presentation system, not arbitrary
letter strings. Parsing and formatting validate titlo placement, thousands signs,
ordering, supported range, and canonical output. Numeral letters in lexical words
are not interpreted numerically without an explicit numeral context.

The current formatter/parser covers canonical 1–9,999 notation and the Alypy §5
special examples for 100,000 and 1,000,000. It validates exactly one balanced
titlo, the thousands sign, letter order, the reversed 11–19 sequence, and
round-trip canonical spelling. Other myriad notation is explicitly out of range.

## Current presentation limits

The validator recognizes standard combining marks, superscript Cyrillic letters,
titlo/pokrytie, payerok, kavyka, and the standard Cyrillic repertoire while
rejecting private-use glyph encodings. The renderer currently automates only
reviewed accent rows and one semantic nomen-sacrum family. Initial `є`, broad
`ѻ`, iotated `ꙗ`, and digraph uk are available through an explicit
`InitialPresentation` operation with a loss/change report. Automatic selection
of those variants remains lexical and grammatical work because Alypy §2 records
exceptions; the engine does not apply a blind spelling rewrite.
