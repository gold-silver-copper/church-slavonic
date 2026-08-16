# synodal-church-slavonic

Offline, typed inflection for **Synodal Russian Church Slavonic**. Callers may
use reviewed dictionary identities or supply explicit lexical metadata. The
facade keeps attestations, irregular overrides, normative generation,
caller-specified predictions, and reviewed OCS-derived predictions distinct.

```rust
use synodal_church_slavonic::{
    noun, Animacy, Case, GenerationPolicy, Inflector, Number, OrthographyProfile,
    Person, Verb,
};

let dative = noun("рабъ", Case::Dative, Number::Plural, Animacy::Animate)?;
assert_eq!(dative.primary_text(), "рабѡмъ");

let inflector = Inflector::builder()
    .generation_policy(GenerationPolicy::Productive)
    .orthography(OrthographyProfile::SynodalLiturgical)
    .build();
let verb = Verb::resolve_with("нести", inflector)?;
assert_eq!(verb.present(Person::First, Number::Singular)?.primary_text(), "несꙋ̀");
# Ok::<(), synodal_church_slavonic::Error>(())
```

## Explicit typed specifications

An unregistered noun needs no fabricated dictionary identity:

```rust
use synodal_church_slavonic::{
    Animacy, Case, Gender, NounCell, NounDeclension, NounSpec, Number,
    SpecificationSource,
};

let source = SpecificationSource::new("local-1", "local-lexicon", "entry 1")?;
let noun = NounSpec::new(
    "псалтирникъ",
    "псалтирник",
    Gender::Masculine,
    NounDeclension::FirstHardMasculine,
    source,
)?;
let form = noun.form(NounCell {
    case: Case::Genitive,
    number: Number::Singular,
    animacy: Animacy::Animate,
})?;
assert_eq!(form.primary_text(), "псалтирника");
# Ok::<(), synodal_church_slavonic::Error>(())
```

The productive noun inventory has thirty-seven reviewed contracts, covering the
regular tables plus Alypy's typed stem-changing, invariant, and lexeme-bounded
families through §44. These include feminine and masculine `-іа` names,
`господь`, `день`, `любовь`, `церковь`,
`ѻко`/`ꙋхо`, `ꙋдъ : ꙋдес-`, postvocalic ancient plurals, and `-инъ`
ethnonyms without allowing arbitrary stem cross-products.
`NounSpec::with_number_inventory` represents plural-only or otherwise defective
number inventories; absent numbers remain `HistoricallyInvalidCell` rows in a
complete paradigm.

Pronouns use the same explicit-metadata route. `PronounSpec` selects one of the
closed suppletive or regular Alypy §§45–48 profiles and can carry a lexical
number inventory, clitic selection, post-prepositional environment, productive
`нѣ-/ни-` prefix, invariant postpositive, and reusable accent paradigm. The
`phrases` module keeps enclitic prosody, negative-pronoun preposition
interposition, and fused `нань`/`вонь` contractions as structured grammatical
results rather than fake space-containing words.

Determiners have their own explicit-metadata route rather than masquerading as
ordinary adjectives. `DeterminerSpec` selects the complete source-owned
`самъ`/`самый`, mixed `весь`, mixed `всѧкъ`/`всѧкїй`, or full
`всѧческїй` class. The facade retains exact spellings first, rejects the
source-excluded dual of `весь` and `всѧкъ`, and exposes a separate
`productive_determiner` capability.

## Injectable application lexicons

The generated registry and application-owned entries compose through one
fail-closed provider contract:

```rust
use synodal_church_slavonic::{
    Animacy, BatchRequest, Case, Gender, GrammarCell, InMemoryLexemeProvider,
    Inflector, LexemeSpec, Lexicon, NounCell, NounDeclension, NounSpec, Number,
    ProviderLexeme, SpecificationSource, StaticLexemeProvider,
};

let source = SpecificationSource::new("app-1", "my-reviewed-lexicon", "entry 1")?;
let spec = NounSpec::new(
    "псалтирникъ",
    "псалтирник",
    Gender::Masculine,
    NounDeclension::FirstHardMasculine,
    source,
)?;
let provider = InMemoryLexemeProvider::new([ProviderLexeme::new(
    "app:noun:psaltirnik",
    "my-reviewed-lexicon",
    LexemeSpec::from(spec),
)?])?;
let lexicon = Lexicon::compose(
    Inflector::default(),
    &[&StaticLexemeProvider, &provider],
)?;
let cell = GrammarCell::Noun(NounCell {
    case: Case::Genitive,
    number: Number::Singular,
    animacy: Animacy::Animate,
});
assert_eq!(lexicon.form("псалтирникъ", cell)?.primary_text(), "псалтирника");

let batch = lexicon.batch([
    BatchRequest::lemma("псалтирникъ", cell),
    BatchRequest::lemma("неизвѣстенъ", cell),
]);
assert_eq!(batch.successes().count(), 1);
assert_eq!(batch.failures().count(), 1);
# Ok::<(), synodal_church_slavonic::Error>(())
```

Provider exact cells precede the entry's caller-specified irregular cells,
which precede its productive background. Ordered variants and failures are not
collapsed. Duplicate stable IDs return `ErrorCode::ProviderConflict`; distinct
homographs remain an explicit `AmbiguousLexeme`.

Short comparison uses a typed, independently supplied formation:

```rust
use synodal_church_slavonic::{
    AdjectiveCell, AdjectiveClass, AdjectiveForm, AdjectiveSpec, Animacy, Case,
    Comparison, ComparisonFormation, Gender, Number, SpecificationSource,
};

let source = SpecificationSource::new("alypy-58", "alypy", "Alypy §§58, 60")?;
let adjective = AdjectiveSpec::new("мꙋдръ", "мꙋдр", AdjectiveClass::Hard, source)?
    .comparison("мꙋдрѣйш", ComparisonFormation::LaterYat)?;
let form = adjective.form(AdjectiveCell {
    case: Case::Nominative,
    number: Number::Singular,
    gender: Gender::Masculine,
    animacy: Animacy::Inanimate,
    form: AdjectiveForm::Short,
    comparison: Comparison::Comparative,
})?;
assert_eq!(form.primary_text(), "мꙋдрѣй");
# Ok::<(), synodal_church_slavonic::Error>(())
```

The same independent comparison metadata realizes the exceptional short
superlative only in predicate-compatible nominative agreement cells. Alypy §59
directly attests the suffix-retaining masculine pattern `и҆́стиннѣйшъ`; oblique
and vocative short-superlative requests fail as historically invalid.

A verb keeps its present edges and non-present stems independent:

```rust
use synodal_church_slavonic::{
    AoristFormation, Aspect, FiniteTense, FiniteVerbCell, GrammarCell, Number,
    Person, SpecificationSource, VerbConjugation, VerbSpec,
};

let source = SpecificationSource::new("alypy-nesti", "alypy", "Alypy §§80, 86")?;
let verb = VerbSpec::builder(
    "нести",
    Aspect::Imperfective,
    VerbConjugation::FirstUnpalatalized,
    source,
)?
.present_series("нес", "несꙋ", "несꙋтъ")?
.aorist("нес", AoristFormation::ConsonantStem)?
.build()?;
let form = verb.form(GrammarCell::FiniteVerb(FiniteVerbCell {
    tense: FiniteTense::Aorist,
    person: Person::First,
    number: Number::Singular,
}))?;
assert_eq!(form.primary_text(), "несохъ");
# Ok::<(), synodal_church_slavonic::Error>(())
```

Specialized paradigms retain failures instead of dropping cells:

```rust
use synodal_church_slavonic::{
    Aspect, ErrorCode, FiniteTense, ParadigmStatus, SpecificationSource,
    VerbConjugation, VerbSpec, VerbSystem,
};

let source = SpecificationSource::new("local-verb", "local-lexicon", "entry 2")?;
let verb = VerbSpec::builder(
    "нести",
    Aspect::Imperfective,
    VerbConjugation::FirstUnpalatalized,
    source,
)?
.build()?;
let paradigm = verb.system_paradigm(VerbSystem::Finite(FiniteTense::Aorist));
assert_eq!(paradigm.iter().count(), 9);
assert!(paradigm
    .iter()
    .all(|row| row.status() == ParadigmStatus::MissingMetadata));
assert_eq!(
    paradigm.iter().next().and_then(|row| row.error_code()),
    Some(ErrorCode::MissingMetadata),
);
# Ok::<(), synodal_church_slavonic::Error>(())
```

A reusable accent paradigm can realize several liturgical cells:

```rust
use synodal_church_slavonic::{
    AccentMark, AccentScope, AdjectiveCell, AdjectiveClass, AdjectiveForm,
    AdjectiveSpec, Animacy, Case, Comparison, Gender, Inflector, Number,
    OrthographyProfile, SpecificationSource,
};

let source = SpecificationSource::new("alypy-57", "alypy", "Alypy §57")?;
let accent = source.fixed_stem_accent(
    "mudr-fixed-stem",
    AccentScope::Adjective {
        form: AdjectiveForm::Long,
        comparison: Comparison::Positive,
        numbers: vec![Number::Singular],
    },
    0,
    AccentMark::Acute,
);
let adjective = AdjectiveSpec::new(
    "мꙋдръ",
    "мꙋдр",
    AdjectiveClass::Hard,
    source,
)?
.with_accent_paradigm(accent)?;
let inflector = Inflector::builder()
    .orthography(OrthographyProfile::SynodalLiturgical)
    .build();
let form = adjective.form_with(
    inflector,
    AdjectiveCell {
        case: Case::Genitive,
        number: Number::Singular,
        gender: Gender::Masculine,
        animacy: Animacy::Inanimate,
        form: AdjectiveForm::Long,
        comparison: Comparison::Positive,
    },
)?;
assert_eq!(form.primary_text(), "мꙋ́драгѡ");
# Ok::<(), synodal_church_slavonic::Error>(())
```

Direct lemma-plus-dimension functions include `noun`, `long_adjective`,
`short_adjective`, `present`, `imperfect`, `aorist`, `imperative`, `infinitive`,
`l_participle`, `pronoun`, `determiner`, `numeral`, `participle`, `supine`, and
`verbal_noun`. The distinct target supine is historically invalid (with
explicit caller provider-exact and irregular interoperability seams);
verbal nouns use Alypy §27's past-passive-platform `-їе` rule or a complete
caller-supplied lexical noun. Neither category becomes a placeholder or
plausible guess.

`Noun`, `Adjective`, `Verb`, `Pronoun`, `Determiner`, `Numeral`, and `Participle`
resolve one stable identity, accept a caller-configured `Inflector`, expose honest
per-system capabilities and missing metadata, and build paradigms. `VerbSystem`
covers every represented finite, imperative, infinitive, l-participle,
participial, supine, and verbal-noun inventory through `system_paradigm`; the
stable `ErrorCode` and per-system principal-part diagnostics are available
without parsing display strings. Paradigms
retain every requested cell and classify it as attested, sourced prediction,
irregular override, caller-specified prediction, inferred/ambiguous prediction,
historically invalid, evidence-incomplete, missing metadata, missing orthographic
metadata, or unsupported.

Generation policies are:

- `Strict`: target exact tables, reviewed overrides, and target normative rules
  supplied with independently sourced target metadata;
- `Productive`: additionally reviewed or calibrated OCS-to-Synodal mappings at a
  configurable confidence threshold; and
- `Exploratory`: every non-rejected compatible mapping, retained as separate
  variants rather than collapsed.

Inherited variants expose their OCS source lexeme, mapping ID, mapping evidence,
target rule, and each transformation in the rule trace. No OCS surface form is
copied into the target exact-form table.

The `phrases` module realizes the closed typed inventory of compound futures,
perfects, pluperfects, future anterior, conditionals, optatives, analytic
passives, periphrastic tenses, copula ellipses, and composite adverbial
participles. `abbreviation` contracts only a resolved
lexeme plus semantic sense and returns all compatible expansions. Static
registries are generated offline; runtime code performs no file or network I/O.

The current registry is intentionally a reviewed seed, not a complete Church
Slavonic lexicon. Coverage and limitations are reported in
`reports/synodal-evaluation.md` and `docs/SYNODAL_MORPHOLOGY.md`.
