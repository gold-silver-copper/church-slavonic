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
| Present | `SYN-VERB-PRESENT-ALYPY-80` | Alypy §§79–82 | independent full 1sg and 3pl plus medial present stem and conjugation |
| Aorist | `SYN-VERB-AORIST-{VOWEL,CONSONANT}-ALYPY-86` | Alypy §86 | independent aorist base and formation; limited final-velar alternation |
| Imperfect | `SYN-VERB-IMPERFECT-{H,YAH,AH}-ALYPY-87` | Alypy §87 | imperfective/biaspectual verb, independent base and formation |
| Imperative | `SYN-VERB-IMPERATIVE-ALYPY-93` | Alypy §93 | independent base and `first-unpalatalized`/`i-series` formation |
| Infinitive | `SYN-VERB-INFINITIVE-LEXICAL` | Alypy §79 | resolved target lemma; no invented infinitive stem |
| l-participle | `SYN-VERB-LPART-ALYPY-97` | Alypy §97 | independent base plus typed gender and number |
| Declined long active participles | `SYN-VERB-PARTICIPLE-{PRESENT,PAST}-ACTIVE-ALYPY-{95,96}` | Alypy §§95–96 | tense/voice-specific full-form stem and adjective class |
| Declined short active participles | `SYN-VERB-PARTICIPLE-{PRESENT,PAST}-ACTIVE-SHORT-ALYPY-{95,96}-98` | Alypy §§95–96 citation edges and §98 complete declension | independent short stem plus `PresentFirstUnpalatalized`, `PresentFirstPalatalized`, `PresentSecond`, `PresentAfterSibilant`, `PastConsonant`, `PastVowel`, or `PastIotated`; source-ordered masculine/neuter citation variants; 63 valid canonical cells; vocative invalid |
| Declined passive participles | `SYN-VERB-PARTICIPLE-{PRESENT,PAST}-PASSIVE-ALYPY-{99,100}` | Alypy §§99–100 | independent short/full stems; past-passive `н`/`нн` distinction is explicit metadata |

Every productive variant carries `alypy-gamanovich-grammar-web-2023` as
normative evidence and the rule ID as its exact citation. The core accepts these
rules only from explicit metadata. The facade adds target lexeme resolution,
exact-table precedence, accent metadata, irregular overrides, and mapping policy.

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

## Explicit specifications and complete paradigms

`NounSpec`, `AdjectiveSpec`, `PronounSpec`, and `VerbSpec` are first-class facade inputs. They
validate Church Slavonic Unicode and closed class/formation enums, preserve
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
complete reviewed present, imperative, and aorist tables of archaic `дати` from
Alypy §103. The determiner registry also has complete productive backgrounds
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
| Compound future | present of `имати` + imperfective infinitive | Alypy §85 |
| Perfect | l-participle + present of `быти` | Alypy §88 |
| Pluperfect | l-participle + either imperfect series of `быти` | Alypy §89 |
| Conditional | l-participle + aorist of `быти` | Alypy §91 |
| Analytic passive | nominative passive participle + selected copula present | Alypy §§101–102 |

The generic `phrases::from_tokens` retains a typed construction and roles for
reviewed periphrases whose auxiliary lexeme is not yet registered. Agreement and
government beyond these construction-specific constraints remain future work.

## Current reviewed lexical surface

The reviewed registry has 875 target lexemes and 877 reviewed senses. The
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

- noun accent paradigms and lexical identities beyond the reviewed registry;
  unknown lexemes still require caller-supplied class/stem metadata, while
  arbitrary irregulars require explicit ordered cell forms;
- automatic velar/sibilant alternation outside the typed noun and aorist seams;
- automatic comparison-stem formation and comparison stems other than explicitly
  supplied typed series; short superlatives are restricted to the exceptional
  predicate nominatives licensed by Alypy §§59, 125, and 128;
- collective, compound, and irregular cardinal numerals beyond the reviewed
  exact cells and productive ordinal background;
- automatic participle stem formation from an undifferentiated verb stem;
  short active participles require an independent stem and typed formation;
- the supine pending a target-recension normative inventory, productive verbal
  nouns pending lexical suffix metadata, and unregistered irregular verbs;
- accent paradigms beyond the four reviewed lexical rules, complete breathing/positional-letter
  realization, and abbreviation families beyond the individually typed
  contraction cells; and
- automatic syntax, free agreement/government, dropped copulas, future
  auxiliaries `хотѣти`/`начати`, and the wider periphrastic inventory of Alypy
  §90.

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
