use synodal_church_slavonic_core::{
    AdjectiveForm, Animacy, Case, Comparison, CompoundAuxiliaryOrder, CompoundFutureAuxiliary,
    Error, FormSet, LexemeId, Number, ParticipleCell, ParticipleTense, ParticipleVoice,
    PassiveAgentGovernment, PassiveFormation, Person, PhraseFormation, PhraseOrder, PhraseRole,
    PhraseToken, RealizedPhrase, Result,
};

#[allow(unused_imports)]
use super::*;
use crate::{Inflector, Noun, PartOfSpeech, Participle, Verb};

/// Compatibility wrapper selecting present or perfect from the participle
/// tense in Alypy §102.
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
    let formation = match participle_cell.tense {
        ParticipleTense::Present => PassiveFormation::Present,
        ParticipleTense::Past => PassiveFormation::Perfect,
    };
    analytic_passive_formation(
        lemma,
        participle_cell,
        formation,
        person,
        number,
        PhraseOrder::PredicateFirst,
        inflector,
    )
}

pub fn analytic_passive_formation(
    lemma: &str,
    participle_cell: ParticipleCell,
    formation: PassiveFormation,
    person: Person,
    number: Number,
    order: PhraseOrder,
    inflector: Inflector,
) -> Result<RealizedPhrase> {
    if matches!(
        formation,
        PassiveFormation::PluperfectPerfectCopula
            | PassiveFormation::PresentParticipleConditional
            | PassiveFormation::PastParticipleConditional
    ) {
        return Err(Error::ContradictoryMetadata {
            reason: "three-token passive auxiliaries require analytic_passive_compound_auxiliary"
                .into(),
        });
    }
    typed_phrase(
        PhraseFormation::AnalyticPassive(formation),
        passive_binary_tokens(
            lemma,
            participle_cell,
            formation,
            person,
            number,
            order,
            inflector,
        )?,
    )
}

pub fn analytic_passive_compound_auxiliary(
    lemma: &str,
    participle_cell: ParticipleCell,
    formation: PassiveFormation,
    person: Person,
    number: Number,
    order: CompoundAuxiliaryOrder,
    inflector: Inflector,
) -> Result<RealizedPhrase> {
    typed_phrase(
        PhraseFormation::AnalyticPassive(formation),
        passive_compound_tokens(
            lemma,
            participle_cell,
            formation,
            person,
            number,
            order,
            inflector,
        )?,
    )
}

pub(super) fn passive_compound_tokens(
    lemma: &str,
    participle_cell: ParticipleCell,
    formation: PassiveFormation,
    person: Person,
    number: Number,
    order: CompoundAuxiliaryOrder,
    inflector: Inflector,
) -> Result<Vec<PhraseToken>> {
    if !matches!(
        formation,
        PassiveFormation::PluperfectPerfectCopula
            | PassiveFormation::PresentParticipleConditional
            | PassiveFormation::PastParticipleConditional
    ) {
        return Err(Error::ContradictoryMetadata {
            reason: "the selected passive formation has no compound auxiliary".into(),
        });
    }
    validate_passive_formation(participle_cell, formation, number)?;
    let copula = byti(inflector)?;
    let finite = if formation == PassiveFormation::PluperfectPerfectCopula {
        copula.present(person, number)?
    } else {
        copula.aorist(person, number)?
    };
    Ok(ordered_compound(
        PhraseToken {
            role: PhraseRole::PassiveParticiple,
            forms: Participle::resolve_with(lemma, inflector)?.form(participle_cell)?,
        },
        PhraseToken {
            role: PhraseRole::AuxiliaryParticiple,
            forms: copula.l_participle(
                participle_cell.agreement.gender,
                participle_cell.agreement.number,
            )?,
        },
        PhraseToken {
            role: PhraseRole::Auxiliary,
            forms: finite,
        },
        order,
    ))
}

/// Morphological inputs shared by passive builders that add an explicit agent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PassivePredicateSpec<'a> {
    pub lemma: &'a str,
    pub participle_cell: ParticipleCell,
    pub formation: PassiveFormation,
    pub person: Person,
    pub number: Number,
}

/// A noun agent together with one of Alypy §101's two government patterns.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PassiveNounAgentSpec<'a> {
    pub lemma: &'a str,
    pub number: Number,
    pub animacy: Animacy,
    pub government: PassiveAgentGovernment,
}

/// Adds a noun agent in either source-licensed government pattern (§101).
pub fn analytic_passive_with_noun_agent(
    predicate: PassivePredicateSpec<'_>,
    order: PhraseOrder,
    agent: PassiveNounAgentSpec<'_>,
    inflector: Inflector,
) -> Result<RealizedPhrase> {
    if matches!(
        predicate.formation,
        PassiveFormation::PluperfectPerfectCopula
            | PassiveFormation::PresentParticipleConditional
            | PassiveFormation::PastParticipleConditional
    ) {
        return Err(Error::ContradictoryMetadata {
            reason: "compound passive auxiliaries require the dedicated compound builder".into(),
        });
    }
    let tokens = passive_binary_tokens(
        predicate.lemma,
        predicate.participle_cell,
        predicate.formation,
        predicate.person,
        predicate.number,
        order,
        inflector,
    )?;
    let case = match agent.government {
        PassiveAgentGovernment::Instrumental => Case::Instrumental,
        PassiveAgentGovernment::OtGenitive => Case::Genitive,
    };
    analytic_passive_tokens_with_agent(
        predicate.formation,
        tokens,
        Noun::resolve_with(agent.lemma, inflector)?.form(case, agent.number, agent.animacy)?,
        agent.government,
        inflector,
    )
}

/// Adds an already inflected agent to any two-token passive. The caller is
/// responsible for supplying instrumental or genitive forms matching the
/// declared government; the noun-specific builder above performs that
/// inflection automatically.
pub fn analytic_passive_with_agent_forms(
    predicate: PassivePredicateSpec<'_>,
    order: PhraseOrder,
    agent_forms: FormSet,
    government: PassiveAgentGovernment,
    inflector: Inflector,
) -> Result<RealizedPhrase> {
    let tokens = passive_binary_tokens(
        predicate.lemma,
        predicate.participle_cell,
        predicate.formation,
        predicate.person,
        predicate.number,
        order,
        inflector,
    )?;
    analytic_passive_tokens_with_agent(
        predicate.formation,
        tokens,
        agent_forms,
        government,
        inflector,
    )
}

/// Adds a noun agent to any of the three §102 passives whose auxiliary is
/// itself compound.
pub fn analytic_passive_compound_with_noun_agent(
    predicate: PassivePredicateSpec<'_>,
    order: CompoundAuxiliaryOrder,
    agent: PassiveNounAgentSpec<'_>,
    inflector: Inflector,
) -> Result<RealizedPhrase> {
    let tokens = passive_compound_tokens(
        predicate.lemma,
        predicate.participle_cell,
        predicate.formation,
        predicate.person,
        predicate.number,
        order,
        inflector,
    )?;
    let case = match agent.government {
        PassiveAgentGovernment::Instrumental => Case::Instrumental,
        PassiveAgentGovernment::OtGenitive => Case::Genitive,
    };
    analytic_passive_tokens_with_agent(
        predicate.formation,
        tokens,
        Noun::resolve_with(agent.lemma, inflector)?.form(case, agent.number, agent.animacy)?,
        agent.government,
        inflector,
    )
}

/// Adds an already inflected agent to any compound-auxiliary passive.
pub fn analytic_passive_compound_with_agent_forms(
    predicate: PassivePredicateSpec<'_>,
    order: CompoundAuxiliaryOrder,
    agent_forms: FormSet,
    government: PassiveAgentGovernment,
    inflector: Inflector,
) -> Result<RealizedPhrase> {
    let tokens = passive_compound_tokens(
        predicate.lemma,
        predicate.participle_cell,
        predicate.formation,
        predicate.person,
        predicate.number,
        order,
        inflector,
    )?;
    analytic_passive_tokens_with_agent(
        predicate.formation,
        tokens,
        agent_forms,
        government,
        inflector,
    )
}

pub(super) fn analytic_passive_tokens_with_agent(
    formation: PassiveFormation,
    mut tokens: Vec<PhraseToken>,
    agent_forms: FormSet,
    government: PassiveAgentGovernment,
    inflector: Inflector,
) -> Result<RealizedPhrase> {
    if government == PassiveAgentGovernment::OtGenitive {
        tokens.push(PhraseToken {
            role: PhraseRole::Preposition,
            forms: indeclinable("ѿ", PartOfSpeech::Preposition, inflector)?,
        });
    }
    tokens.push(PhraseToken {
        role: PhraseRole::Agent,
        forms: agent_forms,
    });
    typed_phrase_with_government(
        PhraseFormation::AnalyticPassive(formation),
        tokens,
        government,
    )
}

pub(super) fn compound_future_auxiliary(
    auxiliary: CompoundFutureAuxiliary,
    person: Person,
    number: Number,
    inflector: Inflector,
) -> Result<FormSet> {
    let (id, future) = match auxiliary {
        CompoundFutureAuxiliary::Byti => ("synodal:verb:byti", true),
        CompoundFutureAuxiliary::Imati => ("synodal:verb:imati", false),
        CompoundFutureAuxiliary::Khoteti => ("synodal:verb:wikt-070505e82800", false),
        CompoundFutureAuxiliary::Nachati => ("synodal:verb:v07-35ce5d83583f3639", true),
    };
    let verb = Verb::from_id_with(&LexemeId::from(id), inflector)?;
    if future {
        verb.future(person, number)
    } else {
        verb.present(person, number)
    }
}

pub(super) fn passive_binary_tokens(
    lemma: &str,
    participle_cell: ParticipleCell,
    formation: PassiveFormation,
    person: Person,
    number: Number,
    order: PhraseOrder,
    inflector: Inflector,
) -> Result<Vec<PhraseToken>> {
    validate_passive_formation(participle_cell, formation, number)?;
    let copula = byti(inflector)?;
    let auxiliary = match formation {
        PassiveFormation::PresentParticipleInfinitive
        | PassiveFormation::PastParticipleInfinitive => copula.infinitive()?,
        PassiveFormation::Present | PassiveFormation::Perfect => copula.present(person, number)?,
        PassiveFormation::PresentParticipleFuture | PassiveFormation::PastParticipleFuture => {
            copula.future(person, number)?
        }
        PassiveFormation::PresentParticipleAorist | PassiveFormation::PastParticipleAorist => {
            copula.aorist(person, number)?
        }
        PassiveFormation::PresentParticipleAoristBe | PassiveFormation::PluperfectAoristBe => {
            copula_be(person, number, inflector)?
        }
        PassiveFormation::Imperfect | PassiveFormation::PluperfectImperfectBya => {
            copula_bya(person, number, inflector)?
        }
        PassiveFormation::PresentParticipleImperative
        | PassiveFormation::PastParticipleImperative => copula.imperative(person, number)?,
        PassiveFormation::PluperfectPerfectCopula
        | PassiveFormation::PresentParticipleConditional
        | PassiveFormation::PastParticipleConditional => {
            return Err(Error::ContradictoryMetadata {
                reason: "the selected passive formation requires a compound auxiliary".into(),
            });
        }
    };
    Ok(ordered_pair(
        PhraseToken {
            role: PhraseRole::PassiveParticiple,
            forms: Participle::resolve_with(lemma, inflector)?.form(participle_cell)?,
        },
        PhraseToken {
            role: PhraseRole::Auxiliary,
            forms: auxiliary,
        },
        order,
    ))
}

pub(super) fn validate_passive_formation(
    participle_cell: ParticipleCell,
    formation: PassiveFormation,
    subject_number: Number,
) -> Result<()> {
    let expected_tense = match formation {
        PassiveFormation::PresentParticipleInfinitive
        | PassiveFormation::Present
        | PassiveFormation::PresentParticipleFuture
        | PassiveFormation::PresentParticipleAorist
        | PassiveFormation::PresentParticipleAoristBe
        | PassiveFormation::Imperfect
        | PassiveFormation::PresentParticipleConditional
        | PassiveFormation::PresentParticipleImperative => ParticipleTense::Present,
        PassiveFormation::PastParticipleInfinitive
        | PassiveFormation::PastParticipleFuture
        | PassiveFormation::PastParticipleAorist
        | PassiveFormation::Perfect
        | PassiveFormation::PluperfectAoristBe
        | PassiveFormation::PluperfectImperfectBya
        | PassiveFormation::PluperfectPerfectCopula
        | PassiveFormation::PastParticipleConditional
        | PassiveFormation::PastParticipleImperative => ParticipleTense::Past,
    };
    validate_predicative_participle(
        participle_cell,
        subject_number,
        ParticipleVoice::Passive,
        expected_tense,
    )
}

pub(super) fn validate_predicative_participle(
    cell: ParticipleCell,
    subject_number: Number,
    voice: ParticipleVoice,
    tense: ParticipleTense,
) -> Result<()> {
    if cell.voice != voice || cell.tense != tense {
        return Err(Error::HistoricallyInvalidCell {
            reason: format!("the construction requires a {tense:?} {voice:?} participle"),
        });
    }
    if cell.agreement.case != Case::Nominative
        || cell.agreement.form != AdjectiveForm::Short
        || cell.agreement.comparison != Comparison::Positive
        || cell.agreement.number != subject_number
    {
        return Err(Error::ContradictoryMetadata {
            reason: "a predicative participle must be short positive nominative and agree with the subject number"
                .into(),
        });
    }
    Ok(())
}
