use synodal_church_slavonic_core::{
    AdverbialParticipleFormation, Aspect, CompoundAuxiliaryOrder, CompoundFutureAuxiliary,
    ConditionalCopulaOrder, ConditionalFormation, CopulaOmissionContext, Error, FormSet, Gender,
    GrammarCell, LexemeId, ModalConditionalAuxiliary, Number, OptativeFiniteSystem, ParticipleCell,
    ParticipleTense, ParticipleVoice, PassiveFormation, PerfectFormation, PeriphrasticFormation,
    PeriphrasticSemiAuxiliary, PeriphrasticTenseFormation, Person, PhraseFormation, PhraseOrder,
    PhraseRole, PhraseToken, PluperfectFormation, RealizedPhrase, Result,
};

#[allow(unused_imports)]
use super::*;
use crate::{Inflector, PartOfSpeech, Participle, Verb};

/// Realizes Alypy §85's default `имати` compound future.
pub fn compound_future(lemma: &str, person: Person, number: Number) -> Result<RealizedPhrase> {
    compound_future_with(lemma, person, number, Inflector::default())
}

pub fn compound_future_with(
    lemma: &str,
    person: Person,
    number: Number,
    inflector: Inflector,
) -> Result<RealizedPhrase> {
    compound_future_with_auxiliary(
        lemma,
        CompoundFutureAuxiliary::Imati,
        person,
        number,
        PhraseOrder::AuxiliaryFirst,
        inflector,
    )
}

/// Realizes all four compound-future auxiliaries in the Alypy §85 /
/// Pletneva–Kravetsky lesson 13 source union.
pub fn compound_future_with_auxiliary(
    lemma: &str,
    auxiliary: CompoundFutureAuxiliary,
    person: Person,
    number: Number,
    order: PhraseOrder,
    inflector: Inflector,
) -> Result<RealizedPhrase> {
    let verb = Verb::resolve_with(lemma, inflector)?;
    if verb.aspect()? != Aspect::Imperfective {
        return Err(Error::HistoricallyInvalidCell {
            reason: "the compound future is restricted to imperfective lexical verbs".into(),
        });
    }
    typed_phrase(
        PhraseFormation::CompoundFuture(auxiliary),
        ordered_pair(
            PhraseToken {
                role: PhraseRole::Infinitive,
                forms: verb.infinitive()?,
            },
            PhraseToken {
                role: PhraseRole::Auxiliary,
                forms: compound_future_auxiliary(auxiliary, person, number, inflector)?,
            },
            order,
        ),
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
    perfect_with_formation(
        lemma,
        person,
        number,
        gender,
        PerfectFormation::PresentCopula,
        PhraseOrder::PredicateFirst,
        inflector,
    )
}

pub fn perfect_with_formation(
    lemma: &str,
    person: Person,
    number: Number,
    gender: Gender,
    formation: PerfectFormation,
    order: PhraseOrder,
    inflector: Inflector,
) -> Result<RealizedPhrase> {
    let verb = Verb::resolve_with(lemma, inflector)?;
    let predicate = PhraseToken {
        role: PhraseRole::LParticiple,
        forms: verb.l_participle(gender, number)?,
    };
    match formation {
        PerfectFormation::PresentCopula => typed_phrase(
            PhraseFormation::Perfect(formation),
            ordered_pair(
                predicate,
                PhraseToken {
                    role: PhraseRole::Auxiliary,
                    forms: byti(inflector)?.present(person, number)?,
                },
                order,
            ),
        ),
        PerfectFormation::OmittedThirdSingularCopula => {
            if person != Person::Third || number != Number::Singular {
                return Err(Error::HistoricallyInvalidCell {
                    reason: "Alypy §88 licenses copula-less perfects only in third singular".into(),
                });
            }
            typed_phrase(PhraseFormation::Perfect(formation), vec![predicate])
        }
        PerfectFormation::SharedPresentCopula => Err(Error::ContradictoryMetadata {
            reason: "a shared perfect requires two lexical predicates; use shared_copula_perfect"
                .into(),
        }),
    }
}

/// Realizes two consecutive l-participles with one shared present copula
/// (Alypy §88).
pub fn shared_copula_perfect(
    first_lemma: &str,
    second_lemma: &str,
    person: Person,
    number: Number,
    gender: Gender,
    inflector: Inflector,
) -> Result<RealizedPhrase> {
    let first = Verb::resolve_with(first_lemma, inflector)?;
    let second = Verb::resolve_with(second_lemma, inflector)?;
    typed_phrase(
        PhraseFormation::Perfect(PerfectFormation::SharedPresentCopula),
        vec![
            PhraseToken {
                role: PhraseRole::LParticiple,
                forms: first.l_participle(gender, number)?,
            },
            PhraseToken {
                role: PhraseRole::LParticiple,
                forms: second.l_participle(gender, number)?,
            },
            PhraseToken {
                role: PhraseRole::Auxiliary,
                forms: byti(inflector)?.present(person, number)?,
            },
        ],
    )
}

/// Realizes the Alypy §89 default pluperfect with the `бѣ-` aorist series.
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
    pluperfect_with_formation(
        lemma,
        person,
        number,
        gender,
        PluperfectFormation::AoristBe,
        PhraseOrder::PredicateFirst,
        inflector,
    )
}

pub fn pluperfect_with_formation(
    lemma: &str,
    person: Person,
    number: Number,
    gender: Gender,
    formation: PluperfectFormation,
    order: PhraseOrder,
    inflector: Inflector,
) -> Result<RealizedPhrase> {
    if formation == PluperfectFormation::PerfectCopula {
        return Err(Error::ContradictoryMetadata {
            reason: "the three-token perfect-copula pluperfect requires a CompoundAuxiliaryOrder"
                .into(),
        });
    }
    let verb = Verb::resolve_with(lemma, inflector)?;
    typed_phrase(
        PhraseFormation::Pluperfect(formation),
        ordered_pair(
            PhraseToken {
                role: PhraseRole::LParticiple,
                forms: verb.l_participle(gender, number)?,
            },
            PhraseToken {
                role: PhraseRole::Auxiliary,
                forms: match formation {
                    PluperfectFormation::AoristBe => copula_be(person, number, inflector)?,
                    PluperfectFormation::ImperfectBya => copula_bya(person, number, inflector)?,
                    PluperfectFormation::PerfectCopula => unreachable!("checked above"),
                },
            },
            order,
        ),
    )
}

pub fn pluperfect_with_perfect_copula(
    lemma: &str,
    person: Person,
    number: Number,
    gender: Gender,
    order: CompoundAuxiliaryOrder,
    inflector: Inflector,
) -> Result<RealizedPhrase> {
    let verb = Verb::resolve_with(lemma, inflector)?;
    let copula = byti(inflector)?;
    typed_phrase(
        PhraseFormation::Pluperfect(PluperfectFormation::PerfectCopula),
        ordered_compound(
            PhraseToken {
                role: PhraseRole::LParticiple,
                forms: verb.l_participle(gender, number)?,
            },
            PhraseToken {
                role: PhraseRole::AuxiliaryParticiple,
                forms: copula.l_participle(gender, number)?,
            },
            PhraseToken {
                role: PhraseRole::Auxiliary,
                forms: copula.present(person, number)?,
            },
            order,
        ),
    )
}

/// Realizes the exceptional future anterior, including its required `аще`
/// clause marker (Alypy §162).
pub fn future_anterior(
    lemma: &str,
    person: Person,
    number: Number,
    gender: Gender,
    order: PhraseOrder,
    inflector: Inflector,
) -> Result<RealizedPhrase> {
    let verb = Verb::resolve_with(lemma, inflector)?;
    let mut tokens = vec![PhraseToken {
        role: PhraseRole::Conjunction,
        forms: indeclinable("аще", PartOfSpeech::Conjunction, inflector)?,
    }];
    tokens.extend(ordered_pair(
        PhraseToken {
            role: PhraseRole::LParticiple,
            forms: verb.l_participle(gender, number)?,
        },
        PhraseToken {
            role: PhraseRole::Auxiliary,
            forms: byti(inflector)?.future(person, number)?,
        },
        order,
    ));
    typed_phrase(PhraseFormation::FutureAnterior, tokens)
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
    conditional_with_formation(
        lemma,
        person,
        number,
        gender,
        ConditionalFormation::PersonalAorist,
        PhraseOrder::PredicateFirst,
        inflector,
    )
}

pub fn conditional_with_formation(
    lemma: &str,
    person: Person,
    number: Number,
    gender: Gender,
    formation: ConditionalFormation,
    order: PhraseOrder,
    inflector: Inflector,
) -> Result<RealizedPhrase> {
    let verb = Verb::resolve_with(lemma, inflector)?;
    let auxiliary = match formation {
        ConditionalFormation::PersonalAorist => PhraseToken {
            role: PhraseRole::Auxiliary,
            forms: byti(inflector)?.aorist(person, number)?,
        },
        ConditionalFormation::InvariantBy => PhraseToken {
            role: PhraseRole::Particle,
            forms: invariant_by(inflector)?,
        },
        ConditionalFormation::PersonalAoristWithPresentCopula
        | ConditionalFormation::InvariantByWithPresentCopula => {
            return Err(Error::ContradictoryMetadata {
                reason: "conditional forms with a present copula require conditional_with_present_copula"
                    .into(),
            });
        }
        ConditionalFormation::InfinitiveWithInvariantBy => {
            return Err(Error::ContradictoryMetadata {
                reason: "the infinitival conditional requires infinitive_conditional".into(),
            });
        }
        ConditionalFormation::ModalImperfect(_) => {
            return Err(Error::ContradictoryMetadata {
                reason: "modal conditional predicates require modal_conditional_infinitive".into(),
            });
        }
    };
    typed_phrase(
        PhraseFormation::Conditional(formation),
        ordered_pair(
            PhraseToken {
                role: PhraseRole::LParticiple,
                forms: verb.l_participle(gender, number)?,
            },
            auxiliary,
            order,
        ),
    )
}

pub fn conditional_with_present_copula(
    lemma: &str,
    person: Person,
    number: Number,
    gender: Gender,
    invariant: bool,
    order: ConditionalCopulaOrder,
    inflector: Inflector,
) -> Result<RealizedPhrase> {
    let verb = Verb::resolve_with(lemma, inflector)?;
    let formation = if invariant {
        ConditionalFormation::InvariantByWithPresentCopula
    } else {
        ConditionalFormation::PersonalAoristWithPresentCopula
    };
    let conditional = PhraseToken {
        role: if invariant {
            PhraseRole::Particle
        } else {
            PhraseRole::Auxiliary
        },
        forms: if invariant {
            invariant_by(inflector)?
        } else {
            byti(inflector)?.aorist(person, number)?
        },
    };
    let predicate = PhraseToken {
        role: PhraseRole::LParticiple,
        forms: verb.l_participle(gender, number)?,
    };
    let present = PhraseToken {
        role: PhraseRole::Auxiliary,
        forms: byti(inflector)?.present(person, number)?,
    };
    let tokens = match order {
        ConditionalCopulaOrder::ConditionalPredicatePresent => {
            vec![conditional, predicate, present]
        }
        ConditionalCopulaOrder::ConditionalPresentPredicate => {
            vec![conditional, present, predicate]
        }
    };
    typed_phrase(PhraseFormation::Conditional(formation), tokens)
}

pub fn infinitive_conditional(
    lemma: &str,
    order: PhraseOrder,
    inflector: Inflector,
) -> Result<RealizedPhrase> {
    typed_phrase(
        PhraseFormation::Conditional(ConditionalFormation::InfinitiveWithInvariantBy),
        ordered_pair(
            PhraseToken {
                role: PhraseRole::Infinitive,
                forms: Verb::resolve_with(lemma, inflector)?.infinitive()?,
            },
            PhraseToken {
                role: PhraseRole::Particle,
                forms: invariant_by(inflector)?,
            },
            order,
        ),
    )
}

/// Realizes the fixed third-singular modal imperfects that carry conditional
/// meaning in Alypy §91.
pub fn modal_conditional_infinitive(
    auxiliary: ModalConditionalAuxiliary,
    infinitive_lemma: &str,
    with_invariant_by: bool,
    inflector: Inflector,
) -> Result<RealizedPhrase> {
    if with_invariant_by && auxiliary != ModalConditionalAuxiliary::Podobati {
        return Err(Error::HistoricallyInvalidCell {
            reason: "Alypy §91 explicitly licenses added бы only with подобаше".into(),
        });
    }
    let id = match auxiliary {
        ModalConditionalAuxiliary::Podobati => "synodal:verb:v06-7572c074fcdb7753",
        ModalConditionalAuxiliary::Dostoyati => "synodal:verb:dostoyati",
        ModalConditionalAuxiliary::Moshchi => "synodal:verb:v06-c8b75d2f425c16e5",
    };
    let mut tokens = vec![PhraseToken {
        role: PhraseRole::Auxiliary,
        forms: Verb::from_id_with(&LexemeId::from(id), inflector)?
            .imperfect(Person::Third, Number::Singular)?,
    }];
    if with_invariant_by {
        tokens.push(PhraseToken {
            role: PhraseRole::Particle,
            forms: invariant_by(inflector)?,
        });
    }
    tokens.push(PhraseToken {
        role: PhraseRole::Infinitive,
        forms: Verb::resolve_with(infinitive_lemma, inflector)?.infinitive()?,
    });
    typed_phrase(
        PhraseFormation::Conditional(ConditionalFormation::ModalImperfect(auxiliary)),
        tokens,
    )
}

/// Realizes the §91 modal conditional with one of §102's passive infinitives,
/// as in `можаше продано быти`. The nested passive phrase retains both its
/// passive evidence and the enclosing conditional trace.
pub fn modal_conditional_passive_infinitive(
    auxiliary: ModalConditionalAuxiliary,
    lemma: &str,
    participle_cell: ParticipleCell,
    with_invariant_by: bool,
    inflector: Inflector,
) -> Result<RealizedPhrase> {
    if with_invariant_by && auxiliary != ModalConditionalAuxiliary::Podobati {
        return Err(Error::HistoricallyInvalidCell {
            reason: "Alypy §91 explicitly licenses added бы only with подобаше".into(),
        });
    }
    let formation = match participle_cell.tense {
        ParticipleTense::Present => PassiveFormation::PresentParticipleInfinitive,
        ParticipleTense::Past => PassiveFormation::PastParticipleInfinitive,
    };
    let complement = analytic_passive_formation(
        lemma,
        participle_cell,
        formation,
        Person::Third,
        participle_cell.agreement.number,
        PhraseOrder::PredicateFirst,
        inflector,
    )?;
    let id = match auxiliary {
        ModalConditionalAuxiliary::Podobati => "synodal:verb:v06-7572c074fcdb7753",
        ModalConditionalAuxiliary::Dostoyati => "synodal:verb:dostoyati",
        ModalConditionalAuxiliary::Moshchi => "synodal:verb:v06-c8b75d2f425c16e5",
    };
    let mut tokens = vec![PhraseToken {
        role: PhraseRole::Auxiliary,
        forms: Verb::from_id_with(&LexemeId::from(id), inflector)?
            .imperfect(Person::Third, Number::Singular)?,
    }];
    if with_invariant_by {
        tokens.push(PhraseToken {
            role: PhraseRole::Particle,
            forms: invariant_by(inflector)?,
        });
    }
    tokens.extend(complement.tokens().iter().cloned());
    typed_phrase(
        PhraseFormation::Conditional(ConditionalFormation::ModalImperfect(auxiliary)),
        tokens,
    )
}

/// Realizes every person/number of `да` plus present or simple future (§92).
pub fn optative(
    lemma: &str,
    system: OptativeFiniteSystem,
    person: Person,
    number: Number,
    inflector: Inflector,
) -> Result<RealizedPhrase> {
    let verb = Verb::resolve_with(lemma, inflector)?;
    let finite = match system {
        OptativeFiniteSystem::Present => verb.present(person, number)?,
        OptativeFiniteSystem::SimpleFuture => verb.future(person, number)?,
    };
    typed_phrase(
        PhraseFormation::Optative(system),
        vec![
            PhraseToken {
                role: PhraseRole::Particle,
                forms: indeclinable("да", PartOfSpeech::Conjunction, inflector)?,
            },
            PhraseToken {
                role: PhraseRole::FiniteVerb,
                forms: finite,
            },
        ],
    )
}

/// Realizes the five `быти` + present-active-participle systems in Alypy
/// §§90 and 163.
pub fn periphrastic_tense(
    lemma: &str,
    participle_cell: ParticipleCell,
    formation: PeriphrasticTenseFormation,
    person: Person,
    number: Number,
    order: PhraseOrder,
    inflector: Inflector,
) -> Result<RealizedPhrase> {
    validate_predicative_participle(
        participle_cell,
        number,
        ParticipleVoice::Active,
        ParticipleTense::Present,
    )?;
    let auxiliary_forms = match formation {
        PeriphrasticTenseFormation::Present => byti(inflector)?.present(person, number)?,
        PeriphrasticTenseFormation::AoristBe => copula_be(person, number, inflector)?,
        PeriphrasticTenseFormation::ImperfectBya => copula_bya(person, number, inflector)?,
        PeriphrasticTenseFormation::Future => byti(inflector)?.future(person, number)?,
        PeriphrasticTenseFormation::Imperative => byti(inflector)?.imperative(person, number)?,
    };
    typed_phrase(
        PhraseFormation::PeriphrasticTense(PeriphrasticFormation::Copular(formation)),
        ordered_pair(
            PhraseToken {
                role: PhraseRole::ActiveParticiple,
                forms: Participle::resolve_with(lemma, inflector)?.form(participle_cell)?,
            },
            PhraseToken {
                role: PhraseRole::Auxiliary,
                forms: auxiliary_forms,
            },
            order,
        ),
    )
}

/// Uses an already evidenced finite semi-auxiliary form while retaining its
/// closed §90 lexical identity in the phrase formation. Caller providers can
/// therefore supply source forms without weakening the construction type.
pub fn semi_auxiliary_periphrasis(
    auxiliary: PeriphrasticSemiAuxiliary,
    auxiliary_cell: GrammarCell,
    lemma: &str,
    participle_cell: ParticipleCell,
    order: PhraseOrder,
    inflector: Inflector,
) -> Result<RealizedPhrase> {
    if !matches!(
        auxiliary_cell,
        GrammarCell::FiniteVerb(_) | GrammarCell::Imperative(_)
    ) {
        return Err(Error::ContradictoryMetadata {
            reason: "a §90 semi-auxiliary must use a finite or imperative cell".into(),
        });
    }
    let summary = inflector.resolve(auxiliary.lemma())?;
    if summary.part_of_speech() != PartOfSpeech::Verb {
        return Err(Error::ContradictoryMetadata {
            reason: format!("{} is not registered as a verb", auxiliary.lemma()),
        });
    }
    let forms = inflector.form_by_id(summary.id(), auxiliary_cell)?;
    semi_auxiliary_periphrasis_from_forms(
        auxiliary,
        forms,
        lemma,
        participle_cell,
        order,
        inflector,
    )
}

pub fn semi_auxiliary_periphrasis_from_forms(
    auxiliary: PeriphrasticSemiAuxiliary,
    auxiliary_forms: FormSet,
    lemma: &str,
    participle_cell: ParticipleCell,
    order: PhraseOrder,
    inflector: Inflector,
) -> Result<RealizedPhrase> {
    let allowed_tense = if auxiliary == PeriphrasticSemiAuxiliary::Prebyvati {
        matches!(
            participle_cell.tense,
            ParticipleTense::Present | ParticipleTense::Past
        )
    } else {
        participle_cell.tense == ParticipleTense::Present
    };
    if !allowed_tense {
        return Err(Error::HistoricallyInvalidCell {
            reason: "only пребывати is source-licensed with a past-active participle".into(),
        });
    }
    validate_predicative_participle(
        participle_cell,
        participle_cell.agreement.number,
        ParticipleVoice::Active,
        participle_cell.tense,
    )?;
    let mut tokens = ordered_pair(
        PhraseToken {
            role: PhraseRole::ActiveParticiple,
            forms: Participle::resolve_with(lemma, inflector)?.form(participle_cell)?,
        },
        PhraseToken {
            role: PhraseRole::Auxiliary,
            forms: auxiliary_forms,
        },
        order,
    );
    if matches!(
        auxiliary,
        PeriphrasticSemiAuxiliary::NePrestavati | PeriphrasticSemiAuxiliary::NeOskudevati
    ) {
        tokens.insert(
            0,
            PhraseToken {
                role: PhraseRole::Particle,
                forms: indeclinable("не", PartOfSpeech::Particle, inflector)?,
            },
        );
    }
    typed_phrase(
        PhraseFormation::PeriphrasticTense(PeriphrasticFormation::SemiAuxiliary(auxiliary)),
        tokens,
    )
}

/// Represents a source-licensed zero copula without inventing a zero-valued
/// word token. The predicate retains its word-level provenance.
pub fn copula_ellipsis(
    predicate: FormSet,
    context: CopulaOmissionContext,
    inflector: Inflector,
) -> Result<RealizedPhrase> {
    let mut tokens = Vec::new();
    if matches!(
        context,
        CopulaOmissionContext::SePresent
            | CopulaOmissionContext::SePastAorist
            | CopulaOmissionContext::SePastImperfect
    ) {
        tokens.push(PhraseToken {
            role: PhraseRole::Particle,
            forms: indeclinable("се", PartOfSpeech::Interjection, inflector)?,
        });
    }
    tokens.push(PhraseToken {
        role: PhraseRole::Complement,
        forms: predicate,
    });
    typed_phrase(PhraseFormation::CopulaEllipsis(context), tokens)
}

/// Realizes the §146 passive circumstantial participle with agreeing past
/// active `быти`.
pub fn composite_passive_adverbial_participle(
    lemma: &str,
    passive_cell: ParticipleCell,
    order: PhraseOrder,
    inflector: Inflector,
) -> Result<RealizedPhrase> {
    validate_predicative_participle(
        passive_cell,
        passive_cell.agreement.number,
        ParticipleVoice::Passive,
        ParticipleTense::Past,
    )?;
    let byti_cell = ParticipleCell {
        tense: ParticipleTense::Past,
        voice: ParticipleVoice::Active,
        agreement: passive_cell.agreement,
    };
    typed_phrase(
        PhraseFormation::CompositeAdverbialParticiple(
            AdverbialParticipleFormation::PastPassiveWithPastActiveByti,
        ),
        ordered_pair(
            PhraseToken {
                role: PhraseRole::PassiveParticiple,
                forms: Participle::resolve_with(lemma, inflector)?.form(passive_cell)?,
            },
            PhraseToken {
                role: PhraseRole::AuxiliaryParticiple,
                forms: Participle::resolve_with("быти", inflector)?.form(byti_cell)?,
            },
            order,
        ),
    )
}

pub fn composite_copular_adverbial_participle(
    predicate: FormSet,
    byti_cell: ParticipleCell,
    formation: AdverbialParticipleFormation,
    order: PhraseOrder,
    inflector: Inflector,
) -> Result<RealizedPhrase> {
    let expected_tense = match formation {
        AdverbialParticipleFormation::PresentCopularNominal => ParticipleTense::Present,
        AdverbialParticipleFormation::PastCopularNominal => ParticipleTense::Past,
        AdverbialParticipleFormation::PastPassiveWithPastActiveByti => {
            return Err(Error::ContradictoryMetadata {
                reason: "use composite_passive_adverbial_participle for a passive predicate".into(),
            });
        }
    };
    validate_predicative_participle(
        byti_cell,
        byti_cell.agreement.number,
        ParticipleVoice::Active,
        expected_tense,
    )?;
    typed_phrase(
        PhraseFormation::CompositeAdverbialParticiple(formation),
        ordered_pair(
            PhraseToken {
                role: PhraseRole::Complement,
                forms: predicate,
            },
            PhraseToken {
                role: PhraseRole::ActiveParticiple,
                forms: Participle::resolve_with("быти", inflector)?.form(byti_cell)?,
            },
            order,
        ),
    )
}
