# Old Church Slavonic morphology specification

This is the normative specification for `old-church-slavonic-core`. It describes
canonical Old Church Slavonic (`cu`), not a later Church Slavonic recension. Every
productive implementation rule has the stable identifier shown below. Dictionary
tables can preserve lexical and orthographic variants that a single-output rule
cannot predict.

## Authorities and conflicts

- The Kaikki/English-Wiktionary snapshot in `data/SOURCES.toml` is authoritative
  only for the versioned dictionary-table target.
- The University of Texas *Old Church Slavonic Online* grammar is the independent
  authority for the grammatical inventory and the productive generalizations:
  <https://lrc.la.utexas.edu/eieol_toc/ocsol>.
- English Wiktionary's `cu-*` templates clarify the table convention under test,
  but agreement with their own extracted output is not independent validation.
- UD OCS PROIEL is optional manuscript/corpus evidence. It is not a complete
  paradigm and is not bundled.

The template revisions inspected on 2026-08-07 were: `cu-decl-noun-o-m`
89325407, `o-n` 88905593, `a` 88905596, `jo-m` 88905589, `jo-n` 88905590,
`ja` 89731588, `i-f` 91799126, `i-m` 88905601, `u` 88905595, `n-m`
88905603, `n-n` 88905604, `nt` 88905606, `r` 88905607, `s` 88905608, `v`
88905609, `cu-decl-adj-hard` 88905585, and `cu-decl-adj-soft` 88905586.
These are review anchors, not the revision identity of every page in the Kaikki
dump. Kaikki does not provide per-page revision IDs in this export.

## Grammatical inventory

Nominals distinguish nominative, genitive, dative, accusative, instrumental,
locative, and vocative in singular, dual, and plural. Adjectives additionally
agree in gender. `AdjectiveForm::Short` names the historical simple/short
paradigm; `Long` names the compound/long paradigm. “Long” is a morphological
description and is not an unconditional claim of semantic definiteness.

Class selection is lexical. In particular, the core never infers an i-, u-, or
consonant-stem class solely from the last letter. Callers use `NounLexeme` or
dictionary metadata. Number restrictions are checked before an ending is selected.

## N-O-M-HARD-01 and N-O-N-HARD-01 — hard o-stems

Masculine citations remove `-ъ`; neuters remove `-о`. The principal ending table is:

| Case | M singular | M dual | M plural | N singular | N dual | N plural |
|---|---|---|---|---|---|---|
| Nom | ъ | а | и | о | ѣ | а |
| Gen | а | оу | ъ | а | оу | ъ |
| Dat | оу | ома | омъ | оу | ома | омъ |
| Acc | Nom/Gen by animacy | а | ꙑ/Gen | о | ѣ | а |
| Ins | омъ | ома | ꙑ | омъ | ома | ꙑ |
| Loc | ѣ | оу | ѣхъ | ѣ | оу | ѣхъ |
| Voc | е | а | и | о | ѣ | а |

The first velar palatalization is `к/г/х → ч/ж/ш` before the masculine vocative
`-е`; the second is `к/г/х → ц/ѕ/с` at the relevant `-ѣ/-и` seams.

## N-JO-M-SOFT-01 and N-JO-N-SOFT-01 — soft o-stems

Soft masculine citations remove `-ь` (or the template's `-и` citation spelling).
The default front-vowel series is `-ꙗ, -ю, -емь, -и`; sibilant seams use the
documented back-vowel alternatives. Soft neuters remove `-е/-ѥ` and use `-ѥ`
singular, `-и` dual, and `-ꙗ` plural in nominative/accusative/vocative. The rule
returns the default documented form; source-listed alternatives remain table
variants. This seam convention is the largest measured soft-stem error source.

## N-A-HARD-01 and N-JA-SOFT-01 — a-stems

Hard a-stems remove `-а` and use the `-а/-ꙑ/-ѣ/-ѫ/-оѭ` singular series,
`-ѣ/-оу/-ама` dual series, and `-ꙑ/-ъ/-амъ/-ами/-ахъ` plural series. The second
velar palatalization applies at `-ѣ`.

Soft ja-stems remove `-ꙗ` (or a template `-и` spelling) and use the corresponding
`-ꙗ/-и/-ѭ/-еѭ`, `-и/-ю/-ꙗма`, and `-ѩ/-ь/-ꙗмъ/-ꙗми/-ꙗхъ` series. Source tables
contain additional orthographic seam choices; the OOV report measures the default.

## N-I-F-01 and N-I-M-01 — i-stems

Both remove citation `-ь`. They share `-и` obliques, `-ью/-ьма` dual obliques,
and `-ьи/-ьмъ/-ьми/-ьхъ` plural obliques. Masculines use instrumental singular
`-ьмь` and nominative/vocative plural `-ьѥ`; feminines use `-ьѭ` and `-и`.

## N-U-M-01 — u-stems

The explicit u-stem class removes citation `-ъ`. Its distinctive cells include
genitive/locative/vocative singular `-оу`, dative singular `-ови`, dual
`-ꙑ/-овоу/-ъма`, and plural `-ове/-овъ/-ъмъ/-ъми/-ъхъ`.

## Consonant stems

The following rules select an extended stem and then attach the documented case
ending. They are deliberately explicit classes:

- `N-N-M-01`: masculine n-stems, citation `-ꙑ`, extension `-ен-`;
- `N-N-N-01`: neuter n-stems, citation `-ѧ`, extension `-ен-`;
- `N-NT-N-01`: nt-stems, citation `-ѧ`, extension `-ѧт-`;
- `N-R-01`: r-stems such as `мати`, extension `-ер-`;
- `N-S-N-01`: neuter s-stems, citation `-о`, extension `-ес-`;
- `N-V-F-01`: feminine v-stems, citation `-ꙑ`, extension `-ъв-`.

The complete seven-case, three-number endings are directly golden-tested for every
class. Rare lexical alternants stay dictionary-backed.

## N-INDECL-01 — explicit indeclinables

Only explicit lexical metadata activates this rule. It returns the unchanged
canonical lemma in otherwise allowed number cells. Shape alone never marks a word
indeclinable.

## Adjective rules

- `ADJ-HARD-SHORT-01` combines the hard o-stem masculine/neuter endings with the
  hard a-stem feminine endings and the nominal velar seam rules.
- `ADJ-HARD-LONG-01` implements the compound series, including masculine
  nominative `-ꙑи`, feminine `-аꙗ`, neuter `-оѥ`, and the `-ꙑи-` obliques.
- `ADJ-SOFT-SHORT-01` implements the soft simple `-ь/-а/-е` citation series.
- `ADJ-SOFT-LONG-01` implements the soft compound `-ии/-аꙗ/-еѥ` series and its
  `-ии-` obliques.

All four rules use the same case × number × gender resolver and handle masculine
accusative animacy. Declined participles reuse this resolver once a caller supplies a
safe participial stem; malformed Wiktextract rows never supply that stem.

Dictionary adjective records retain `adj-hard`/`adj-soft` metadata when their
canonical masculine citation has the unambiguous `-ъ` versus `-ь/-и` shape. Exact
table cells still win; this metadata is used only for a missing cell. A caller can
always bypass citation-shape inference with an explicit `AdjectiveLexeme`.

Polivanova §§285 and 305 exhaustively list three *plenum tantum* adjectives:
`которыи`, `прокыи`, and `прочии` (canonical engine spellings `которꙑи`,
`прокꙑи`, `прочии`). Their starting form is already the long masculine
nominative singular. `LongOnlyAdjectiveIdentity` records those lexical facts;
all long cells use the ordinary hard or soft compound endings, while every short
cell is historically invalid. Generic explicit metadata likewise recognizes
unambiguous hard long citations in `-ꙑи/-ыи`. Soft `-ии` remains lexically
ambiguous (`прочии` is long-only, while `божии` is short-only), so only the
exhaustive typed inventory interprets that ending as a long citation. A
productive comparative still requires a short positive citation.

### Determiners

Determiner is lexical and syntactic ownership, not a fifth declensional set.
`DeterminerDeclension` therefore selects one of the grammar's actual productive
profiles: regular hard/soft/j-stem class `2/p`, or short/long hard/soft class
`2/a`. `DeterminerLexeme` is the strict arbitrary-lexeme contract and never
infers one of those profiles from spelling. Its lemma must be a citation of the
declared short/long type; this explicitly resolves otherwise ambiguous soft
`-ии` citations.

`DeterminerIdentity` exhaustively allocates eleven reviewed source identities:
the eight determiner-owned members of Polivanova §314's regular `2/p` list
(`ꙗкъ`, `ѥликъ`, `какъ`, `коликъ`, `селикъ`, `такъ`, `толикъ`, `чии`),
exceptional `кꙑи`, long-only `которꙑи`, and `2/a` indefinite `ѥтеръ`.
Polivanova §§285, 303–305, 314–316, and 375–376 plus Paradigmatic Dictionary
entry 343 provide the lexical/class allocation; UT OCS Online §§13 and 23.2
independently crosscheck the pronominal terminals and `кꙑи`/`чии` profiles. The terminal rules
remain `PRON-2P-*`, `DET-UNIQUE-KYI-01`, and `ADJ-*-SHORT/LONG-01`; the facade
does not duplicate them.

`DeterminerCell` carries case, number, gender, and animacy. Regular `2/p` and
`кꙑи` forms are animacy-syncretic and have no vocative. Adjectival `ѥтеръ` and
`которꙑи` retain the adjective system's licensed animacy contrasts and cell
inventory. Every reviewed identity has a complete 126-row paradigm containing
both successful forms and typed historical failures.

## Numeral rules

### Cardinals through 10,000

`CardinalNumeralIdentity` gives stable grammatical identities to the reviewed
simple-cardinal lexicon. One agrees in the singular, two and both in the dual,
three and four in the plural, and five through ten govern a genitive-plural
complement. `NumeralCell` makes the inherent number and optional agreement
gender explicit. The unique three, four, and mixed ten profiles use
`NUM-CARD-THREE-01`, `NUM-CARD-FOUR-01`, and `NUM-CARD-TEN-01`; the other
simple forms reuse independently reviewed nominal or pronominal kernels.

Structured cardinal composition retains every component as a `PhraseToken`.
The engine covers teens in `на десѧте`, multiplicative tens, hundreds,
thousands, `тъма`, and additive `и` through 10,000. It never flattens correlated
alternatives into invalid cross-products. Stable rules are `NUM-CARD-TEEN-01`,
`NUM-CARD-TENS-01`, `NUM-CARD-HUNDRED-01`, `NUM-CARD-THOUSAND-01`,
`NUM-CARD-MYRIAD-01`, and `NUM-CARD-ADDITIVE-01`.

### NUM-CARD-DISTRIBUTIVE-01 — distributive `по` with dative cardinals

OCS does not require an independent synthetic distributive paradigm. Leuta and
Havryliuk's numeral inventory separates cardinal, ordinal, collective, and
fractional classes, while primary-text witnesses realize distributive meaning
with `по` plus a dative cardinal: `по ѥдиномоу`, `по дьвѣма`,
`по пѧти десѧтъ`, and composed magnitude phrases. The engine therefore exposes
`distributive_cardinal` as a structured phrase for every value from 1 through
10,000, reusing the complete cardinal component machinery at dative case.

`DistributiveCardinalCell` carries only optional gender. It requires a gender
exactly when the lowest governing unit is one through four, retains the
cardinal's agreement or genitive-plural government, and has no case field. That
last boundary deliberately excludes temporal `по` plus a locative. The leading
`по` receives reviewed-grammar evidence under `NUM-CARD-DISTRIBUTIVE-01`; every
following token retains its own source, variants, warnings, and trace. Exact
corpus values are attestations of the construction. Larger unattested values
through the already bounded 10,000 cardinal inventory are labeled productive
applications, not corpus forms.

Authority: Leuta and Havryliuk 2018 pp. 154, 156, and 164; pinned UD OCS PROIEL
r2.18 and native Syntacticus witnesses in Codex Zographensis Mark 14:19 and
6:40, Codex Marianus Luke 9:14 and 10:1 and John 8:9 and 21:25, and Codex
Suprasliensis sentences 245344 and 253762.

### NUM-ORD-HARD-01 and NUM-ORD-J-01 — simple ordinals

Polivanova's OSD spreadsheet contains exactly ten ordinal class-`2/a` lexical
identities: `прьвъ`, `въторъ`, `третии`, `четврьтъ`, `пѧтъ`, `шестъ`,
`седмъ`, `осмъ`, `девѧтъ`, and `десѧтъ`. The first, second, and fourth through
tenth use the ordinary hard short/long adjective kernels. Each agrees in case,
number, gender, and masculine-accusative animacy.

The third ordinal is not an ordinary consonant-soft adjective. Its source
workstem is `трет.ьj`; Polivanova §§70 and 72 require the yer before `j` to
surface as `и` and resolve the following terminal across the yod boundary.
Thus the reviewed short profile includes `третии`, `третиѥ`, `третиꙗ`,
`третиѭ`, and `третиѩ`. The same synthesis remains productive for every long
cell. Where that suffix produces multiply adjacent vowels, the canonical rule
output is retained even when unattested, for example `третиии`, `третиѥѥ`,
`третиꙗѥго`, and `третиюѥмоу`; Polivanova §305 explicitly warns that `ьj`
suffixes create morphophonological problems for long forms.

`OrdinalNumeralParadigm` enumerates all 252 form × case × number × gender ×
animacy requests for each identity. The OSD citation has reviewed-table
evidence, productive standard cells have productive-rule evidence, and pinned
UD OCS PROIEL r2.18 spellings remain separate corpus observations. In
particular, the engine retains the unambiguously long `третиѣаго`,
`третию҄моу`, and `третиее` only in their observed cells; it does not assign
form-ambiguous corpus spellings or turn deformations into a global paradigm.

Authority: Polivanova 2023 §§70, 72, 285, 299, and 303–306 plus OSD spreadsheet
rows 939, 1181, 1188, 3073, 4224, 4525, 4864, 5596, 6149, and 6243. UD OCS
PROIEL r2.18 is an attestation crosscheck, not the productive authority.

### NUM-ORD-TEEN-01 through NUM-ORD-CIRCUMLOCUTIVE-01 — compound ordinals

`compound_ordinal` covers every integer from 11 through 1,000 as a structured
ordinal phrase. Ordinary conjunctive and directly attested asyndetic analyses
apply the complete 252-cell short/long × case × number × gender × animacy
request to every agreeing component. Competing source constructions can instead
freeze later components or fixed syntactic tails. Each component and invariant
connector remains a separate token with its own provenance. `RealizedOrdinal`
keeps correlated analyses intact instead of flattening their variants into
unattested cross-products.

`NUM-ORD-TEEN-01` makes the analytic teen the deterministic first analysis: an
inflected simple ordinal followed by invariant `на десѧте`. Reviewed fused teen
stems remain alternative single-token analyses. `NUM-ORD-DECADE-01`,
`NUM-ORD-HUNDRED-01`, and `NUM-ORD-THOUSAND-01` decline the reviewed fused
decade, hundred, and thousand stems as hard adjectives. Forms productively
extended from a source-listed stem remain productive evidence; the inferred
fourteenth and the parallel sixtieth and seven- through nine-hundredth stems are
explicitly reconstructed.

`NUM-ORD-ADDITIVE-01` composes non-head values with `и`, `ти`, or no connector.
The conjunctive analyses decline every component. The source conflict inside
the asyndetic account is represented instead of silently adjudicated:
`Asyndetic` retains the directly attested all-agreeing Suprasliensis
`сътьнааго четврьтааго`, while `AsyndeticFirstComponent` implements
Leuta–Havryliuk's general statement that only the first component declines and
freezes every later part in its masculine short nominative citation form. The
construction tag describes the outermost join, so a three-component analysis
may retain a different source-licensed connector inside its frozen lower chunk.

`NUM-ORD-CIRCUMLOCUTIVE-01` adds the two source-described alternative patterns
for 21–29. `BetweenTens` inflects the unit ordinal and appends fixed
`междю десетма`; `UnitWithinThirdTen` inflects the unit and appends fixed
genitival `третиаго десѧте`. The source's latter witness is
`въ четврьтьи третиаго десѧте`; `въ` is the external governor of the requested
locative unit and is therefore not repeated in every adjective-agreement cell.
The grammar explicitly licenses the patterns for all nine values, so ordinary
unit cells are productive predictions. The two cited unit spellings remain
construction evidence rather than being assigned to an underdetermined global
adjective cell. Fixed tail tokens retain reviewed-table evidence and the exact
source spellings.

The public range constants close this contract at 11–1,000. The reviewed OCS
grammars independently enumerate the thousandth head but do not determine both
the stem and component-inflection rule of a higher ordinal. Requests above
1,000 therefore return an explicit `NUM-SCOPE-BOUNDARY-01` failure rather than
silently importing later Russian composition.

Authority: Gorshkov 2002 §§118–119; Leuta and Havryliuk 2018 pp. 155 and
161–162; Polivanova OSD entries for `сътьнъ` and `тꙑсѧщьнъ`; pinned UD OCS
PROIEL r2.18 and its native Syntacticus lineage for cell-specific manuscript
crosschecks.

### NUM-COLL-PRON-01 and NUM-COLL-ADJ-01 — inherited collectives

The inherited two-through-ten collective series divides between two historical
declension classes. `дъвои`, `обои`, and `трои` use the full class-`2/p`
j-pronominal case × singular/dual/plural × gender system. Their 63-cell typed
product retains all nine vocatives as historical failures. Krys'ko's manuscript
audit independently confirms that `дъвои` and `обои` are not restricted to one
inherent number.

Four through ten use class `2/a`: `четворъ`, `пѧтеръ`, `шестеръ`, `седморъ`,
`осмеръ`, `девѧтеръ`, and `десѧторъ`. Each accepts all 252 adjective cells,
including short/long form and masculine-accusative animacy. Polivanova's OSD
directly lists `четворъ`, `седморъ`, and `десѧторъ`. ESSJa supplies the inherited
parallel `-ер-/-ор-` series and direct OCS citations including `четвѣръ` and
`осмеръ`. Cells productively extended from those direct stems remain productive
rule output; the five, six, and nine stems and unattested parallel variants are
marked reconstructed and always carry a reconstruction warning. The engine does
not promote reconstruction to attestation.

Pinned UD OCS PROIEL r2.18 supplies independent low-collective singular, dual,
and plural checks and direct higher examples such as `четворо`, `четворꙑ`, and
`десꙙторо`. The last spelling is preserved only in its observed short accusative
singular neuter cell. Adverbial `седморо` and `седморицеѭ` remain derivative
evidence, not adjective cells. Likewise, `дъвоица`, `троица`, `четворица`, and
the other `-ица` group formations are noun lexemes rather than collective
agreement forms.

Authority: Polivanova 2023 §§285, 287–299, 303–306, and 314–316 plus OSD rows
1111, 2552, 5608, 6148, 4863, and 1187; Krys'ko 2020 pp. 55–67; ESSJa
collective-series entries; UD OCS PROIEL r2.18 as an attestation crosscheck.

### NUM-FRAC-NOUN-01 — substantival fractional numerals

The OCS fractional system is a closed lexical allocation over ordinary noun
declension, not a productive rule that derives a numeral for every arithmetic
denominator. Leuta and Havryliuk list `полъ` “one half”, `четврьть` “one
quarter”, and `десѧтина` “one tenth” among the rare fractions of the oldest
monuments, and their declensional account additionally names a-stem
`половина` as the synonymous half noun. The resulting stable inventory has
four identities: u-stem masculine `полъ`, a-stem feminine `половина`, i-stem
feminine `четврьть`, and a-stem feminine `десѧтина`.

`FractionalNumeralParadigm` exposes all 21 case × singular/dual/plural noun
cells for each identity. The listed citation is reviewed source evidence;
remaining cells are productive applications of the explicitly assigned noun
class. This permits grammatically coherent but unattested dual and plural
forms without presenting them as corpus observations. The specialized rule
trace ends in `NUM-FRAC-NOUN-01` while retaining the inherited noun rule in its
earlier steps.

Pinned UD OCS PROIEL r2.18 independently attests fractional accusative singular
`полъ` with a genitive whole in Codex Marianus Luke 19:8 and `десѧтинѫ` in
Luke 11:42 and 18:12. Those exact uses remain separate corpus analyses even
where their surface equals the productive output. The OSD spreadsheet also
independently lists `полъ` and `десѧтина`; it does not supply the missing full
fractional inventory. Gorshkov §86 independently lists `полъ` “half” among the
six secure masculine u-stem nouns and gives that class's complete declension.

The same source explicitly assigns `третина` “one third” to Church Slavonic
redactions and compounds such as `полътора` and `полътретиꙗ` to later
monuments. The OCS fractional resolver therefore rejects them. It also does
not invent an arbitrary numerator/denominator composition API: ordinary
cardinals and fractional nouns remain independently available for syntax once
a source licenses a particular construction.

Authority: Leuta and Havryliuk 2018 p. 162; Gorshkov 2002 §86; Polivanova OSD
spreadsheet rows 1186 and 3629; pinned UD OCS PROIEL r2.18 as a cell-specific
attestation crosscheck.

### NUM-INDEF-NOUN-01 — indefinite quantity `несъвѣда`

OCS `несъвѣда` denotes an incalculable or unbounded quantity, not a stable
integer magnitude in the engine's small/church number profile. Lvov's
manuscript-grounded lexical comparison distinguishes it from exact `тъма`
“10,000” and cites instrumental plural `несъвѣдами` in both Codex
Suprasliensis and John the Exarch. The identity therefore reuses the complete
21-cell hard feminine a-stem noun paradigm. Nominative singular is reviewed
lexical evidence, `несъвѣдами` carries primary-text-attestation provenance, and
the remaining grammatically licensed cells are explicit productive predictions.

`IndefiniteNumeralIdentity` deliberately exposes no numeric value. It cannot be
selected as a cardinal magnitude or enter integer composition. This resolves
Gorshkov §115's compressed exact-10,000 gloss in favor of Lvov's named primary
texts; Simonov's complete-context study of Кирик and Pronin's chronological
synthesis independently show why later exact-value systems must not be
projected back into the OCS API.

Authority: Lvov 1966 pp. 247–249; Gorshkov 2002 §115 as a recorded conflict;
Simonov 2006 pp. 81–85 and Pronin 2024 pp. 47–52 as semantic-boundary
crosschecks.

### Comparatives

`ComparativeFormation` records a word-formation strategy independently of
`AdjectiveForm`, because both old and new comparatives decline in short/simple and
long/compound forms. The two principal parts are the syncopated short masculine
nominative singular and expanded short feminine nominative singular. The strict
contract requires both for an old comparative because suffix deletion and
substitutive consonant softening are lexical: `грѫбъ` therefore supplies
`грѫбл҄ь` and `грѫбл҄ьши`. It never guesses the old alternation from the positive.

- `ADJ-COMP-NEW-01` forms the productive new principal parts from an explicitly
  classified positive workstem. It adds surface `-ѣи/-ѣиши`, or first-palatalizes
  final `к/г/х → ч/ж/ш` and adds `-аи/-аиши`: `новъ → новѣи`,
  `горькъ → горьчаи`, `драгъ → дражаи`.
- `ADJ-COMP-OLD-01` inflects independently supplied old principal parts such as
  `грѫбл҄ь/грѫбл҄ьши`. This includes the closed old-comparative and
  *comparativum tantum* inventories without pretending their positive base or
  consonant alternation is predictable.

The short nominative/accusative singular masculine and neuter and the long
nominative/accusative singular masculine use the syncopated stem; animate
masculine accusatives instead use the expanded genitive-shaped form. Every other
cell uses the expanded stem with ordinary soft-adjective endings, except short
feminine nominative singular `-и`, long feminine nominative singular `-иꙗ`, short
masculine nominative plural `-е`, and long masculine nominative plural `-еи`.
Thus the canonical new forms include `новѣи`, `новѣѥ`, `новѣии`, `новѣиши`,
`новѣишиꙗ`, `новѣише`, and `новѣишеи`. The engine deliberately preserves the
canonical short/long contrast even though source spelling frequently collapses
`-ии/-и`.

Authority: Polivanova 2023 §§279–281, 307–313, and 919–924; UT lesson 5
§§22.1–22.3, <https://lrc.la.utexas.edu/eieol/ocsol/50#grammar_1005>. UT's
“short” versus “long” comparator suffix terminology corresponds to the old versus
new formation strategy; it must not be confused with short versus long adjective
declension. Lunt 2001 §§4.19, 4.31, and 4.7 are currently a contents-level
crosswalk only; no claim here depends on inaccessible text.

OCS superlative constructions are a separate typed system:

- `PHRASE-SUP-REL-GEN-01` keeps an inflected comparative and an independently
  inflected genitive reference as separate `PhraseToken`s. Both reference-first
  (`вьсѣхъ бол҄ии`) and head-first orders are representable.
- `PHRASE-SUP-ZELO-01` combines invariant `ѕѣло` with an independently declined
  positive adjective in either attested order.
- `ADJ-SUP-PRE-01` prefixes `прѣ-` to the positive lexeme and then applies its
  ordinary short/long declension, producing one derived word rather than a fake
  phrase.

Lexical `наи-` comparatives in Polivanova's inventory use the already typed old
comparative principal-part contract; the engine does not mislabel them as a
universal synthetic superlative. `RealizedPhrase` preserves the complete `FormSet`
of every token, including variants, evidence, warnings, and word-level traces,
while `primary_text()` is only a convenient source-first rendering.
Authority: UT lesson 5 §22.4 and Polivanova 2023 §§281 and 922.

## Verb rules

### Independent lexical dimensions

`VerbClass` owns only the present conjugation. `VerbLexeme` separately records the
present stem and first-singular allomorph, imperfect stem, formation, and variant
policy, aorist main stem, independent sigmatic 2sg/3sg principal part, and
formation, imperative stem and formation, and one
stem/formation pair for each productive participle. A complete verbal-noun
platform is independent metadata, with the past-passive stem and formation as a
convenience fallback when they are available. Lexical aspect is metadata and
never chooses an aorist.
Root and irregular lexemes are explicit classes. A caller may use a productive
past-system formation with a root only by declaring the needed stem and formation;
the engine never derives those principal parts from the infinitive.

The facade's advanced `DictionaryVerbMetadata` keeps each system as an ordered
array of typed analyses. Every stem, class/formation, source feature, source
spelling, cross-check set, authority, and provenance travels together. Analysis
rank is stable source order; two defensible analyses are generated separately and
are not collapsed into a bag of strings. Ordinary root functions and resolved
handles reach these facts only through the canonical dictionary resolver. Aspect
is an independent sourced field.

Every stem is canonicalized and validated before use. A missing stem and a missing
formation report their distinct `MetadataField`; contradictory normalized fields,
an unimplemented formation, and a historically invalid cell have separate typed
errors. This model follows the separation of present, aorist, imperative, and
participial systems in UT *Old Church Slavonic Online*.

### Present, infinitive, supine, and l-participle

`V-IA1-01`, `V-IA2-01`, and `V-II1-01` through `V-II3-01` attach the e-series or
i-series present endings to an explicit present stem. Second-conjugation 1sg cells
require `present_first_singular`; this replaces the former broad consonant-mutation
guess. A bare infinitive never selects a class or invents an allomorph. Known table
cells still precede every rule.

`V-INF-01` validates and returns an explicit `-ти` citation. `V-SUP-01` supplies the
regular `-ти → -тъ` supine component. `V-LPART-01` attaches l-participle gender and
number endings to an explicitly supplied l-participle stem. Irregular/root supines and
stems remain dictionary-backed. Authority: UT lessons 2 and 7,
<https://lrc.la.utexas.edu/eieol/ocsol/20#grammar_979> and
<https://lrc.la.utexas.edu/eieol/ocsol/70#grammar_1023>.

### V-VERBAL-NOUN-01 — productive derived soft-neuter noun

This is the one productive derivational system included in the OCS completeness
claim. UT lesson 8 §36 defines the verbal substantive from the past-passive
platform plus tense `-ьj-` and assigns the result to the soft neuter `jo`-stem
declension. Polivanova §§483 and 865 independently analyze `-аниѥ`, `-ениѥ`,
and `-тиѥ` as a verbal platform plus a nominal suffix. Polivanova §407 excludes
the result from the verb's finite and nominal representations, so the engine
models a derived noun paradigm attached to a verb identity rather than a
nonfinite verb cell.

`V-VERBAL-NOUN-01` forms a canonical citation in `-иѥ`; `N-JO-N-SOFT-01`
then supplies all seven cases in singular, dual, and plural. The fixed noun
features are neuter and inanimate. A caller may supply either the complete
platform before `-иѥ` or a past-passive stem plus `T`, `N`, or `En`. The
independent input is essential: UT licenses the same formation for intransitive
verbs without an actual passive participle, and Polivanova §276 n.4 cites
`притѧжаниѥ` and `слутиѥ` where the corresponding participle is absent.

Exact citation spelling always precedes the productive profile. In the locked
dictionary, 191 citations seed complete 21-cell paradigms. Of the 134 citations
with an independently extracted passive platform, 117 equal the productive
`-иѥ` output and 17 retain `-ьѥ`; UT describes tense-jer realization as frequent,
not exceptionless. Oblique or number-expanded cells derived from either exact
citation spelling are predictions and carry the citation plus productive-rule
evidence. The generated reverse index includes those declined forms.

The boundary is intentionally narrow. The engine does not derive unrestricted
agent, instrument, result, diminutive, or other nominal lexemes, and a verbal
noun never proves that a passive participle was attested. LOVe's official verb
schema likewise records verbal stems, aspect, arguments, and derivative links
without treating verbal nouns as verb-paradigm cells. Lunt's accessible endpoint
did not expose the relevant body text, so no rule claim depends on it.

Authorities: UT OCS Online lesson 8 §36,
<https://lrc.la.utexas.edu/eieol/ocsol/80>; Polivanova 2023 §§407,
483, and 865 plus §276 n.4; official LMU LOVe schema reviewed 2026-08-15.

### Imperfect

The imperfect needs `ImperfectStem`, `ImperfectFormation`, and
`ImperfectVariantPolicy`. The stem is the lexically selected base before the
formation marker. `UncontractedOnly`, `ContractedOnly`, and `IotatedOnly` are
separate source-order analyses; the API never silently emits their cross-product.
`PresentA` and `PresentYatA` record that the supplied base belongs to the present
rather than the infinitive-aorist system; their separate short and yat-initial
contracts ensure that stem spelling never guesses the terminal series.

| Policy | Formation | Platform-to-ending seam | Rule |
|---|---|---|---|
| `UncontractedOnly` | `A` | `а` | `V-IMPF-A-01` |
| `UncontractedOnly` | `YatA` | `ѣа` | `V-IMPF-YAT-A-01` |
| `UncontractedOnly` | `PalatalizedA` | `к/г/х → ч/ж/ш`, then `аа` | `V-IMPF-PAL-A-01` |
| `UncontractedOnly` | `PresentA` | `а` | `V-IMPF-PRESENT-01` |
| `UncontractedOnly` | `PresentYatA` | `ѣа` | `V-IMPF-PRESENT-01` |
| `ContractedOnly` | `A` | zero | `V-IMPF-CONTRACTED-A-01` |
| `ContractedOnly` | `YatA` | `ѣ` | `V-IMPF-CONTRACTED-YAT-A-01` |
| `ContractedOnly` | `PalatalizedA` | `к/г/х → ч/ж/ш`, then `а` | `V-IMPF-CONTRACTED-PAL-A-01` |
| `ContractedOnly` | `PresentA` | zero | `V-IMPF-PRESENT-CONTRACTED-01` |
| `ContractedOnly` | `PresentYatA` | `ѣ` | `V-IMPF-PRESENT-CONTRACTED-01` |
| `IotatedOnly` | any platform | `ꙗ`, `ѣꙗ`, or `аꙗ` as typed by the formation | `V-IMPF-IOTATED-01` |

| Cell | Personal ending after the marker |
|---|---|
| 1sg | `хъ` |
| 2sg, 3sg | `ше` |
| 1du | `ховѣ` |
| 2du | `шета` |
| 3du | `шете` |
| 1pl | `хомъ` |
| 2pl | `шете` |
| 3pl | `хѫ` |

Thus `нес-` + `YatA` gives `несѣахъ, несѣаше, …, несѣахѫ`, while
`мог-` + `PalatalizedA` gives `можаахъ`. Their contracted analyses give
`несѣхъ` and `можахъ` and use the same full person-number inventory. An explicit
present stem `зов-` plus `PresentYatA` gives `зовѣаше` or contracted `зовѣше`.
The rare iotated policy produces such sourced formations as `исъхнѣꙗше`,
`трьпѣꙗхъ`, and present stem `раду-` plus `PresentA` as `радуꙗше`.
Suppletive platforms and class-specific exceptions remain independent metadata or
exact cells rather than guessed stems.
Authority: UT lesson 1 §4.2,
<https://lrc.la.utexas.edu/eieol/ocsol/10#grammar_967>.

Polivanova 2023 §§455 and 467–472 supplies the complete contracted, present-stem,
and iotated terminal sets. Contracted forms occur in every reviewed source, while
Savvina kniga uses the contracted series almost exclusively; the policy therefore
remains explicit per lexical/source analysis instead of becoming a global default.
Sections 914–915 define the imperfect platform and document further exceptional
analyses. The pinned native-corpus audit found 1,349 diplomatic mismatches in
1,725 oracle-generated imperfect tokens: 890 in the `YatA` slice and 459 in the
explicit-base `A` slice. Of those mismatches, 630 belong to suppletive `бꙑти` and
235 to abbreviation-heavy `глаголати`; 1,100 occur in 3sg. The remainder mixes
contraction, editorial marks, orthographic substitutions, and unsafe principal-part
selection across manuscripts. Those aggregates still do not select one analysis;
the newly implemented policies make the independently established rule available
without treating frequency as lexical or manuscript evidence.

### Aorists

`AoristFormation` is independent of present class and `VerbAspect`.

- `V-AOR-ASIG-01` implements the source-described asigmatic endings. It preserves
  the explicit stem in 1sg/1du/1pl/3pl and first-palatalizes a final velar in 2sg,
  3sg, 2du, 3du, and 2pl. The endings are `-ъ/-е`, `-овѣ/-ета/-ете`, and
  `-омъ/-ете/-ѫ`.
- `V-AOR-NEW-01` implements the new *ox*-aorist: `-охъ/-е`,
  `-оховѣ/-оста/-осте`, and `-охомъ/-осте/-ошѧ`. Only the 2sg/3sg seam applies
  first palatalization. For example, explicit `рек-` yields `рекохъ` but `рече`.
- `V-AOR-SIG-PRIMARY-01` implements Polivanova's old sigmatic 1 `-с-` main
  subbundle: `-съ/-совѣ/-ста/-сте/-сомъ/-сте/-сѧ`.
- `V-AOR-SIG-SECONDARY-01` implements the old sigmatic 2 `-х-` main subbundle:
  `-хъ/-ховѣ/-ста/-сте/-хомъ/-сте/-шѧ`.
- `V-AOR-SIG-VOWEL-01` preserves the morphologically distinct standard
  vowel-stem sigmatic analysis, which selects the zero-`о` `-х-` series. Thus
  explicit `зна-` produces `знахъ, знаховѣ, знаста, ... , знашѧ`; it is not
  relabeled as the old `рѣхъ` subtype merely because most endings coincide.

All three sigmatic formations require an already graded surface main stem and a
complete, independent syncretic 2sg/3sg principal part. The engine therefore does
not guess compensatory lengthening, an `s/x` boundary, first palatalization, or the
lexically restricted zero/`-тъ`/`-стъ` choice. For example, separate analyses with
`ѧ` and `ѧтъ` produce those two sourced singular variants without admitting their
cross-product. Missing singular metadata is a typed
`AoristSecondThirdSingular` failure. The validated builder rejects a sigmatic
formation passed through the one-stem `aorist` method and rejects a consonant-final
stem for `SigmaticVowel`.

Multiple aorist formations for one lemma remain separate lexical analyses; aspect
never selects among them. Authority: UT lesson 3 §§14.1–14.3; Polivanova 2023
§§93, 455, 460, 462, and 476–482,
<https://lrc.la.utexas.edu/eieol/ocsol/30#grammar_987> and
<https://lrc.la.utexas.edu/eieol/ocsol/30#grammar_989>.

### Irregular and defective lexical contract

`VerbLexeme` can carry source-reviewed exact forms keyed by the typed
`VerbMorphologyCell` inventory. It can also classify a complete
`VerbMorphologySystem` or an individual cell as either historically invalid or
unattested and unreconstructable. Resolution order is exact cell, cell defect,
system defect, then productive principal-part rules. An attested survivor can
therefore remain available inside an otherwise unreconstructable sparse system.
The two defect classes remain observably different:
`HistoricallyInvalidCell` states a grammatical exclusion, while
`UnattestedUnreconstructableCell` states that the reviewed evidence and current
analysis do not license an output. `UnsupportedCell` remains reserved for an
engine path that has not been implemented.

`V-IRREG-EXACT-01` records exact-cell selection in the rule trace. Exact forms
are still rule-core values rather than claims of corpus provenance; the facade
must attach the source evidence that licensed each form. Every productive
subsystem retains its own independently supplied principal part. In particular,
the l-participle now uses `VerbStems::l_participle`, never the aorist stem: this
is required by suppletive profiles such as `ити` (`ид-` aorist versus `шь-`
l-participle). The closed unique-verb inventory and reusable irregular workstem
groups are separately source-audited lexical layers built on this contract.
Authority: Polivanova 2023 §§417, 434–440, 516–605, especially Table 516.3 and
§§561–562.

`UniqueVerbIdentity` now closes the first of those layers over exactly nineteen
base profiles: `дати`, `ꙗсти`, `вѣдѣти`, `имѣти`, independent defective
`ѥсмь`, `бꙑти`, `хотѣти`, `довьлѣти`, `ити`, `ꙗти`, `стати`, `съпати`,
`въпити`, `сѣсти`, `лещи`, `обрѣсти`, `гънати`, `плѣти`, and `дѣти`.
Every identity supplies a nine-cell reviewed present profile, aspect, profile
kind, source section, and independent metadata for each reconstructable
subsystem. The exhaustive profile test requires every public cell either to
produce a form or to return one of the two lexical defect errors; missing
metadata and generic unsupported formation are forbidden inside this closed
layer.

`UniqueVerbFamilyMember` closes the dictionary family union listed in
Polivanova §§520–604: 106 distinct source spellings across those nineteen
profiles. It preserves source order and excludes nearby productive lexemes;
notably, §604 lists five prefixed members of the `дѣти` profile but no
prefixless dictionary headword. Family assembly prefixes the independent
principal parts only where the profile permits it, and uses explicit
system-specific allomorph maps for `ꙗсти`, `ити`, `ꙗти`, `въпити`,
`°рѣсти`, and `гънати`. Thus forms such as `придѫ` / `пришьлъ`,
`сърѧщѫ` / `сърѣтохъ`, and `иженѫ` / `изгънахъ` do not rely on a single
string-prefix heuristic. The source-specific defect contract also preserves
the unattested imperfect of prefixed `бꙑти` members and the attested
`забъвенъ` past passive. Exhaustive tests apply the same no-missing-metadata
rule to every cell of all 106 members.

Source dashes in Table 516 mean that no forms license reconstruction, so they
map to `UnattestedUnreconstructableCell`, not automatically to grammatical
impossibility. The independent `ѥсмь` profile is the stronger exception:
Polivanova §539 defines it as genuinely defective outside its present and
present-active participle, and those cells return `HistoricallyInvalidCell`.
For the extremely sparse `плѣти`, Polivanova §598 directly supports only
`плѣвемъ` and `плѣвом-`; LOVe supplies the comparative `plěv-/plě-/plěvi`
principal parts used for its reviewed reconstruction, while the remaining
systems stay explicitly unreconstructable.

Two additional present-active participle seams preserve unique-profile
oppositions that the ordinary hard/soft formations cannot express:
`MixedYushtSoft` forms `-ѧ/-ѫшт-` (`дѣти`, `°рѣсти`), and
`IotatedYushtSoft` forms `-ѩ/-ѭшт-` (`въпити`, `довьлѣти`). These are typed
formations rather than spelling postprocessing.

`IrregularVerbGroup` inventories all thirteen reusable workstem groups of
Polivanova Tables 434 and 440. `IrregularVerbFamilyMember` closes all seventy
source-listed members with explicit prefix and root allomorph maps; every member
realizes every finite, imperative, infinitive, supine, l-participle, and
declined-participle cell. The model keeps the table's subparadigm distributions
explicit: for example, `крꙑти` uses `крꙑ-` in its present and past-active
participle but `кръв-` in the past passive, while `мрѣти` distinguishes `мьр-`,
`мрь-`, and `мрѣ-`. It likewise preserves prefixed root alternations such as
`възѧти` / `възьмѫ`, `разскврѣти` / `разсквьрѫ`, and `брѣщи` / `брѣгѫ` /
`брьженъ`. The vowel/j-present groups required a separate
`PresentPassiveParticipleFormation::IotatedEm`, which forms `-ѥм-` directly
(`плюѥмъ`, `крꙑѥмъ`, `поѥмъ`) and is also accepted by the typed metadata and
extractor contracts.

Table 434 appears to transpose the displayed past stems of `мрѣти` and
`разскврѣти`. The implementation follows each lexeme's own root and the uniform
Table 440 distribution (`мрьлъ/мрьтъ`, `разскврьлъ/разскврьтъ`) rather than
copying the crossed labels. LOVe also assigns `метати` a `мещ-/метај-` present
analysis, while Polivanova places it in the 3° `мет-/метѫ` group; these remain
competing lexical analyses rather than being silently conflated. Ordinary
facade calls return the ordered `метѫ`, `мещѫ`, and `метаѭ` analyses where the
sources diverge. Cells with identical output merge into one surface analysis
while retaining all source evidence.

All unique-family and reusable irregular-family members route through the
ordinary facade. Resolution order is exact dictionary cell, reviewed manual
override, closed reviewed profile, then open-class dictionary metadata.
Reviewed source spellings absent from the bundled dictionary receive stable
`reviewed:ocs:verb:*` identities, so free calls, handles, by-ID calls, and full
paradigms share the same behavior. Every profile cell carries direct-table or
productive-rule evidence; predicted continuations are warned as predictions,
and source conflicts use `ReviewedGrammarAnalyses` without choosing a winner.

`ImpersonalVerbIdentity` closes the two impersonal senses in the pinned OCS
dictionary: lexically impersonal `достоꙗти` ‘befit’ and reflexive impersonal
`мьнѣти сѧ` ‘seem’. Both govern a dative experiencer. Impersonality is modeled
as syntactic third-person-singular selection, not as deletion of mechanically
possible word forms: the personal senses and full morphology of `мьнѣти` remain
available. `достоꙗти` and `мьнѣти` retain their dictionary-backed present and
imperfect forms, and their regular vowel aorists are explicit reviewed
reconstructions. Passive participles of intransitive `достоꙗти` are typed as
historically invalid; all source-listed passive forms of personal `мьнѣти`
remain available. `PHRASE-IMPERSONAL-PRED-01` preserves the finite word's exact
or reviewed provenance and keeps reflexive `сѧ` as a separate token.

### Imperative

`V-IMP-01` requires an explicit imperative stem and either `ISeries` or
`YatSeries`. Both use `-и` in 2sg/3sg. The i-series uses
`-ивѣ/-ита/-имъ/-ите`; the yat-series uses
`-ѣвѣ/-ѣта/-ѣмъ/-ѣте` in 1du/2du/1pl/2pl. These are the only six morphological
cells. First singular, third dual, and third plural periphrases with `да` are outside
the word inflector and return `HistoricallyInvalidCell`.

`PHRASE-IMPV-DA-01` represents the distinct analytic imperative/optative as the
particle `да` followed by an independently resolved present-tense token. It
covers all nine person-number combinations: the construction supplies missing
first/third-person commands and can also be used where a synthetic imperative
exists. The phrase never changes the six-cell synthetic inventory and never stores
spaces in a `FormSet`. Authority: UT lesson 2 §9,
<https://lrc.la.utexas.edu/eieol/ocsol/20#grammar_979>.

### Analytic verb constructions

Analytic tense, mood, and voice forms are `RealizedPhrase`s, never whitespace-
bearing word forms. The engine first resolves every independently inflected token
to a full `FormSet`, then validates the construction's exact role signature and
agreement constraints. Both auxiliary-first and lexical-head-first orders are
available where the reviewed sources license them.

The auxiliary API keeps six historically distinct copular series separate:

- `V-COP-ES-PRES-01`: present `ѥсмь, ѥси, ... сѫтъ`, used in the
  perfect;
- `V-COP-BUD-PRES-01`: future `бѫдѫ, бѫдеши, ... бѫдѫтъ`;
- `V-COP-BE-IMPF-01` and `V-COP-BE-AOR-01`: the `бѣ-` imperfect and
  aorist series used by the pluperfect;
- `V-COP-BI-COND-01`: the dedicated `би-` conditional series, including
  source-marked reconstructions for the unattested dual and expected plural
  `бите`;
- `V-COP-BY-COND-AOR-01`: the `бꙑ-` aorist series in its independently
  licensed conditional function.

Reviewed grammar-table forms use `FormSource::ReviewedGrammarTable`. A
reconstructed alternative uses `ExplicitMetadataRule` with
`ProductiveRuleOutput` evidence, a rule trace, and `IncludesReconstructedForms`;
it is therefore usable without being presented as attested. This separation
also prevents the dictionary's aggregated `бꙑти`
record and its negative `нѣс-` spellings from leaking into positive perfects.

The productive construction inventory is:

- `PHRASE-PERFECT-01`: agreeing l-participle + present `ѥс-`;
- `PHRASE-PLUPERFECT-01`: agreeing l-participle + imperfect `бѣ-`,
  aorist `бѣ-`, or the three-token perfect of `be`;
- `PHRASE-FUT-INF-01`: infinitive with present `въчѧти`, `начѧти`,
  `имѣти`, or `хотѣти`; imperfect/aorist future-in-the-past is
  admitted only for `имѣти` and `хотѣти`;
- `PHRASE-FUT-PTCP-01`: short nominative present- or past-active participle
  + future `бѫд-`;
- `PHRASE-FUT-PERFECT-01`: agreeing l-participle + future `бѫд-`;
- `PHRASE-COND-OPT-01`, `PHRASE-COND-OPT-DA-01`, and
  `PHRASE-COND-OPT-ELLIP-01`: ordinary, `да`-marked, and auxiliary-elliptical
  conditional-optatives;
- `PHRASE-COND-OPT-PASS-01`: the source-attested conditional with a passive
  predicate;
- `PHRASE-PASSIVE-01`: a short nominative present/past passive participle
  with explicitly selected present, past, future, or conditional copula;
- `PHRASE-IMPERSONAL-PRED-01`: third-person-singular `достоꙗти`, or
  third-person-singular `мьнѣти` plus independently written reflexive `сѧ`, in
  the present, imperfect, or reconstructable aorist.

Predicative participles must be short nominatives and match the subject number;
voice mismatches and unlicensed auxiliary/tense combinations are typed failures.
The complete `имѣти` finite profile and missing `хотѣти` aorist cells
are source-reviewed curated overrides rather than guessed regular forms.
Authorities: UT OCS Online lessons 5 §24, 6 §27, 7 §§32/35, and 9
§45.2; Polivanova 2023 §§516 and 532–555; LMU Digital Editions Reference
Grammar §§5.3–5.5.

The pinned Wiktionary target sometimes spells 1du with final `-ве`; such exact table
variants still win, while the productive rule follows the grammar's `-вѣ`. Optional
plural `-ꙗмъ/-ꙗте` variants for some types remain table-backed until the typed result
can associate them with a justified formation. Authority: UT lesson 2 §9,
<https://lrc.la.utexas.edu/eieol/ocsol/20#grammar_979>.

### Participial stem formation and agreement

All four rules require an explicit participial stem and formation. Stem formation
is the verb rule's first trace step; agreement is the second. The agreement step is
owned exclusively by `adjective::decline_stem`.

- `V-PTCP-PRES-ACT-01`: `YushtHard`, `YushtSoft`, or `YeshtSoft` builds the
  `-ѫшт-/-ѧшт-` oblique stem and its source-described special short nominatives.
  Other short cells and every long cell use soft adjective agreement.
- `V-PTCP-PRES-PASS-01`: `Im`, `Em`, or `Om` builds `-им-/-ем-/-ом-`, then uses
  hard short/long adjective agreement.
- `V-PTCP-PAST-ACT-01`: `Ush`, `Ish`, or a typed `Vush` seam builds
  `-ъш-/-ьш-/-въш-` plus the special active short nominative, then uses soft
  adjective agreement. `VushAfterJDeletion` declares that the supplied Cyrillic
  base already reflects loss of underlying final *j*; `VushAfterOvToU` requires
  final `-ов`, changes it to `-оу`, and then attaches `-въш-`; plain `Vush`
  performs no extra seam. A malformed `VushAfterOvToU` input is a typed error.
- `V-PTCP-PAST-PASS-01`: `T`, `N`, or `En` builds `-т-/-н-/-ен-`, then uses hard
  adjective agreement.

`ParticipleCell` carries kind, short/long form, case, number, gender, and animacy;
it is distinct from a citation participle and from `LParticipleCell`. The active
short nominative seams and every ordinary adjective agreement cell are golden or
metamorphically tested. Authorities: UT lesson 6 §26 and lesson 7 §§31.1–32,
<https://lrc.la.utexas.edu/eieol/ocsol/60#grammar_1017>,
<https://lrc.la.utexas.edu/eieol/ocsol/70#grammar_1023>,
<https://lrc.la.utexas.edu/eieol/ocsol/70#grammar_1024>, and
<https://lrc.la.utexas.edu/eieol/ocsol/70#grammar_1025>.

The core therefore covers independently declared `-ъш-`, transformed i-stem
`-ьш-`, ordinary `-въш-`, final-j deletion, and `ov → u` seams. Automatic
selection of the two special seams from a spelling remains deliberately absent;
dictionary extraction admits only what the citation itself diagnoses, while an
explicit caller may select the more precise typed seam.

### OCS accent boundary and explicit reconstruction

Canonical OCS Cyrillic remains unaccented. The canonical manuscripts do not
provide a complete, standardized surface-accent orthography from which every
morphological cell can be recovered: Polivanova §§648 and 842 describe accent as
unstable and sometimes unobservable, and Trager's public description of the Kiev
Folia treats that manuscript as an exceptional early accented witness while most
accented Church Slavonic texts are later and recension-influenced. Lunt's
accessible contents locate historical Common Slavic accent in §§51–52, but the
body was inaccessible and no rule claim is inferred from a heading.

`OCS-ACCENT-RECON-01` is therefore a separate compositional analysis, exposed as
`reconstruct_accent`. An `AccentParadigm` must name its evidence and assign every
requested cell exactly one nonoverlapping rule. Each rule either declares the
cell atonic or addresses a vowel from the left or right edge of the complete
generated wordform; yers remain possible stress bearers because placement is an
explicit scholarly analysis rather than a spelling heuristic. The atonic state
also covers clitics and vowel-less orthographic words without inventing a mark.
Stressed results use U+0301 as a neutral scholarly stress mark, and every path
returns `ReconstructedAccent`, whose comparative or disputed status, source
citation, paradigm identity, and trace prevent the form from being mistaken for
attested spelling.

Unicode TN41 revision 1 §§2, 3.2–3.3, and 5.2 controls code points and combining
order, not morphology. This profile never generates historical breathing marks,
never overwrites an already accented or breathing-marked form, and does not
accept Glagolitic input; Glagolitic orthography is a separate system. Missing,
overlapping, or out-of-range metadata fails with a typed error instead of falling
back to inference.

Exact source spellings retain precedence. The locked Wiktionary snapshot contains
231 acute-marked form rows across 11 lexemes and 21 psili-marked rows for one
lexeme, with no grave, kamora, or dasia rows. Those 252 rows remain exact
dictionary evidence and are never generalized into an OCS-wide accent or
breathing rule.

### Dictionary principal-part derivation contracts

All contracts run after held-cell exclusions. An available non-source diagnostic
cell must reproduce exactly; any contradiction rejects the analysis. Source order
sets `analysis_rank`, and every output stores authority
`wiktionary-kaikki-2026-07-06` plus either `dictionary-principal-part` or
`dictionary-headword-metadata` provenance. Productive metadata derivation admits
only Cyrillic lemmas and stems because the current rules emit Cyrillic endings;
Glagolitic remains exact-source-only rather than producing mixed-script forms.

| Field/system | Admitted source and operation | Prerequisite, cross-check, and rejection policy |
|---|---|---|
| aspect | unique `cu-verb` argument or `head` gender/aspect argument | unknown or conflicting codes reject; never selects tense or aorist |
| present | class from the audited head template; remove class-specific `-еши/-иши` from 2sg; remove `-ѫ/-ѭ` from 1sg allomorph | only IA1/IA2/II1/II2/II3; second conjugation requires 1sg; every other available present cell must agree; no consonant mutation |
| imperfect | remove `-ѣахъ` as `YatA` or `-ахъ` as `A` from 1sg; attach `UncontractedOnly` under UT lesson 1 §4.2 | all other available imperfect cells must agree; a surface palatalized `-аахъ` is rejected because it cannot recover the underlying velar; no corpus-derived contraction policy is inferred |
| new aorist | remove `-охъ` from 1sg and declare `New` | all other available aorist cells must agree; asigmatic/sigmatic shapes are not inferred |
| imperative | remove final `-и` from 2sg | i/yat series must be diagnosed by an exact 1du/2du/1pl/2pl match; missing or contradictory diagnostics reject; both defensible series remain separate |
| l-participle | remove `-лъ` from masculine singular | every other available gender/number cell must agree; at least one cross-check is required |
| present active citation | remove short masculine `-ꙑ/-ѩ/-ѧ` and select `YushtHard/YushtSoft/YeshtSoft` under the declared present class | malformed/empty bases reject; source alternatives remain ranked analyses |
| present passive citation | remove `-имъ/-емъ/-омъ` | suffix directly selects `Im/Em/Om`; ambiguous or empty bases reject |
| past active citation | remove `-ъ/-ь/-въ` | selects `Ush/Ish/Vush`; special j-loss and `ov → u` are never guessed from this source |
| past passive citation | remove `-тъ/-нъ/-енъ` longest suffix first | selects `T/N/En`; empty bases reject |
| verbal-noun citation | preserve the exact listed `-иѥ/-ьѥ` noun identity; otherwise reuse a complete past-passive platform | exact citation precedes production; every non-citation case-number cell is declined as soft neuter and labeled predictive; an independent caller platform covers nouns without an attested participle |

Citation-participle contracts intentionally have no independent table cross-check in
this snapshot. Leakage-controlled evaluation removes the citation target before
derivation, so a citation can never prove its own predicted form. Declined source
rows remain excluded rather than being used as an unsafe cross-check.

### Curated irregular overrides

Overrides are complete-cell records outside the source table and metadata arrays.
The initial reviewed set covers six otherwise absent imperfect cells of `бꙑти`,
using the pinned dictionary 3sg `бѣаше` as the suppletive base and the endings in UT
lesson 1 §4.2. The source 3sg remains `DictionaryTable`; overrides carry
`CuratedGrammarOverride` evidence and their full authority. No development override
is inferred from the final lemma partition, and other high-value irregular verbs
remain exact-table or explicitly unsupported until an equally specific audit exists.

### Personal, reflexive, and anaphoric pronouns

The reviewed closed grammar inventory has four intrinsic identities rather than one
dictionary-shaped table: first person `азъ`, second person `тꙑ`, the numberless
reflexive `сѧ`, and the defective third-person anaphoric `*и`. First and second
person each license all 18 case-number cells except vocative. Their person is a
lexical property, not a freely selectable cell dimension. The reflexive licenses
only its five oblique cases and has no number dimension; it is not copied into three
grammatical paradigms merely because a source adapter once exposed it that way.

The anaphoric pronoun has 45 licensed oblique case-number-gender cells. Nominative
and vocative are historically absent; demonstratives such as `тъ` and `онъ` fill
nominative syntax and remain in the separate other-pronoun system. Every oblique
cell has a free `j-` realization and a typed `н҄-` realization selected by
`AnaphoricEnvironment::AfterPreposition`. The environment is part of the request,
so the resolver never guesses from an absent preposition string.

Grammar-table order is preserved with `PronounFormSelection`. `TablePrimary`
selects forms not specially marked as clitics by the reviewed tables,
`MarkedClitic` selects only explicitly marked variants, and `All` retains both.
These names do not infer the prosody of unmarked short forms. UT lesson 2 §8.1
lists first-person dual dative `на`, whereas Polivanova §382.3 reports no OCS
clitic attestation in that cell and compares the later Church Slavonic form. The
variant is retained with `DisputedGrammarTable` evidence and
`IncludesDisputedForms`, never silently promoted or discarded.

All 13 spellings in the pinned dictionary source union that belong to these four
identities route through their canonical reviewed paradigm. A form-page spelling
adds `LexicalAliasUsed` but cannot acquire the combined table copied onto that
page. Raw dictionary APIs continue to expose the normalized extraction record for
source audit. Exhaustive goldens cover the 36 first/second cells, five numberless
reflexive cells, and both 45-cell anaphoric environments; exhaustive public
paradigms retain every valid result and every historically invalid cell.

## Recorded source conflicts and limitations

- Current Wiktextract verb rows often carry a spurious `l-participle` tag. A finite
  row is accepted only when tense, person, and number are independently complete.
  Rows with `error-unrecognized-form` are always rejected.
- Declined participles cannot be safely assigned to a participle kind from this
  snapshot and are excluded with a counted reason; citation participles are kept.
- Verbal nouns are nominal derivations, not an extra participle or absolutive.
  Only the explicitly sourced `-ьj-/-иѥ` system is productive; exact retained-jer
  spelling remains lexical evidence and all other nominal derivation is out of scope.
- The adjective export flattens short and long table blocks without per-row form
  tags. The extractor uses the two sentinel-delimited blocks and fixture-tests that
  positional interpretation.
- Source tables sometimes omit lexical animacy. The dictionary returns only the
  source-listed forms; the rule engine requires explicit animacy where it changes a
  masculine accusative.
- Glagolitic display forms are never generated from Cyrillic. Only source-backed
  Glagolitic records are returned.
- Wiktextract repeats a combined first/second-person table on headword and form
  pages and exposes reflexive rows without a number dimension. Ordinary resolution
  classifies all 13 affected source-union spellings into four intrinsic identities
  and uses the reviewed grammar tables above. The normalized raw dictionary record
  remains available for source audit and does not define lexical ownership.
