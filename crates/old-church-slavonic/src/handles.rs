//! Resolved dictionary identities for repeated inflection.

use crate::{
    AdjectiveForm, AdjectiveParadigm, Animacy, Case, FiniteTense, FiniteVerbParadigm, FormSet,
    Gender, ImperativeParadigm, InflectionError, LParticipleParadigm, NounParadigm, Number,
    PartOfSpeech, ParticipleKind, ParticipleParadigm, Person, VerbParadigm, lookup, resolver,
};
use old_church_slavonic_core::{
    AdjectiveCell, FiniteVerbCell, ImperativeCell, LParticipleCell, NounCell, ParticipleCell,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedIdentity {
    id: String,
    lemma: String,
}

impl ResolvedIdentity {
    fn new(lemma: &str, part_of_speech: PartOfSpeech) -> Result<Self, InflectionError> {
        let record = lookup::resolve_one(lemma, part_of_speech)?;
        Ok(Self {
            id: record.id.to_string(),
            lemma: record.lemma.to_string(),
        })
    }

    fn from_id(id: &str, part_of_speech: PartOfSpeech) -> Result<Self, InflectionError> {
        let record = lookup::find_lexeme(id).ok_or(InflectionError::UnknownLemma)?;
        if record.pos != part_of_speech.code() {
            return Err(InflectionError::InvalidInput {
                reason: format!("lexeme {id} is {}, not {part_of_speech}", record.pos),
            });
        }
        Ok(Self {
            id: record.id.to_string(),
            lemma: record.lemma.to_string(),
        })
    }
}

/// A uniquely resolved dictionary noun.
///
/// ```
/// use old_church_slavonic::{Case, Noun, Number};
///
/// let noun = Noun::new("обѣдъ")?;
/// assert_eq!(noun.form(Case::Dative, Number::Dual)?.primary_text(), "обѣдома");
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Noun {
    identity: ResolvedIdentity,
}

impl Noun {
    /// Resolve exactly one dictionary noun by lemma.
    pub fn new(lemma: &str) -> Result<Self, InflectionError> {
        Ok(Self {
            identity: ResolvedIdentity::new(lemma, PartOfSpeech::Noun)?,
        })
    }

    /// Bind a stable dictionary ID after validating its part of speech.
    pub fn from_id(id: &str) -> Result<Self, InflectionError> {
        Ok(Self {
            identity: ResolvedIdentity::from_id(id, PartOfSpeech::Noun)?,
        })
    }

    /// The canonical dictionary lemma.
    pub fn lemma(&self) -> &str {
        &self.identity.lemma
    }

    /// The stable dictionary lexeme ID.
    pub fn id(&self) -> &str {
        &self.identity.id
    }

    /// Resolve one case-number cell through the canonical by-ID path.
    pub fn form(&self, case: Case, number: Number) -> Result<FormSet, InflectionError> {
        resolver::noun_by_id(self.id(), NounCell { case, number })
    }

    /// Enumerate all 21 noun cells through [`Self::form`]'s resolver.
    pub fn paradigm(&self) -> NounParadigm {
        resolver::build_noun_paradigm(self.id(), self.lemma())
    }
}

/// A uniquely resolved dictionary adjective.
///
/// ```
/// use old_church_slavonic::{Adjective, Animacy, Case, Gender, Number};
///
/// let adjective = Adjective::new("добръ")?;
/// assert_eq!(
///     adjective.long(
///         Case::Nominative,
///         Number::Singular,
///         Gender::Masculine,
///         Animacy::Inanimate,
///     )?.primary_text(),
///     "добрꙑи",
/// );
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Adjective {
    identity: ResolvedIdentity,
}

impl Adjective {
    /// Resolve exactly one dictionary adjective by lemma.
    pub fn new(lemma: &str) -> Result<Self, InflectionError> {
        Ok(Self {
            identity: ResolvedIdentity::new(lemma, PartOfSpeech::Adjective)?,
        })
    }

    /// Bind a stable dictionary ID after validating its part of speech.
    pub fn from_id(id: &str) -> Result<Self, InflectionError> {
        Ok(Self {
            identity: ResolvedIdentity::from_id(id, PartOfSpeech::Adjective)?,
        })
    }

    /// The canonical dictionary lemma.
    pub fn lemma(&self) -> &str {
        &self.identity.lemma
    }

    /// The stable dictionary lexeme ID.
    pub fn id(&self) -> &str {
        &self.identity.id
    }

    /// Resolve one long/compound adjective agreement cell.
    pub fn long(
        &self,
        case: Case,
        number: Number,
        gender: Gender,
        animacy: Animacy,
    ) -> Result<FormSet, InflectionError> {
        self.form(AdjectiveForm::Long, case, number, gender, animacy)
    }

    /// Resolve one short/simple adjective agreement cell.
    pub fn short(
        &self,
        case: Case,
        number: Number,
        gender: Gender,
        animacy: Animacy,
    ) -> Result<FormSet, InflectionError> {
        self.form(AdjectiveForm::Short, case, number, gender, animacy)
    }

    fn form(
        &self,
        form: AdjectiveForm,
        case: Case,
        number: Number,
        gender: Gender,
        animacy: Animacy,
    ) -> Result<FormSet, InflectionError> {
        resolver::adjective_by_id(
            self.id(),
            AdjectiveCell {
                case,
                number,
                gender,
                animacy,
                form,
            },
        )
    }

    /// Return dictionary-listed comparative citations.
    pub fn comparative(&self) -> Result<FormSet, InflectionError> {
        resolver::adjective_comparatives_by_id(self.id())
    }

    /// Enumerate both adjective paradigms through the canonical cell resolver.
    pub fn paradigm(&self) -> AdjectiveParadigm {
        resolver::build_adjective_paradigm(self.id(), self.lemma())
    }
}

/// A uniquely resolved dictionary verb.
///
/// ```
/// use old_church_slavonic::{Number, Person, Verb};
///
/// let verb = Verb::new("благословити")?;
/// assert_eq!(
///     verb.present(Person::First, Number::Singular)?.primary_text(),
///     "благословлѭ",
/// );
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verb {
    identity: ResolvedIdentity,
}

impl Verb {
    /// Resolve exactly one dictionary verb by lemma.
    pub fn new(lemma: &str) -> Result<Self, InflectionError> {
        Ok(Self {
            identity: ResolvedIdentity::new(lemma, PartOfSpeech::Verb)?,
        })
    }

    /// Bind a stable dictionary ID after validating its part of speech.
    pub fn from_id(id: &str) -> Result<Self, InflectionError> {
        Ok(Self {
            identity: ResolvedIdentity::from_id(id, PartOfSpeech::Verb)?,
        })
    }

    /// The canonical dictionary lemma.
    pub fn lemma(&self) -> &str {
        &self.identity.lemma
    }

    /// The stable dictionary lexeme ID.
    pub fn id(&self) -> &str {
        &self.identity.id
    }

    /// Resolve one present-indicative person-number cell.
    pub fn present(&self, person: Person, number: Number) -> Result<FormSet, InflectionError> {
        self.finite(FiniteTense::Present, person, number)
    }

    /// Resolve one imperfect person-number cell.
    pub fn imperfect(&self, person: Person, number: Number) -> Result<FormSet, InflectionError> {
        self.finite(FiniteTense::Imperfect, person, number)
    }

    /// Resolve one aorist person-number cell.
    pub fn aorist(&self, person: Person, number: Number) -> Result<FormSet, InflectionError> {
        self.finite(FiniteTense::Aorist, person, number)
    }

    /// Resolve one finite cell in an explicitly selected synthetic tense.
    pub fn finite(
        &self,
        tense: FiniteTense,
        person: Person,
        number: Number,
    ) -> Result<FormSet, InflectionError> {
        resolver::finite_verb_by_id(
            self.id(),
            FiniteVerbCell {
                tense,
                person,
                number,
            },
        )
    }

    /// Resolve one historically represented imperative cell.
    pub fn imperative(&self, person: Person, number: Number) -> Result<FormSet, InflectionError> {
        resolver::imperative_by_id(self.id(), ImperativeCell { person, number })
    }

    /// Resolve the dictionary infinitive.
    pub fn infinitive(&self) -> Result<FormSet, InflectionError> {
        resolver::infinitive_by_id(self.id())
    }

    /// Resolve the dictionary or table-backed supine.
    pub fn supine(&self) -> Result<FormSet, InflectionError> {
        resolver::supine_by_id(self.id())
    }

    /// Resolve a dictionary-listed verbal noun.
    pub fn verbal_noun(&self) -> Result<FormSet, InflectionError> {
        resolver::verbal_noun_by_id(self.id())
    }

    /// Resolve one gender-number l-participle cell.
    pub fn l_participle(&self, gender: Gender, number: Number) -> Result<FormSet, InflectionError> {
        resolver::l_participle_by_id(self.id(), LParticipleCell { gender, number })
    }

    /// Bind one independently represented participle system.
    pub fn participle(&self, kind: ParticipleKind) -> Result<Participle, InflectionError> {
        let participle = Participle {
            identity: self.identity.clone(),
            kind,
        };
        participle.citation()?;
        Ok(participle)
    }

    /// Bind the present active participle system.
    pub fn present_active_participle(&self) -> Result<Participle, InflectionError> {
        self.participle(ParticipleKind::PresentActive)
    }

    /// Bind the present passive participle system.
    pub fn present_passive_participle(&self) -> Result<Participle, InflectionError> {
        self.participle(ParticipleKind::PresentPassive)
    }

    /// Bind the past active participle system.
    pub fn past_active_participle(&self) -> Result<Participle, InflectionError> {
        self.participle(ParticipleKind::PastActive)
    }

    /// Bind the past passive participle system.
    pub fn past_passive_participle(&self) -> Result<Participle, InflectionError> {
        self.participle(ParticipleKind::PastPassive)
    }

    /// Enumerate the nine present-indicative cells.
    pub fn paradigm(&self) -> VerbParadigm {
        resolver::build_verb_paradigm(self.id(), self.lemma())
    }

    /// Enumerate present, imperfect, and aorist cells.
    pub fn finite_paradigm(&self) -> FiniteVerbParadigm {
        resolver::build_finite_verb_paradigm(self.id(), self.lemma())
    }

    /// Enumerate the six historically represented imperative cells.
    pub fn imperative_paradigm(&self) -> ImperativeParadigm {
        resolver::build_imperative_paradigm(self.id(), self.lemma())
    }

    /// Enumerate every gender-number l-participle cell.
    pub fn l_participle_paradigm(&self) -> LParticipleParadigm {
        resolver::build_l_participle_paradigm(self.id(), self.lemma())
    }
}

/// One resolved verb and one independently represented participle system.
///
/// ```
/// use old_church_slavonic::{Animacy, Case, Gender, Number, Verb};
///
/// let participle = Verb::new("благословити")?.past_active_participle()?;
/// let forms = participle.short(
///     Case::Genitive,
///     Number::Singular,
///     Gender::Masculine,
///     Animacy::Inanimate,
/// )?;
/// assert_eq!(forms.texts().collect::<Vec<_>>(), ["благословл҄ьша", "благословивъша"]);
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Participle {
    identity: ResolvedIdentity,
    kind: ParticipleKind,
}

impl Participle {
    /// The canonical dictionary verb lemma.
    pub fn lemma(&self) -> &str {
        &self.identity.lemma
    }

    /// The stable dictionary verb ID.
    pub fn id(&self) -> &str {
        &self.identity.id
    }

    /// The independently represented participle system.
    pub fn kind(&self) -> ParticipleKind {
        self.kind
    }

    /// Resolve the short masculine nominative singular citation cell.
    pub fn citation(&self) -> Result<FormSet, InflectionError> {
        resolver::participle_citation_by_id(self.id(), self.kind)
    }

    /// Resolve one short/simple declined participle cell.
    pub fn short(
        &self,
        case: Case,
        number: Number,
        gender: Gender,
        animacy: Animacy,
    ) -> Result<FormSet, InflectionError> {
        self.form(AdjectiveForm::Short, case, number, gender, animacy)
    }

    /// Resolve one long/compound declined participle cell.
    pub fn long(
        &self,
        case: Case,
        number: Number,
        gender: Gender,
        animacy: Animacy,
    ) -> Result<FormSet, InflectionError> {
        self.form(AdjectiveForm::Long, case, number, gender, animacy)
    }

    fn form(
        &self,
        form: AdjectiveForm,
        case: Case,
        number: Number,
        gender: Gender,
        animacy: Animacy,
    ) -> Result<FormSet, InflectionError> {
        resolver::participle_by_id(
            self.id(),
            ParticipleCell {
                kind: self.kind,
                adjective: AdjectiveCell {
                    case,
                    number,
                    gender,
                    animacy,
                    form,
                },
            },
        )
    }

    /// Enumerate both adjective agreement paradigms for this participle.
    pub fn paradigm(&self) -> ParticipleParadigm {
        resolver::build_participle_paradigm(self.id(), self.lemma(), self.kind)
    }
}
