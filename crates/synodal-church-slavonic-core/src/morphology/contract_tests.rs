use crate::{
    AdjectiveCell, AdjectiveForm, Animacy, Case, Comparison, Error, Gender, ImperativeCell,
    LParticipleCell, Number, OrthographyProfile, ParticipleCell, ParticipleTense, ParticipleVoice,
    Person,
};

use super::*;

use super::test_support::*;

#[test]
fn productive_rule_inventory_contracts_are_complete() {
    for declension in NounDeclension::ALL {
        let (lemma, stem, gender) = match declension {
            NounDeclension::FirstHardMasculine => ("рабъ", "раб", Gender::Masculine),
            NounDeclension::FirstHardMasculineUStem => ("сынъ", "сын", Gender::Masculine),
            NounDeclension::FirstHardMasculineInEthnonym => {
                ("галїлеанинъ", "галїлеанин", Gender::Masculine)
            }
            NounDeclension::FirstHardMasculineUdEs => ("ꙋдъ", "ꙋдес", Gender::Masculine),
            NounDeclension::FirstHardVelarMasculine => ("ѻтрокъ", "ѻтрок", Gender::Masculine),
            NounDeclension::FirstMixedMasculine => ("мꙋжъ", "мꙋж", Gender::Masculine),
            NounDeclension::FirstMixedTsMasculine => ("младенецъ", "младенц", Gender::Masculine),
            NounDeclension::FirstHardNeuter => ("слово", "слов", Gender::Neuter),
            NounDeclension::FirstSoftMasculine => ("царь", "цар", Gender::Masculine),
            NounDeclension::FirstSoftMasculineAgentTel => {
                ("свидѣтель", "свидѣтел", Gender::Masculine)
            }
            NounDeclension::FirstSoftMasculineLord => ("господь", "господ", Gender::Masculine),
            NounDeclension::FirstSoftMasculineJ => ("край", "кра", Gender::Masculine),
            NounDeclension::FirstSoftMasculineEy => ("їерей", "їере", Gender::Masculine),
            NounDeclension::FirstSoftNeuter => ("море", "мор", Gender::Neuter),
            NounDeclension::FirstSoftNeuterIshche => ("соборище", "соборищ", Gender::Neuter),
            NounDeclension::FirstSoftNeuterIe => ("знаменїе", "знаменї", Gender::Neuter),
            NounDeclension::SecondHard => ("жена", "жен", Gender::Feminine),
            NounDeclension::SecondHardVelar => ("рꙋка", "рꙋк", Gender::Feminine),
            NounDeclension::SecondSoft => ("землѧ", "земл", Gender::Feminine),
            NounDeclension::SecondSoftPostvocalicAncientPlural => {
                ("молнїѧ", "молнї", Gender::Feminine)
            }
            NounDeclension::SecondSoftMasculineIa => ("исаїа", "исаї", Gender::Masculine),
            NounDeclension::SecondSoftFeminineIa => ("маріа", "марі", Gender::Feminine),
            NounDeclension::SecondMixed => ("юноша", "юнош", Gender::Masculine),
            NounDeclension::ThirdFeminine => ("кость", "кост", Gender::Feminine),
            NounDeclension::ThirdMasculine => ("пꙋть", "пꙋт", Gender::Masculine),
            NounDeclension::FourthNeuterEn => ("имѧ", "имен", Gender::Neuter),
            NounDeclension::FourthNeuterEs => ("небо", "небес", Gender::Neuter),
            NounDeclension::FourthNeuterEsAlternatingFirst => ("чꙋдо", "чꙋдес", Gender::Neuter),
            NounDeclension::FourthNeuterEsPairedDual => ("ѻко", "очес", Gender::Neuter),
            NounDeclension::FourthNeuterAt => ("ѻтроча", "ѻтрочат", Gender::Neuter),
            NounDeclension::FourthFeminineEr => ("мати", "матер", Gender::Feminine),
            NounDeclension::FourthFeminineErDaughter => ("дщерь", "дщер", Gender::Feminine),
            NounDeclension::FourthFeminineOv => ("свекры", "свекров", Gender::Feminine),
            NounDeclension::FourthFeminineOvSyncopating => ("церковь", "церкв", Gender::Feminine),
            NounDeclension::FourthMasculineEn => ("степень", "степен", Gender::Masculine),
            NounDeclension::FourthMasculineEnDay => ("день", "дн", Gender::Masculine),
            NounDeclension::FourthMasculineEnKamen => ("камень", "камен", Gender::Masculine),
            NounDeclension::Indeclinable => ("адѡнаі", "адѡнаі", Gender::Masculine),
        };
        let lexeme = NounLexeme {
            lemma: word(lemma),
            stem: word(stem),
            gender,
            declension,
            number_inventory: NounNumberInventory::All,
            animacy_inventory: NounAnimacyInventory::All,
        };
        for number in Number::ALL {
            for case in Case::ALL {
                for animacy in if case == Case::Accusative {
                    Animacy::ALL.as_slice()
                } else {
                    &[Animacy::Inanimate]
                } {
                    assert_productive_contract(
                        &decline_noun(
                            &lexeme,
                            crate::NounCell {
                                case,
                                number,
                                animacy: *animacy,
                            },
                            OrthographyProfile::Expanded,
                        )
                        .expect("declared noun inventory"),
                    );
                }
            }
        }
    }

    for (class, lemma, stem) in [
        (AdjectiveClass::Hard, "мꙋдръ", "мꙋдр"),
        (AdjectiveClass::Soft, "синь", "син"),
        (AdjectiveClass::Velar, "благъ", "благ"),
    ] {
        let lexeme = AdjectiveLexeme {
            lemma: word(lemma),
            stem: word(stem),
            class,
            short_masculine_stem: None,
            short_masculine_formation: None,
            comparative_stem: Some(word("мꙋдрѣйш")),
            comparison_formation: Some(ComparisonFormation::LaterYat),
        };
        for form in [AdjectiveForm::Short, AdjectiveForm::Long] {
            for comparison in [
                Comparison::Positive,
                Comparison::Comparative,
                Comparison::Superlative,
            ] {
                for number in Number::ALL {
                    for case in Case::ALL {
                        for gender in Gender::ALL {
                            for animacy in if case == Case::Accusative {
                                Animacy::ALL.as_slice()
                            } else {
                                &[Animacy::Inanimate]
                            } {
                                let outcome = decline_adjective(
                                    &lexeme,
                                    AdjectiveCell {
                                        case,
                                        number,
                                        gender,
                                        animacy: *animacy,
                                        form,
                                        comparison,
                                    },
                                    OrthographyProfile::Expanded,
                                );
                                if form == AdjectiveForm::Short
                                    && comparison == Comparison::Superlative
                                    && case != Case::Nominative
                                {
                                    assert!(matches!(
                                        outcome,
                                        Err(Error::HistoricallyInvalidCell { .. })
                                    ));
                                } else {
                                    assert_productive_contract(
                                        &outcome.expect("declared adjective inventory"),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let base = regular_verb();
    for number in Number::ALL {
        for person in Person::ALL {
            assert_productive_contract(
                &present(&base, person, number, OrthographyProfile::Expanded)
                    .expect("declared present inventory"),
            );
            for formation in [AoristFormation::VowelStem, AoristFormation::ConsonantStem] {
                let mut verb = base.clone();
                verb.aorist_formation = Some(formation);
                assert_productive_contract(
                    &aorist(&verb, person, number, OrthographyProfile::Expanded)
                        .expect("declared aorist inventory"),
                );
            }
            for formation in [
                ImperfectFormation::H,
                ImperfectFormation::Yah,
                ImperfectFormation::Ah,
            ] {
                let mut verb = base.clone();
                verb.imperfect_formation = Some(formation);
                assert_productive_contract(
                    &imperfect(&verb, person, number, OrthographyProfile::Expanded)
                        .expect("declared imperfect inventory"),
                );
            }
            for formation in [
                ImperativeFormation::FirstUnpalatalized,
                ImperativeFormation::ISeries,
            ] {
                let mut verb = base.clone();
                verb.imperative_formation = Some(formation);
                let outcome = imperative(
                    &verb,
                    ImperativeCell { person, number },
                    OrthographyProfile::Expanded,
                );
                if (person == Person::First && number == Number::Singular)
                    || (person == Person::Third && number != Number::Singular)
                {
                    assert!(matches!(
                        outcome,
                        Err(Error::HistoricallyInvalidCell { .. })
                    ));
                } else {
                    assert_productive_contract(&outcome.expect("declared imperative inventory"));
                }
            }
        }
        for gender in Gender::ALL {
            assert_productive_contract(
                &l_participle(
                    &base,
                    LParticipleCell { gender, number },
                    OrthographyProfile::Expanded,
                )
                .expect("declared l-participle inventory"),
            );
        }
    }

    for tense in ParticipleTense::ALL {
        for voice in ParticipleVoice::ALL {
            for form in [AdjectiveForm::Short, AdjectiveForm::Long] {
                if voice == ParticipleVoice::Active && form == AdjectiveForm::Short {
                    continue;
                }
                for number in Number::ALL {
                    for case in Case::ALL {
                        for gender in Gender::ALL {
                            for animacy in if case == Case::Accusative {
                                Animacy::ALL.as_slice()
                            } else {
                                &[Animacy::Inanimate]
                            } {
                                assert_productive_contract(
                                    &decline_participle(
                                        &base,
                                        ParticipleCell {
                                            tense,
                                            voice,
                                            agreement: AdjectiveCell {
                                                case,
                                                number,
                                                gender,
                                                animacy: *animacy,
                                                form,
                                                comparison: Comparison::Positive,
                                            },
                                        },
                                        OrthographyProfile::Expanded,
                                    )
                                    .expect("declared ordinary participle inventory"),
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}
