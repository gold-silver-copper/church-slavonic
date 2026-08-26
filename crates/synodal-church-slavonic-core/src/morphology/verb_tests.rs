use crate::{
    AdjectiveCell, AdjectiveForm, Animacy, Case, Comparison, Error, FiniteTense, FormSource,
    Gender, LParticipleCell, MetadataField, Number, OrthographyProfile, ParticipleCell,
    ParticipleTense, ParticipleVoice, Person, SynodalWord, VerbSystem,
};

use super::*;

use super::test_support::*;

use crate::NounCell;

#[test]
fn present_uses_independent_edge_principal_parts() {
    let lexeme = regular_verb();
    assert_eq!(
        present(
            &lexeme,
            Person::First,
            Number::Singular,
            OrthographyProfile::Expanded
        )
        .expect("first singular")
        .primary_text(),
        "несꙋ"
    );
    assert_eq!(
        present(
            &lexeme,
            Person::Third,
            Number::Plural,
            OrthographyProfile::Expanded
        )
        .expect("third plural")
        .primary_text(),
        "несꙋтъ"
    );
}

#[test]
fn simple_future_is_the_complete_perfective_present_shape() {
    let present_lexeme = regular_verb();
    let mut future_lexeme = present_lexeme.clone();
    future_lexeme.lemma = word("понести");
    future_lexeme.aspect = Aspect::Perfective;

    for number in Number::ALL {
        for person in Person::ALL {
            let present_form = present(
                &present_lexeme,
                person,
                number,
                OrthographyProfile::Expanded,
            )
            .expect("complete present-shaped source paradigm");
            let future_form = future(&future_lexeme, person, number, OrthographyProfile::Expanded)
                .expect("Alypy §84 perfective simple future");
            assert_eq!(
                future_form.texts().collect::<Vec<_>>(),
                present_form.texts().collect::<Vec<_>>()
            );
            assert!(future_form.variants().iter().all(|variant| {
                matches!(
                    &variant.source,
                    FormSource::SynodalNormativeGeneration { rule }
                        if rule.as_str() == "SYN-VERB-FUTURE-PERFECTIVE-ALYPY-84"
                )
            }));
        }
    }

    let mut suppletive = future_lexeme.clone();
    suppletive.lemma = word("възѧти");
    suppletive.present_stem = Some(word("вземл"));
    suppletive.present_first_singular = Some(word("вземлю"));
    suppletive.present_third_plural = Some(word("вземлютъ"));
    suppletive.future_stem = Some(word("возм"));
    suppletive.future_first_singular = Some(word("возмꙋ"));
    suppletive.future_third_plural = Some(word("возмꙋтъ"));
    assert_eq!(
        present(
            &suppletive,
            Person::Second,
            Number::Singular,
            OrthographyProfile::Expanded,
        )
        .expect("independent present series")
        .primary_text(),
        "вземлеши"
    );
    assert_eq!(
        future(
            &suppletive,
            Person::Second,
            Number::Singular,
            OrthographyProfile::Expanded,
        )
        .expect("independent future series")
        .primary_text(),
        "возмеши"
    );

    suppletive.future_third_plural = None;
    assert_eq!(
        suppletive.missing_principal_parts(VerbSystem::Finite(FiniteTense::Future)),
        vec![MetadataField::FutureThirdPlural]
    );

    assert!(matches!(
        future(
            &present_lexeme,
            Person::Third,
            Number::Singular,
            OrthographyProfile::Expanded,
        ),
        Err(Error::EvidenceIncompleteCell {
            field: MetadataField::Aspect,
            ..
        })
    ));

    let mut biaspectual = present_lexeme.clone();
    biaspectual.aspect = Aspect::Biaspectual;
    assert!(matches!(
        future(
            &biaspectual,
            Person::Third,
            Number::Singular,
            OrthographyProfile::Expanded,
        ),
        Err(Error::EvidenceIncompleteCell {
            field: MetadataField::Aspect,
            ..
        })
    ));

    let mut unknown = present_lexeme;
    unknown.aspect = Aspect::Unknown;
    assert_eq!(
        future(
            &unknown,
            Person::Third,
            Number::Singular,
            OrthographyProfile::Expanded,
        ),
        Err(Error::MissingMetadata {
            field: MetadataField::Aspect,
        })
    );
}

#[test]
fn alpy_104_mobile_vowel_l_participle_keeps_two_typed_stems() {
    let mut verb = regular_verb();
    verb.lemma = word("изити");
    verb.l_participle_stem = Some(word("изш"));
    verb.l_participle_masculine_singular_stem = Some(word("изше"));

    let expected = [
        (Gender::Masculine, Number::Singular, "изшелъ"),
        (Gender::Feminine, Number::Singular, "изшла"),
        (Gender::Neuter, Number::Singular, "изшло"),
        (Gender::Masculine, Number::Dual, "изшла"),
        (Gender::Feminine, Number::Dual, "изшли"),
        (Gender::Neuter, Number::Dual, "изшли"),
        (Gender::Masculine, Number::Plural, "изшли"),
        (Gender::Feminine, Number::Plural, "изшли"),
        (Gender::Neuter, Number::Plural, "изшли"),
    ];
    for (gender, number, surface) in expected {
        assert_eq!(
            l_participle(
                &verb,
                LParticipleCell { gender, number },
                OrthographyProfile::Expanded,
            )
            .expect("typed two-stem l-participle")
            .primary_text(),
            surface
        );
    }

    verb.l_participle_masculine_singular_stem = None;
    assert_eq!(
        l_participle(
            &verb,
            LParticipleCell {
                gender: Gender::Masculine,
                number: Number::Singular,
            },
            OrthographyProfile::Expanded,
        )
        .expect("legacy one-stem l-participle")
        .primary_text(),
        "изшлъ"
    );
}

#[test]
fn conjugates_consonant_aorist_from_alypy_86() {
    let lexeme = regular_verb();
    assert_eq!(
        aorist(
            &lexeme,
            Person::First,
            Number::Singular,
            OrthographyProfile::Expanded
        )
        .expect("aorist")
        .primary_text(),
        "несохъ"
    );
    assert_eq!(
        aorist(
            &lexeme,
            Person::Third,
            Number::Singular,
            OrthographyProfile::Expanded
        )
        .expect("aorist")
        .primary_text(),
        "несе"
    );
}

#[test]
fn rejects_perfective_imperfect() {
    let mut lexeme = regular_verb();
    lexeme.aspect = Aspect::Perfective;
    assert!(matches!(
        imperfect(
            &lexeme,
            Person::Third,
            Number::Singular,
            OrthographyProfile::Expanded
        ),
        Err(Error::HistoricallyInvalidCell { .. })
    ));
}

#[test]
fn liturgical_profile_requires_accent_metadata() {
    let lexeme = regular_verb();
    assert!(matches!(
        present(
            &lexeme,
            Person::Second,
            Number::Singular,
            OrthographyProfile::SynodalLiturgical
        ),
        Err(Error::OrthographicMetadataRequired { .. })
    ));
}

#[test]
fn vowel_t_aorist_offers_the_alypy_86_t_variant_in_the_second_and_third_singular() {
    let mut lexeme = regular_verb();
    lexeme.lemma = SynodalWord::parse("клѧти").expect("lemma");
    lexeme.aorist_stem = Some(SynodalWord::parse("клѧ").expect("stem"));
    lexeme.aorist_formation = Some(AoristFormation::VowelStemWithT);
    let third = aorist(
        &lexeme,
        Person::Third,
        Number::Singular,
        OrthographyProfile::Expanded,
    )
    .expect("aorist");
    assert_eq!(third.texts().collect::<Vec<_>>(), vec!["клѧтъ", "клѧ"]);
    let first = aorist(
        &lexeme,
        Person::First,
        Number::Singular,
        OrthographyProfile::Expanded,
    )
    .expect("aorist");
    assert_eq!(first.primary_text(), "клѧхъ");
    let plural = aorist(
        &lexeme,
        Person::Third,
        Number::Plural,
        OrthographyProfile::Expanded,
    )
    .expect("aorist");
    assert_eq!(plural.primary_text(), "клѧша");
}

#[test]
fn verbal_noun_ie_has_the_complete_alypy_27_34_paradigm() {
    let mut verb = regular_verb();
    verb.lemma = word("молити");
    verb.past_passive_participle = Some(ParticiplePrincipalPart {
        short_stem: Some(word("молен")),
        short_formation: None,
        long_stem: Some(word("моленн")),
        class: AdjectiveClass::Hard,
    });
    let expected: &[&[&str]] = &[
        &["моленїе"],
        &["моленїѧ"],
        &["моленїю"],
        &["моленїе"],
        &["моленїемъ"],
        &["моленїи"],
        &["моленїе"],
        &["молєнїи"],
        &["молєнїю"],
        &["моленїема"],
        &["молєнїи"],
        &["моленїема"],
        &["молєнїю"],
        &["молєнїи"],
        &["молєнїѧ"],
        &["моленїй"],
        &["моленїємъ"],
        &["молєнїѧ"],
        &["моленїи", "моленьми", "моленми"],
        &["моленїихъ"],
        &["молєнїѧ"],
    ];

    for animacy in Animacy::ALL {
        for (index, (number, case)) in Number::ALL
            .into_iter()
            .flat_map(|number| Case::ALL.into_iter().map(move |case| (number, case)))
            .enumerate()
        {
            let forms = decline_verbal_noun(
                &verb,
                NounCell {
                    case,
                    number,
                    animacy,
                },
                OrthographyProfile::Expanded,
            )
            .unwrap_or_else(|error| panic!("{animacy:?} {number:?} {case:?}: {error}"));
            assert_eq!(
                forms.texts().collect::<Vec<_>>().as_slice(),
                expected[index],
                "{animacy:?} {number:?} {case:?}"
            );
            assert!(matches!(
                &forms.primary().source,
                FormSource::SynodalNormativeGeneration { rule }
                    if rule.as_str() == "SYN-VERB-VERBAL-NOUN-IE-ALYPY-27"
            ));
            assert_eq!(
                forms.primary().rule_trace.steps()[0].stage,
                "verbal-noun-formation-past-passive-ie"
            );
        }
    }
}

#[test]
fn verbal_noun_keeps_lexical_suffix_choice_explicit() {
    let mut verb = regular_verb();
    verb.lemma = word("молитися");
    verb.past_passive_participle = None;
    assert_eq!(
        decline_verbal_noun(
            &verb,
            NounCell {
                case: Case::Nominative,
                number: Number::Singular,
                animacy: Animacy::Inanimate,
            },
            OrthographyProfile::Expanded,
        ),
        Err(Error::MissingPrincipalPart {
            field: MetadataField::VerbalNounStem,
        })
    );

    verb.verbal_noun = Some(
        VerbalNounPrincipalPart::explicit_lexical(
            NounLexeme::new(
                word("молитва"),
                word("молитв"),
                Gender::Feminine,
                NounDeclension::SecondHard,
            )
            .with_number_inventory(NounNumberInventory::SingularAndPlural),
        )
        .expect("complete lexical deverbal noun"),
    );
    let nominative = decline_verbal_noun(
        &verb,
        NounCell {
            case: Case::Nominative,
            number: Number::Singular,
            animacy: Animacy::Inanimate,
        },
        OrthographyProfile::Expanded,
    )
    .expect("explicit lexical suffix family");
    assert_eq!(nominative.primary_text(), "молитва");
    assert!(matches!(
        &nominative.primary().source,
        FormSource::SynodalNormativeGeneration { rule }
            if rule.as_str() == "SYN-VERB-VERBAL-NOUN-LEXICAL-ALYPY-27"
    ));
    assert!(matches!(
        decline_verbal_noun(
            &verb,
            NounCell {
                case: Case::Nominative,
                number: Number::Dual,
                animacy: Animacy::Inanimate,
            },
            OrthographyProfile::Expanded,
        ),
        Err(Error::HistoricallyInvalidCell { .. })
    ));
}

#[test]
fn productive_verbal_noun_rejects_a_non_participial_platform() {
    assert!(matches!(
        VerbalNounPrincipalPart::past_passive_ie("моли"),
        Err(Error::ContradictoryMetadata { .. })
    ));

    let mut verb = regular_verb();
    verb.verbal_noun = None;
    verb.past_passive_participle = Some(ParticiplePrincipalPart {
        short_stem: Some(word("моли")),
        short_formation: None,
        long_stem: Some(word("моленн")),
        class: AdjectiveClass::Hard,
    });
    assert_eq!(
        verb.missing_principal_parts(VerbSystem::VerbalNoun {
            animacy: Animacy::Inanimate,
        }),
        [MetadataField::VerbalNounStem]
    );
}

#[test]
fn lexical_verbal_noun_requires_an_alypy_27_suffix_family() {
    for (lemma, stem, gender, declension) in [
        (
            "работа",
            "работ",
            Gender::Feminine,
            NounDeclension::SecondHard,
        ),
        (
            "сꙋета",
            "сꙋет",
            Gender::Feminine,
            NounDeclension::SecondHard,
        ),
        (
            "слꙋжба",
            "слꙋжб",
            Gender::Feminine,
            NounDeclension::SecondHard,
        ),
        (
            "падежъ",
            "падеж",
            Gender::Masculine,
            NounDeclension::FirstMixedMasculine,
        ),
        (
            "дань",
            "дан",
            Gender::Feminine,
            NounDeclension::ThirdFeminine,
        ),
        (
            "пѣснь",
            "пѣсн",
            Gender::Feminine,
            NounDeclension::ThirdFeminine,
        ),
        (
            "жизнь",
            "жизн",
            Gender::Feminine,
            NounDeclension::ThirdFeminine,
        ),
        (
            "молитва",
            "молитв",
            Gender::Feminine,
            NounDeclension::SecondHard,
        ),
        (
            "власть",
            "власт",
            Gender::Feminine,
            NounDeclension::ThirdFeminine,
        ),
        (
            "ꙋкоризна",
            "ꙋкоризн",
            Gender::Feminine,
            NounDeclension::SecondHard,
        ),
    ] {
        VerbalNounPrincipalPart::explicit_lexical(NounLexeme::new(
            word(lemma),
            word(stem),
            gender,
            declension,
        ))
        .unwrap_or_else(|error| panic!("{lemma}: {error}"));
    }

    assert!(matches!(
        VerbalNounPrincipalPart::explicit_lexical(NounLexeme::new(
            word("столъ"),
            word("стол"),
            Gender::Masculine,
            NounDeclension::FirstHardMasculine,
        )),
        Err(Error::ContradictoryMetadata { .. })
    ));
}

#[test]
fn accented_verbal_noun_platform_supports_the_liturgical_profile() {
    let mut verb = regular_verb();
    verb.lemma = word("молити");
    verb.verbal_noun =
        Some(VerbalNounPrincipalPart::past_passive_ie("моле́н").expect("accented source platform"));
    let forms = decline_verbal_noun(
        &verb,
        NounCell {
            case: Case::Genitive,
            number: Number::Singular,
            animacy: Animacy::Inanimate,
        },
        OrthographyProfile::SynodalLiturgical,
    )
    .expect("accented productive verbal noun");
    assert_eq!(forms.primary_text(), "моле́нїѧ");
}

#[test]
fn aspect_sensitive_rules_reject_unknown_aspect_as_missing_metadata() {
    let mut verb = regular_verb();
    verb.aspect = Aspect::Unknown;
    assert_eq!(
        imperfect(
            &verb,
            Person::First,
            Number::Singular,
            OrthographyProfile::Expanded,
        ),
        Err(Error::MissingMetadata {
            field: MetadataField::Aspect,
        })
    );
    assert_eq!(
        decline_participle(
            &verb,
            ParticipleCell {
                tense: ParticipleTense::Present,
                voice: ParticipleVoice::Active,
                agreement: AdjectiveCell {
                    case: Case::Nominative,
                    number: Number::Singular,
                    gender: Gender::Masculine,
                    animacy: Animacy::Inanimate,
                    form: AdjectiveForm::Long,
                    comparison: Comparison::Positive,
                },
            },
            OrthographyProfile::Expanded,
        ),
        Err(Error::MissingMetadata {
            field: MetadataField::Aspect,
        })
    );
}
