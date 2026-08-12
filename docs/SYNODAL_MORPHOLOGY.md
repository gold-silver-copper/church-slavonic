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
| First-declension soft masculine/neuter nouns | `SYN-NOUN-I-SOFT-M-ALYPY-34`, `SYN-NOUN-I-SOFT-N-ALYPY-34` | Alypy §§34–35 | explicit stem, lexical gender, class, case, number, animacy |
| Second-declension hard/soft nouns | `SYN-NOUN-II-HARD-ALYPY-39`, `SYN-NOUN-II-SOFT-ALYPY-39` | Alypy §§39, 44 | explicit stem and class; no automatic sibilant/velar alternation |
| Third-declension feminine nouns | `SYN-NOUN-III-F-ALYPY-41` | Alypy §41 | explicit feminine stem and class |
| Short adjectives, hard/soft | `SYN-ADJ-SHORT-{HARD,SOFT}-ALYPY-53` | Alypy §§53–55 | positive degree only; explicit stem/class/agreement |
| Long adjectives, hard/soft | `SYN-ADJ-LONG-{HARD,SOFT}-ALYPY-57` | Alypy §§56–57 | positive degree only; explicit stem/class/agreement |
| Comparative/superlative full adjectives | `SYN-ADJ-{COMPARATIVE,SUPERLATIVE}-LONG-ALYPY-{58,59}` | Alypy §§58–59 | independently reviewed comparison stem; full forms only |
| Present | `SYN-VERB-PRESENT-ALYPY-80` | Alypy §§79–82 | independent full 1sg and 3pl plus medial present stem and conjugation |
| Aorist | `SYN-VERB-AORIST-{VOWEL,CONSONANT}-ALYPY-86` | Alypy §86 | independent aorist base and formation; limited final-velar alternation |
| Imperfect | `SYN-VERB-IMPERFECT-{H,YAH,AH}-ALYPY-87` | Alypy §87 | imperfective/biaspectual verb, independent base and formation |
| Imperative | `SYN-VERB-IMPERATIVE-ALYPY-93` | Alypy §93 | independent base and `first-unpalatalized`/`i-series` formation |
| Infinitive | `SYN-VERB-INFINITIVE-LEXICAL` | Alypy §79 | resolved target lemma; no invented infinitive stem |
| l-participle | `SYN-VERB-LPART-ALYPY-97` | Alypy §97 | independent base plus typed gender and number |
| Declined active participles | `SYN-VERB-PARTICIPLE-{PRESENT,PAST}-ACTIVE-ALYPY-{95,96}` | Alypy §§95–96 | tense/voice-specific full-form stem and adjective class; special short nominatives remain exact-only |
| Declined passive participles | `SYN-VERB-PARTICIPLE-{PRESENT,PAST}-PASSIVE-ALYPY-{99,100}` | Alypy §§99–100 | independent short/full stems; past-passive `н`/`нн` distinction is explicit metadata |

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

The generated v0.7 target registry contains 3,041 reviewable exact rows: normative
tables and variants, plus passage-identified target attestations admitted by the
v0.3 lexical review overlay. It includes the complete nine-cell present, simple
future, aorist, and imperfect paradigms of `быти`, its sourced imperative and
representative active participles; the
complete nine-cell future auxiliary present of `имати`; a full demonstrative
paradigm for `той`; reviewed exact cells for `сей`, `иже`, `кто`, `что`, and
`нѣкто`; the reviewed cardinal paradigms of `два`, `три`, and `четыре`; one
exceptional determiner cell; representative active/passive participle cells for
`нести`; the first-, second-, and complete gendered third-person
personal-pronoun paradigms; and the
complete reviewed present, imperative, and aorist tables of archaic `дати` from
Alypy §103.

The v0.4 family review adds the complete, accent-specified `землѧ` paradigm from
Alypy §39; independently delimited exact future/aorist/imperative cells for
`рещи` from §104; five cited mixed cells for `весь` with the §48 no-dual
restriction; five `сынъ` consonantal/`сынов-` overrides from §37; and seven typed
`господь` contractions under §3. Only `землѧ` is admitted as a complete new
productive class. The other admissions remain exact-cell families and do not
license an inferred paradigm.

The v0.5 review adds complete 57-cell, accent-specified possessive adjective
tables for `мо́й`, `тво́й`, `на́шъ`, `ва́шъ`, and `сво́й`; eight reviewed
reflexive-pronoun cells; additional exact cells for `сей` and `иже`; 66
high-frequency lexical identities admitted as exact target evidence; bounded
exact-form batches; and 42 additional typed abbreviation evaluation cases.
These additions are deliberately data-driven: they do not introduce a new
productive morphology rule, infer a class for an exact-only lexeme, or infer
unreviewed abbreviation cells.

`normative-table` is deliberately classified as sourced prediction, not corpus
attestation. A future `synodal-attestation` row must carry edition and passage
evidence before `FormVariant::is_attested()` can be true.

The productive verb registry contains independently sourced principal parts for
`нести`, `писати`, `любити`, and the supported non-present systems of `дати`: the 1sg and 3pl present edges, imperfect base and
formation, aorist base and formation, imperative base and formation, and
l-participle base. `нести` additionally has four separate tense/voice
participial systems, and `дати` has independently reviewed past-active and
past-passive stems. Present class and aspect never choose a past stem.

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

The reviewed registry has 855 target lexemes and 855 reviewed senses. The
original productive seed remains deliberately small: corpus-prioritized hard and
soft nouns, regular adjectives, six verbs, reviewed closed-class paradigms, and
five numerals. The v0.3 additions are primarily high-frequency exact lexical
evidence. Inflectable additions use `LexicalForm` unless a class or independent
principal parts have actually been reviewed; that cell supports only its exact
target form and never enables a productive rule. Productive rules can fill many
cells of a resolved regular lexeme, but they do not infer an unknown lexeme's
class or silently import an OCS surface table.

Five accepted OCS/Synodal mappings and one rejected negative control form the
initial alignment gold set. One target (`градъ`) deliberately obtains only its
class analysis through a reviewed OCS mapping under `Productive`; the output is
then realized by the Synodal noun rule. `Strict` abstains on that inherited-only
path. Identity and transformed mappings are explicit registry operations, not
string equality.

## Explicitly unsupported or partial formations

The closed grammar enums represent these gaps so paradigms retain failures:

- mixed, consonantal and heteroclitic nouns, lexical stem alternants, number restrictions, and most
  irregular/suppletive nouns;
- automatic velar/sibilant alternation outside the narrowly reviewed aorist
  operation, and the several ending variants in Alypy §§34–44 not yet modeled;
- automatic comparison-stem formation, short comparison series, and irregular
  comparison stems other than the reviewed `мꙋдръ` series;
- reflexive, relative, interrogative, indefinite, and negative pronouns beyond
  reviewed exact cells; the third-person paradigm is exact, not productive;
  velar-stem determiners such as full `всѧкъ`; collective,
  compound, and irregular cardinal numerals;
- automatic participle stem formation from an undifferentiated verb stem and
  active short-participle allomorphs outside reviewed exact cells;
- the supine pending a target-recension normative inventory, productive verbal
  nouns pending lexical suffix metadata, and unregistered irregular verbs;
- automatic productive accent classes, complete breathing/positional-letter
  realization, and abbreviation families beyond the 159 individually typed
  contraction cells; and
- automatic syntax, free agreement/government, dropped copulas, future
  auxiliaries `хотѣти`/`начати`, and the wider periphrastic inventory of Alypy
  §90.

These are coverage gaps, not invitations to guess. Direct calls return
`MissingPrincipalPart`, `UnsupportedFormation`, `UnsupportedCell`,
`HistoricallyInvalidCell`, or an orthographic metadata error as appropriate.
The real-text evaluation now contains 2,291 passage-disjoint morphology cells plus
five analytic phrase cases and 74 separately scored typed contractions. Under
Productive and Exploratory, expanded output is 2,220/2,291 top-1 and 2,291/2,291 top-k;
printed output is 2,135/2,291 top-1 and 2,291/2,291 top-k. Strict preserves the intended
inherited-cell abstention. All 74 contraction cases pass expansion and typed
reverse lookup, with 65/74 top-1, and all 501 masked-cell leakage controls retain
the expected top-k result. These remain regression metrics for registered forms,
not language-wide accuracy.
