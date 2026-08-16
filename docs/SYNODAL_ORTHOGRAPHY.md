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
reviewed or provider-supplied accented/printed cell; an explicit lexical
irregular printed override; caller-supplied positional presentation followed by
a reviewed reusable `AccentParadigm`; otherwise a typed
`OrthographicMetadataRequired` failure. Positional presentation runs before
initial breathing and accent so `его` can become canonical `є҆́го`, never a
misordered combining sequence. Neither paradigm is inferred from corpus
frequency or one surface witness.

The typed model distinguishes a stem vowel counted from the lexical left edge
from an ending vowel counted from the right edge. Multiple disjoint cell/number
scopes can express documented mobility and acute, grave, or kamora selection.
Lexical metadata selects the stressed vowel and any exceptional mark; Alypy
§3's language-wide surface rules are then applied independently. An initial
vowel receives psili automatically (over the second component of digraph uk),
and a stressed final vowel receives grave in isolation but acute before the
closed `же`, `бо`, `ли`, or short-personal/reflexive-pronoun environments.
`AccentEnvironment` and `AccentEnclitic` make that syntactic choice explicit.
An optional lexical `BreathingRule` can document the same initial placement but
cannot move psili to a medial vowel. The generated result is validated again
through `SynodalWord`, preserving canonical combining order and rejecting
hostile sequences.

The reviewed `synodal-accent:mudr-fixed-stem` rule applies first-stem-vowel acute
stress to multiple long positive singular forms of `мꙋдръ` under Alypy §57. It
coexists with, and is lower precedence than, the existing exact nominative
accent row.

Alypy §43 additionally licenses `synodal-accent:mati-fixed-stem`,
`synodal-accent:imya-mobile`, and `synodal-accent:nebo-mobile` across complete
noun paradigms. `AccentScope::NounCases` restricts each mobile rule by both
number and case. The `имѧ` rules place psili and stress independently on the
initial vowel, so canonical output retains breathing before acute. Missing
scopes return `OrthographicMetadataRequired`; overlapping scopes return
`ContradictoryMetadata`.

Phrase-valued enclisis stays out of the single-word API.
`enclitic_particle_after_host` resolves `же`, `бо`, or `ли` as a separate
reviewed token and changes only a final host varia to acute; it never moves
nonfinal lexical stress. The existing typed short-pronoun phrase path applies
the corresponding §47 rule after validating the pronoun identity and cell.

### Semantic abbreviation families

The reviewed exact abbreviation registry remains the highest-precedence layer:
191 cells preserve their complete accent, breathing, capitalization,
positional-letter, superscript, and titlo spelling. Beneath it,
`abbreviation_families.tsv` records 55 stable lexical/sense identities and 61
source-backed stem allomorphs. Each family replaces only an initial stem
pattern after semantic identity and a typed grammatical cell are already known;
it is never a global substring rewrite. The extractor proves that the family
skeleton reproduces every exact row after only prosodic, capitalization, and
positional-letter comparison normalization.

`contract_variants_for_cell_by_id` returns every compatible exact row when one
exists. Only an otherwise uncovered cell may fall through to productive
morphology and a family pattern, and the result is labeled
`AbbreviationRealization::ProductiveFamily`. Family evidence is combined with
the productive form's morphological evidence. Multiple allomorphs, such as
`бог-/боз-`, `отец-/отц-/отч-`, and `небо-/небес-`, use a deterministic
longest-prefix rule. The matcher recognizes only the engine's closed
positional-letter equivalences at that initial boundary, so productive forms
such as soft-neuter `е/є` alternants keep their original suffix without opening
a general spelling rewrite. Unknown senses, compounds that merely contain a
listed stem, and forms that do not match a licensed allomorph fail typed.

The family layer supplies the contraction skeleton and its required titlo,
superscript, pokrytie, or initial breathing. It does not guess missing lexical
stress or positional spelling in a previously unattested cell; those remain
separate orthographic metadata and exact overrides.

Injected providers do not receive a stress-guessing path. Their exact
`SpecifiedForm` may carry an explicit liturgical realization; otherwise their
`LexemeSpec` must carry a complete applicable accent paradigm. Composition
cannot borrow an accent rule from another stable identity.

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

`PositionalParadigm` is the complete arbitrary-lexeme interface. Its disjoint
`AccentScope` rules cover typed grammar cells and compose explicit operations:
preserve; initial wide `є`, broad `ѻ`, iotated `ꙗ`, or digraph uk; `и → ї`
before a vowel; the §36 wide plural ending; and a checked occurrence-specific
`е → є`, `о → ѻ/ѡ`, `и → ї`, or `ѧ → ꙗ` replacement. An empty operation list
is an affirmative preserve decision. A missing scope returns
`OrthographicMetadataRequired(PositionalParadigm)`; overlapping scopes or a
replacement whose input occurrence is absent return `ContradictoryMetadata`.
Every noun, adjective, determiner, numeral, pronoun, and verb specification
exposes this same contract, including injected-provider fallthrough.

The occurrence operation does not license arbitrary Cyrillic rewriting. Alypy
§2's closed `ѕ` families and foreign-word `і/ї`, `ѡ`, `ѳ`, `ѯ`, `ѱ`, and `ѵ`
spellings remain validated lexical stem/lemma metadata because they depend on
identity and Greek etymology. Likewise, `ѧзыкъ` “organ” versus `ꙗзыкъ`
“people,” and `сиѡна` the king versus `сїѡнъ` the holy mountain, require the
caller's resolved semantic identity. The engine can express either result but
never chooses by string shape alone.

Alypy §36 calls `-ѡвъ/-євъ` and `-ѡмъ/-ємъ` obligatory, while the same
grammar's §43 extended-stem tables print ordinary `-емъ` in forms such as
plural dative `матеремъ`. The engine retains this conflict: reviewed exact
tables win, and arbitrary lexemes opt into `WidePluralEnding` through sourced,
cell-scoped metadata. It does not erase a table reading with a global rewrite.
`AccentMark::Kamora` supplies §36's alternative case distinction where a
reviewed paradigm selects it.

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

## Presentation completion boundary

The validator recognizes standard combining marks, superscript Cyrillic letters,
titlo/pokrytie, payerok, kavyka, and the standard Cyrillic repertoire while
rejecting private-use glyph encodings. The renderer currently automates reviewed
exact accent rows, six bundled reusable lexical paradigms, the language-wide
initial-breathing and contextual final-accent rules, 191 exact abbreviation
cells, and 55 semantic contraction families with 61 allomorphs. The reusable
`AccentParadigm` and `PositionalParadigm` APIs are not limited to those bundled
rows: a caller can source every cell of any arbitrary lexeme, including fixed
or mobile stress, breathing, initial variants, contextual `ї`, case-distinguishing
`є/ѡ` or kamora, etymological spelling retained in the stem, and exact printed
exceptions. Exact printed cells always win before reusable metadata.

Alypy §3.c's printed inventory contains 48 named abbreviation entries.
`abbreviation_inventory.tsv` classifies all 48 in source order and now maps every
entry to a stable semantic family. The current 55-family runtime additionally
includes seven independently reviewed derivatives or corpus-listed contractions,
so the family and source-table counts are not a one-to-one denominator. The ten
last source entries introduced a typed mobile-`е`/`ц` noun, velar-adjective
palatalization, `-нн-` reduction and mobile-`е` short-masculine principal parts,
the suppletive `благїй : ᲂу҆́н-` comparison, and closed invariant policies for
`благочестно` and the `имярекъ` rubric.

Unknown lexical stress, semantics, or Greek etymology is intentionally not a
grammar implementation gap. For an arbitrary caller specification it is a
typed metadata boundary; for a bundled lexeme an exact row or reviewed reusable
paradigm supplies the known result, and an uncovered cell fails rather than
guessing. The finite source inventory in `positional_rules.tsv` records all
Alypy §§2 and 36 decision classes and their exception boundaries. Thus the
presentation engine is operationally complete for complete lexical metadata
without claiming that six bundled accent paradigms exhaust an open lexicon.
