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

## Verb rules

### Independent lexical dimensions

`VerbClass` owns only the present conjugation. `VerbLexeme` separately records the
present stem and first-singular allomorph, imperfect stem and formation, aorist stem
and formation, imperative stem and formation, and one stem/formation pair for each
productive participle. Lexical aspect is metadata and never chooses an aorist.
Root and irregular lexemes are explicit classes. A caller may use a productive
past-system formation with a root only by declaring the needed stem and formation;
the engine never derives those principal parts from the infinitive.

Every stem is canonicalized and validated before use. A missing stem and a missing
formation report their distinct `MetadataField`; an unimplemented formation reports
`UnsupportedCell`. This model follows the separation of present, aorist, imperative,
and participial systems in UT *Old Church Slavonic Online*.

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

The imperfect needs both `ImperfectStem` and `ImperfectFormation`. The stem is the
lexically selected base before the formation marker.

| Rule | Formation | Operation |
|---|---|---|
| `V-IMPF-A-01` | `A` | stem + `а` + personal ending |
| `V-IMPF-YAT-A-01` | `YatA` | stem + `ѣа` + personal ending |
| `V-IMPF-PAL-A-01` | `PalatalizedA` | final `к/г/х → ч/ж/ш`, then `аа` + personal ending |

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
`мог-` + `PalatalizedA` gives `можаахъ`. Contracted, suppletive, and lexically
exceptional imperfects require an explicit dictionary cell rather than a guessed
variant. Authority: UT lesson 1 §4.2,
<https://lrc.la.utexas.edu/eieol/ocsol/10#grammar_967>.

### Aorists

`AoristFormation` is independent of present class and `VerbAspect`.

- `V-AOR-ASIG-01` implements the source-described asigmatic endings. It preserves
  the explicit stem in 1sg/1du/1pl/3pl and first-palatalizes a final velar in 2sg,
  3sg, 2du, 3du, and 2pl. The endings are `-ъ/-е`, `-овѣ/-ета/-ете`, and
  `-омъ/-ете/-ѫ`.
- `V-AOR-NEW-01` implements the new *ox*-aorist: `-охъ/-е`,
  `-оховѣ/-оста/-осте`, and `-охомъ/-осте/-ошѧ`. Only the 2sg/3sg seam applies
  first palatalization. For example, explicit `рек-` yields `рекохъ` but `рече`.
- `Sigmatic` is represented so metadata can be honest, but generation remains
  explicitly unsupported. The attested sigmatic alternations and stem loss are not
  yet independently encoded.

Multiple aorist formations for one lemma remain separate lexical analyses; aspect
never selects among them. Suppletive forms of `бꙑти`, `дати`, `ѣсти`, `вѣдѣти`,
`хотѣти`, and motion verbs stay table-backed unless a caller supplies a deliberately
audited root analysis. Authority: UT lesson 3 §§14.1–14.3,
<https://lrc.la.utexas.edu/eieol/ocsol/30#grammar_987> and
<https://lrc.la.utexas.edu/eieol/ocsol/30#grammar_989>.

### Imperative

`V-IMP-01` requires an explicit imperative stem and either `ISeries` or
`YatSeries`. Both use `-и` in 2sg/3sg. The i-series uses
`-ивѣ/-ита/-имъ/-ите`; the yat-series uses
`-ѣвѣ/-ѣта/-ѣмъ/-ѣте` in 1du/2du/1pl/2pl. These are the only six morphological
cells. First singular, third dual, and third plural periphrases with `да` are outside
the word inflector and return `UnsupportedCell`.

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
- `V-PTCP-PAST-ACT-01`: `Ush` or `Vush` builds `-ъш-/-въш-` plus the special
  active short nominative, then uses soft adjective agreement.
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

The primary `-ьш-` past-active formation of transformed i-stems, plus automatic
final-j deletion and `ov → u` before `-въш-`, are not inferred by this version.
Such citations remain table-backed unless the caller supplies an already appropriate
stem for one of the represented formations.

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
