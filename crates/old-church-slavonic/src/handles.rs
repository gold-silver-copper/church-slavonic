//! Resolved dictionary identities for repeated inflection.

use crate::{
    AdjectiveForm, AdjectiveParadigm, Animacy, Case, DeterminerParadigm, FiniteTense,
    FiniteVerbParadigm, FormSet, Gender, GenderedNumeralParadigm, GenderedPronounParadigm,
    ImperativeParadigm, InflectionError, LParticipleParadigm, NounParadigm, Number,
    NumeralParadigm, PartOfSpeech, ParticipleKind, ParticipleParadigm, Person,
    PersonalPronounParadigm, PronounParadigm, VerbParadigm, lookup, resolver,
};
use old_church_slavonic_core::{
    AdjectiveCell, DeterminerCell, FiniteVerbCell, GenderedCell, ImperativeCell, LParticipleCell,
    NounCell, ParticipleCell, PersonalPronounCell, UngenderedCell,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedIdentity {
    id: String,
    lemma: String,
}

impl ResolvedIdentity {
    fn resolve(lemma: &str, part_of_speech: PartOfSpeech) -> Result<Self, InflectionError> {
        let record = lookup::resolve_one(lemma, part_of_speech)?;
        Ok(Self {
            id: record.id.to_string(),
            lemma: record.lemma.to_string(),
        })
    }

    fn from_id(id: &str, part_of_speech: PartOfSpeech) -> Result<Self, InflectionError> {
        let record = lookup::find_lexeme(id)
            .ok_or_else(|| InflectionError::unknown_id(id, Some(part_of_speech)))?;
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

// A trait would make these constructors available only when callers import
// that trait. This small declaration macro keeps the ordinary inherent API
// while leaving every linguistic operation in the explicit per-handle impl.
macro_rules! identity_methods {
    ($handle:ident, $part_of_speech:expr) => {
        impl $handle {
            /// Resolve exactly one dictionary identity of this lexical class.
            pub fn resolve(lemma: &str) -> Result<Self, InflectionError> {
                Ok(Self {
                    identity: ResolvedIdentity::resolve(lemma, $part_of_speech)?,
                })
            }

            /// Bind a stable dictionary ID after validating its lexical class.
            pub fn from_id(id: &str) -> Result<Self, InflectionError> {
                Ok(Self {
                    identity: ResolvedIdentity::from_id(id, $part_of_speech)?,
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
        }
    };
}

/// A uniquely resolved dictionary noun.
///
/// ```
/// use old_church_slavonic::{Case, Noun, Number};
///
/// let noun = Noun::resolve("обѣдъ")?;
/// assert_eq!(noun.form(Case::Dative, Number::Dual)?.primary_text(), "обѣдома");
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Noun {
    identity: ResolvedIdentity,
}

identity_methods!(Noun, PartOfSpeech::Noun);

impl Noun {
    /// Resolve one case-number cell through the canonical by-ID path.
    pub fn form(&self, case: Case, number: Number) -> Result<FormSet, InflectionError> {
        resolver::noun_by_id(self.id(), NounCell { case, number })
    }

    /// Enumerate all 21 noun cells through [`Self::form`]'s resolver.
    pub fn paradigm(&self) -> NounParadigm {
        resolver::build_noun_paradigm(self.id(), self.lemma())
    }
}

/// A uniquely resolved dictionary determiner.
///
/// ```
/// use old_church_slavonic::{Animacy, Case, Determiner, Gender, Number};
/// let word = Determiner::resolve("кꙑи")?;
/// assert_eq!(
///     word.form(
///         Case::Accusative, Number::Singular, Gender::Feminine,
///         Animacy::Inanimate,
///     )?
///         .primary_text(),
///     "кѫѭ",
/// );
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Determiner {
    identity: ResolvedIdentity,
}

identity_methods!(Determiner, PartOfSpeech::Determiner);

impl Determiner {
    /// Resolve one case-number-gender cell.
    pub fn form(
        &self,
        case: Case,
        number: Number,
        gender: Gender,
        animacy: Animacy,
    ) -> Result<FormSet, InflectionError> {
        resolver::determiner_by_id(
            self.id(),
            DeterminerCell {
                case,
                number,
                gender,
                animacy,
            },
        )
    }

    /// Enumerate the typed determiner inventory through the canonical resolver.
    pub fn paradigm(&self) -> Result<DeterminerParadigm, InflectionError> {
        resolver::determiner_paradigm_by_id(self.id())
    }
}

/// A uniquely resolved source-backed dictionary pronoun.
///
/// A source record can expose an case-number-only, person-indexed, or
/// gender-indexed system. The three method families remain separate so callers
/// never build a catch-all request from unrelated optional dimensions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pronoun {
    identity: ResolvedIdentity,
}

identity_methods!(Pronoun, PartOfSpeech::Pronoun);

impl Pronoun {
    /// Resolve one case-number cell in an case-number-only pronoun table.
    pub fn form(&self, case: Case, number: Number) -> Result<FormSet, InflectionError> {
        resolver::pronoun_by_id(self.id(), UngenderedCell { case, number })
    }

    /// Resolve one case-number-person cell in a personal-pronoun table.
    pub fn personal(
        &self,
        case: Case,
        number: Number,
        person: Person,
    ) -> Result<FormSet, InflectionError> {
        resolver::personal_pronoun_by_id(
            self.id(),
            PersonalPronounCell {
                case,
                number,
                person,
            },
        )
    }

    /// Resolve one case-number-gender cell in an agreeing pronoun table.
    pub fn gendered(
        &self,
        case: Case,
        number: Number,
        gender: Gender,
    ) -> Result<FormSet, InflectionError> {
        resolver::gendered_pronoun_by_id(
            self.id(),
            GenderedCell {
                case,
                number,
                gender,
            },
        )
    }

    /// Enumerate the case-number-only case-number inventory.
    pub fn paradigm(&self) -> PronounParadigm {
        resolver::build_ungendered_closed_class_paradigm(
            self.id(),
            self.lemma(),
            PartOfSpeech::Pronoun,
        )
    }

    /// Enumerate the person-indexed inventory.
    pub fn personal_paradigm(&self) -> PersonalPronounParadigm {
        resolver::build_personal_pronoun_paradigm(self.id(), self.lemma())
    }

    /// Enumerate the gender-indexed inventory.
    pub fn gendered_paradigm(&self) -> GenderedPronounParadigm {
        resolver::build_gendered_closed_class_paradigm(
            self.id(),
            self.lemma(),
            PartOfSpeech::Pronoun,
        )
    }
}

/// A uniquely resolved source-backed dictionary numeral.
///
/// Ungendered cardinal-like and gendered agreeing tables have separate methods.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Numeral {
    identity: ResolvedIdentity,
}

identity_methods!(Numeral, PartOfSpeech::Numeral);

impl Numeral {
    /// Resolve one case-number cell in an case-number-only numeral table.
    pub fn form(&self, case: Case, number: Number) -> Result<FormSet, InflectionError> {
        resolver::numeral_by_id(self.id(), UngenderedCell { case, number })
    }

    /// Resolve one case-number-gender cell in an agreeing numeral table.
    pub fn gendered(
        &self,
        case: Case,
        number: Number,
        gender: Gender,
    ) -> Result<FormSet, InflectionError> {
        resolver::gendered_numeral_by_id(
            self.id(),
            GenderedCell {
                case,
                number,
                gender,
            },
        )
    }

    /// Enumerate the case-number-only case-number inventory.
    pub fn paradigm(&self) -> NumeralParadigm {
        resolver::build_numeral_paradigm(self.id(), self.lemma())
    }

    /// Enumerate the gender-indexed inventory.
    pub fn gendered_paradigm(&self) -> GenderedNumeralParadigm {
        resolver::build_gendered_numeral_paradigm(self.id(), self.lemma())
    }
}

/// A uniquely resolved dictionary adjective.
///
/// ```
/// use old_church_slavonic::{Adjective, Animacy, Case, Gender, Number};
///
/// let adjective = Adjective::resolve("добръ")?;
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

identity_methods!(Adjective, PartOfSpeech::Adjective);

impl Adjective {
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
    pub fn comparative_citation(&self) -> Result<FormSet, InflectionError> {
        resolver::comparative_citation_by_id(self.id())
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
/// let verb = Verb::resolve("благословити")?;
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

identity_methods!(Verb, PartOfSpeech::Verb);

impl Verb {
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
        resolver::finite_by_id(
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
    pub fn present_paradigm(&self) -> VerbParadigm {
        resolver::build_present_paradigm(self.id(), self.lemma())
    }

    /// Enumerate present, imperfect, and aorist cells.
    pub fn finite_paradigm(&self) -> FiniteVerbParadigm {
        resolver::build_finite_paradigm(self.id(), self.lemma())
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
/// let participle = Verb::resolve("благословити")?.past_active_participle()?;
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
