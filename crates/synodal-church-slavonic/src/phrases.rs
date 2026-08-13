use synodal_church_slavonic_core::{
    AnalyticConstruction, Aspect, Error, FormSet, Gender, Number, ParticipleCell, ParticipleVoice,
    Person, PhraseRole, PhraseToken, RealizedPhrase, Result,
};

use crate::{Inflector, Participle, Verb};

fn token(role: PhraseRole, forms: FormSet) -> PhraseToken {
    PhraseToken { role, forms }
}

fn copular_l_participle(
    construction: AnalyticConstruction,
    lemma: &str,
    person: Person,
    number: Number,
    gender: Gender,
    inflector: Inflector,
    auxiliary_form: impl FnOnce(&Verb, Person, Number) -> Result<FormSet>,
) -> Result<RealizedPhrase> {
    let verb = Verb::resolve_with(lemma, inflector)?;
    let copula = Verb::resolve_with("быти", inflector)?;
    RealizedPhrase::new(
        construction,
        vec![
            token(PhraseRole::LParticiple, verb.l_participle(gender, number)?),
            token(
                PhraseRole::Auxiliary,
                auxiliary_form(&copula, person, number)?,
            ),
        ],
    )
}

/// Realizes the Alypy §85 compound future: a present form of the auxiliary
/// `имати` followed by the infinitive of an imperfective lexical verb.
pub fn compound_future(lemma: &str, person: Person, number: Number) -> Result<RealizedPhrase> {
    compound_future_with(lemma, person, number, Inflector::default())
}

pub fn compound_future_with(
    lemma: &str,
    person: Person,
    number: Number,
    inflector: Inflector,
) -> Result<RealizedPhrase> {
    let verb = Verb::resolve_with(lemma, inflector)?;
    if verb.aspect()? != Aspect::Imperfective {
        return Err(Error::HistoricallyInvalidCell {
            reason: "Alypy §85 restricts the compound future to imperfective verbs".into(),
        });
    }
    let auxiliary = Verb::resolve_with("имати", inflector)?;
    RealizedPhrase::new(
        AnalyticConstruction::CompoundFuture,
        vec![
            token(PhraseRole::Auxiliary, auxiliary.present(person, number)?),
            token(PhraseRole::Infinitive, verb.infinitive()?),
        ],
    )
}

/// Realizes the Alypy §88 perfect as an l-participle plus the present copula.
pub fn perfect(
    lemma: &str,
    person: Person,
    number: Number,
    gender: Gender,
) -> Result<RealizedPhrase> {
    perfect_with(lemma, person, number, gender, Inflector::default())
}

pub fn perfect_with(
    lemma: &str,
    person: Person,
    number: Number,
    gender: Gender,
    inflector: Inflector,
) -> Result<RealizedPhrase> {
    copular_l_participle(
        AnalyticConstruction::Perfect,
        lemma,
        person,
        number,
        gender,
        inflector,
        Verb::present,
    )
}

/// Realizes the Alypy §89 pluperfect as an l-participle plus an imperfect
/// form of the copula. Alternate copular series remain variants on the token.
pub fn pluperfect(
    lemma: &str,
    person: Person,
    number: Number,
    gender: Gender,
) -> Result<RealizedPhrase> {
    pluperfect_with(lemma, person, number, gender, Inflector::default())
}

pub fn pluperfect_with(
    lemma: &str,
    person: Person,
    number: Number,
    gender: Gender,
    inflector: Inflector,
) -> Result<RealizedPhrase> {
    copular_l_participle(
        AnalyticConstruction::Pluperfect,
        lemma,
        person,
        number,
        gender,
        inflector,
        Verb::imperfect,
    )
}

/// Realizes the Alypy §91 conditional as an l-participle plus the aorist of
/// `быти`; it remains a structured phrase rather than a space-bearing word.
pub fn conditional(
    lemma: &str,
    person: Person,
    number: Number,
    gender: Gender,
) -> Result<RealizedPhrase> {
    conditional_with(lemma, person, number, gender, Inflector::default())
}

pub fn conditional_with(
    lemma: &str,
    person: Person,
    number: Number,
    gender: Gender,
    inflector: Inflector,
) -> Result<RealizedPhrase> {
    copular_l_participle(
        AnalyticConstruction::Conditional,
        lemma,
        person,
        number,
        gender,
        inflector,
        Verb::aorist,
    )
}

/// Realizes a reviewed passive participle plus the present copula as a typed
/// analytic passive construction (Alypy §§101–102).
pub fn analytic_passive(
    lemma: &str,
    participle_cell: ParticipleCell,
    person: Person,
    number: Number,
) -> Result<RealizedPhrase> {
    analytic_passive_with(lemma, participle_cell, person, number, Inflector::default())
}

pub fn analytic_passive_with(
    lemma: &str,
    participle_cell: ParticipleCell,
    person: Person,
    number: Number,
    inflector: Inflector,
) -> Result<RealizedPhrase> {
    if participle_cell.voice != ParticipleVoice::Passive {
        return Err(Error::HistoricallyInvalidCell {
            reason: "analytic passive requires a passive participle".into(),
        });
    }
    let participle = Participle::resolve_with(lemma, inflector)?;
    let copula = Verb::resolve_with("быти", inflector)?;
    RealizedPhrase::new(
        AnalyticConstruction::AnalyticPassive,
        vec![
            token(
                PhraseRole::PassiveParticiple,
                participle.form(participle_cell)?,
            ),
            token(PhraseRole::Auxiliary, copula.present(person, number)?),
        ],
    )
}

/// Constructs a typed analytic phrase from already inflected tokens. This is the
/// escape hatch for reviewed future, pluperfect, conditional, passive, and
/// periphrastic combinations whose auxiliary lexeme is not yet in the registry.
pub fn from_tokens(
    construction: AnalyticConstruction,
    tokens: Vec<PhraseToken>,
) -> Result<RealizedPhrase> {
    RealizedPhrase::new(construction, tokens)
}

#[cfg(test)]
mod tests {
    use super::*;
    use synodal_church_slavonic_core::{
        AdjectiveCell, AdjectiveForm, Animacy, Case, Comparison, ParticipleTense,
    };

    #[test]
    fn perfect_is_structured_not_a_fake_word() {
        let phrase = perfect("нести", Person::First, Number::Singular, Gender::Masculine)
            .expect("supported phrase");
        assert_eq!(phrase.tokens().len(), 2);
        assert_eq!(phrase.primary_text(), "неслъ єсмь");
    }

    #[test]
    fn compound_future_uses_reviewed_auxiliary_and_infinitive() {
        let expanded = compound_future("нести", Person::Third, Number::Plural)
            .expect("supported compound future");
        assert_eq!(expanded.primary_text(), "имꙋтъ нести");

        let liturgical = compound_future_with(
            "нести",
            Person::Third,
            Number::Plural,
            Inflector::builder()
                .orthography(synodal_church_slavonic_core::OrthographyProfile::SynodalLiturgical)
                .build(),
        )
        .expect("accented compound future");
        assert_eq!(liturgical.primary_text(), "и҆́мꙋтъ нестѝ");
    }

    #[test]
    fn pluperfect_and_conditional_use_independent_copular_systems() {
        let pluperfect = pluperfect("писати", Person::Third, Number::Singular, Gender::Masculine)
            .expect("supported pluperfect");
        assert_eq!(pluperfect.primary_text(), "писалъ бѣ");
        let copulas = pluperfect.tokens()[1].forms.variants();
        assert_eq!(copulas.len(), 3);
        assert_eq!(
            copulas
                .iter()
                .filter(|variant| variant.is_attested())
                .count(),
            1
        );
        assert_eq!(
            copulas
                .iter()
                .filter(|variant| variant.is_predicted())
                .count(),
            2
        );

        let conditional = conditional("писати", Person::First, Number::Singular, Gender::Masculine)
            .expect("supported conditional");
        assert_eq!(conditional.primary_text(), "писалъ быхъ");
    }

    #[test]
    fn analytic_passive_is_structured_and_voice_checked() {
        let phrase = analytic_passive(
            "нести",
            ParticipleCell {
                tense: ParticipleTense::Past,
                voice: ParticipleVoice::Passive,
                agreement: AdjectiveCell {
                    case: Case::Nominative,
                    number: Number::Singular,
                    gender: Gender::Masculine,
                    animacy: Animacy::Inanimate,
                    form: AdjectiveForm::Short,
                    comparison: Comparison::Positive,
                },
            },
            Person::Third,
            Number::Singular,
        )
        .expect("reviewed passive cell");
        assert_eq!(phrase.primary_text(), "несенъ єсть");
    }
}
