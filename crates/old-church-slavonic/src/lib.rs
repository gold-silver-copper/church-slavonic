//! Dictionary-backed Old Church Slavonic inflection with a direct typed API.
//!
//! Give the facade a lemma and the grammatical dimensions of one cell. A
//! successful result remains structured because the source can contain ordered
//! variants and competing analyses.
//!
//! ```
//! use old_church_slavonic::{noun, Case, Number};
//!
//! let forms = noun("обѣдъ", Case::Dative, Number::Dual)?;
//! assert_eq!(forms.primary_text(), "обѣдома");
//! # Ok::<(), old_church_slavonic::InflectionError>(())
//! ```
//!
//! Resolve a lexical identity once for repeated calls:
//!
//! ```
//! use old_church_slavonic::{Number, Person, Verb};
//!
//! let verb = Verb::new("благословити")?;
//! assert_eq!(
//!     verb.present(Person::First, Number::Singular)?.primary_text(),
//!     "благословлѭ",
//! );
//! # Ok::<(), old_church_slavonic::InflectionError>(())
//! ```
//!
//! Exact table cells take precedence over dictionary principal-part rules and
//! reviewed overrides. Ambiguity, missing metadata, unsupported formations, and
//! historically invalid cells remain distinct [`InflectionError`] values. See
//! [`advanced`] for by-ID, explicit-rule, metadata, and raw-feature operations.

#![forbid(unsafe_code)]

pub mod advanced;
mod dictionary;
mod handles;
mod lookup;
mod metadata;
mod paradigm;
mod resolver;

pub use handles::{Adjective, Noun, Participle, Verb};
pub use lookup::lookup;
pub use old_church_slavonic_core::{
    AdjectiveForm, Animacy, Case, FiniteTense, FormSet, FormSource, FormVariant, Gender,
    InflectionError, InflectionWarning, LexemeSummary, Number, PartOfSpeech, ParticipleKind,
    Person,
};
pub use paradigm::{
    AdjectiveParadigm, CellOutcome, FiniteVerbParadigm, ImperativeParadigm, LParticipleParadigm,
    NounParadigm, ParticipleParadigm, VerbParadigm,
};

/// Rule traces and source-evidence diagnostics.
pub mod trace {
    pub use old_church_slavonic_core::{
        FormAnalysis, MetadataEvidence, MetadataField, MetadataProvenance, RuleId, RuleStep,
    };
}

/// The structured result returned by ordinary inflection calls.
pub type InflectionResult = Result<FormSet, InflectionError>;

/// Decline one dictionary noun cell.
///
/// ```
/// use old_church_slavonic::{noun, Case, Number};
/// assert_eq!(noun("обѣдъ", Case::Dative, Number::Dual)?.primary_text(), "обѣдома");
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn noun(lemma: &str, case: Case, number: Number) -> InflectionResult {
    resolver::noun(lemma, old_church_slavonic_core::NounCell { case, number })
}

/// Decline one long/compound adjective cell.
///
/// ```
/// use old_church_slavonic::{adjective, Animacy, Case, Gender, Number};
/// assert_eq!(
///     adjective(
///         "добръ", Case::Nominative, Number::Singular,
///         Gender::Masculine, Animacy::Inanimate,
///     )?.primary_text(),
///     "добрꙑи",
/// );
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn adjective(
    lemma: &str,
    case: Case,
    number: Number,
    gender: Gender,
    animacy: Animacy,
) -> InflectionResult {
    adjective_form(lemma, AdjectiveForm::Long, case, number, gender, animacy)
}

/// Decline one short/simple adjective cell.
///
/// ```
/// use old_church_slavonic::{short_adjective, Animacy, Case, Gender, Number};
/// assert_eq!(
///     short_adjective(
///         "добръ", Case::Nominative, Number::Singular,
///         Gender::Masculine, Animacy::Inanimate,
///     )?.primary_text(),
///     "добръ",
/// );
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn short_adjective(
    lemma: &str,
    case: Case,
    number: Number,
    gender: Gender,
    animacy: Animacy,
) -> InflectionResult {
    adjective_form(lemma, AdjectiveForm::Short, case, number, gender, animacy)
}

fn adjective_form(
    lemma: &str,
    form: AdjectiveForm,
    case: Case,
    number: Number,
    gender: Gender,
    animacy: Animacy,
) -> InflectionResult {
    resolver::adjective(
        lemma,
        old_church_slavonic_core::AdjectiveCell {
            case,
            number,
            gender,
            animacy,
            form,
        },
    )
}

/// Inflect a present-indicative verb cell.
///
/// ```
/// use old_church_slavonic::{verb, Number, Person};
/// assert_eq!(
///     verb("благословити", Person::First, Number::Singular)?.primary_text(),
///     "благословлѭ",
/// );
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn verb(lemma: &str, person: Person, number: Number) -> InflectionResult {
    finite_verb(lemma, FiniteTense::Present, person, number)
}

/// Inflect one imperfect person-number cell.
///
/// ```
/// use old_church_slavonic::{imperfect, Number, Person};
/// let forms = imperfect("бꙑти", Person::First, Number::Singular)?;
/// assert!(!forms.primary_text().is_empty());
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn imperfect(lemma: &str, person: Person, number: Number) -> InflectionResult {
    finite_verb(lemma, FiniteTense::Imperfect, person, number)
}

/// Inflect one aorist person-number cell.
///
/// ```
/// use old_church_slavonic::{aorist, Number, Person};
/// let forms = aorist("бꙑти", Person::First, Number::Singular)?;
/// assert_eq!(forms.texts().collect::<Vec<_>>(), ["бѣхъ", "бꙑхъ"]);
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn aorist(lemma: &str, person: Person, number: Number) -> InflectionResult {
    finite_verb(lemma, FiniteTense::Aorist, person, number)
}

/// Inflect one cell in an explicitly selected synthetic finite tense.
///
/// ```
/// use old_church_slavonic::{finite_verb, FiniteTense, Number, Person};
/// let forms = finite_verb(
///     "благословити", FiniteTense::Present, Person::First, Number::Singular,
/// )?;
/// assert_eq!(forms.primary_text(), "благословлѭ");
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn finite_verb(
    lemma: &str,
    tense: FiniteTense,
    person: Person,
    number: Number,
) -> InflectionResult {
    resolver::finite_verb(
        lemma,
        old_church_slavonic_core::FiniteVerbCell {
            tense,
            person,
            number,
        },
    )
}

/// Inflect one historically represented imperative cell.
///
/// ```
/// use old_church_slavonic::{imperative, Number, Person};
/// assert_eq!(
///     imperative("благословити", Person::Second, Number::Singular)?.primary_text(),
///     "благослови",
/// );
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn imperative(lemma: &str, person: Person, number: Number) -> InflectionResult {
    resolver::imperative(
        lemma,
        old_church_slavonic_core::ImperativeCell { person, number },
    )
}

/// Inflect one gender-number l-participle cell.
///
/// ```
/// use old_church_slavonic::{l_participle, Gender, Number};
/// assert_eq!(
///     l_participle("благословити", Gender::Feminine, Number::Dual)?.primary_text(),
///     "благословилѣ",
/// );
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn l_participle(lemma: &str, gender: Gender, number: Number) -> InflectionResult {
    resolver::l_participle(
        lemma,
        old_church_slavonic_core::LParticipleCell { gender, number },
    )
}

/// Return the dictionary infinitive.
///
/// ```
/// assert_eq!(
///     old_church_slavonic::infinitive("благословити")?.primary_text(),
///     "благословити",
/// );
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn infinitive(lemma: &str) -> InflectionResult {
    resolver::infinitive(lemma)
}

/// Return a dictionary or safely generated supine.
///
/// ```
/// assert_eq!(old_church_slavonic::supine("бости")?.primary_text(), "бостъ");
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn supine(lemma: &str) -> InflectionResult {
    resolver::supine(lemma)
}

/// Return a dictionary-listed verbal noun.
///
/// ```
/// let forms = old_church_slavonic::verbal_noun("благословити")?;
/// assert_eq!(forms.primary_text(), "благословлѥниѥ");
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn verbal_noun(lemma: &str) -> InflectionResult {
    resolver::verbal_noun(lemma)
}

/// Return dictionary-listed comparative citations.
///
/// ```
/// let result = old_church_slavonic::comparative("добръ");
/// assert!(result.is_ok() || matches!(result, Err(old_church_slavonic::InflectionError::UnsupportedCell)));
/// ```
pub fn comparative(lemma: &str) -> InflectionResult {
    resolver::adjective_comparatives(lemma)
}

/// Bind the present active participle system of one verb.
///
/// ```
/// let participle = old_church_slavonic::present_active_participle("благословити")?;
/// assert!(!participle.citation()?.primary_text().is_empty());
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn present_active_participle(lemma: &str) -> Result<Participle, InflectionError> {
    Verb::new(lemma)?.present_active_participle()
}

/// Bind the present passive participle system of one verb.
///
/// ```
/// let participle = old_church_slavonic::present_passive_participle("благословити")?;
/// assert!(!participle.citation()?.primary_text().is_empty());
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn present_passive_participle(lemma: &str) -> Result<Participle, InflectionError> {
    Verb::new(lemma)?.present_passive_participle()
}

/// Bind the past active participle system of one verb.
///
/// ```
/// let participle = old_church_slavonic::past_active_participle("благословити")?;
/// assert_eq!(participle.citation()?.texts().count(), 2);
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn past_active_participle(lemma: &str) -> Result<Participle, InflectionError> {
    Verb::new(lemma)?.past_active_participle()
}

/// Bind the past passive participle system of one verb.
///
/// ```
/// let participle = old_church_slavonic::past_passive_participle("благословити")?;
/// assert!(!participle.citation()?.primary_text().is_empty());
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn past_passive_participle(lemma: &str) -> Result<Participle, InflectionError> {
    Verb::new(lemma)?.past_passive_participle()
}

/// Enumerate all noun cells after resolving one dictionary lemma.
///
/// ```
/// let paradigm = old_church_slavonic::noun_paradigm("обѣдъ")?;
/// assert_eq!(paradigm.len(), 21);
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn noun_paradigm(lemma: &str) -> Result<NounParadigm, InflectionError> {
    Ok(Noun::new(lemma)?.paradigm())
}

/// Enumerate long and short adjective cells after resolving one lemma.
///
/// ```
/// let paradigm = old_church_slavonic::adjective_paradigm("добръ")?;
/// assert!(!paradigm.is_empty());
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn adjective_paradigm(lemma: &str) -> Result<AdjectiveParadigm, InflectionError> {
    Ok(Adjective::new(lemma)?.paradigm())
}

/// Enumerate the nine present-indicative person-number cells.
///
/// ```
/// let paradigm = old_church_slavonic::verb_paradigm("благословити")?;
/// assert_eq!(paradigm.len(), 9);
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn verb_paradigm(lemma: &str) -> Result<VerbParadigm, InflectionError> {
    Ok(Verb::new(lemma)?.paradigm())
}

/// Enumerate present, imperfect, and aorist cells after resolving one lemma.
///
/// ```
/// let paradigm = old_church_slavonic::finite_verb_paradigm("благословити")?;
/// assert_eq!(paradigm.len(), 27);
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn finite_verb_paradigm(lemma: &str) -> Result<FiniteVerbParadigm, InflectionError> {
    Ok(Verb::new(lemma)?.finite_paradigm())
}

/// Enumerate the six historically represented imperative cells.
///
/// ```
/// let paradigm = old_church_slavonic::imperative_paradigm("благословити")?;
/// assert_eq!(paradigm.len(), 6);
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn imperative_paradigm(lemma: &str) -> Result<ImperativeParadigm, InflectionError> {
    Ok(Verb::new(lemma)?.imperative_paradigm())
}

/// Enumerate every gender-number l-participle cell.
///
/// ```
/// let paradigm = old_church_slavonic::l_participle_paradigm("благословити")?;
/// assert_eq!(paradigm.len(), 9);
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn l_participle_paradigm(lemma: &str) -> Result<LParticipleParadigm, InflectionError> {
    Ok(Verb::new(lemma)?.l_participle_paradigm())
}

/// Enumerate both agreement paradigms for one declined participle system.
///
/// ```
/// use old_church_slavonic::{participle_paradigm, ParticipleKind};
/// let paradigm = participle_paradigm("благословити", ParticipleKind::PastActive)?;
/// assert_eq!(paradigm.kind(), ParticipleKind::PastActive);
/// assert!(!paradigm.is_empty());
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn participle_paradigm(
    lemma: &str,
    kind: ParticipleKind,
) -> Result<ParticipleParadigm, InflectionError> {
    Ok(Verb::new(lemma)?.participle(kind)?.paradigm())
}
