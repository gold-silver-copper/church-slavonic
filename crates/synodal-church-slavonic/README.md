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

Short comparison uses a typed, independently supplied formation:

```rust
use synodal_church_slavonic::{
    AdjectiveCell, AdjectiveClass, AdjectiveForm, AdjectiveSpec, Animacy, Case,
    Comparison, ComparisonFormation, Gender, Number, SpecificationSource,
};

let source = SpecificationSource::new("alypy-58", "alypy", "Alypy §§58, 98")?;
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
.present_stem("нес")?
.present_first_singular("несꙋ")?
.present_third_plural("несꙋтъ")?
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
    Aspect, FiniteTense, ParadigmStatus, SpecificationSource, VerbConjugation, VerbSpec,
};

let source = SpecificationSource::new("local-verb", "local-lexicon", "entry 2")?;
let verb = VerbSpec::builder(
    "нести",
    Aspect::Imperfective,
    VerbConjugation::FirstUnpalatalized,
    source,
)?
.build()?;
let paradigm = verb.finite_paradigm(FiniteTense::Aorist);
assert_eq!(paradigm.iter().count(), 9);
assert!(paradigm
    .iter()
    .all(|row| row.status() == ParadigmStatus::MissingMetadata));
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
`verbal_noun`. Conceptually represented but unsupported cells return a typed
error; they never become placeholders or plausible guesses.

`Noun`, `Adjective`, `Verb`, `Pronoun`, `Determiner`, `Numeral`, and `Participle`
resolve one stable identity, accept a caller-configured `Inflector`, expose honest
per-system capabilities and missing metadata, and build paradigms. Paradigms
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

The `phrases` module realizes a typed compound future, perfect, pluperfect,
conditional, and analytic passive. `abbreviation` contracts only a resolved
lexeme plus semantic sense and returns all compatible expansions. Static
registries are generated offline; runtime code performs no file or network I/O.

The current registry is intentionally a reviewed seed, not a complete Church
Slavonic lexicon. Coverage and limitations are reported in
`reports/synodal-evaluation.md` and `docs/SYNODAL_MORPHOLOGY.md`.
