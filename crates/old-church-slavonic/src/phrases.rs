//! Structured Old Church Slavonic superlative and verbal periphrases.

use old_church_slavonic_core::adjective::{AdjectiveLexeme, ComparativeLexeme};
use old_church_slavonic_core::{
    AdjectiveCell, AdjectiveForm, AnalyticConstruction, Case, ConditionalAuxiliary, CopulaSeries,
    FiniteTense, FormSet, FutureInfinitiveAuxiliary, FutureReferenceTense, Gender, InflectionError,
    Number, ParticipleKind, PassiveAuxiliary, Person, PhraseOrder, PhraseRole, PhraseToken,
    PluperfectAuxiliary, RealizedPhrase, RuleId,
};

use crate::{Verb, resolver};

/// Build the usual relative superlative: a declined comparative together with
/// an independently inflected genitive reference. The caller supplies the
/// reference `FormSet`, so its dictionary identity, variants, and evidence stay
/// intact and no noun/adjective/pronoun distinction is guessed here.
pub fn relative_superlative_with(
    comparative: &ComparativeLexeme,
    cell: AdjectiveCell,
    genitive_reference: FormSet,
    order: PhraseOrder,
) -> Result<RealizedPhrase, InflectionError> {
    let head = PhraseToken {
        role: PhraseRole::ComparativeAdjective,
        forms: resolver::comparative_with(comparative, cell)?,
    };
    let dependent = PhraseToken {
        role: PhraseRole::ComparisonReference,
        forms: genitive_reference,
    };
    RealizedPhrase::new(
        AnalyticConstruction::RelativeSuperlative,
        ordered(dependent, head, order),
    )
}

/// Build the source-described absolute superlative with invariant `ѕѣло` and a
/// declined positive adjective. Both attested modifier orders are representable.
pub fn absolute_superlative_adverb_with(
    positive: &AdjectiveLexeme,
    cell: AdjectiveCell,
    order: PhraseOrder,
) -> Result<RealizedPhrase, InflectionError> {
    let head = PhraseToken {
        role: PhraseRole::PositiveAdjective,
        forms: resolver::adjective_with(positive, cell)?,
    };
    let dependent = PhraseToken {
        role: PhraseRole::Adverb,
        forms: resolver::grammar_token(
            "ѕѣло",
            RuleId::PhraseAbsoluteSuperlativeAdverb,
            "supply the invariant absolute-superlative adverb",
        )?,
    };
    RealizedPhrase::new(
        AnalyticConstruction::AbsoluteSuperlativeAdverb,
        ordered(dependent, head, order),
    )
}

/// Build a `да` + present imperative/optative for any person-number cell.
///
/// This is deliberately distinct from the six-cell synthetic imperative. OCS
/// sources use the periphrasis for missing first/third-person commands and also
/// for persons that possess a synthetic imperative when its modal force is
/// appropriate.
pub fn da_imperative(
    lemma: &str,
    person: Person,
    number: Number,
) -> Result<RealizedPhrase, InflectionError> {
    let verb = Verb::resolve(lemma)?;
    RealizedPhrase::new(
        AnalyticConstruction::DaImperative,
        vec![
            PhraseToken {
                role: PhraseRole::Particle,
                forms: resolver::grammar_token(
                    "да",
                    RuleId::PhraseDaImperative,
                    "supply the proclitic imperative/optative particle",
                )?,
            },
            PhraseToken {
                role: PhraseRole::FiniteVerb,
                forms: verb.present(person, number)?,
            },
        ],
    )
}

/// Resolve one source-reviewed copular series. This intentionally distinguishes
/// present `ѥс-` from future `бѫд-` and both from the past and modal
/// series that dictionary headwords often aggregate under `бꙑти`.
pub fn copula(
    series: CopulaSeries,
    person: Person,
    number: Number,
) -> Result<FormSet, InflectionError> {
    resolver::copula(series, person, number)
}

/// Build the OCS perfect from an agreeing l-participle and present `ѥс-`
/// copula.
pub fn perfect(
    lemma: &str,
    person: Person,
    number: Number,
    gender: Gender,
    order: PhraseOrder,
) -> Result<RealizedPhrase, InflectionError> {
    let verb = Verb::resolve(lemma)?;
    paired_phrase(
        AnalyticConstruction::Perfect,
        PhraseToken {
            role: PhraseRole::Auxiliary,
            forms: copula(CopulaSeries::PresentEs, person, number)?,
        },
        PhraseToken {
            role: PhraseRole::LParticiple,
            forms: verb.l_participle(gender, number)?,
        },
        order,
    )
}

/// Build any of the three source-described pluperfect formations: l-participle
/// plus imperfect or aorist `be`, or the three-token l-participle + perfect of
/// `be` construction.
pub fn pluperfect(
    lemma: &str,
    person: Person,
    number: Number,
    gender: Gender,
    auxiliary: PluperfectAuxiliary,
    order: PhraseOrder,
) -> Result<RealizedPhrase, InflectionError> {
    let verb = Verb::resolve(lemma)?;
    let head = PhraseToken {
        role: PhraseRole::LParticiple,
        forms: verb.l_participle(gender, number)?,
    };
    match auxiliary {
        PluperfectAuxiliary::Imperfect | PluperfectAuxiliary::Aorist => {
            let series = if auxiliary == PluperfectAuxiliary::Imperfect {
                CopulaSeries::ImperfectBe
            } else {
                CopulaSeries::AoristBe
            };
            paired_phrase(
                AnalyticConstruction::Pluperfect,
                PhraseToken {
                    role: PhraseRole::Auxiliary,
                    forms: copula(series, person, number)?,
                },
                head,
                order,
            )
        }
        PluperfectAuxiliary::Perfect => {
            let copular_verb = Verb::resolve("бꙑти")?;
            let auxiliary_participle = PhraseToken {
                role: PhraseRole::AuxiliaryParticiple,
                forms: copular_verb.l_participle(gender, number)?,
            };
            let auxiliary = PhraseToken {
                role: PhraseRole::Auxiliary,
                forms: copula(CopulaSeries::PresentEs, person, number)?,
            };
            let tokens = match order {
                PhraseOrder::HeadFirst => vec![head, auxiliary_participle, auxiliary],
                PhraseOrder::DependentFirst => vec![auxiliary_participle, auxiliary, head],
            };
            RealizedPhrase::new(AnalyticConstruction::Pluperfect, tokens)
        }
    }
}

/// Build an infinitival future. Present auxiliaries are licensed for all four
/// source-listed verbs; past-reference futures are restricted to `имѣти` and
/// `хотѣти`, as stated by the reviewed grammar.
pub fn infinitival_future(
    lemma: &str,
    auxiliary: FutureInfinitiveAuxiliary,
    reference_tense: FutureReferenceTense,
    person: Person,
    number: Number,
    order: PhraseOrder,
) -> Result<RealizedPhrase, InflectionError> {
    if reference_tense != FutureReferenceTense::Present
        && !matches!(
            auxiliary,
            FutureInfinitiveAuxiliary::Imeti | FutureInfinitiveAuxiliary::Khoteti
        )
    {
        return Err(InflectionError::InvalidInput {
            reason: format!(
                "{auxiliary:?} is not source-licensed as a past-reference future auxiliary"
            ),
        });
    }
    let verb = Verb::resolve(lemma)?;
    let auxiliary_verb = Verb::resolve(auxiliary.lemma())?;
    let tense = match reference_tense {
        FutureReferenceTense::Present => FiniteTense::Present,
        FutureReferenceTense::Imperfect => FiniteTense::Imperfect,
        FutureReferenceTense::Aorist => FiniteTense::Aorist,
    };
    paired_phrase(
        AnalyticConstruction::FutureInfinitive,
        PhraseToken {
            role: PhraseRole::Auxiliary,
            forms: auxiliary_verb.finite(tense, person, number)?,
        },
        PhraseToken {
            role: PhraseRole::Infinitive,
            forms: verb.infinitive()?,
        },
        order,
    )
}

/// Build the occasional active-participle future with future `бѫд-`.
pub fn participial_future(
    lemma: &str,
    kind: ParticipleKind,
    cell: AdjectiveCell,
    person: Person,
    number: Number,
    order: PhraseOrder,
) -> Result<RealizedPhrase, InflectionError> {
    validate_predicative_participle(kind, cell, number, true)?;
    let participle = Verb::resolve(lemma)?.participle(kind)?;
    paired_phrase(
        AnalyticConstruction::FutureParticiple,
        PhraseToken {
            role: PhraseRole::Auxiliary,
            forms: copula(CopulaSeries::FutureBud, person, number)?,
        },
        PhraseToken {
            role: PhraseRole::ActiveParticiple,
            forms: participle.short(cell.case, cell.number, cell.gender, cell.animacy)?,
        },
        order,
    )
}

/// Build the future perfect from an agreeing l-participle and future `бѫд-`.
pub fn future_perfect(
    lemma: &str,
    person: Person,
    number: Number,
    gender: Gender,
    order: PhraseOrder,
) -> Result<RealizedPhrase, InflectionError> {
    let verb = Verb::resolve(lemma)?;
    paired_phrase(
        AnalyticConstruction::FuturePerfect,
        PhraseToken {
            role: PhraseRole::Auxiliary,
            forms: copula(CopulaSeries::FutureBud, person, number)?,
        },
        PhraseToken {
            role: PhraseRole::LParticiple,
            forms: verb.l_participle(gender, number)?,
        },
        order,
    )
}

/// Build the conditional-optative from an l-participle and either the dedicated
/// conditional series or its source-described aorist replacement.
pub fn conditional_optative(
    lemma: &str,
    person: Person,
    number: Number,
    gender: Gender,
    auxiliary: ConditionalAuxiliary,
    order: PhraseOrder,
) -> Result<RealizedPhrase, InflectionError> {
    conditional_optative_tokens(lemma, person, number, gender, auxiliary, order, false)
}

/// Build the independently described `да`-marked optative construction.
pub fn da_conditional_optative(
    lemma: &str,
    person: Person,
    number: Number,
    gender: Gender,
    auxiliary: ConditionalAuxiliary,
    order: PhraseOrder,
) -> Result<RealizedPhrase, InflectionError> {
    conditional_optative_tokens(lemma, person, number, gender, auxiliary, order, true)
}

/// Represent the source-described ellipsis of the conditional auxiliary while
/// retaining the agreeing l-participle as a structured construction.
pub fn elliptical_conditional_optative(
    lemma: &str,
    number: Number,
    gender: Gender,
) -> Result<RealizedPhrase, InflectionError> {
    let verb = Verb::resolve(lemma)?;
    RealizedPhrase::new(
        AnalyticConstruction::EllipticalConditionalOptative,
        vec![PhraseToken {
            role: PhraseRole::LParticiple,
            forms: verb.l_participle(gender, number)?,
        }],
    )
}

/// Build the source-attested conditional with a passive rather than
/// l-participle predicate.
pub fn conditional_passive(
    lemma: &str,
    kind: ParticipleKind,
    cell: AdjectiveCell,
    person: Person,
    number: Number,
    auxiliary: ConditionalAuxiliary,
    order: PhraseOrder,
) -> Result<RealizedPhrase, InflectionError> {
    validate_predicative_participle(kind, cell, number, false)?;
    let participle = Verb::resolve(lemma)?.participle(kind)?;
    paired_phrase(
        AnalyticConstruction::ConditionalOptativePassive,
        PhraseToken {
            role: PhraseRole::Auxiliary,
            forms: copula(conditional_series(auxiliary), person, number)?,
        },
        PhraseToken {
            role: PhraseRole::PassiveParticiple,
            forms: participle.short(cell.case, cell.number, cell.gender, cell.animacy)?,
        },
        order,
    )
}

/// Build a present, past, future, or modal analytic passive from an agreeing
/// passive participle and the explicitly selected copular series.
pub fn analytic_passive(
    lemma: &str,
    kind: ParticipleKind,
    cell: AdjectiveCell,
    person: Person,
    number: Number,
    auxiliary: PassiveAuxiliary,
    order: PhraseOrder,
) -> Result<RealizedPhrase, InflectionError> {
    validate_predicative_participle(kind, cell, number, false)?;
    let participle = Verb::resolve(lemma)?.participle(kind)?;
    paired_phrase(
        AnalyticConstruction::AnalyticPassive,
        PhraseToken {
            role: PhraseRole::Auxiliary,
            forms: copula(passive_series(auxiliary), person, number)?,
        },
        PhraseToken {
            role: PhraseRole::PassiveParticiple,
            forms: participle.short(cell.case, cell.number, cell.gender, cell.animacy)?,
        },
        order,
    )
}

fn conditional_optative_tokens(
    lemma: &str,
    person: Person,
    number: Number,
    gender: Gender,
    auxiliary: ConditionalAuxiliary,
    order: PhraseOrder,
    with_da: bool,
) -> Result<RealizedPhrase, InflectionError> {
    let verb = Verb::resolve(lemma)?;
    let dependent = PhraseToken {
        role: PhraseRole::Auxiliary,
        forms: copula(conditional_series(auxiliary), person, number)?,
    };
    let head = PhraseToken {
        role: PhraseRole::LParticiple,
        forms: verb.l_participle(gender, number)?,
    };
    let mut tokens = ordered(dependent, head, order);
    let construction = if with_da {
        tokens.insert(
            0,
            PhraseToken {
                role: PhraseRole::Particle,
                forms: resolver::grammar_token(
                    "да",
                    RuleId::PhraseConditionalOptativeDa,
                    "supply the optative particle",
                )?,
            },
        );
        AnalyticConstruction::DaConditionalOptative
    } else {
        AnalyticConstruction::ConditionalOptative
    };
    RealizedPhrase::new(construction, tokens)
}

fn conditional_series(auxiliary: ConditionalAuxiliary) -> CopulaSeries {
    match auxiliary {
        ConditionalAuxiliary::Conditional => CopulaSeries::ConditionalBi,
        ConditionalAuxiliary::AoristReplacement => CopulaSeries::ConditionalAoristBy,
    }
}

fn passive_series(auxiliary: PassiveAuxiliary) -> CopulaSeries {
    match auxiliary {
        PassiveAuxiliary::Present => CopulaSeries::PresentEs,
        PassiveAuxiliary::Imperfect => CopulaSeries::ImperfectBe,
        PassiveAuxiliary::Aorist => CopulaSeries::AoristBe,
        PassiveAuxiliary::Future => CopulaSeries::FutureBud,
        PassiveAuxiliary::Conditional => CopulaSeries::ConditionalBi,
        PassiveAuxiliary::ConditionalAoristReplacement => CopulaSeries::ConditionalAoristBy,
    }
}

fn validate_predicative_participle(
    kind: ParticipleKind,
    cell: AdjectiveCell,
    subject_number: Number,
    active: bool,
) -> Result<(), InflectionError> {
    let valid_kind = if active {
        matches!(
            kind,
            ParticipleKind::PresentActive | ParticipleKind::PastActive
        )
    } else {
        matches!(
            kind,
            ParticipleKind::PresentPassive | ParticipleKind::PastPassive
        )
    };
    if !valid_kind {
        return Err(InflectionError::InvalidInput {
            reason: format!(
                "the requested analytic construction requires an {} participle",
                if active { "active" } else { "passive" }
            ),
        });
    }
    if cell.case != Case::Nominative
        || cell.form != AdjectiveForm::Short
        || cell.number != subject_number
    {
        return Err(InflectionError::InvalidInput {
            reason: "a predicative participle must be short nominative and agree with the subject number"
                .to_string(),
        });
    }
    Ok(())
}

fn paired_phrase(
    construction: AnalyticConstruction,
    dependent: PhraseToken,
    head: PhraseToken,
    order: PhraseOrder,
) -> Result<RealizedPhrase, InflectionError> {
    RealizedPhrase::new(construction, ordered(dependent, head, order))
}

fn ordered(dependent: PhraseToken, head: PhraseToken, order: PhraseOrder) -> Vec<PhraseToken> {
    match order {
        PhraseOrder::DependentFirst => vec![dependent, head],
        PhraseOrder::HeadFirst => vec![head, dependent],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use old_church_slavonic_core::adjective::productive_new_comparative;
    use old_church_slavonic_core::{AdjectiveClass, AdjectiveForm, Animacy, Case, Gender, Number};

    fn nominative_masculine_short() -> AdjectiveCell {
        AdjectiveCell {
            case: Case::Nominative,
            number: Number::Singular,
            gender: Gender::Masculine,
            animacy: Animacy::Inanimate,
            form: AdjectiveForm::Short,
        }
    }

    fn predicative_cell(number: Number, gender: Gender) -> AdjectiveCell {
        AdjectiveCell {
            case: Case::Nominative,
            number,
            gender,
            animacy: Animacy::Inanimate,
            form: AdjectiveForm::Short,
        }
    }

    #[test]
    fn superlative_strategies_keep_component_provenance_and_order() {
        let positive = AdjectiveLexeme {
            lemma: "свѧтъ".to_string(),
            class: AdjectiveClass::Hard,
        };
        let absolute = absolute_superlative_adverb_with(
            &positive,
            nominative_masculine_short(),
            PhraseOrder::HeadFirst,
        )
        .expect("absolute superlative");
        assert_eq!(absolute.primary_text(), "свѧтъ ѕѣло");
        assert_eq!(absolute.tokens().len(), 2);
        assert_eq!(absolute.rule_id(), RuleId::PhraseAbsoluteSuperlativeAdverb);

        let comparative = productive_new_comparative(&positive).expect("new comparative");
        let reference = resolver::grammar_token(
            "вьсѣхъ",
            RuleId::PhraseRelativeSuperlative,
            "supply an explicit genitive comparison reference",
        )
        .expect("reference token");
        let relative = relative_superlative_with(
            &comparative,
            nominative_masculine_short(),
            reference,
            PhraseOrder::DependentFirst,
        )
        .expect("relative superlative");
        assert_eq!(relative.primary_text(), "вьсѣхъ свѧтѣи");
        assert_eq!(relative.tokens()[0].forms.primary_text(), "вьсѣхъ");
        assert_eq!(relative.tokens()[1].role, PhraseRole::ComparativeAdjective);
    }

    #[test]
    fn prefixed_absolute_superlative_remains_one_inflected_word() {
        let positive = AdjectiveLexeme {
            lemma: "свѧтъ".to_string(),
            class: AdjectiveClass::Hard,
        };
        let form = resolver::pre_superlative_with(&positive, nominative_masculine_short())
            .expect("prefixed superlative");
        assert_eq!(form.primary_text(), "прѣсвѧтъ");
        assert_eq!(
            form.source(),
            &old_church_slavonic_core::FormSource::ExplicitMetadataRule {
                rule_id: RuleId::AdjectiveSuperlativePre,
            }
        );
        assert_eq!(form.trace().len(), 2);
    }

    #[test]
    fn da_imperative_covers_every_person_number_cell() {
        let phrases = Number::ALL
            .into_iter()
            .flat_map(|number| {
                Person::ALL
                    .into_iter()
                    .map(move |person| da_imperative("благословити", person, number))
            })
            .collect::<Result<Vec<_>, _>>()
            .expect("all analytic imperative cells");
        assert_eq!(phrases.len(), 9);
        assert!(
            phrases
                .iter()
                .all(|phrase| phrase.primary_text().starts_with("да "))
        );
        assert_eq!(phrases[0].primary_text(), "да благословлѭ");
        assert_eq!(phrases[8].primary_text(), "да благословѧтъ");
    }

    #[test]
    fn copular_series_are_complete_and_keep_reconstruction_visible() {
        let forms = CopulaSeries::ALL
            .into_iter()
            .flat_map(|series| {
                Number::ALL.into_iter().flat_map(move |number| {
                    Person::ALL
                        .into_iter()
                        .map(move |person| copula(series, person, number))
                })
            })
            .collect::<Result<Vec<_>, _>>()
            .expect("all six complete copular series");
        assert_eq!(forms.len(), 54);
        assert_eq!(forms[0].primary_text(), "ѥсмь");
        assert_eq!(
            copula(CopulaSeries::FutureBud, Person::Third, Number::Plural)
                .expect("future copula")
                .primary_text(),
            "бѫдѫтъ"
        );
        let reconstructed = copula(CopulaSeries::ConditionalBi, Person::First, Number::Dual)
            .expect("reconstructed conditional dual");
        assert_eq!(reconstructed.primary_text(), "бивѣ");
        assert_eq!(
            reconstructed.source(),
            &old_church_slavonic_core::FormSource::ExplicitMetadataRule {
                rule_id: RuleId::VerbCopulaConditionalBi,
            }
        );
        assert!(
            reconstructed
                .warnings()
                .contains(&old_church_slavonic_core::InflectionWarning::IncludesReconstructedForms)
        );
        assert_eq!(reconstructed.analyses()[0].trace.len(), 1);
    }

    #[test]
    fn perfect_pluperfect_future_perfect_and_conditionals_cover_all_cells() {
        let mut perfect_count = 0;
        let mut pluperfect_count = 0;
        let mut future_perfect_count = 0;
        let mut conditional_count = 0;
        let mut da_conditional_count = 0;
        for number in Number::ALL {
            for person in Person::ALL {
                for gender in Gender::ALL {
                    perfect(
                        "благословити",
                        person,
                        number,
                        gender,
                        PhraseOrder::HeadFirst,
                    )
                    .expect("perfect cell");
                    perfect_count += 1;
                    future_perfect(
                        "благословити",
                        person,
                        number,
                        gender,
                        PhraseOrder::HeadFirst,
                    )
                    .expect("future-perfect cell");
                    future_perfect_count += 1;
                    for auxiliary in [
                        PluperfectAuxiliary::Imperfect,
                        PluperfectAuxiliary::Aorist,
                        PluperfectAuxiliary::Perfect,
                    ] {
                        pluperfect(
                            "благословити",
                            person,
                            number,
                            gender,
                            auxiliary,
                            PhraseOrder::HeadFirst,
                        )
                        .expect("pluperfect cell");
                        pluperfect_count += 1;
                    }
                    for auxiliary in [
                        ConditionalAuxiliary::Conditional,
                        ConditionalAuxiliary::AoristReplacement,
                    ] {
                        conditional_optative(
                            "благословити",
                            person,
                            number,
                            gender,
                            auxiliary,
                            PhraseOrder::DependentFirst,
                        )
                        .expect("conditional cell");
                        conditional_count += 1;
                        da_conditional_optative(
                            "благословити",
                            person,
                            number,
                            gender,
                            auxiliary,
                            PhraseOrder::DependentFirst,
                        )
                        .expect("da conditional cell");
                        da_conditional_count += 1;
                    }
                }
            }
        }
        assert_eq!(perfect_count, 27);
        assert_eq!(pluperfect_count, 81);
        assert_eq!(future_perfect_count, 27);
        assert_eq!(conditional_count, 54);
        assert_eq!(da_conditional_count, 54);
        let elliptical_count = Number::ALL
            .into_iter()
            .flat_map(|number| {
                Gender::ALL.into_iter().map(move |gender| {
                    elliptical_conditional_optative("благословити", number, gender)
                        .expect("elliptical conditional cell")
                })
            })
            .count();
        assert_eq!(elliptical_count, 9);
        assert_eq!(
            perfect(
                "благословити",
                Person::First,
                Number::Singular,
                Gender::Masculine,
                PhraseOrder::HeadFirst,
            )
            .expect("perfect")
            .primary_text(),
            "благословилъ ѥсмь"
        );
        assert_eq!(
            elliptical_conditional_optative("благословити", Number::Plural, Gender::Feminine,)
                .expect("elliptical conditional")
                .tokens()
                .len(),
            1
        );
    }

    #[test]
    fn all_licensed_infinitival_future_auxiliary_cells_are_realized() {
        let mut count = 0;
        for auxiliary in FutureInfinitiveAuxiliary::ALL {
            for number in Number::ALL {
                for person in Person::ALL {
                    infinitival_future(
                        "благословити",
                        auxiliary,
                        FutureReferenceTense::Present,
                        person,
                        number,
                        PhraseOrder::DependentFirst,
                    )
                    .expect("present-reference future");
                    count += 1;
                    if matches!(
                        auxiliary,
                        FutureInfinitiveAuxiliary::Imeti | FutureInfinitiveAuxiliary::Khoteti
                    ) {
                        for tense in [
                            FutureReferenceTense::Imperfect,
                            FutureReferenceTense::Aorist,
                        ] {
                            infinitival_future(
                                "благословити",
                                auxiliary,
                                tense,
                                person,
                                number,
                                PhraseOrder::DependentFirst,
                            )
                            .expect("past-reference future");
                            count += 1;
                        }
                    }
                }
            }
        }
        assert_eq!(count, 72);
        let imeti_future = infinitival_future(
            "благословити",
            FutureInfinitiveAuxiliary::Imeti,
            FutureReferenceTense::Present,
            Person::Third,
            Number::Plural,
            PhraseOrder::DependentFirst,
        )
        .expect("имѣти future");
        assert_eq!(imeti_future.primary_text(), "имѫтъ благословити");
        assert_eq!(
            imeti_future.tokens()[0].forms.source(),
            &old_church_slavonic_core::FormSource::ManualOverride
        );
        assert_eq!(
            imeti_future.tokens()[0].forms.texts().collect::<Vec<_>>(),
            vec!["имѫтъ", "имѣютъ"]
        );
        let khoteti_past_future = infinitival_future(
            "благословити",
            FutureInfinitiveAuxiliary::Khoteti,
            FutureReferenceTense::Aorist,
            Person::First,
            Number::Dual,
            PhraseOrder::DependentFirst,
        )
        .expect("хотѣти future-in-the-past");
        assert_eq!(
            khoteti_past_future.tokens()[0].forms.source(),
            &old_church_slavonic_core::FormSource::ManualOverride
        );
        assert_eq!(
            khoteti_past_future.tokens()[0].forms.primary_text(),
            "хотѣховѣ"
        );
        assert!(matches!(
            infinitival_future(
                "благословити",
                FutureInfinitiveAuxiliary::Vochati,
                FutureReferenceTense::Imperfect,
                Person::First,
                Number::Singular,
                PhraseOrder::DependentFirst,
            ),
            Err(InflectionError::InvalidInput { .. })
        ));
    }

    #[test]
    fn participial_futures_and_passives_enforce_and_cover_agreement() {
        let mut active_count = 0;
        let mut passive_count = 0;
        let mut conditional_passive_count = 0;
        for number in Number::ALL {
            for person in Person::ALL {
                for gender in Gender::ALL {
                    let cell = predicative_cell(number, gender);
                    for kind in [ParticipleKind::PresentActive, ParticipleKind::PastActive] {
                        participial_future(
                            "благословити",
                            kind,
                            cell,
                            person,
                            number,
                            PhraseOrder::HeadFirst,
                        )
                        .expect("active-participle future cell");
                        active_count += 1;
                    }
                    for kind in [ParticipleKind::PresentPassive, ParticipleKind::PastPassive] {
                        for auxiliary in [
                            PassiveAuxiliary::Present,
                            PassiveAuxiliary::Imperfect,
                            PassiveAuxiliary::Aorist,
                            PassiveAuxiliary::Future,
                            PassiveAuxiliary::Conditional,
                            PassiveAuxiliary::ConditionalAoristReplacement,
                        ] {
                            analytic_passive(
                                "благословити",
                                kind,
                                cell,
                                person,
                                number,
                                auxiliary,
                                PhraseOrder::HeadFirst,
                            )
                            .expect("analytic passive cell");
                            passive_count += 1;
                        }
                        for auxiliary in [
                            ConditionalAuxiliary::Conditional,
                            ConditionalAuxiliary::AoristReplacement,
                        ] {
                            conditional_passive(
                                "благословити",
                                kind,
                                cell,
                                person,
                                number,
                                auxiliary,
                                PhraseOrder::HeadFirst,
                            )
                            .expect("conditional passive cell");
                            conditional_passive_count += 1;
                        }
                    }
                }
            }
        }
        assert_eq!(active_count, 54);
        assert_eq!(passive_count, 324);
        assert_eq!(conditional_passive_count, 108);
        assert!(matches!(
            participial_future(
                "благословити",
                ParticipleKind::PastPassive,
                predicative_cell(Number::Singular, Gender::Masculine),
                Person::Third,
                Number::Singular,
                PhraseOrder::HeadFirst,
            ),
            Err(InflectionError::InvalidInput { .. })
        ));
        assert!(matches!(
            analytic_passive(
                "благословити",
                ParticipleKind::PresentActive,
                predicative_cell(Number::Singular, Gender::Masculine),
                Person::Third,
                Number::Singular,
                PassiveAuxiliary::Present,
                PhraseOrder::HeadFirst,
            ),
            Err(InflectionError::InvalidInput { .. })
        ));
        assert!(matches!(
            analytic_passive(
                "благословити",
                ParticipleKind::PastPassive,
                AdjectiveCell {
                    form: AdjectiveForm::Long,
                    ..predicative_cell(Number::Singular, Gender::Masculine)
                },
                Person::Third,
                Number::Singular,
                PassiveAuxiliary::Present,
                PhraseOrder::HeadFirst,
            ),
            Err(InflectionError::InvalidInput { .. })
        ));
        assert!(matches!(
            analytic_passive(
                "благословити",
                ParticipleKind::PastPassive,
                AdjectiveCell {
                    case: Case::Genitive,
                    ..predicative_cell(Number::Singular, Gender::Masculine)
                },
                Person::Third,
                Number::Singular,
                PassiveAuxiliary::Present,
                PhraseOrder::HeadFirst,
            ),
            Err(InflectionError::InvalidInput { .. })
        ));
        assert!(matches!(
            analytic_passive(
                "благословити",
                ParticipleKind::PastPassive,
                predicative_cell(Number::Plural, Gender::Masculine),
                Person::Third,
                Number::Singular,
                PassiveAuxiliary::Present,
                PhraseOrder::HeadFirst,
            ),
            Err(InflectionError::InvalidInput { .. })
        ));
    }
}
