# synodal-church-slavonic

Offline, dictionary-backed inflection for **Synodal Russian Church Slavonic**.
The facade keeps exact target tables, target normative generation, and reviewed
OCS-derived predictions distinct in every result.

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

Direct lemma-plus-dimension functions include `noun`, `long_adjective`,
`short_adjective`, `present`, `imperfect`, `aorist`, `imperative`, `infinitive`,
`l_participle`, `pronoun`, `determiner`, `numeral`, `participle`, `supine`, and
`verbal_noun`. Conceptually represented but unsupported cells return a typed
error; they never become placeholders or plausible guesses.

`Noun`, `Adjective`, `Verb`, `Pronoun`, `Determiner`, `Numeral`, and `Participle`
resolve one stable identity, accept a caller-configured `Inflector`, expose honest
per-system capabilities and missing metadata, and build paradigms. Paradigms
retain every requested cell and classify it as attested, sourced prediction,
inferred prediction, ambiguous prediction, historically invalid, or unsupported.

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
