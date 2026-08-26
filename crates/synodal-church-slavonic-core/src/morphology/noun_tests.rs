use crate::{Animacy, Case, Error, Gender, Number, OrthographyProfile};

use super::*;

use super::test_support::*;

use crate::NounCell;

#[test]
fn declines_first_hard_noun_from_alypy_34() {
    let lexeme = NounLexeme {
        lemma: word("рабъ"),
        stem: word("раб"),
        gender: Gender::Masculine,
        declension: NounDeclension::FirstHardMasculine,
        number_inventory: NounNumberInventory::All,
        animacy_inventory: NounAnimacyInventory::All,
    };
    let form = decline_noun(
        &lexeme,
        NounCell {
            case: Case::Dative,
            number: Number::Plural,
            animacy: Animacy::Animate,
        },
        OrthographyProfile::Expanded,
    )
    .expect("supported form");
    assert_eq!(form.primary_text(), "рабомъ");
}

#[test]
fn alpy_37_38_complete_u_stem_background_is_bounded() {
    let son = NounLexeme::new(
        word("сынъ"),
        word("сын"),
        Gender::Masculine,
        NounDeclension::FirstHardMasculineUStem,
    );
    assert_noun_paradigm(
        &son,
        Animacy::Animate,
        &[
            &["сынъ"],
            &["сына", "сынꙋ"],
            &["сынꙋ", "сынови"],
            &["сына", "сынъ"],
            &["сыномъ"],
            &["сынѣ", "сынꙋ"],
            &["сыне"],
            &["сына"],
            &["сынꙋ"],
            &["сынома"],
            &["сына"],
            &["сынома"],
            &["сынꙋ"],
            &["сына"],
            &["сыни", "сынове"],
            &["сыновъ"],
            &["сыномъ", "сыновомъ"],
            &["сыны", "сыновъ"],
            &["сыны", "сынми"],
            &["сынѣхъ", "сыновѣхъ", "сынахъ"],
            &["сыни", "сынове"],
        ],
    );
}

#[test]
fn alpy_34_37_j_ey_and_ie_stem_goldens() {
    let kraj = NounLexeme::new(
        word("край"),
        word("кра"),
        Gender::Masculine,
        NounDeclension::FirstSoftMasculineJ,
    );
    assert_noun_paradigm(
        &kraj,
        Animacy::Animate,
        &[
            &["край"],
            &["краѧ"],
            &["краю"],
            &["краѧ", "край"],
            &["краемъ"],
            &["краи", "краѣ"],
            &["краю"],
            &["краѧ"],
            &["краю"],
            &["краема"],
            &["краѧ"],
            &["краема"],
            &["краю"],
            &["краѧ"],
            &["краи"],
            &["краєвъ"],
            &["краємъ"],
            &["краи", "краєвъ"],
            &["краи"],
            &["краехъ"],
            &["краи"],
        ],
    );

    let priest = NounLexeme::new(
        word("їерей"),
        word("їере"),
        Gender::Masculine,
        NounDeclension::FirstSoftMasculineEy,
    );
    assert_noun_paradigm(
        &priest,
        Animacy::Animate,
        &[
            &["їерей"],
            &["їереа"],
            &["їерею", "їереови"],
            &["їереа", "їерей"],
            &["їереемъ"],
            &["їереи", "їереѣ"],
            &["їерею", "їерее"],
            &["їерєа"],
            &["їерєю"],
            &["їереема", "їереома"],
            &["їерєа"],
            &["їереема", "їереома"],
            &["їерєю"],
            &["їерєа"],
            &["їереє"],
            &["їерєй"],
            &["їереємъ", "їереѡмъ"],
            &["їерєи", "їерєй"],
            &["їерєи"],
            &["їереехъ"],
            &["їереє"],
        ],
    );

    let sign = NounLexeme::new(
        word("знаменїе"),
        word("знаменї"),
        Gender::Neuter,
        NounDeclension::FirstSoftNeuterIe,
    );
    assert_noun_paradigm(
        &sign,
        Animacy::Inanimate,
        &[
            &["знаменїе"],
            &["знаменїѧ"],
            &["знаменїю"],
            &["знаменїе"],
            &["знаменїемъ"],
            &["знаменїи"],
            &["знаменїе"],
            &["знамєнїи"],
            &["знамєнїю"],
            &["знаменїема"],
            &["знамєнїи"],
            &["знаменїема"],
            &["знамєнїю"],
            &["знамєнїи"],
            &["знамєнїѧ"],
            &["знаменїй"],
            &["знаменїємъ"],
            &["знамєнїѧ"],
            &["знаменїи", "знаменьми", "знаменми"],
            &["знаменїихъ"],
            &["знамєнїѧ"],
        ],
    );
}

#[test]
fn rejects_second_declension_with_neuter_gender() {
    let lexeme = NounLexeme {
        lemma: word("жена"),
        stem: word("жен"),
        gender: Gender::Neuter,
        declension: NounDeclension::SecondHard,
        number_inventory: NounNumberInventory::All,
        animacy_inventory: NounAnimacyInventory::All,
    };
    assert!(matches!(
        decline_noun(
            &lexeme,
            NounCell {
                case: Case::Nominative,
                number: Number::Singular,
                animacy: Animacy::Inanimate,
            },
            OrthographyProfile::Expanded,
        ),
        Err(Error::ContradictoryMetadata { .. })
    ));
}

#[test]
fn alpy_39_40_velar_and_mixed_second_declension_goldens() {
    let hand = NounLexeme::new(
        word("рꙋка"),
        word("рꙋк"),
        Gender::Feminine,
        NounDeclension::SecondHardVelar,
    );
    assert_noun_paradigm(
        &hand,
        Animacy::Inanimate,
        &[
            &["рꙋка"],
            &["рꙋки"],
            &["рꙋцѣ"],
            &["рꙋкꙋ"],
            &["рꙋкою"],
            &["рꙋцѣ"],
            &["рꙋко"],
            &["рꙋцѣ"],
            &["рꙋкꙋ"],
            &["рꙋкама"],
            &["рꙋцѣ"],
            &["рꙋкама"],
            &["рꙋкꙋ"],
            &["рꙋцѣ"],
            &["рꙋки"],
            &["рꙋкъ"],
            &["рꙋкамъ"],
            &["рꙋки"],
            &["рꙋками"],
            &["рꙋкахъ"],
            &["рꙋки"],
        ],
    );

    let youth = NounLexeme::new(
        word("юноша"),
        word("юнош"),
        Gender::Masculine,
        NounDeclension::SecondMixed,
    );
    assert_noun_paradigm(
        &youth,
        Animacy::Animate,
        &[
            &["юноша"],
            &["юноши"],
            &["юноши", "юношѣ"],
            &["юношꙋ"],
            &["юношею"],
            &["юноши"],
            &["юноше"],
            &["юнѡши"],
            &["юношꙋ"],
            &["юношама"],
            &["юнѡши"],
            &["юношама"],
            &["юношꙋ"],
            &["юнѡши"],
            &["юноши"],
            &["юношъ"],
            &["юношамъ"],
            &["юношы", "юношъ"],
            &["юношами"],
            &["юношахъ"],
            &["юноши"],
        ],
    );
}

#[test]
fn alpy_32_40_postvocalic_and_gendered_ia_boundaries() {
    let lightning = NounLexeme::new(
        word("молнїѧ"),
        word("молнї"),
        Gender::Feminine,
        NounDeclension::SecondSoftPostvocalicAncientPlural,
    );
    for case in [Case::Nominative, Case::Accusative, Case::Vocative] {
        assert_eq!(
            decline_noun(
                &lightning,
                NounCell {
                    case,
                    number: Number::Plural,
                    animacy: Animacy::Inanimate,
                },
                OrthographyProfile::Expanded,
            )
            .expect("ancient postvocalic plural")
            .primary_text(),
            "молнїѧ"
        );
    }
    assert_eq!(
        decline_noun(
            &lightning,
            NounCell {
                case: Case::Genitive,
                number: Number::Plural,
                animacy: Animacy::Inanimate,
            },
            OrthographyProfile::Expanded,
        )
        .expect("ordinary noncitation plural")
        .primary_text(),
        "молнїй"
    );

    let isaiah = NounLexeme::new(
        word("исаїа"),
        word("исаї"),
        Gender::Masculine,
        NounDeclension::SecondSoftMasculineIa,
    );
    assert_eq!(
        decline_noun(
            &isaiah,
            NounCell {
                case: Case::Instrumental,
                number: Number::Singular,
                animacy: Animacy::Animate,
            },
            OrthographyProfile::Expanded,
        )
        .expect("§40 masculine -їа instrumental")
        .primary_text(),
        "исаїемъ"
    );

    let mary = NounLexeme::new(
        word("маріа"),
        word("марі"),
        Gender::Feminine,
        NounDeclension::SecondSoftFeminineIa,
    );
    assert_noun_paradigm(
        &mary,
        Animacy::Animate,
        &[
            &["маріа"],
            &["маріи"],
            &["маріи"],
            &["марію"],
            &["маріею"],
            &["маріи"],
            &["маріе"],
            &["маріи"],
            &["марію"],
            &["маріѧма"],
            &["маріи"],
            &["маріѧма"],
            &["марію"],
            &["маріи"],
            &["маріи"],
            &["марій"],
            &["маріѧмъ"],
            &["маріи", "марій"],
            &["маріѧми"],
            &["маріѧхъ"],
            &["маріи"],
        ],
    );
}

#[test]
fn animate_accusatives_retain_alypy_35_variants_in_normative_order() {
    let lexeme = NounLexeme {
        lemma: word("рабъ"),
        stem: word("раб"),
        gender: Gender::Masculine,
        declension: NounDeclension::FirstHardMasculine,
        number_inventory: NounNumberInventory::All,
        animacy_inventory: NounAnimacyInventory::All,
    };
    let singular = decline_noun(
        &lexeme,
        NounCell {
            case: Case::Accusative,
            number: Number::Singular,
            animacy: Animacy::Animate,
        },
        OrthographyProfile::Expanded,
    )
    .expect("supported singular");
    assert_eq!(
        singular
            .variants()
            .iter()
            .map(|variant| variant.printed.as_str())
            .collect::<Vec<_>>(),
        ["раба", "рабъ"]
    );

    let plural = decline_noun(
        &lexeme,
        NounCell {
            case: Case::Accusative,
            number: Number::Plural,
            animacy: Animacy::Animate,
        },
        OrthographyProfile::Expanded,
    )
    .expect("supported plural");
    assert_eq!(
        plural
            .variants()
            .iter()
            .map(|variant| variant.printed.as_str())
            .collect::<Vec<_>>(),
        ["рабы", "рабовъ"]
    );
    assert!(
        plural
            .variants()
            .iter()
            .all(|variant| !variant.evidence.is_empty())
    );
}

#[test]
fn alpy_34_complete_mixed_masculine_golden() {
    let lexeme = NounLexeme::new(
        word("мꙋжъ"),
        word("мꙋж"),
        Gender::Masculine,
        NounDeclension::FirstMixedMasculine,
    );
    assert_noun_paradigm(
        &lexeme,
        Animacy::Animate,
        &[
            &["мꙋжъ"],
            &["мꙋжа"],
            &["мꙋжꙋ", "мꙋжеви"],
            &["мꙋжа", "мꙋжъ"],
            &["мꙋжемъ"],
            &["мꙋжи", "мꙋжѣ"],
            &["мꙋжꙋ"],
            &["мꙋжа"],
            &["мꙋжꙋ"],
            &["мꙋжема"],
            &["мꙋжа"],
            &["мꙋжема"],
            &["мꙋжꙋ"],
            &["мꙋжа"],
            &["мꙋжи", "мꙋжїе"],
            &["мꙋжей"],
            &["мꙋжемъ"],
            &["мꙋжы", "мꙋжей"],
            &["мꙋжы", "мꙋжьми", "мꙋжами"],
            &["мꙋжахъ"],
            &["мꙋжи", "мꙋжїе"],
        ],
    );
}

#[test]
fn alpy_34_velar_alternations_cover_g_k_and_h_boundaries() {
    for (lemma, stem, locative, vocative, nominative_plural) in [
        ("богъ", "бог", "бозѣ", "боже", "бози"),
        ("ѻтрокъ", "ѻтрок", "ѻтроцѣ", "ѻтроче", "ѻтроцы"),
        ("дꙋхъ", "дꙋх", "дꙋсѣ", "дꙋше", "дꙋси"),
    ] {
        let lexeme = NounLexeme::new(
            word(lemma),
            word(stem),
            Gender::Masculine,
            NounDeclension::FirstHardVelarMasculine,
        );
        for (case, number, expected) in [
            (Case::Locative, Number::Singular, locative),
            (Case::Vocative, Number::Singular, vocative),
            (Case::Nominative, Number::Plural, nominative_plural),
        ] {
            let forms = decline_noun(
                &lexeme,
                NounCell {
                    case,
                    number,
                    animacy: Animacy::Inanimate,
                },
                OrthographyProfile::Expanded,
            )
            .expect("reviewed velar cell");
            assert_eq!(
                forms.primary_text(),
                expected,
                "{lemma} {number:?} {case:?}"
            );
        }
    }
}

#[test]
fn productive_noun_classes_reject_incompatible_stem_shapes() {
    for lexeme in [
        NounLexeme::new(
            word("рабъ"),
            word("раб"),
            Gender::Masculine,
            NounDeclension::FirstHardVelarMasculine,
        ),
        NounLexeme::new(
            word("сынь"),
            word("сын"),
            Gender::Masculine,
            NounDeclension::FirstHardMasculineUStem,
        ),
        NounLexeme::new(
            word("галїлеанъ"),
            word("галїлеан"),
            Gender::Masculine,
            NounDeclension::FirstHardMasculineInEthnonym,
        ),
        NounLexeme::new(
            word("удъ"),
            word("удес"),
            Gender::Masculine,
            NounDeclension::FirstHardMasculineUdEs,
        ),
        NounLexeme::new(
            word("свидѣтель"),
            word("свидѣт"),
            Gender::Masculine,
            NounDeclension::FirstSoftMasculineAgentTel,
        ),
        NounLexeme::new(
            word("господинъ"),
            word("господин"),
            Gender::Masculine,
            NounDeclension::FirstSoftMasculineLord,
        ),
        NounLexeme::new(
            word("краь"),
            word("кра"),
            Gender::Masculine,
            NounDeclension::FirstSoftMasculineJ,
        ),
        NounLexeme::new(
            word("їерей"),
            word("їер"),
            Gender::Masculine,
            NounDeclension::FirstSoftMasculineEy,
        ),
        NounLexeme::new(
            word("знаменїе"),
            word("знамен"),
            Gender::Neuter,
            NounDeclension::FirstSoftNeuterIe,
        ),
        NounLexeme::new(
            word("море"),
            word("мор"),
            Gender::Neuter,
            NounDeclension::FirstSoftNeuterIshche,
        ),
        NounLexeme::new(
            word("домъ"),
            word("дом"),
            Gender::Masculine,
            NounDeclension::FirstMixedMasculine,
        ),
        NounLexeme::new(
            word("жена"),
            word("жен"),
            Gender::Feminine,
            NounDeclension::SecondHardVelar,
        ),
        NounLexeme::new(
            word("жена"),
            word("жен"),
            Gender::Feminine,
            NounDeclension::SecondMixed,
        ),
        NounLexeme::new(
            word("землѧ"),
            word("земл"),
            Gender::Feminine,
            NounDeclension::SecondSoftPostvocalicAncientPlural,
        ),
        NounLexeme::new(
            word("исаїѧ"),
            word("исаї"),
            Gender::Masculine,
            NounDeclension::SecondSoftMasculineIa,
        ),
        NounLexeme::new(
            word("маріѧ"),
            word("марі"),
            Gender::Feminine,
            NounDeclension::SecondSoftFeminineIa,
        ),
        NounLexeme::new(
            word("имѧ"),
            word("имес"),
            Gender::Neuter,
            NounDeclension::FourthNeuterEn,
        ),
        NounLexeme::new(
            word("небо"),
            word("небен"),
            Gender::Neuter,
            NounDeclension::FourthNeuterEs,
        ),
        NounLexeme::new(
            word("чꙋдо"),
            word("чꙋден"),
            Gender::Neuter,
            NounDeclension::FourthNeuterEsAlternatingFirst,
        ),
        NounLexeme::new(
            word("ѻко"),
            word("очен"),
            Gender::Neuter,
            NounDeclension::FourthNeuterEsPairedDual,
        ),
        NounLexeme::new(
            word("мати"),
            word("матес"),
            Gender::Feminine,
            NounDeclension::FourthFeminineEr,
        ),
        NounLexeme::new(
            word("дщи"),
            word("дщер"),
            Gender::Feminine,
            NounDeclension::FourthFeminineErDaughter,
        ),
        NounLexeme::new(
            word("ѻтроча"),
            word("ѻтрочен"),
            Gender::Neuter,
            NounDeclension::FourthNeuterAt,
        ),
        NounLexeme::new(
            word("свекры"),
            word("свекрер"),
            Gender::Feminine,
            NounDeclension::FourthFeminineOv,
        ),
        NounLexeme::new(
            word("степень"),
            word("степес"),
            Gender::Masculine,
            NounDeclension::FourthMasculineEn,
        ),
        NounLexeme::new(
            word("камень"),
            word("камен"),
            Gender::Masculine,
            NounDeclension::FourthMasculineEn,
        ),
        NounLexeme::new(
            word("степень"),
            word("степен"),
            Gender::Masculine,
            NounDeclension::FourthMasculineEnKamen,
        ),
        NounLexeme::new(
            word("день"),
            word("ден"),
            Gender::Masculine,
            NounDeclension::FourthMasculineEnDay,
        ),
        NounLexeme::new(
            word("адѡнаі"),
            word("адонаи"),
            Gender::Masculine,
            NounDeclension::Indeclinable,
        ),
        NounLexeme::new(
            word("любовь"),
            word("любов"),
            Gender::Feminine,
            NounDeclension::FourthFeminineOv,
        ),
        NounLexeme::new(
            word("любовь"),
            word("любов"),
            Gender::Feminine,
            NounDeclension::FourthFeminineOvSyncopating,
        ),
    ] {
        assert!(matches!(
            validate_noun_lexeme(&lexeme),
            Err(Error::ContradictoryMetadata { .. })
        ));
    }
}

#[test]
fn alpy_41_complete_third_masculine_golden() {
    let lexeme = NounLexeme::new(
        word("пꙋть"),
        word("пꙋт"),
        Gender::Masculine,
        NounDeclension::ThirdMasculine,
    );
    assert_noun_paradigm(
        &lexeme,
        Animacy::Inanimate,
        &[
            &["пꙋть"],
            &["пꙋти"],
            &["пꙋти"],
            &["пꙋть"],
            &["пꙋтемъ"],
            &["пꙋти"],
            &["пꙋть", "пꙋтю"],
            &["пꙋти"],
            &["пꙋтїю"],
            &["пꙋтьма"],
            &["пꙋти"],
            &["пꙋтьма"],
            &["пꙋтїю"],
            &["пꙋти"],
            &["пꙋтїе"],
            &["пꙋтій", "пꙋтей"],
            &["пꙋтємъ"],
            &["пꙋти"],
            &["пꙋтьми"],
            &["пꙋтехъ"],
            &["пꙋтїе"],
        ],
    );
}
