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

OCS superlative constructions are a separate system. UT lesson 5 §22.4 describes
relative superlatives chiefly as a comparative plus a genitive reference and
absolute superlatives as `ѕѣло` plus a positive or `прѣ-` plus a positive. Lexical
`наи-` comparatives also occur in Polivanova's old-comparative inventory. The
word-level comparator does not mislabel any of those as a universal synthetic
superlative; they require the structured analytic/derivational contract tracked in
the completion matrix.

## Verb rules

### Independent lexical dimensions

`VerbClass` owns only the present conjugation. `VerbLexeme` separately records the
present stem and first-singular allomorph, imperfect stem, formation, and variant
policy, aorist main stem, independent sigmatic 2sg/3sg principal part, and
formation, imperative stem and formation, and one
stem/formation pair for each productive participle. Lexical aspect is metadata and
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
number endings to an explicitly supplied aorist stem. Irregular/root supines and
stems remain dictionary-backed. Authority: UT lessons 2 and 7,
<https://lrc.la.utexas.edu/eieol/ocsol/20#grammar_979> and
<https://lrc.la.utexas.edu/eieol/ocsol/70#grammar_1023>.

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
never selects among them. Suppletive forms of `бꙑти`, `дати`, `ѣсти`, `вѣдѣти`,
`хотѣти`, and motion verbs stay table-backed unless a caller supplies a deliberately
audited root analysis. Authority: UT lesson 3 §§14.1–14.3; Polivanova 2023
§§93, 455, 460, 462, and 476–482,
<https://lrc.la.utexas.edu/eieol/ocsol/30#grammar_987> and
<https://lrc.la.utexas.edu/eieol/ocsol/30#grammar_989>.

### Imperative

`V-IMP-01` requires an explicit imperative stem and either `ISeries` or
`YatSeries`. Both use `-и` in 2sg/3sg. The i-series uses
`-ивѣ/-ита/-имъ/-ите`; the yat-series uses
`-ѣвѣ/-ѣта/-ѣмъ/-ѣте` in 1du/2du/1pl/2pl. These are the only six morphological
cells. First singular, third dual, and third plural periphrases with `да` are outside
the word inflector and return `HistoricallyInvalidCell`.

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

## Recorded source conflicts and limitations

- Current Wiktextract verb rows often carry a spurious `l-participle` tag. A finite
  row is accepted only when tense, person, and number are independently complete.
  Rows with `error-unrecognized-form` are always rejected.
- Declined participles cannot be safely assigned to a participle kind from this
  snapshot and are excluded with a counted reason; citation participles are kept.
- The adjective export flattens short and long table blocks without per-row form
  tags. The extractor uses the two sentinel-delimited blocks and fixture-tests that
  positional interpretation.
- Source tables sometimes omit lexical animacy. The dictionary returns only the
  source-listed forms; the rule engine requires explicit animacy where it changes a
  masculine accusative.
- Glagolitic display forms are never generated from Cyrillic. Only source-backed
  Glagolitic records are returned.
- Wiktextract repeats the combined personal/reflexive table on form-of pages. Those
  pages are not independent lexemes. Personal rows retain their person dimension;
  numberless reflexive rows belong to `сѧ` and are made available for singular, dual,
  and plural without changing their surface variants.
