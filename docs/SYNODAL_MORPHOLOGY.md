# Synodal Russian Church Slavonic morphology specification

## Authority and scope

Alypy (Gamanovich), *Grammar of the Church Slavonic Language*, corrected web
edition based on the 1991 grammar, is the initial normative morphological anchor.
Named Synodal editions test surface spelling and actual distribution. OCS grammars,
Wiktionary, Polivanova, and OCS corpora may supply inherited analyses and
counterexamples but cannot attest a Synodal surface form.

Every productive rule receives a stable rule ID, exact citation, input contract,
ending/allomorph table, accent behavior, and golden examples before entering
`Strict` or `Productive` output. Until that review exists, the corresponding cell
returns `UnsupportedFormation`; it is not filled by an OCS ending.

## Grammatical inventory

The typed inventory investigates and represents:

- nouns: seven cases and singular, dual, plural, with lexical gender, animacy,
  declension, number restrictions, stem alternants, and accent class;
- adjectives and adjectival participles: short/simple and long/compound forms,
  gender, case, number, animacy, and comparison;
- personal, reflexive, demonstrative, relative, interrogative, indefinite, and
  negative pronouns, plus determiners;
- cardinal, ordinal, collective, and compound numerals with their agreement and
  government;
- verbs: present, imperfect, independent aorist systems, imperative, infinitive,
  supine where normative, l-participles, active/passive participles, verbal nouns,
  and irregular/suppletive paradigms; and
- structured analytic future, perfect, pluperfect, conditional, and passive
  constructions.

Single-word morphology and phrase realization are distinct APIs. Analytic forms
are structured tokens with agreement and provenance, never a space-containing
word result.

## OCS delta

The historical category inventory is a useful checklist, but no OCS rule table is
adopted wholesale. For each system the Synodal implementation records whether it
is retained, transformed, restricted, expanded, lexicalized, or absent. Important
audit points include later ending distributions, spelling conventions, accentual
paradigms, productive versus bookish past systems, dual usage, and changes in
participle and analytic-tense behavior.

The OCS core's present class does not select a past stem, and the Synodal core
preserves that good separation. A verb record stores independent present stem and
allomorphs, infinitive, aorist principal parts, imperfect base, imperative base,
l-participle base, each participial base, accent metadata, and irregular overrides.
Aspect alone never chooses an aorist.

## Generation policies

- `Strict`: exact Synodal cells, reviewed overrides, and normative Synodal rules
  with independently sourced Synodal principal parts. It abstains otherwise.
- `Productive`: additionally admits uniquely compatible, confidence-calibrated
  analyses from reviewed/automatically validated OCS alignments, but realizes the
  output through Synodal rules and labels it prediction.
- `Exploratory`: returns every compatible ranked analysis, including uncertain
  mappings and analogical models, with assumptions and contradictions.

No policy calls a generated form attested. A higher-coverage policy may add
candidates but may not hide precision or ambiguity.

## Canonical resolution

Direct functions, stable-ID calls, resolved handles, reverse analysis, and
paradigms delegate to one canonical cell resolver. The resolver applies:

1. validate target and input orthography;
2. resolve a stable target lexeme identity;
3. check an exact Synodal cell;
4. check a reviewed target irregular override;
5. run a supported Synodal normative rule from sourced target metadata;
6. when policy permits, evaluate explicit inherited mappings and realize them
   with target rules;
7. retain all compatible analyses, evidence, conflicts, assumptions, and trace;
8. return a nonempty `FormSet` or a typed error.

## Implemented productive rule tables

The first release implements the following target rules. `SynodalLiturgical`
realization additionally requires a matching accent-registry row; the expanded
rules do not invent stress.

| System | Stable rule IDs | Normative basis | Implemented input contract |
|---|---|---|---|
| First-declension hard masculine/neuter nouns | `SYN-NOUN-I-HARD-M-ALYPY-34`, `SYN-NOUN-I-HARD-N-ALYPY-34` | Alypy §§34–35 | explicit stem, lexical gender, class, case, number, animacy |
| Second-declension hard/soft nouns | `SYN-NOUN-II-HARD-ALYPY-39`, `SYN-NOUN-II-SOFT-ALYPY-39` | Alypy §§39, 44 | explicit stem and class; no automatic sibilant/velar alternation |
| Third-declension feminine nouns | `SYN-NOUN-III-F-ALYPY-41` | Alypy §41 | explicit feminine stem and class |
| Short adjectives, hard/soft | `SYN-ADJ-SHORT-{HARD,SOFT}-ALYPY-53` | Alypy §§53–55 | positive degree only; explicit stem/class/agreement |
| Long adjectives, hard/soft | `SYN-ADJ-LONG-{HARD,SOFT}-ALYPY-57` | Alypy §§56–57 | positive degree only; explicit stem/class/agreement |
| Present | `SYN-VERB-PRESENT-ALYPY-80` | Alypy §§79–82 | independent full 1sg and 3pl plus medial present stem and conjugation |
| Aorist | `SYN-VERB-AORIST-{VOWEL,CONSONANT}-ALYPY-86` | Alypy §86 | independent aorist base and formation; limited final-velar alternation |
| Imperfect | `SYN-VERB-IMPERFECT-{H,YAH,AH}-ALYPY-87` | Alypy §87 | imperfective/biaspectual verb, independent base and formation |
| Imperative | `SYN-VERB-IMPERATIVE-ALYPY-93` | Alypy §93 | independent base and `first-unpalatalized`/`i-series` formation |
| Infinitive | `SYN-VERB-INFINITIVE-LEXICAL` | Alypy §79 | resolved target lemma; no invented infinitive stem |
| l-participle | `SYN-VERB-LPART-ALYPY-97` | Alypy §97 | independent base plus typed gender and number |

Every productive variant carries `alypy-gamanovich-grammar-web-2023` as
normative evidence and the rule ID as its exact citation. The core accepts these
rules only from explicit metadata. The facade adds target lexeme resolution,
exact-table precedence, accent metadata, irregular overrides, and mapping policy.

### Accusative variation

Alypy §35 states that animate singular noun accusatives are usually
genitive-like but can be nominative-like, while animate plural accusatives are
usually nominative-like but can be genitive-like. The engine therefore returns
both for supported nouns, in that normative order. The adjective tables in
§§53–57 preserve the analogous parenthesized alternatives. Animacy is not
treated as a modern-Russian equation that erases attested Church Slavonic
variation.

### Exact and irregular systems

The target exact-form registry contains 138 reviewable normative-table rows. It
includes the complete nine-cell present, aorist, and imperfect paradigms of
`быти`, its sourced imperative and representative active participles; the
complete nine-cell future auxiliary present of `имати`; a full demonstrative
paradigm for `той`; the reviewed cardinal paradigm of `два`; one determiner cell;
and representative active/passive participle cells for `нести`.

`normative-table` is deliberately classified as sourced prediction, not corpus
attestation. A future `synodal-attestation` row must carry edition and passage
evidence before `FormVariant::is_attested()` can be true.

The productive verb registry contains independently sourced principal parts for
`нести`, `писати`, and `любити`: the 1sg and 3pl present edges, imperfect base and
formation, aorist base and formation, imperative base and formation, and
l-participle base. Present class and aspect never choose a past stem.

## Structured analytic constructions

Single words and phrases have separate result types. Implemented phrase builders
are:

| Construction | Formation | Basis |
|---|---|---|
| Compound future | present of `имати` + imperfective infinitive | Alypy §85 |
| Perfect | l-participle + present of `быти` | Alypy §88 |
| Pluperfect | l-participle + either imperfect series of `быти` | Alypy §89 |
| Conditional | l-participle + aorist of `быти` | Alypy §91 |
| Analytic passive | nominative passive participle + selected copula present | Alypy §§101–102 |

The generic `phrases::from_tokens` retains a typed construction and roles for
reviewed periphrases whose auxiliary lexeme is not yet registered. Agreement and
government beyond these construction-specific constraints remain future work.

## Current reviewed lexical surface

The generated seed registry has 16 target lexemes: seven nouns, one adjective,
five verbs (including two auxiliaries), one pronoun, one determiner, and one
numeral. It is intentionally too small for broad lexical coverage. Productive
rules can fill many cells of a resolved regular lexeme, but they do not infer an
unknown lexeme's class or silently import a surface table from OCS.

Five accepted OCS/Synodal mappings and one rejected negative control form the
initial alignment gold set. One target (`градъ`) deliberately obtains only its
class analysis through a reviewed OCS mapping under `Productive`; the output is
then realized by the Synodal noun rule. `Strict` abstains on that inherited-only
path. Identity and transformed mappings are explicit registry operations, not
string equality.

## Explicitly unsupported or partial formations

The closed grammar enums represent these gaps so paradigms retain failures:

- first-declension soft/mixed masculine and neuter classes, consonantal and
  heteroclitic nouns, lexical stem alternants, number restrictions, and most
  irregular/suppletive nouns;
- automatic velar/sibilant alternation outside the narrowly reviewed aorist
  operation, and the several ending variants in Alypy §§34–44 not yet modeled;
- productive comparative/superlative stems and their irregular lexical series;
- pronouns other than `той`, arbitrary determiners, ordinal/collective/compound
  numerals, and productive declension of numeral lexemes;
- productive active/passive participle stem formation and full declined
  participle tables (only representative exact cells exist);
- the supine pending a target-recension normative inventory, productive verbal
  nouns pending lexical suffix metadata, and unregistered irregular verbs;
- automatic productive accent classes, complete breathing/positional-letter
  realization, and abbreviation families beyond the reviewed Christian sense
  of `богъ`; and
- automatic syntax, free agreement/government, dropped copulas, future
  auxiliaries `хотѣти`/`начати`, and the wider periphrastic inventory of Alypy
  §90.

These are coverage gaps, not invitations to guess. Direct calls return
`MissingPrincipalPart`, `UnsupportedFormation`, `UnsupportedCell`,
`HistoricallyInvalidCell`, or an orthographic metadata error as appropriate.
The real-text fixture is deliberately small; its perfect score is a regression
check for the seed slice and must not be read as language-wide accuracy.
