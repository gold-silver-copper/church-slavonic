use std::collections::BTreeSet;

use synodal_church_slavonic_core::{
    AdjectiveCell, AdjectiveForm, AdverbialParticipleFormation, AnalyticConstruction, Animacy,
    Case, Comparison, CompoundAuxiliaryOrder, CompoundFutureAuxiliary, ConditionalCopulaOrder,
    ConditionalFormation, CopulaOmissionContext, EncliticParticle, Error, Gender, GrammarCell,
    LexemeId, ModalConditionalAuxiliary, NegativePronounBase, Number, OptativeFiniteSystem,
    OrthographyProfile, ParticipleCell, ParticipleTense, ParticipleVoice, PassiveAgentGovernment,
    PassiveFormation, PerfectFormation, PeriphrasticSemiAuxiliary, PeriphrasticTenseFormation,
    Person, PhraseFormation, PhraseOrder, PhraseRole, PluperfectFormation, PronounCell,
    PronounCliticProsody, PronounPostpositive, RealizedPhrase,
};

use super::*;
use crate::{Inflector, Verb};

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
    let expanded =
        compound_future("нести", Person::Third, Number::Plural).expect("supported compound future");
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
    assert!(perfect_with_formation(
        "писати",
        Person::First,
        Number::Singular,
        Gender::Masculine,
        PerfectFormation::OmittedThirdSingularCopula,
        PhraseOrder::PredicateFirst,
        Inflector::default(),
    )
    .is_err());
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
    assert!(modal_conditional_infinitive(
        ModalConditionalAuxiliary::Moshchi,
        "писати",
        true,
        Inflector::default(),
    )
    .is_err());
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
                    .unwrap_or_else(|error| panic!("{system:?} {person:?} {number:?}: {error}"));
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
    assert!(semi_auxiliary_periphrasis_from_forms(
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
    .is_err());

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
    assert!(analytic_passive_formation(
        "нести",
        wrong_voice,
        PassiveFormation::Perfect,
        Person::Third,
        Number::Singular,
        PhraseOrder::PredicateFirst,
        Inflector::default(),
    )
    .is_err());
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
    assert!(analytic_passive_formation(
        "нести",
        wrong_form,
        PassiveFormation::Perfect,
        Person::Third,
        Number::Singular,
        PhraseOrder::PredicateFirst,
        Inflector::default(),
    )
    .is_err());
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
fn postpositive_particles_condition_only_a_word_final_grave() {
    assert_eq!(final_grave_to_acute("землѧ̀"), "землѧ́");
    assert_eq!(final_grave_to_acute("ма́ти"), "ма́ти");
    assert_eq!(final_grave_to_acute("гра́дъ"), "гра́дъ");

    let inflector = Inflector::builder()
        .orthography(OrthographyProfile::SynodalLiturgical)
        .build();
    for (particle, expected) in [
        (EncliticParticle::Zhe, "се́ же"),
        (EncliticParticle::Bo, "се́ бо"),
        (EncliticParticle::Li, "се́ ли"),
    ] {
        let host = inflector
            .form_by_id(
                &LexemeId::from("synodal:interjection:se"),
                GrammarCell::Indeclinable,
            )
            .expect("reviewed host");
        let phrase = enclitic_particle_after_host_with(host, particle, inflector)
            .expect("typed postpositive particle");
        assert_eq!(phrase.primary_text(), expected);
        assert_eq!(
            phrase.construction(),
            AnalyticConstruction::EncliticParticle
        );
        assert_eq!(
            phrase
                .tokens()
                .iter()
                .map(|token| token.role)
                .collect::<Vec<_>>(),
            [PhraseRole::Host, PhraseRole::Particle]
        );
        assert!(phrase.tokens().iter().all(|token| {
            token.forms.rule_traces().all(|trace| {
                trace.steps().iter().any(|step| {
                    step.rule.as_ref() == "SYN-ORTH-FINAL-ACUTE-BEFORE-ENCLITIC-ALYPY-3"
                })
            })
        }));
    }
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
        trace
            .steps()
            .iter()
            .any(|step| step.rule.as_ref() == "SYN-PRONOUN-THIRD-PREPOSITION-CONTRACTION-ALYPY-47")
    }));

    assert!(matches!(
        contracted_third_person_accusative("за"),
        Err(Error::HistoricallyInvalidCell { .. })
    ));
}
