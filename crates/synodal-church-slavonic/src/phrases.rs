use std::collections::BTreeSet;

use synodal_church_slavonic_core::{
    AdjectiveForm, AdverbialParticipleFormation, AnalyticConstruction, Animacy, Aspect,
    AuthorityRole, Case, Comparison, CompoundAuxiliaryOrder, CompoundFutureAuxiliary,
    ConditionalCopulaOrder, ConditionalFormation, CopulaOmissionContext, EpistemicRole, Error,
    Evidence, EvidenceId, EvidenceKind, FormSet, Gender, GrammarCell, LexemeId,
    ModalConditionalAuxiliary, NegativePronounBase, Number, OptativeFiniteSystem,
    OrthographyProfile, ParticipleCell, ParticipleTense, ParticipleVoice, PassiveAgentGovernment,
    PassiveFormation, PerfectFormation, PeriphrasticFormation, PeriphrasticSemiAuxiliary,
    PeriphrasticTenseFormation, Person, PhraseFormation, PhraseOrder, PhraseRole, PhraseToken,
    PluperfectFormation, PronounCell, PronounCliticProsody, PronounFormSelection,
    PronounPostpositive, RealizedPhrase, Recension, Result, RuleId, SourceId, TraceStep,
    decline_pronoun,
};

use crate::{Inflector, Noun, PartOfSpeech, Participle, Pronoun, Verb};

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

fn passive_compound_tokens(
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

fn analytic_passive_tokens_with_agent(
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

fn compound_future_auxiliary(
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

fn passive_binary_tokens(
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

fn validate_passive_formation(
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

fn validate_predicative_participle(
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

fn byti(inflector: Inflector) -> Result<Verb> {
    Verb::from_id_with(&LexemeId::from("synodal:verb:byti"), inflector)
}

fn copula_be(person: Person, number: Number, inflector: Inflector) -> Result<FormSet> {
    select_evidence(
        byti(inflector)?.imperfect(person, number)?,
        "alypy-81-byti-imperfect-be",
    )
}

fn copula_bya(person: Person, number: Number, inflector: Inflector) -> Result<FormSet> {
    select_evidence(
        byti(inflector)?.imperfect(person, number)?,
        "alypy-81-byti-imperfect-bya",
    )
}

fn invariant_by(inflector: Inflector) -> Result<FormSet> {
    let forms = byti(inflector)?.aorist(Person::Second, Number::Singular)?;
    let variants = forms
        .variants()
        .iter()
        .filter(|variant| variant.expanded == "бы")
        .cloned()
        .collect();
    FormSet::try_from_variants(variants)
}

fn select_evidence(forms: FormSet, evidence_id: &str) -> Result<FormSet> {
    let variants = forms
        .variants()
        .iter()
        .filter(|variant| {
            variant
                .evidence
                .iter()
                .any(|evidence| evidence.id.as_ref() == evidence_id)
        })
        .cloned()
        .collect();
    FormSet::try_from_variants(variants)
}

fn indeclinable(lemma: &str, expected: PartOfSpeech, inflector: Inflector) -> Result<FormSet> {
    let summary = inflector.resolve(lemma)?;
    if summary.part_of_speech() != expected {
        return Err(Error::ContradictoryMetadata {
            reason: format!(
                "analytic token {lemma:?} must be {}, not {}",
                expected.code(),
                summary.part_of_speech().code()
            ),
        });
    }
    inflector.form_by_id(summary.id(), GrammarCell::Indeclinable)
}

fn ordered_pair(
    predicate: PhraseToken,
    auxiliary: PhraseToken,
    order: PhraseOrder,
) -> Vec<PhraseToken> {
    match order {
        PhraseOrder::AuxiliaryFirst => vec![auxiliary, predicate],
        PhraseOrder::PredicateFirst => vec![predicate, auxiliary],
    }
}

fn ordered_compound(
    predicate: PhraseToken,
    participle: PhraseToken,
    finite: PhraseToken,
    order: CompoundAuxiliaryOrder,
) -> Vec<PhraseToken> {
    match order {
        CompoundAuxiliaryOrder::PredicateParticipleFinite => {
            vec![predicate, participle, finite]
        }
        CompoundAuxiliaryOrder::PredicateFiniteParticiple => {
            vec![predicate, finite, participle]
        }
        CompoundAuxiliaryOrder::ParticipleFinitePredicate => {
            vec![participle, finite, predicate]
        }
        CompoundAuxiliaryOrder::FiniteParticiplePredicate => {
            vec![finite, participle, predicate]
        }
    }
}

fn typed_phrase(formation: PhraseFormation, tokens: Vec<PhraseToken>) -> Result<RealizedPhrase> {
    typed_phrase_inner(formation, tokens, None)
}

fn typed_phrase_with_government(
    formation: PhraseFormation,
    tokens: Vec<PhraseToken>,
    government: PassiveAgentGovernment,
) -> Result<RealizedPhrase> {
    typed_phrase_inner(formation, tokens, Some(government))
}

fn typed_phrase_inner(
    formation: PhraseFormation,
    tokens: Vec<PhraseToken>,
    government: Option<PassiveAgentGovernment>,
) -> Result<RealizedPhrase> {
    let (rule, evidence) = analytic_phrase_evidence(formation);
    let tokens = tokens
        .into_iter()
        .map(|mut token| {
            token.forms = append_analytic_trace(token.forms, rule, evidence.clone())?;
            Ok(token)
        })
        .collect::<Result<Vec<_>>>()?;
    RealizedPhrase::new_typed_with_government(formation, tokens, government)
}

fn append_analytic_trace(
    forms: FormSet,
    rule: &'static str,
    evidence: Evidence,
) -> Result<FormSet> {
    let rule = RuleId::from(rule);
    let mut variants = Vec::with_capacity(forms.variants().len());
    for source in forms.variants() {
        let mut variant = source.clone();
        if !variant.evidence.iter().any(|item| item.id == evidence.id) {
            variant.evidence.push(evidence.clone());
        }
        let evidence_ids = variant
            .evidence
            .iter()
            .map(|item| item.id.clone())
            .collect();
        variant.rule_trace.push(TraceStep {
            rule: rule.clone(),
            stage: "compose-typed-analytic-phrase".into(),
            input: variant.expanded.clone(),
            output: variant.printed.clone(),
            source_recension: Some(Recension::SynodalRussian),
            target_recension: Recension::SynodalRussian,
            mapping: None,
            evidence: evidence_ids,
        });
        variants.push(variant);
    }
    FormSet::try_from_variants(variants)
}

fn analytic_phrase_evidence(formation: PhraseFormation) -> (&'static str, Evidence) {
    let (rule, id, source, citation) = match formation {
        PhraseFormation::CompoundFuture(CompoundFutureAuxiliary::Byti) => (
            "SYN-PHRASE-FUTURE-ALYPY-85-PK-13",
            "syn-phrase-future-alypy-85-pk-13",
            "pletneva-kravetsky-church-slavonic-2005",
            "Pletneva–Kravetsky, lesson 13, table 36",
        ),
        PhraseFormation::CompoundFuture(_) => (
            "SYN-PHRASE-FUTURE-ALYPY-85-PK-13",
            "syn-phrase-future-alypy-85-pk-13",
            "alypy-gamanovich-grammar-web-2023",
            "Alypy §85; Pletneva–Kravetsky lesson 13",
        ),
        PhraseFormation::Perfect(_) => (
            "SYN-PHRASE-PERFECT-ALYPY-88",
            "syn-phrase-perfect-alypy-88",
            "alypy-gamanovich-grammar-web-2023",
            "Alypy §88",
        ),
        PhraseFormation::Pluperfect(_) => (
            "SYN-PHRASE-PLUPERFECT-ALYPY-89-168",
            "syn-phrase-pluperfect-alypy-89-168",
            "alypy-gamanovich-grammar-web-2023",
            "Alypy §§89 and 168",
        ),
        PhraseFormation::FutureAnterior => (
            "SYN-PHRASE-FUTURE-ANTERIOR-ALYPY-162",
            "syn-phrase-future-anterior-alypy-162",
            "alypy-gamanovich-grammar-web-2023",
            "Alypy §162",
        ),
        PhraseFormation::Conditional(_) => (
            "SYN-PHRASE-CONDITIONAL-ALYPY-91",
            "syn-phrase-conditional-alypy-91",
            "alypy-gamanovich-grammar-web-2023",
            "Alypy §91",
        ),
        PhraseFormation::Optative(_) => (
            "SYN-PHRASE-OPTATIVE-ALYPY-92",
            "syn-phrase-optative-alypy-92",
            "alypy-gamanovich-grammar-web-2023",
            "Alypy §92",
        ),
        PhraseFormation::AnalyticPassive(_) => (
            "SYN-PHRASE-PASSIVE-ALYPY-101-102",
            "syn-phrase-passive-alypy-101-102",
            "alypy-gamanovich-grammar-web-2023",
            "Alypy §§101–102",
        ),
        PhraseFormation::PeriphrasticTense(_) => (
            "SYN-PHRASE-PERIPHRASTIC-ALYPY-90-163",
            "syn-phrase-periphrastic-alypy-90-163",
            "alypy-gamanovich-grammar-web-2023",
            "Alypy §§90 and 163",
        ),
        PhraseFormation::CopulaEllipsis(_) => (
            "SYN-PHRASE-COPULA-ELLIPSIS-ALYPY-123-124",
            "syn-phrase-copula-ellipsis-alypy-123-124",
            "alypy-gamanovich-grammar-web-2023",
            "Alypy §§123–124",
        ),
        PhraseFormation::CompositeAdverbialParticiple(_) => (
            "SYN-PHRASE-ADVERBIAL-PARTICIPLE-ALYPY-146",
            "syn-phrase-adverbial-participle-alypy-146",
            "alypy-gamanovich-grammar-web-2023",
            "Alypy §146",
        ),
    };
    (
        rule,
        Evidence {
            id: EvidenceId::from(id),
            source: SourceId::from(source),
            source_recension: Recension::SynodalRussian,
            kind: EvidenceKind::NormativeRule,
            authority_roles: vec![AuthorityRole::Grammatical],
            epistemic_role: EpistemicRole::SynodalNormativeAuthority,
            citation: citation.into(),
            note: Some("typed analytic-construction contract".into()),
        },
    )
}

/// Realizes Alypy §48 negative-pronoun interposition as three independently
/// evidenced tokens: `ни + preposition + inflected interrogative base`.
pub fn negative_pronoun_prepositional(
    preposition: &str,
    base: NegativePronounBase,
    postpositive: Option<PronounPostpositive>,
    cell: PronounCell,
) -> Result<RealizedPhrase> {
    negative_pronoun_prepositional_with(preposition, base, postpositive, cell, Inflector::default())
}

pub fn negative_pronoun_prepositional_with(
    preposition: &str,
    base: NegativePronounBase,
    postpositive: Option<PronounPostpositive>,
    cell: PronounCell,
    inflector: Inflector,
) -> Result<RealizedPhrase> {
    let negative_id = match (base, postpositive) {
        (NegativePronounBase::Who, None) => "synodal:pronoun:nikto",
        (NegativePronounBase::Who, Some(PronounPostpositive::Zhe)) => {
            "synodal:pronoun:v06-niktozhe"
        }
        (NegativePronounBase::What, None) => "synodal:pronoun:nichto",
        (NegativePronounBase::What, Some(PronounPostpositive::Zhe)) => {
            "synodal:pronoun:v06-nichtozhe"
        }
        (NegativePronounBase::Kii, None) => "synodal:pronoun:nikii",
        (NegativePronounBase::Kotoryi, None) => "synodal:pronoun:nikotoryi",
        (_, Some(_)) => {
            return Err(Error::ContradictoryMetadata {
                reason:
                    "preposition-interposed negative pronouns license only optional -же on кто/что"
                        .into(),
            });
        }
    };

    let preposition_summary = inflector.resolve(preposition)?;
    if preposition_summary.part_of_speech() != PartOfSpeech::Preposition {
        return Err(Error::ContradictoryMetadata {
            reason: format!(
                "negative-pronoun interposition requires a preposition, not {}",
                preposition_summary.part_of_speech().code()
            ),
        });
    }
    let negative = Pronoun::from_id_with(&LexemeId::from(negative_id), inflector)?;
    let base_forms = separated_negative_base(negative.form(cell)?)?;
    RealizedPhrase::new(
        AnalyticConstruction::NegativePronounPrepositional,
        vec![
            PhraseToken {
                role: PhraseRole::Particle,
                forms: inflector.form_by_id(
                    &LexemeId::from("synodal:particle:negative-ni"),
                    GrammarCell::Indeclinable,
                )?,
            },
            PhraseToken {
                role: PhraseRole::Preposition,
                forms: inflector.form_by_id(preposition_summary.id(), GrammarCell::Indeclinable)?,
            },
            PhraseToken {
                role: PhraseRole::Pronoun,
                forms: base_forms,
            },
        ],
    )
}

/// Joins a previously realized host and one source-licensed short personal or
/// reflexive pronoun as a typed enclitic construction (Alypy §47).
pub fn pronoun_enclitic_after_host(
    host: FormSet,
    pronoun_id: &LexemeId,
    cell: PronounCell,
    prosody: PronounCliticProsody,
) -> Result<RealizedPhrase> {
    pronoun_enclitic_after_host_with(host, pronoun_id, cell, prosody, Inflector::default())
}

pub fn pronoun_enclitic_after_host_with(
    host: FormSet,
    pronoun_id: &LexemeId,
    cell: PronounCell,
    prosody: PronounCliticProsody,
    inflector: Inflector,
) -> Result<RealizedPhrase> {
    let mut lexeme = crate::registry::pronoun_lexeme(pronoun_id)?;
    lexeme.selection = PronounFormSelection::Enclitic;
    let enclitic_expanded = decline_pronoun(&lexeme, cell, OrthographyProfile::Expanded)?
        .variants()
        .iter()
        .map(|variant| variant.expanded.clone())
        .collect::<BTreeSet<_>>();
    let registered = inflector.form_by_id(pronoun_id, GrammarCell::Pronoun(cell))?;
    let variants = registered
        .variants()
        .iter()
        .filter(|variant| enclitic_expanded.contains(&variant.expanded))
        .cloned()
        .collect::<Vec<_>>();
    if variants.is_empty() {
        return Err(Error::ContradictoryMetadata {
            reason: "the registered pronoun cell did not realize its licensed enclitic form".into(),
        });
    }
    let clitic = FormSet::try_from_variants(variants)?;
    let (host, clitic) = match prosody {
        PronounCliticProsody::AfterFinalVowelStress => {
            (stress_final_host_vowel(host)?, unaccent_enclitic(clitic)?)
        }
        PronounCliticProsody::LogicallyStressed => (
            trace_pronoun_clitic(host, "logically-stressed-host")?,
            trace_pronoun_clitic(clitic, "logically-stressed-short-pronoun")?,
        ),
    };
    RealizedPhrase::new(
        AnalyticConstruction::EncliticPronoun,
        vec![
            PhraseToken {
                role: PhraseRole::Host,
                forms: host,
            },
            PhraseToken {
                role: PhraseRole::Pronoun,
                forms: clitic,
            },
        ],
    )
}

/// Realizes the source-listed masculine singular inanimate accusative
/// contractions `на(н)и → нань` and `въ(н)и → вонь` (Alypy §47).
pub fn contracted_third_person_accusative(preposition: &str) -> Result<RealizedPhrase> {
    contracted_third_person_accusative_with(preposition, Inflector::default())
}

pub fn contracted_third_person_accusative_with(
    preposition: &str,
    inflector: Inflector,
) -> Result<RealizedPhrase> {
    let preposition = inflector.resolve(preposition)?;
    if preposition.part_of_speech() != PartOfSpeech::Preposition {
        return Err(Error::ContradictoryMetadata {
            reason: format!(
                "third-person contraction requires a preposition, not {}",
                preposition.part_of_speech().code()
            ),
        });
    }
    let contraction_id = match preposition.lemma() {
        "на" => "synodal:pronoun:v07-2f384f6138f1b4bd",
        "въ" => "synodal:pronoun:v07-61cdc8112ee912fe",
        other => {
            return Err(Error::HistoricallyInvalidCell {
                reason: format!(
                    "Alypy §47 explicitly licenses contracted -нь only after на and въ, not {other}"
                ),
            });
        }
    };
    let forms = append_phrase_trace(
        inflector.form_by_id(&LexemeId::from(contraction_id), GrammarCell::LexicalForm)?,
        "SYN-PRONOUN-THIRD-PREPOSITION-CONTRACTION-ALYPY-47",
        "third-person-preposition-contraction",
    )?;
    RealizedPhrase::new(
        AnalyticConstruction::ThirdPersonPrepositionalContraction,
        vec![PhraseToken {
            role: PhraseRole::FusedPrepositionPronoun,
            forms,
        }],
    )
}

fn stress_final_host_vowel(forms: FormSet) -> Result<FormSet> {
    transform_pronoun_clitic_forms(forms, "final-vowel-stressed-host", |text| {
        let unaccented = strip_stress_marks(text);
        let final_base = unaccented.chars().rev().find(|character| {
            !matches!(
                *character as u32,
                0x0300..=0x036f | 0x0483..=0x0489 | 0x2de0..=0x2dff | 0xfe20..=0xfe2f
            )
        });
        if !final_base.is_some_and(is_synodal_vowel) {
            return Err(Error::ContradictoryMetadata {
                reason: "AfterFinalVowelStress requires a host ending in a vowel".into(),
            });
        }
        Ok(format!("{unaccented}\u{0301}"))
    })
}

fn unaccent_enclitic(forms: FormSet) -> Result<FormSet> {
    transform_pronoun_clitic_forms(forms, "unaccented-enclitic", |text| {
        Ok(strip_stress_marks(text))
    })
}

fn trace_pronoun_clitic(forms: FormSet, stage: &'static str) -> Result<FormSet> {
    transform_pronoun_clitic_forms(forms, stage, |text| Ok(text.to_owned()))
}

fn transform_pronoun_clitic_forms(
    forms: FormSet,
    stage: &'static str,
    transform: impl Fn(&str) -> Result<String>,
) -> Result<FormSet> {
    let rule = RuleId::from("SYN-PRONOUN-ENCLITIC-PROSODY-ALYPY-47");
    let mut variants = Vec::with_capacity(forms.variants().len());
    for source in forms.variants() {
        let mut variant = source.clone();
        let input = variant.printed.clone();
        variant.printed = transform(&variant.printed)?;
        variant.accented = match &variant.accented {
            Some(accented) => Some(transform(accented)?),
            None if stage == "final-vowel-stressed-host" => Some(variant.printed.clone()),
            None => None,
        };
        variant.romanization = None;
        let construction_evidence = pronoun_phrase_evidence(rule.as_ref())?;
        if !variant
            .evidence
            .iter()
            .any(|item| item.id == construction_evidence.id)
        {
            variant.evidence.push(construction_evidence);
        }
        let evidence = variant
            .evidence
            .iter()
            .map(|item| item.id.clone())
            .collect();
        variant.rule_trace.push(TraceStep {
            rule: rule.clone(),
            stage: stage.into(),
            input,
            output: variant.printed.clone(),
            source_recension: Some(Recension::SynodalRussian),
            target_recension: Recension::SynodalRussian,
            mapping: None,
            evidence,
        });
        variants.push(variant);
    }
    FormSet::try_from_variants(variants)
}

fn strip_stress_marks(text: &str) -> String {
    text.chars()
        .filter(|character| !matches!(character, '\u{0300}' | '\u{0301}' | '\u{0311}' | '\u{0484}'))
        .collect()
}

fn is_synodal_vowel(character: char) -> bool {
    matches!(
        character.to_lowercase().next().unwrap_or(character),
        'а' | 'е'
            | 'є'
            | 'ё'
            | 'и'
            | 'і'
            | 'ї'
            | 'о'
            | 'ѡ'
            | 'ꙍ'
            | 'у'
            | 'ꙋ'
            | 'ы'
            | 'э'
            | 'ю'
            | 'я'
            | 'ѧ'
            | 'ѩ'
            | 'ѣ'
    )
}

fn append_phrase_trace(forms: FormSet, rule: &'static str, stage: &'static str) -> Result<FormSet> {
    let rule = RuleId::from(rule);
    let mut variants = Vec::with_capacity(forms.variants().len());
    for source in forms.variants() {
        let mut variant = source.clone();
        let construction_evidence = pronoun_phrase_evidence(rule.as_ref())?;
        if !variant
            .evidence
            .iter()
            .any(|item| item.id == construction_evidence.id)
        {
            variant.evidence.push(construction_evidence);
        }
        let evidence = variant
            .evidence
            .iter()
            .map(|item| item.id.clone())
            .collect();
        variant.rule_trace.push(TraceStep {
            rule: rule.clone(),
            stage: stage.into(),
            input: variant.expanded.clone(),
            output: variant.printed.clone(),
            source_recension: Some(Recension::SynodalRussian),
            target_recension: Recension::SynodalRussian,
            mapping: None,
            evidence,
        });
        variants.push(variant);
    }
    FormSet::try_from_variants(variants)
}

fn pronoun_phrase_evidence(rule: &str) -> Result<Evidence> {
    let (id, citation, note, roles) = match rule {
        "SYN-PRONOUN-ENCLITIC-PROSODY-ALYPY-47" => (
            "alypy-47-pronoun-enclisis",
            "Alypy (Gamanovich), §47 note 2",
            "short-pronoun enclisis and accent",
            vec![AuthorityRole::Grammatical, AuthorityRole::Accentual],
        ),
        "SYN-PRONOUN-THIRD-PREPOSITION-CONTRACTION-ALYPY-47" => (
            "alypy-47-third-person-contraction",
            "Alypy (Gamanovich), §47 note 1",
            "third-person prepositional contraction",
            vec![AuthorityRole::Grammatical, AuthorityRole::Morphological],
        ),
        "SYN-PRONOUN-NEGATIVE-PREPOSITION-ALYPY-48" => (
            "alypy-48-negative-pronoun-interposition",
            "Alypy (Gamanovich), §48",
            "negative-pronoun preposition interposition",
            vec![AuthorityRole::Grammatical, AuthorityRole::Morphological],
        ),
        _ => {
            return Err(Error::ContradictoryMetadata {
                reason: format!("unknown pronoun phrase rule {rule}"),
            });
        }
    };
    Ok(Evidence {
        id: EvidenceId::from(id),
        source: SourceId::from("alypy-gamanovich-grammar-web-2023"),
        source_recension: Recension::SynodalRussian,
        kind: EvidenceKind::NormativeRule,
        authority_roles: roles,
        epistemic_role: EpistemicRole::SynodalNormativeAuthority,
        citation: citation.into(),
        note: Some(note.into()),
    })
}

fn separated_negative_base(forms: FormSet) -> Result<FormSet> {
    let rule = RuleId::from("SYN-PRONOUN-NEGATIVE-PREPOSITION-ALYPY-48");
    let mut variants = Vec::with_capacity(forms.variants().len());
    for source in forms.variants() {
        let mut variant = source.clone();
        let input = variant.printed.clone();
        variant.expanded = strip_negative_prefix(&variant.expanded)?;
        variant.printed = strip_negative_prefix(&variant.printed)?;
        variant.accented = variant
            .accented
            .as_deref()
            .map(strip_negative_prefix)
            .transpose()?;
        let construction_evidence = pronoun_phrase_evidence(rule.as_ref())?;
        if !variant
            .evidence
            .iter()
            .any(|item| item.id == construction_evidence.id)
        {
            variant.evidence.push(construction_evidence);
        }
        let evidence = variant
            .evidence
            .iter()
            .map(|item| item.id.clone())
            .collect();
        variant.rule_trace.push(TraceStep {
            rule: rule.clone(),
            stage: "negative-pronoun-preposition-interposition".into(),
            input,
            output: variant.printed.clone(),
            source_recension: Some(Recension::SynodalRussian),
            target_recension: Recension::SynodalRussian,
            mapping: None,
            evidence,
        });
        variants.push(variant);
    }
    FormSet::try_from_variants(variants)
}

fn strip_negative_prefix(text: &str) -> Result<String> {
    text.strip_prefix("ни")
        .or_else(|| text.strip_prefix("Ни"))
        .map(str::to_owned)
        .ok_or_else(|| Error::ContradictoryMetadata {
            reason: "negative-pronoun interposition requires a surface ни- prefix".into(),
        })
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
    use synodal_church_slavonic_core::{AdjectiveCell, AdjectiveForm, Animacy, Case, Comparison};

    fn predicative_participle(
        tense: ParticipleTense,
        voice: ParticipleVoice,
        number: Number,
        gender: Gender,
    ) -> ParticipleCell {
        ParticipleCell {
            tense,
            voice,
            agreement: AdjectiveCell {
                case: Case::Nominative,
                number,
                gender,
                animacy: Animacy::Inanimate,
                form: AdjectiveForm::Short,
                comparison: Comparison::Positive,
            },
        }
    }

    fn assert_analytic_provenance(phrase: &RealizedPhrase, rule: &str) {
        assert!(phrase.formation().is_some());
        assert!(phrase.tokens().iter().all(|token| {
            token.forms.variants().iter().all(|variant| {
                variant
                    .evidence
                    .iter()
                    .any(|evidence| evidence.id.as_ref().starts_with("syn-phrase-"))
                    && variant
                        .rule_trace
                        .steps()
                        .iter()
                        .any(|step| step.rule.as_ref() == rule)
            })
        }));
    }

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
        assert_eq!(copulas.len(), 2);
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
            1
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

    #[test]
    fn compound_future_covers_every_auxiliary_person_number_and_order() {
        let expected_third_plural = [
            (CompoundFutureAuxiliary::Byti, "бꙋдꙋтъ нести"),
            (CompoundFutureAuxiliary::Imati, "имꙋтъ нести"),
            (CompoundFutureAuxiliary::Khoteti, "хотѧтъ нести"),
            (CompoundFutureAuxiliary::Nachati, "начнꙋтъ нести"),
        ];
        for (auxiliary, expected) in expected_third_plural {
            let phrase = compound_future_with_auxiliary(
                "нести",
                auxiliary,
                Person::Third,
                Number::Plural,
                PhraseOrder::AuxiliaryFirst,
                Inflector::default(),
            )
            .expect("source-union compound future");
            assert_eq!(phrase.primary_text(), expected);
            assert_eq!(
                phrase.formation(),
                Some(PhraseFormation::CompoundFuture(auxiliary))
            );
            assert_analytic_provenance(&phrase, "SYN-PHRASE-FUTURE-ALYPY-85-PK-13");

            for person in Person::ALL {
                for number in Number::ALL {
                    for order in [PhraseOrder::AuxiliaryFirst, PhraseOrder::PredicateFirst] {
                        compound_future_with_auxiliary(
                            "нести",
                            auxiliary,
                            person,
                            number,
                            order,
                            Inflector::default(),
                        )
                        .unwrap_or_else(|error| {
                            panic!("{auxiliary:?} {person:?} {number:?} {order:?}: {error}")
                        });
                    }
                }
            }
        }
        assert!(matches!(
            compound_future_with_auxiliary(
                "дати",
                CompoundFutureAuxiliary::Imati,
                Person::Third,
                Number::Singular,
                PhraseOrder::AuxiliaryFirst,
                Inflector::default(),
            ),
            Err(Error::HistoricallyInvalidCell { .. })
        ));
    }

    #[test]
    fn perfect_pluperfect_and_future_anterior_cover_every_source_subtype() {
        for order in [PhraseOrder::AuxiliaryFirst, PhraseOrder::PredicateFirst] {
            let perfect = perfect_with_formation(
                "писати",
                Person::Third,
                Number::Singular,
                Gender::Masculine,
                PerfectFormation::PresentCopula,
                order,
                Inflector::default(),
            )
            .expect("present-copula perfect");
            assert_analytic_provenance(&perfect, "SYN-PHRASE-PERFECT-ALYPY-88");

            for formation in [
                PluperfectFormation::AoristBe,
                PluperfectFormation::ImperfectBya,
            ] {
                pluperfect_with_formation(
                    "писати",
                    Person::Third,
                    Number::Singular,
                    Gender::Masculine,
                    formation,
                    order,
                    Inflector::default(),
                )
                .expect("binary pluperfect");
            }
        }
        let omitted = perfect_with_formation(
            "писати",
            Person::Third,
            Number::Singular,
            Gender::Masculine,
            PerfectFormation::OmittedThirdSingularCopula,
            PhraseOrder::PredicateFirst,
            Inflector::default(),
        )
        .expect("third-singular copula ellipsis");
        assert_eq!(omitted.primary_text(), "писалъ");
        assert!(
            perfect_with_formation(
                "писати",
                Person::First,
                Number::Singular,
                Gender::Masculine,
                PerfectFormation::OmittedThirdSingularCopula,
                PhraseOrder::PredicateFirst,
                Inflector::default(),
            )
            .is_err()
        );
        shared_copula_perfect(
            "писати",
            "нести",
            Person::Third,
            Number::Singular,
            Gender::Masculine,
            Inflector::default(),
        )
        .expect("shared copula");
        for order in [
            CompoundAuxiliaryOrder::PredicateParticipleFinite,
            CompoundAuxiliaryOrder::PredicateFiniteParticiple,
            CompoundAuxiliaryOrder::ParticipleFinitePredicate,
            CompoundAuxiliaryOrder::FiniteParticiplePredicate,
        ] {
            pluperfect_with_perfect_copula(
                "писати",
                Person::Third,
                Number::Singular,
                Gender::Masculine,
                order,
                Inflector::default(),
            )
            .expect("three-token pluperfect");
        }
        let anterior = future_anterior(
            "писати",
            Person::Third,
            Number::Singular,
            Gender::Masculine,
            PhraseOrder::AuxiliaryFirst,
            Inflector::default(),
        )
        .expect("future anterior");
        assert_eq!(anterior.primary_text(), "аще бꙋдетъ писалъ");
        assert_analytic_provenance(&anterior, "SYN-PHRASE-FUTURE-ANTERIOR-ALYPY-162");
    }

    #[test]
    fn conditional_and_optative_cover_every_source_subtype() {
        for formation in [
            ConditionalFormation::PersonalAorist,
            ConditionalFormation::InvariantBy,
        ] {
            for order in [PhraseOrder::AuxiliaryFirst, PhraseOrder::PredicateFirst] {
                conditional_with_formation(
                    "писати",
                    Person::First,
                    Number::Singular,
                    Gender::Masculine,
                    formation,
                    order,
                    Inflector::default(),
                )
                .expect("binary conditional");
            }
        }
        for invariant in [false, true] {
            for order in [
                ConditionalCopulaOrder::ConditionalPredicatePresent,
                ConditionalCopulaOrder::ConditionalPresentPredicate,
            ] {
                conditional_with_present_copula(
                    "писати",
                    Person::First,
                    Number::Singular,
                    Gender::Masculine,
                    invariant,
                    order,
                    Inflector::default(),
                )
                .expect("three-token conditional");
            }
        }
        infinitive_conditional("писати", PhraseOrder::PredicateFirst, Inflector::default())
            .expect("infinitive conditional");
        for auxiliary in [
            ModalConditionalAuxiliary::Podobati,
            ModalConditionalAuxiliary::Dostoyati,
            ModalConditionalAuxiliary::Moshchi,
        ] {
            modal_conditional_infinitive(auxiliary, "писати", false, Inflector::default())
                .unwrap_or_else(|error| panic!("{auxiliary:?}: {error}"));
        }
        modal_conditional_infinitive(
            ModalConditionalAuxiliary::Podobati,
            "писати",
            true,
            Inflector::default(),
        )
        .expect("подобаше with optional бы");
        assert!(
            modal_conditional_infinitive(
                ModalConditionalAuxiliary::Moshchi,
                "писати",
                true,
                Inflector::default(),
            )
            .is_err()
        );
        modal_conditional_passive_infinitive(
            ModalConditionalAuxiliary::Moshchi,
            "нести",
            predicative_participle(
                ParticipleTense::Past,
                ParticipleVoice::Passive,
                Number::Singular,
                Gender::Neuter,
            ),
            false,
            Inflector::default(),
        )
        .expect("можаше plus passive infinitive");

        for system in [
            OptativeFiniteSystem::Present,
            OptativeFiniteSystem::SimpleFuture,
        ] {
            let lemma = if system == OptativeFiniteSystem::Present {
                "нести"
            } else {
                "дати"
            };
            for person in Person::ALL {
                for number in Number::ALL {
                    let phrase = optative(lemma, system, person, number, Inflector::default())
                        .unwrap_or_else(|error| {
                            panic!("{system:?} {person:?} {number:?}: {error}")
                        });
                    assert_eq!(phrase.tokens()[0].forms.primary_text(), "да");
                }
            }
        }
    }

    #[test]
    fn periphrastic_ellipsis_and_composite_participles_are_closed_and_checked() {
        let active_present = predicative_participle(
            ParticipleTense::Present,
            ParticipleVoice::Active,
            Number::Singular,
            Gender::Masculine,
        );
        for formation in PeriphrasticTenseFormation::ALL {
            let person = if formation == PeriphrasticTenseFormation::Imperative {
                Person::Second
            } else {
                Person::Third
            };
            let phrase = periphrastic_tense(
                "нести",
                active_present,
                formation,
                person,
                Number::Singular,
                PhraseOrder::PredicateFirst,
                Inflector::default(),
            )
            .unwrap_or_else(|error| panic!("{formation:?}: {error}"));
            assert_analytic_provenance(&phrase, "SYN-PHRASE-PERIPHRASTIC-ALYPY-90-163");
        }
        let supplied_auxiliary = Verb::resolve("быти")
            .expect("copula")
            .present(Person::Third, Number::Singular)
            .expect("finite form");
        for auxiliary in PeriphrasticSemiAuxiliary::ALL {
            let phrase = semi_auxiliary_periphrasis_from_forms(
                auxiliary,
                supplied_auxiliary.clone(),
                "нести",
                active_present,
                PhraseOrder::AuxiliaryFirst,
                Inflector::default(),
            )
            .unwrap_or_else(|error| panic!("{auxiliary:?}: {error}"));
            let expected_len = if matches!(
                auxiliary,
                PeriphrasticSemiAuxiliary::NePrestavati | PeriphrasticSemiAuxiliary::NeOskudevati
            ) {
                3
            } else {
                2
            };
            assert_eq!(phrase.tokens().len(), expected_len);
        }
        semi_auxiliary_periphrasis_from_forms(
            PeriphrasticSemiAuxiliary::Prebyvati,
            supplied_auxiliary.clone(),
            "нести",
            predicative_participle(
                ParticipleTense::Past,
                ParticipleVoice::Active,
                Number::Singular,
                Gender::Masculine,
            ),
            PhraseOrder::AuxiliaryFirst,
            Inflector::default(),
        )
        .expect("пребывати with past-active participle");
        assert!(
            semi_auxiliary_periphrasis_from_forms(
                PeriphrasticSemiAuxiliary::Prestati,
                supplied_auxiliary,
                "нести",
                predicative_participle(
                    ParticipleTense::Past,
                    ParticipleVoice::Active,
                    Number::Singular,
                    Gender::Masculine,
                ),
                PhraseOrder::AuxiliaryFirst,
                Inflector::default(),
            )
            .is_err()
        );

        let predicate = Verb::resolve("нести")
            .expect("verb")
            .present(Person::Third, Number::Singular)
            .expect("predicate form");
        for context in [
            CopulaOmissionContext::PresentNominalPredicate,
            CopulaOmissionContext::SePresent,
            CopulaOmissionContext::SePastAorist,
            CopulaOmissionContext::SePastImperfect,
            CopulaOmissionContext::Imperative,
            CopulaOmissionContext::NarrativePast,
            CopulaOmissionContext::ImpersonalPredicate,
        ] {
            copula_ellipsis(predicate.clone(), context, Inflector::default())
                .unwrap_or_else(|error| panic!("{context:?}: {error}"));
        }
        composite_passive_adverbial_participle(
            "нести",
            predicative_participle(
                ParticipleTense::Past,
                ParticipleVoice::Passive,
                Number::Singular,
                Gender::Masculine,
            ),
            PhraseOrder::PredicateFirst,
            Inflector::default(),
        )
        .expect("past passive plus past-active быти");
        for (formation, tense) in [
            (
                AdverbialParticipleFormation::PresentCopularNominal,
                ParticipleTense::Present,
            ),
            (
                AdverbialParticipleFormation::PastCopularNominal,
                ParticipleTense::Past,
            ),
        ] {
            composite_copular_adverbial_participle(
                predicate.clone(),
                predicative_participle(
                    tense,
                    ParticipleVoice::Active,
                    Number::Singular,
                    Gender::Masculine,
                ),
                formation,
                PhraseOrder::PredicateFirst,
                Inflector::default(),
            )
            .expect("copular composite adverbial participle");
        }
    }

    #[test]
    fn passive_table_covers_all_seventeen_formations_orders_and_agent_government() {
        for formation in PassiveFormation::ALL {
            let tense = match formation {
                PassiveFormation::PresentParticipleInfinitive
                | PassiveFormation::Present
                | PassiveFormation::PresentParticipleFuture
                | PassiveFormation::PresentParticipleAorist
                | PassiveFormation::PresentParticipleAoristBe
                | PassiveFormation::Imperfect
                | PassiveFormation::PresentParticipleConditional
                | PassiveFormation::PresentParticipleImperative => ParticipleTense::Present,
                _ => ParticipleTense::Past,
            };
            let cell = predicative_participle(
                tense,
                ParticipleVoice::Passive,
                Number::Singular,
                Gender::Masculine,
            );
            let person = if matches!(
                formation,
                PassiveFormation::PresentParticipleImperative
                    | PassiveFormation::PastParticipleImperative
            ) {
                Person::Second
            } else {
                Person::Third
            };
            let is_compound = matches!(
                formation,
                PassiveFormation::PluperfectPerfectCopula
                    | PassiveFormation::PresentParticipleConditional
                    | PassiveFormation::PastParticipleConditional
            );
            if is_compound {
                for order in [
                    CompoundAuxiliaryOrder::PredicateParticipleFinite,
                    CompoundAuxiliaryOrder::PredicateFiniteParticiple,
                    CompoundAuxiliaryOrder::ParticipleFinitePredicate,
                    CompoundAuxiliaryOrder::FiniteParticiplePredicate,
                ] {
                    analytic_passive_compound_auxiliary(
                        "нести",
                        cell,
                        formation,
                        person,
                        Number::Singular,
                        order,
                        Inflector::default(),
                    )
                    .unwrap_or_else(|error| panic!("{formation:?} {order:?}: {error}"));
                }
                for government in [
                    PassiveAgentGovernment::Instrumental,
                    PassiveAgentGovernment::OtGenitive,
                ] {
                    let phrase = analytic_passive_compound_with_noun_agent(
                        PassivePredicateSpec {
                            lemma: "нести",
                            participle_cell: cell,
                            formation,
                            person,
                            number: Number::Singular,
                        },
                        CompoundAuxiliaryOrder::PredicateParticipleFinite,
                        PassiveNounAgentSpec {
                            lemma: "рабъ",
                            number: Number::Singular,
                            animacy: Animacy::Animate,
                            government,
                        },
                        Inflector::default(),
                    )
                    .unwrap_or_else(|error| panic!("{formation:?} {government:?}: {error}"));
                    assert_eq!(phrase.agent_government(), Some(government));
                }
            } else {
                for order in [PhraseOrder::AuxiliaryFirst, PhraseOrder::PredicateFirst] {
                    analytic_passive_formation(
                        "нести",
                        cell,
                        formation,
                        person,
                        Number::Singular,
                        order,
                        Inflector::default(),
                    )
                    .unwrap_or_else(|error| panic!("{formation:?} {order:?}: {error}"));
                }
                for government in [
                    PassiveAgentGovernment::Instrumental,
                    PassiveAgentGovernment::OtGenitive,
                ] {
                    let phrase = analytic_passive_with_noun_agent(
                        PassivePredicateSpec {
                            lemma: "нести",
                            participle_cell: cell,
                            formation,
                            person,
                            number: Number::Singular,
                        },
                        PhraseOrder::PredicateFirst,
                        PassiveNounAgentSpec {
                            lemma: "рабъ",
                            number: Number::Singular,
                            animacy: Animacy::Animate,
                            government,
                        },
                        Inflector::default(),
                    )
                    .unwrap_or_else(|error| panic!("{formation:?} {government:?}: {error}"));
                    assert_eq!(phrase.agent_government(), Some(government));
                }
            }
        }

        let wrong_voice = predicative_participle(
            ParticipleTense::Past,
            ParticipleVoice::Active,
            Number::Singular,
            Gender::Masculine,
        );
        assert!(
            analytic_passive_formation(
                "нести",
                wrong_voice,
                PassiveFormation::Perfect,
                Person::Third,
                Number::Singular,
                PhraseOrder::PredicateFirst,
                Inflector::default(),
            )
            .is_err()
        );
        let wrong_form = ParticipleCell {
            agreement: AdjectiveCell {
                form: AdjectiveForm::Long,
                ..predicative_participle(
                    ParticipleTense::Past,
                    ParticipleVoice::Passive,
                    Number::Singular,
                    Gender::Masculine,
                )
                .agreement
            },
            ..predicative_participle(
                ParticipleTense::Past,
                ParticipleVoice::Passive,
                Number::Singular,
                Gender::Masculine,
            )
        };
        assert!(
            analytic_passive_formation(
                "нести",
                wrong_form,
                PassiveFormation::Perfect,
                Person::Third,
                Number::Singular,
                PhraseOrder::PredicateFirst,
                Inflector::default(),
            )
            .is_err()
        );
    }

    #[test]
    fn negative_pronoun_preposition_is_a_typed_three_token_construction() {
        let phrase = negative_pronoun_prepositional(
            "ѡ",
            NegativePronounBase::Who,
            Some(PronounPostpositive::Zhe),
            PronounCell {
                case: Case::Locative,
                number: Number::Singular,
                gender: None,
                person: None,
                animacy: Animacy::Animate,
            },
        )
        .expect("Alypy §48 interposed negative pronoun");
        assert_eq!(phrase.primary_text(), "ни ѡ комъже");
        assert_eq!(
            phrase
                .tokens()
                .iter()
                .map(|token| token.role)
                .collect::<Vec<_>>(),
            [
                PhraseRole::Particle,
                PhraseRole::Preposition,
                PhraseRole::Pronoun
            ]
        );
        assert!(phrase.tokens()[2].forms.rule_traces().all(|trace| {
            trace
                .steps()
                .iter()
                .any(|step| step.rule.as_ref() == "SYN-PRONOUN-NEGATIVE-PREPOSITION-ALYPY-48")
        }));

        let kii = negative_pronoun_prepositional(
            "въ",
            NegativePronounBase::Kii,
            None,
            PronounCell {
                case: Case::Locative,
                number: Number::Singular,
                gender: Some(Gender::Neuter),
                person: None,
                animacy: Animacy::Inanimate,
            },
        )
        .expect("negative кій interposition");
        assert_eq!(kii.primary_text(), "ни въ коемъ");

        let kotoryi = negative_pronoun_prepositional(
            "въ",
            NegativePronounBase::Kotoryi,
            None,
            PronounCell {
                case: Case::Locative,
                number: Number::Singular,
                gender: Some(Gender::Feminine),
                person: None,
                animacy: Animacy::Inanimate,
            },
        )
        .expect("negative который interposition");
        assert_eq!(kotoryi.primary_text(), "ни въ которой");

        for (base, postpositive) in [
            (NegativePronounBase::Who, PronounPostpositive::Zhdo),
            (NegativePronounBase::Kii, PronounPostpositive::Zhe),
        ] {
            assert!(matches!(
                negative_pronoun_prepositional(
                    "въ",
                    base,
                    Some(postpositive),
                    PronounCell {
                        case: Case::Locative,
                        number: Number::Singular,
                        gender: None,
                        person: None,
                        animacy: Animacy::Inanimate,
                    },
                ),
                Err(Error::ContradictoryMetadata { .. })
            ));
        }
    }

    #[test]
    fn short_pronoun_enclisis_selects_the_clitic_and_realizes_source_prosody() {
        let cell = PronounCell {
            case: Case::Accusative,
            number: Number::Singular,
            gender: None,
            person: Some(Person::First),
            animacy: Animacy::Animate,
        };
        let host = Verb::resolve("писати")
            .expect("registered verb")
            .imperative(Person::Second, Number::Singular)
            .expect("imperative host");
        let enclitic = pronoun_enclitic_after_host(
            host,
            &LexemeId::from("synodal:pronoun:az"),
            cell,
            PronounCliticProsody::AfterFinalVowelStress,
        )
        .expect("Alypy §47 final-vowel enclisis");
        assert_eq!(enclitic.primary_text(), "пиши\u{0301} мѧ");
        assert_eq!(
            enclitic.construction(),
            AnalyticConstruction::EncliticPronoun
        );
        assert_eq!(
            enclitic
                .tokens()
                .iter()
                .map(|token| token.role)
                .collect::<Vec<_>>(),
            [PhraseRole::Host, PhraseRole::Pronoun]
        );
        assert_eq!(
            enclitic.tokens()[1].forms.texts().collect::<BTreeSet<_>>(),
            BTreeSet::from(["мѧ"])
        );
        assert!(enclitic.tokens().iter().all(|token| {
            token.forms.rule_traces().all(|trace| {
                trace
                    .steps()
                    .iter()
                    .any(|step| step.rule.as_ref() == "SYN-PRONOUN-ENCLITIC-PROSODY-ALYPY-47")
            })
        }));

        let liturgical = Inflector::builder()
            .orthography(OrthographyProfile::SynodalLiturgical)
            .build();
        let logical_host = Verb::resolve("писати")
            .expect("registered verb")
            .imperative(Person::Second, Number::Singular)
            .expect("imperative host");
        let logical = pronoun_enclitic_after_host_with(
            logical_host,
            &LexemeId::from("synodal:pronoun:az"),
            cell,
            PronounCliticProsody::LogicallyStressed,
            liturgical,
        )
        .expect("logically stressed short pronoun");
        assert!(logical.primary_text().ends_with(" мѧ̀"));
    }

    #[test]
    fn third_person_prepositional_contractions_are_typed_exact_forms() {
        let na = contracted_third_person_accusative("на").expect("нань contraction");
        assert_eq!(na.primary_text(), "нань");
        assert_eq!(
            na.construction(),
            AnalyticConstruction::ThirdPersonPrepositionalContraction
        );
        assert_eq!(na.tokens()[0].role, PhraseRole::FusedPrepositionPronoun);
        assert!(na.tokens()[0].forms.primary().is_attested());

        let liturgical = Inflector::builder()
            .orthography(OrthographyProfile::SynodalLiturgical)
            .build();
        let vo = contracted_third_person_accusative_with("въ", liturgical)
            .expect("accented вонь contraction");
        assert_eq!(vo.primary_text(), "во́нь");
        assert!(vo.tokens()[0].forms.rule_traces().all(|trace| {
            trace.steps().iter().any(|step| {
                step.rule.as_ref() == "SYN-PRONOUN-THIRD-PREPOSITION-CONTRACTION-ALYPY-47"
            })
        }));

        assert!(matches!(
            contracted_third_person_accusative("за"),
            Err(Error::HistoricallyInvalidCell { .. })
        ));
    }
}
