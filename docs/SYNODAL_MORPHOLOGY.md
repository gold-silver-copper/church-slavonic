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
- structured compound-future, perfect, pluperfect, future-anterior,
  conditional, optative, passive, periphrastic-tense, copula-ellipsis, and
  composite-adverbial-participle constructions.

Alypy §32's feminine `маріа : марі-` contract is kept distinct from masculine
names such as `исаїа : исаї-`: its instrumental singular is `маріею`, not the
masculine `-емъ`. It is also a separate lexical identity from `марїамъ` /
`марїамь`, whose third-declension or invariant treatment and borrowed oblique
forms require their own policy. The source-table contraction `мр҃і́ѧ` remains an
exact orthographic form of `маріа`; productive fallback uses the validated
feminine paradigm in every cell without a reviewed exact contraction.

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

Direct functions, stable-ID calls, resolved handles, explicit specifications,
provider lexicons, reverse analysis, batches, and paradigms share one productive
generation kernel. The registry, provider, and explicit-spec resolvers apply the
same precedence:

1. validate target and input orthography;
2. use either a stable target identity or validated `NounSpec`, `AdjectiveSpec`,
   or `VerbSpec` metadata;
3. check an exact or explicit cell;
4. check a reviewed or caller-specified irregular override;
5. run a supported Synodal normative rule from typed metadata;
6. when policy permits, evaluate explicit inherited mappings and realize them
   with target rules;
7. retain all compatible analyses, evidence, conflicts, assumptions, and trace;
8. return a nonempty `FormSet` or a typed error.

## Implemented productive rule tables

The engine implements the following target rules. `SynodalLiturgical`
realization requires either an exact accent override or an applicable reviewed
reusable accent paradigm; expanded rules never invent stress.

| System | Stable rule IDs | Normative basis | Implemented input contract |
|---|---|---|---|
| First-declension hard masculine/neuter nouns | `SYN-NOUN-I-HARD-M-ALYPY-34`, `SYN-NOUN-I-HARD-N-ALYPY-34` | Alypy §§34–35 | explicit stem, lexical gender, class, case, number, animacy |
| First-declension velar masculine nouns | `SYN-NOUN-I-HARD-VELAR-M-ALYPY-34` | Alypy §§34–35 | explicit final-velar stem; reviewed г/к/х → з/ц/с and ж/ч/ш seams; no general phonological rewrite |
| First-declension mixed masculine nouns | `SYN-NOUN-I-MIXED-M-ALYPY-33-34` | Alypy §§33–35 | explicit sibilant stem; complete mixed endings and ordered `-и`/`-їе` nominative plural; lexical `-(ь)ми` is not guessed |
| First-declension soft masculine/neuter nouns | `SYN-NOUN-I-SOFT-M-ALYPY-34`, `SYN-NOUN-I-SOFT-N-ALYPY-34` | Alypy §§34–35 | explicit stem, lexical gender, class, case, number, animacy |
| Second-declension hard/soft nouns | `SYN-NOUN-II-HARD-ALYPY-39`, `SYN-NOUN-II-SOFT-ALYPY-39` | Alypy §§39, 44 | explicit stem and class; no automatic sibilant/velar alternation |
| Third-declension feminine nouns | `SYN-NOUN-III-F-ALYPY-41` | Alypy §41 | explicit feminine stem and class |
| Third-declension masculine nouns | `SYN-NOUN-III-M-ALYPY-41` | Alypy §41 | explicit consonantal stem and masculine gender; ordered vocative and genitive-plural variants; optional closed number inventory |
| Fourth-declension neuter `-ен-` nouns | `SYN-NOUN-IV-N-EN-ALYPY-42-43` | Alypy §§42–43 | citation lemma plus independent extended `-ен-` stem; wide-letter dual seam; ordered table variants |
| Fourth-declension neuter `-ес-` nouns | `SYN-NOUN-IV-N-ES-ALYPY-42-43` | Alypy §§42–43 | citation lemma plus independent extended `-ес-` stem; wide-letter dual seam |
| Fourth-declension neuter `-ат-` nouns | `SYN-NOUN-IV-N-AT-ALYPY-42-43` | Alypy §§42–43 | citation lemma plus independent extended `-ат-` stem; final stem `о` widens only in source-defined dual citation cells |
| Fourth-declension feminine `-ер-` nouns | `SYN-NOUN-IV-F-ER-ALYPY-42-43` | Alypy §§42–43 | citation lemma plus independent extended `-ер-` stem; reviewed wide-letter seams and ordered plural variants |
| Fourth-declension feminine `-ов-`/`-в-` nouns | `SYN-NOUN-IV-F-OV-ALYPY-42-44` | Alypy §§42–44 | citation lemma plus independent oblique stem; `свекры` endings, wide-letter seams, and ordered animate accusative variants |
| Fourth-declension masculine `-ен-` nouns | `SYN-NOUN-IV-M-EN-ALYPY-42-44` | Alypy §§42–44 | citation lemma plus independent `-ен-` stem and regular table endings; no lexeme-specific variants |
| Lexeme-specific `камень` contract | `SYN-NOUN-IV-M-EN-KAMEN-ALYPY-43` | Alypy §43 | independently supplied `камен-`; ordered cited singular, dual, and plural alternatives; the separate `каменїе` collective is never emitted |
| Short adjectives, hard/soft | `SYN-ADJ-SHORT-{HARD,SOFT}-ALYPY-53` | Alypy §§53–55 | positive degree only; explicit stem/class/agreement |
| Long adjectives, hard/soft | `SYN-ADJ-LONG-{HARD,SOFT}-ALYPY-57` | Alypy §§56–57 | positive degree only; explicit stem/class/agreement |
| Comparative/superlative full adjectives | `SYN-ADJ-{COMPARATIVE,SUPERLATIVE}-LONG-ALYPY-{58,59}` | Alypy §§58–59 | independently reviewed comparison stem; full agreement inventory |
| Short comparison | `SYN-ADJ-COMPARATIVE-SHORT-ALYPY-58-60` | Alypy §§58 and 60 | independent comparison stem plus `AncientHard`, `AncientSoft`, `LaterYat`, or `LaterAi`; all 72 canonical cells, including the source-defined vocatives, locatives, dual endings, and masculine-plural variant |
| Predicate short superlative | `SYN-ADJ-SUPERLATIVE-SHORT-PREDICATE-ALYPY-59-60-125-128` | Alypy §§59–60, 125, 128 | nine nominative gender/number cells only; directly attested suffix-retaining singular masculine first, followed by the ordinary §60 citation form; oblique and vocative cells are historically invalid |
| Pronouns | `SYN-PRONOUN-{PERSONAL-FIRST,PERSONAL-SECOND,REFLEXIVE,THIRD-PERSON,SEI,SOFT,SOFT-I-ALTERNATING,HARD,MIXED-POSSESSIVE,SHORT-HARD,SHORT-OV-MIXED,SHORT-VELAR,QUANTITY-VELAR,FULL-HARD,FULL-SOFT,FULL-VELAR,KII,KTO,CHTO,DERIVED}-ALYPY-*` | Alypy §§45–48 and §57 | all closed suppletive and regular pronoun profiles; complete licensed case/number/agreement products, number restrictions, ordered variants, typed prefixes, postpositives, clitic selection, and third-person environments |
| Pronoun phrases and contractions | `SYN-PRONOUN-{ENCLITIC-PROSODY,THIRD-PREPOSITION-CONTRACTION,NEGATIVE-PREPOSITION}-ALYPY-*` | Alypy §§47–48 | typed host + enclitic prosody, exact fused `нань`/`вонь`, and structured `ни + preposition + pronoun` interposition |
| Simple numeral words | `SYN-NUMERAL-{CARDINAL,ORDINAL,COLLECTIVE,MULTIPLICATIVE,FRACTIONAL}-*` | Alypy §§61–70; Synodal Bible III Esdras 14:11–12 | 23 typed declensions over cardinal, ordinal, collective, multiplicative, substantival-fractional, and adjectival-fractional words; lexical number and agreement dimensions are enforced |
| Composed cardinals and ordinals | `SYN-NUMERAL-{CARDINAL,ORDINAL}-*-ALYPY-{63,64,68}` | Alypy §§63–64, 68 and appendix | correlated teen, tens, hundreds, ordinary-thousands, named-magnitude, additive, and compound-ordinal analyses; cardinals 1–1,000,000 and ordinals 1–1,000 |
| Numeral government and phrases | `SYN-NUMERAL-{GOVERNMENT,DISTRIBUTIVE,MULTIPLICATIVE,FRACTION}-*` | Alypy §§61, 65–70; locked target texts | typed following/preceding agreement and government; repeated distributives; invariant `кратъ`; cardinal, ordinal, and `полдесѧтый` expressions with inflected `часть` |
| Present | `SYN-VERB-PRESENT-ALYPY-80` | Alypy §§79–82 | independent full 1sg and 3pl plus medial present stem and conjugation |
| Simple future | `SYN-VERB-FUTURE-PERFECTIVE-ALYPY-84` | Alypy §84; Pletneva–Kravetsky lesson 13 | perfective aspect plus the same independently supplied full 1sg, medial present stem, and 3pl; contextual non-perfective readings require exact evidence |
| Aorist | `SYN-VERB-AORIST-{VOWEL,CONSONANT}-ALYPY-86` | Alypy §86 | independent aorist base and formation; limited final-velar alternation |
| Imperfect | `SYN-VERB-IMPERFECT-{H,YAH,AH}-ALYPY-87` | Alypy §87 | imperfective/biaspectual verb, independent base and formation |
| Imperative | `SYN-VERB-IMPERATIVE-ALYPY-93` | Alypy §93 | independent base and `first-unpalatalized`/`i-series` formation |
| Infinitive | `SYN-VERB-INFINITIVE-LEXICAL` | Alypy §79 | resolved target lemma; no invented infinitive stem |
| Supine boundary | `SYN-VERB-SUPINE-ABSENT-ALYPY-143` | Alypy §143.7–8 n.1; Pletneva–Kravetsky lesson 6 §II; Izotov p. 73 | no distinct productive target category; explicit caller provider-exact or irregular compatibility only |
| l-participle | `SYN-VERB-LPART-ALYPY-97` | Alypy §97 | independent base plus typed gender and number |
| Declined long active participles | `SYN-VERB-PARTICIPLE-{PRESENT,PAST}-ACTIVE-ALYPY-{95,96}` | Alypy §§95–96 | tense/voice-specific full-form stem and adjective class |
| Declined short active participles | `SYN-VERB-PARTICIPLE-{PRESENT,PAST}-ACTIVE-SHORT-ALYPY-{95,96}-98` | Alypy §§95–96 citation edges and §98 complete declension | independent short stem plus `PresentFirstUnpalatalized`, `PresentFirstPalatalized`, `PresentSecond`, `PresentAfterSibilant`, `PastConsonant`, `PastVowel`, or `PastIotated`; source-ordered masculine/neuter citation variants; 63 valid canonical cells; vocative invalid |
| Declined passive participles | `SYN-VERB-PARTICIPLE-{PRESENT,PAST}-PASSIVE-ALYPY-{99,100}` | Alypy §§99–100 | independent short/full stems; past-passive `н`/`нн` distinction is explicit metadata |

Every productive variant carries `alypy-gamanovich-grammar-web-2023` as
normative evidence and the rule ID as its exact citation. The core accepts these
rules only from explicit metadata. The facade adds target lexeme resolution,
exact-table precedence, accent metadata, irregular overrides, and mapping policy.

## Complete finite-past classification audit

The generic `FiniteTense::Past` is a source-adapter compatibility tag, not a
target grammatical tense. Alypy §§86–87 and Pletneva–Kravetsky lessons 5–6
define separate aorist and imperfect systems, so the bundled registry contains
no `past:*` cells and advertises no finite-past capability. Caller-supplied exact
specifications may retain the tag when their external source genuinely lacks
the distinction; productive target generation rejects it and directs callers to
aorist or imperfect.

`data/synodal/past_classification_reviews.tsv` exhausts all 73 historical v0.6
admissions. Sixty-four rows reclassify directly as aorist, four `-ше` rows as
imperfect, and `глаголахъ` splits contextually: Acts 25:20 is imperfect, while
Daniel 10:16 and an independent Deuteronomy 1:29 witness are aorist. Four old
admissions are invalid: two `живи` analyses are predicates or imperatives,
`дѣла` is nominal, and the second-singular `прїѧтъ` contradicts both cited
third-singular contexts and the lesson 5 rule restricting exceptional `-тъ` to
third singular. The resulting historical set contributes 65 aorist and five
imperfect exact rows. Extraction fails if the ledger is incomplete or changed,
if a replacement disappears, or if any `past:*` target/evaluation row returns.

## Complete simple-future contract

Alypy §84 and Pletneva–Kravetsky lesson 13 independently establish that the
simple future has no separate endings: it uses the complete present-shaped
person × singular/dual/plural paradigm. `future`, `Verb::future`, and
`VerbSpec::finite_paradigm(FiniteTense::Future)` therefore reuse the same three
independent lexical inputs as the present system—the complete first singular,
the medial stem, and the complete third plural—but classify their output under
`SYN-VERB-FUTURE-PERFECTIVE-ALYPY-84` only for a lexeme explicitly typed as
perfective.

The same sources document contextual future readings of formally present
imperfective forms. Those cannot be predicted from aspect or shape alone:
without a reviewed exact override they return `EvidenceIncompleteCell`. Unknown
aspect returns `MissingMetadata`; missing present-shaped principal parts retain
their specific `MissingPrincipalPart` failures. Exact future cells always win
before productive generation. The registry includes complete exact nine-cell
suppletive futures for `быти` and archaic `дати`; `дати` is no longer
misclassified as a present table. Other irregular or contextual cells retain
their own source evidence and do not license unattested siblings.

## Complete pronoun contract

The Synodal pronoun kernel follows Alypy §§45–48 rather than treating the
closed-class exact registry as the grammar. It has independent profiles for the
suppletive personal, reflexive, third-person, `сей/сій`, two-base `кій`, `кто`,
and `что` paradigms, plus explicit regular soft, hard, mixed possessive, short,
velar, quantity, and full-form types. The `чій` profile makes the source-defined
`і/ї` vowel-edge spelling explicit. Compound `-ов-` pronouns retain the
noun-like genitive and dative alternatives stated in §48 before their ordinary
pronominal forms.

The lemma `онъ` has one stable lexical identity with two disjoint grammatical
profiles: `person=third` selects the suppletive anaphoric series, while
`person=none` selects the regular short demonstrative paradigm. This preserves
the §46 relationship without making ordinary lemma lookup ambiguous.

Every agreeing profile attempts all case × number × gender × animacy cells and
rejects vocatives; the person-indexed and nonagreeing profiles enforce their own
dimensions. `PronounNumberInventory`, `PronounFormSelection`, and
`PronounEnvironment` keep lexical number defectiveness, table-primary versus
enclitic forms, and the post-prepositional third-person `н-` series out of
untyped booleans. Productive `нѣ-/ни-` prefixes and invariant `-же/-ждо`
postpositives compose with the inflected base while preserving ordered variants
and rule evidence.

Phrase-valued behavior remains outside the single-word API.
`pronoun_enclitic_after_host` realizes the §47 final-vowel host accent and
unaccented clitic or retains the logically stressed short pronoun.
`contracted_third_person_accusative` returns exact fused `нань` and `вонь`
tokens. `negative_pronoun_prepositional` returns the §48 three-token
`ни + preposition + inflected base` construction. Each component keeps its own
evidence and receives a trace step for the construction rule.

Expanded output never invents stress. Liturgical output uses exact reviewed
cells or a caller/registry `AccentParadigm`; `AccentScope::PronounCases` and
`AccentScope::PronounAgreement` can express case-, number-, gender-, and
animacy-conditioned stress. A missing accent contract is an explicit
`OrthographicMetadataRequired` result, not a guessed form.

## Complete numeral contract

`NumeralDeclension` has 23 source-typed word profiles. The closed lower
cardinals and their ordered variants are independent of noun-like five through
ten, hundred, and the large magnitude nouns. Ordinals, collectives,
multiplicatives, fractional nouns, and the target-attested fractional adjective
`полдесѧтый` retain their own grammatical kind, lexical number inventory, and
agreement dimensions. `NumeralSpec` exposes the same kernel for unregistered
caller lexemes. Exact registry cells win first; every other licensed cell is a
normative prediction and never an implied attestation.

`numeral_phrases` keeps component alternatives correlated. It covers every
cardinal from 1 through 1,000,000 and every ordinal from 1 through 1,000,
including Alypy's alternative teen and decade inflection, fused/spaced
hundreds, ordinary complete multipliers of `тысѧща`, distributed thousand
heads found in the Synodal Bible, and the named `тьма`, `легеѡнъ`, and `леѡдръ`
analyses. Additive `и`, all-component `и`, and asyndetic orders are separate
analyses rather than cross-token variant mixing.

`CompoundNumeralCell` makes cardinal case, required agreement gender, and
animacy explicit. `RealizedCardinal::government` distinguishes a following
from a preceding counted noun and returns agreement, genitive-plural
government, and the marked contextual nominative-plural alternative with
source evidence. Repeated distributives, cardinal + invariant `кратъ`, and
cardinal/ordinal/fractional + inflected `часть` are structured phrase values.
The exact target witnesses include `два два`, ordinary and distributed
thousands, asyndetic `двадесѧть два`, compound ordinals, modern neuter `два`,
and `полдесѧтыѧ части`.

Expanded output is total across the licensed word and composition inventories.
The liturgical profile uses exact accents where available and otherwise returns
`OrthographicMetadataRequired`; composition never guesses stress by joining
accentless components.

## Explicit specifications and complete paradigms

`NounSpec`, `AdjectiveSpec`, `DeterminerSpec`, `NumeralSpec`, `PronounSpec`, and
`VerbSpec` are first-class facade inputs. They validate Church Slavonic Unicode
and closed class/formation enums, preserve
independent principal parts, attach caller provenance, and never label their
outputs attestations. `Inflector::form_spec` delegates to the same pure kernel as
registered words. Specialized noun and adjective paradigms retain the canonical
inventory of attempted cells and structured failures. `VerbSystem` provides one
selector for every represented finite, imperative, infinitive, l-participle,
participial, supine, and verbal-noun inventory; both `Verb` and `VerbSpec` expose
`system_paradigm` and stable-order `all_system_paradigms`.

`PresentPrincipalParts` and `VerbSpecBuilder::present_series` set the medial
present stem, complete first-person singular, and complete third-person plural
atomically. Neither edge is derived from the medial stem. Per-system
`missing_principal_parts` diagnostics report the exact closed `MetadataField`
values absent from the productive background; exact registered cells may still
override individual rows.

`ParadigmStatus` distinguishes attested, irregular, sourced, caller-specified,
inherited, and ambiguous successes from historical invalidity, incomplete
evidence, missing morphological metadata, missing orthographic metadata, and
unsupported behavior. The underlying typed `Error` remains available on every
row. `Error::code`, `ParadigmRow::error_code`, `Paradigm::successes`, and
`Paradigm::with_status` provide stable machine-readable inspection without
parsing English diagnostics.

### Historically merged supine boundary

The Russian/Synodal recension has no distinct productive supine. Alypy §143.7–8
uses the infinitive after motion and related verbs to express purpose; its note 1
assigns the supine construction to the ancient language. Pletneva–Kravetsky
lesson 6 §II independently teaches the same target construction with `-ти` and
`-щи` infinitives in Synodal passages. Izotov p. 73 gives the diachronic
boundary explicitly: Russian history merged the supine and infinitive, and
Church Slavonic grammars normally do not distinguish a supine.

### Source-bounded verbal nouns

Alypy §27 defines action/state nouns in `-їе` from the complete base of a
past passive participle, including `осꙋжденїе`, `ѹченїе`, `моленїе`,
`распѧтїе`, and `житїе`. Alypy §34 supplies the complete soft-neuter
`-їе` declension. `VerbalNounPrincipalPart::past_passive_ie` encodes an
independent platform; when a verb already has a reviewed short past-passive
principal part, the same typed platform is reused automatically. Every case,
number, and animacy request then delegates to the noun inflector and retains a
separate verbal-noun formation step in its evidence and trace.

Section 27 also lists `-ота/-ета`, `-ба`, `-ежъ`, `-нь/-снь/-знь`, `-тва`,
`-ть`, and `-изна` families, but gives no rule that selects one of them for an arbitrary
verb. `VerbalNounPrincipalPart::explicit_lexical` therefore requires the
resulting noun's complete lemma, stem, declension, gender, number inventory,
and—when liturgical output is requested—accent. This closes their inflection
without claiming unrestricted derivational morphology. The locked target data
independently crosscheck the productive analysis with `моленїе`.

`GrammarCell::Supine` and `VerbSystem::Supine` remain in the public type system
so source adapters can preserve an explicitly labeled external category. A
productive target request returns `HistoricallyInvalidCell`, and the bundled
target registry and held-out evaluation are guarded against every `supine`
cell. Missing-metadata diagnostics do not request a `SupineStem`, because no
principal part can enable an absent target category. An application-owned
provider may still supply an exact cell, and an explicit specification may
supply the same cell as a caller irregular override. Exact-first resolution
returns that caller prediction without making it Synodal evidence. The OCS
`-ти/-щи` → `-тъ/-щь` rule is never imported into this target.

## Injectable lexical providers and batches

`LexemeProvider` exposes deterministic snapshots of `ProviderLexeme` values;
it has no filesystem, network, serialization-format, or database methods.
`StaticLexemeProvider` adapts the generated registry to this contract, while
`InMemoryLexemeProvider` is application-owned. `Lexicon` sorts composed entries
by stable ID. Duplicate IDs return `ProviderConflict`; distinct homographs
remain `AmbiguousLexeme` rather than being silently shadowed.

A supplied entry contains a stable target identity, part of speech, source ID,
typed `LexemeSpec`, and optional ordered exact cells. Resolution is exact
provider cell, caller irregular cell, then the shared productive kernel.
`Lexicon::batch` retains input order and one `Result<FormSet>` per request;
filters expose successes, failures, and individual `ErrorCode` values. Provider
noun and `VerbSystem` paradigms likewise retain all failed cells.

## Reusable accent paradigms

An `AccentParadigm` contains one or more typed, cell-scoped accent rules plus
independently positioned breathing rules and source evidence. Placement retains
the linguistic distinction between a fixed stem vowel counted from the left and
an ending vowel counted from the right. Rules can be scoped by number and
morphological system, so mobility and acute/grave/kamora choices do not require
precomputed strings for every cell.

Resolution order is exact reviewed accented cell, lexical irregular printed
override, applicable reusable paradigm, then
`OrthographicMetadataRequired { field: AccentParadigm }`. The first reviewed
runtime paradigm, `synodal-accent:mudr-fixed-stem`, applies acute stress to the
first stem vowel throughout the long positive singular of `мꙋдръ` (Alypy §57).
The v0.9 registry adds the complete §43 `synodal-accent:mati-fixed-stem`,
`synodal-accent:imya-mobile`, and `synodal-accent:nebo-mobile` paradigms. The two
mobile paradigms use disjoint number-and-case scopes, acute/grave selection, and
stem-versus-ending placement; `имѧ` also positions psili independently before
the stress mark. Missing or overlapping rules are typed failures. Exact printed
cells continue to win before all reusable paradigms.

### Accusative variation

Alypy §35 states that animate singular noun accusatives are usually
genitive-like but can be nominative-like, while animate plural accusatives are
usually nominative-like but can be genitive-like. The engine therefore returns
both for supported nouns, in that normative order. The adjective tables in
§§53–57 preserve the analogous parenthesized alternatives. Animacy is not
treated as a modern-Russian equation that erases attested Church Slavonic
variation.

### Exact and irregular systems

The frozen v0.7 target registry retains its reviewable exact rows: normative
tables and variants, plus passage-identified target attestations. It includes
the complete nine-cell present, simple
future, aorist, and imperfect paradigms of `быти`, its sourced imperative and
representative active participles; the
complete nine-cell future auxiliary present of `имати`; a full demonstrative
paradigm for `той`; reviewed exact cells for `сей`, `иже`, `кто`, `что`, and
`нѣкто`; the reviewed cardinal paradigms of `два`, `три`, and `четыре`; one
exceptional determiner cell; representative active/passive participle cells for
`нести`; the first-, second-, and complete gendered third-person
personal-pronoun paradigms; and the
complete reviewed simple-future, imperative, and aorist tables of archaic
`дати` from Alypy §103. The determiner registry also has complete productive backgrounds
for `самъ`/`самый`, mixed dual-less `весь`, short/full dual-less
`всѧкъ`/`всѧкїй`, and full `всѧческїй`; its older exact rows remain
higher-precedence spelling, accent, and attestation evidence.

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

The productive noun registry now gives `мꙋжъ`, `имѧ`, `небо`, `мати`, and
plural-only `людїе` reviewed classes and independent stems in addition to their
exact cells. The `людїе` restriction is a separately evidenced registry row,
not an inference from its spelling. Exact normative or attested rows keep
precedence, while an uncovered cell can use only its licensed productive
background. No suffix inference or frequency-based bulk conversion is involved.

The v0.10 registry adds complete productive backgrounds for `ѻтроча`,
`свекры`, and `камень`. The source's collective `каменїе` remains a separate
lexical item. The ordinary paradigm's cited `-їѧ`/`-ема` alternatives are
licensed only by the closed `камень` contract and are not generalized.
The productive upgrade reuses the existing reviewed `камꙑ`/`камень` stable
identity and its exact target attestations rather than creating a second
semantic identity.
The §§35–44 completion adds typed u-stems, `-инъ` ethnonyms, `-тель` agents,
`-й`/`-ей` and `-їе` profiles, `-ище` locative variants, second-declension
velar/mixed/postvocalic families, paired `ѻко`/`ꙋхо`, `дщерь`, `день`,
`ꙋдъ : ꙋдес-`, invariant Hebrew loans, and complete cell-scoped
`церковь`/`любовь` alternation. The public specification API additionally
accepts multiple ordered irregular forms per cell and typed defective cells.

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
| Compound future | imperfective infinitive + finite `быти`, `имати`, `хотѣти`, or `начати`; both orders | Alypy §85; Pletneva–Kravetsky lesson 13 table 36 |
| Perfect | l-participle + present `быти`, third-singular copula omission, or two predicates sharing one copula | Alypy §88 |
| Pluperfect | l-participle + `бѣ` aorist series, `бѧ` imperfect series, or l-participle of `быти` + present copula | Alypy §§89, 168 |
| Future anterior | required `аще` + l-participle + future `быти` | Alypy §162 |
| Conditional | personal aorist or invariant `бы`, both present-copula orders, infinitive + `бы`, and the three fixed modal imperfects | Alypy §91 |
| Optative | `да` + every person/number of present or simple future | Alypy §92 |
| Analytic passive | all 17 participle/copula combinations, both binary orders or four nested compound orders, plus instrumental or `ѿ` + genitive agent | Alypy §§101–102 |
| Periphrastic tense | short nominative present-active participle + five `быти` systems or the closed eight-member semi-auxiliary inventory | Alypy §§90, 163; Petrukhin 2016 |
| Copula ellipsis | seven explicitly licensed zero-copula contexts, with no invented zero token | Alypy §§123–124 |
| Composite adverbial participle | past passive + past active `быти`, or nominal predicate + present/past active `быти` | Alypy §146 |
| Compound numeral | correlated inflected numeral components and optional `и` | Alypy §§63–64, 68 |
| Repeated distributive | repeated inflected cardinal | Alypy §61; Mark 6:7 |
| Multiplicative | inflected cardinal + invariant `кратъ` | Alypy §70 |
| Fractional part | cardinal/ordinal/fractional adjective + inflected `часть` | Alypy §70; III Esdras 14:11–12 |

`PhraseFormation` exposes the closed analytic subtype, while `PhraseRole`
retains the role of every independently evidenced token. Builders reject wrong
aspect, tense, voice, short/long form, case, comparison, agreement, role shape,
agent government, or source-unlicensed omission/order. Every component receives
the construction evidence and rule trace in addition to its word-level
provenance. `phrases::from_tokens` remains a deliberately generic
interoperability escape hatch; it is not needed to realize any documented
analytic morphology above. Unrestricted clause generation remains outside the
inflection engine.

## Current reviewed lexical surface

The reviewed registry has 929 target lexemes and 932 reviewed senses. The
productive layer now covers every source-reviewed noun, adjective, pronoun,
determiner, and numeral class while retaining exact-only identities whose class
or principal parts remain underdetermined. Inflectable additions use
`LexicalForm` unless a class or independent
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

- noun accent paradigms and lexical identities beyond the reviewed registry;
  unknown lexemes still require caller-supplied class/stem metadata, while
  arbitrary irregulars require explicit ordered cell forms;
- automatic velar/sibilant alternation outside the typed noun and aorist seams;
- automatic comparison-stem formation and comparison stems other than explicitly
  supplied typed series; short superlatives are restricted to the exceptional
  predicate nominatives licensed by Alypy §§59, 125, and 128;
- automatic participle stem formation from an undifferentiated verb stem;
  short active participles require an independent stem and typed formation;
- lexical verbal-noun suffix selection without a complete derived noun and
  unregistered irregular verbs; the separately represented supine is closed as historically
  merged with the infinitive and fails with `HistoricallyInvalidCell` unless a
  caller supplies an explicit compatibility cell;
- accent paradigms beyond the four reviewed lexical rules, complete breathing/positional-letter
  realization, and abbreviation families beyond the individually typed
  contraction cells; and
- unrestricted clause syntax and semantic control beyond the closed analytic
  formations above. Agreement, passive-agent government, documented copula
  omission, all four future auxiliaries, and Alypy §§90/163 periphrases are
  represented by typed builders rather than delegated to a sentence generator.

These are coverage gaps, not invitations to guess. Direct calls return
`MissingPrincipalPart`, `UnsupportedFormation`, `UnsupportedCell`,
`HistoricallyInvalidCell`, or an orthographic metadata error as appropriate.
Corpus and passage-disjoint evaluation reports remain downstream regression
signals for registered behavior. The v0.10 capability source of truth is
`data/synodal/engine_capabilities.tsv`, rendered deterministically in
`docs/SYNODAL_V10_PRODUCTIVE_MORPHOLOGY_AND_LEXICON_AUDIT.md`; the small
`data/synodal/linguistic_evaluation.tsv` fixture is evaluated by behavioral
contract, and corpus percentages are not used to select or justify morphology
work.
