use crate::{Animacy, Case, Error, Gender, Number, OrthographyProfile};

use super::*;

use super::test_support::*;

use crate::NounCell;

#[test]
fn alpy_43_complete_extended_stem_goldens() {
    let imya = NounLexeme::new(
        word("имѧ"),
        word("имен"),
        Gender::Neuter,
        NounDeclension::FourthNeuterEn,
    );
    assert_noun_paradigm(
        &imya,
        Animacy::Inanimate,
        &[
            &["имѧ"],
            &["имене"],
            &["имени"],
            &["имѧ"],
            &["именемъ"],
            &["имени"],
            &["имѧ"],
            &["имєни"],
            &["именꙋ"],
            &["именема", "именама"],
            &["имєни"],
            &["именема", "именама"],
            &["именꙋ"],
            &["имєни"],
            &["имена"],
            &["именъ"],
            &["именємъ", "именѡмъ"],
            &["имена"],
            &["имены"],
            &["именѣхъ"],
            &["имена"],
        ],
    );

    let nebo = NounLexeme::new(
        word("небо"),
        word("небес"),
        Gender::Neuter,
        NounDeclension::FourthNeuterEs,
    );
    assert_noun_paradigm(
        &nebo,
        Animacy::Inanimate,
        &[
            &["небо"],
            &["небесе"],
            &["небеси"],
            &["небо"],
            &["небесемъ"],
            &["небеси"],
            &["небо"],
            &["небєси"],
            &["небесꙋ"],
            &["небесема"],
            &["небєси"],
            &["небесема"],
            &["небесꙋ"],
            &["небєси"],
            &["небеса"],
            &["небесъ"],
            &["небесємъ"],
            &["небеса"],
            &["небесы"],
            &["небесѣхъ"],
            &["небеса"],
        ],
    );

    let mati = NounLexeme::new(
        word("мати"),
        word("матер"),
        Gender::Feminine,
        NounDeclension::FourthFeminineEr,
    );
    assert_noun_paradigm(
        &mati,
        Animacy::Animate,
        &[
            &["мати"],
            &["матере"],
            &["матери"],
            &["матерь"],
            &["матерїю"],
            &["матери"],
            &["мати"],
            &["матєри"],
            &["матєрїю"],
            &["матерема"],
            &["матєри"],
            &["матерема"],
            &["матєрїю"],
            &["матєри"],
            &["матєри"],
            &["матерїй", "матерей"],
            &["матеремъ"],
            &["матерей", "матери"],
            &["матерьми"],
            &["матерехъ"],
            &["матєри"],
        ],
    );
}

#[test]
fn alpy_43_44_additional_extended_stem_goldens() {
    let otrocha = NounLexeme::new(
        word("ѻтроча"),
        word("ѻтрочат"),
        Gender::Neuter,
        NounDeclension::FourthNeuterAt,
    );
    assert_noun_paradigm(
        &otrocha,
        Animacy::Inanimate,
        &[
            &["ѻтроча"],
            &["ѻтрочате"],
            &["ѻтрочати"],
            &["ѻтроча"],
            &["ѻтрочатемъ"],
            &["ѻтрочати"],
            &["ѻтроча"],
            &["ѻтрѡчати"],
            &["ѻтрочатꙋ"],
            &["ѻтрочатема", "ѻтрочатама"],
            &["ѻтрѡчати"],
            &["ѻтрочатема", "ѻтрочатама"],
            &["ѻтрочатꙋ"],
            &["ѻтрѡчати"],
            &["ѻтрочата"],
            &["ѻтрочатъ"],
            &["ѻтрочатємъ", "ѻтрочатѡмъ"],
            &["ѻтрочата"],
            &["ѻтрочаты"],
            &["ѻтрочатѣхъ"],
            &["ѻтрочата"],
        ],
    );

    let svekry = NounLexeme::new(
        word("свекры"),
        word("свекров"),
        Gender::Feminine,
        NounDeclension::FourthFeminineOv,
    );
    assert_noun_paradigm(
        &svekry,
        Animacy::Animate,
        &[
            &["свекры"],
            &["свекрове"],
            &["свекрови"],
            &["свекровь"],
            &["свекровїю"],
            &["свекрови"],
            &["свекры"],
            &["свекрѡви"],
            &["свекрѡвїю"],
            &["свекровама"],
            &["свекрѡви"],
            &["свекровама"],
            &["свекрѡвїю"],
            &["свекрѡви"],
            &["свекрѡви"],
            &["свекровей"],
            &["свекровамъ"],
            &["свекровей", "свекрови"],
            &["свекровами"],
            &["свекровахъ"],
            &["свекрѡви"],
        ],
    );

    let kamen = NounLexeme::new(
        word("камень"),
        word("камен"),
        Gender::Masculine,
        NounDeclension::FourthMasculineEnKamen,
    );
    assert_noun_paradigm(
        &kamen,
        Animacy::Inanimate,
        &[
            &["камень"],
            &["камене", "каменѧ"],
            &["камени", "каменю"],
            &["камень"],
            &["каменемъ"],
            &["камени"],
            &["камень"],
            &["камєни"],
            &["каменꙋ"],
            &["каменьма", "каменема"],
            &["камєни"],
            &["каменьма", "каменема"],
            &["каменꙋ"],
            &["камєни"],
            &["камєни", "каменїѧ"],
            &["каменїй"],
            &["каменємъ"],
            &["камєни", "каменїѧ"],
            &["каменьми"],
            &["каменехъ", "каменїѧхъ"],
            &["камєни", "каменїѧ"],
        ],
    );
    assert!(
        decline_noun(
            &kamen,
            NounCell {
                case: Case::Nominative,
                number: Number::Plural,
                animacy: Animacy::Inanimate,
            },
            OrthographyProfile::Expanded,
        )
        .expect("ordinary plural")
        .variants()
        .iter()
        .all(|variant| variant.expanded != "каменїе")
    );
    assert_eq!(
        decline_noun(
            &kamen,
            NounCell {
                case: Case::Accusative,
                number: Number::Plural,
                animacy: Animacy::Animate,
            },
            OrthographyProfile::Expanded,
        )
        .expect("ordered animate variants")
        .variants()
        .iter()
        .map(|variant| variant.expanded.as_str())
        .collect::<Vec<_>>(),
        ["камєни", "каменїѧ", "каменїй"]
    );
}

#[test]
fn alpy_43_44_cell_scoped_irregular_stem_goldens() {
    let eye = NounLexeme::new(
        word("ѻко"),
        word("очес"),
        Gender::Neuter,
        NounDeclension::FourthNeuterEsPairedDual,
    );
    assert_noun_paradigm(
        &eye,
        Animacy::Inanimate,
        &[
            &["ѻко"],
            &["очесе"],
            &["очеси"],
            &["ѻко"],
            &["очесемъ"],
            &["очеси", "ѻцѣ"],
            &["ѻко"],
            &["очи", "оцѣ"],
            &["очїю"],
            &["очима"],
            &["очи", "оцѣ"],
            &["очима"],
            &["очїю"],
            &["очи", "оцѣ"],
            &["очеса"],
            &["очесъ"],
            &["очесємъ"],
            &["очеса"],
            &["очесы"],
            &["очесѣхъ"],
            &["очеса"],
        ],
    );

    let ear = NounLexeme::new(
        word("ꙋхо"),
        word("ушес"),
        Gender::Neuter,
        NounDeclension::FourthNeuterEsPairedDual,
    );
    assert_eq!(
        decline_noun(
            &ear,
            NounCell {
                case: Case::Genitive,
                number: Number::Dual,
                animacy: Animacy::Inanimate,
            },
            OrthographyProfile::Expanded,
        )
        .expect("Alypy §44 paired dual")
        .primary_text(),
        "ушїю"
    );
    assert_eq!(
        decline_noun(
            &ear,
            NounCell {
                case: Case::Nominative,
                number: Number::Plural,
                animacy: Animacy::Inanimate,
            },
            OrthographyProfile::Expanded,
        )
        .expect("Alypy §44 extended plural")
        .primary_text(),
        "ушеса"
    );

    let church = NounLexeme::new(
        word("церковь"),
        word("церкв"),
        Gender::Feminine,
        NounDeclension::FourthFeminineOvSyncopating,
    );
    assert_noun_paradigm(
        &church,
        Animacy::Inanimate,
        &[
            &["церковь"],
            &["церкве"],
            &["церкви"],
            &["церковь"],
            &["церковїю"],
            &["церкви"],
            &["церковь", "церкве"],
            &["цєркви"],
            &["цєрковїю"],
            &["церквама"],
            &["цєркви"],
            &["церквама"],
            &["цєрковїю"],
            &["цєркви"],
            &["цєркви"],
            &["церквей"],
            &["церквамъ"],
            &["церкви"],
            &["церквами"],
            &["церквахъ"],
            &["цєркви"],
        ],
    );

    let love = NounLexeme::new(
        word("любовь"),
        word("любв"),
        Gender::Feminine,
        NounDeclension::FourthFeminineOvSyncopating,
    );
    let cells = [
        (Number::Singular, Case::Genitive, "любве"),
        (Number::Singular, Case::Instrumental, "любовїю"),
        (Number::Dual, Case::Genitive, "любовїю"),
        (Number::Dual, Case::Dative, "любвама"),
        (Number::Plural, Case::Genitive, "любвей"),
    ];
    for (number, case, expected) in cells {
        assert_eq!(
            decline_noun(
                &love,
                NounCell {
                    case,
                    number,
                    animacy: Animacy::Inanimate,
                },
                OrthographyProfile::Expanded,
            )
            .expect("cell-scoped любовь stem")
            .primary_text(),
            expected
        );
    }

    let daughter = NounLexeme::new(
        word("дщерь"),
        word("дщер"),
        Gender::Feminine,
        NounDeclension::FourthFeminineErDaughter,
    );
    for (case, expected) in [
        (Case::Nominative, "дщи"),
        (Case::Accusative, "дщерь"),
        (Case::Genitive, "дщере"),
        (Case::Vocative, "дщи"),
    ] {
        assert_eq!(
            decline_noun(
                &daughter,
                NounCell {
                    case,
                    number: Number::Singular,
                    animacy: Animacy::Animate,
                },
                OrthographyProfile::Expanded,
            )
            .expect("Alypy §44 daughter family")
            .primary_text(),
            expected
        );
    }
}

#[test]
fn plural_only_nouns_retain_absent_numbers_as_typed_failures() {
    let people = NounLexeme::new(
        word("людїе"),
        word("люд"),
        Gender::Masculine,
        NounDeclension::ThirdMasculine,
    )
    .with_number_inventory(NounNumberInventory::PluralOnly);
    assert_eq!(
        decline_noun(
            &people,
            NounCell {
                case: Case::Nominative,
                number: Number::Plural,
                animacy: Animacy::Animate,
            },
            OrthographyProfile::Expanded,
        )
        .expect("licensed plural")
        .primary_text(),
        "людїе"
    );
    assert!(matches!(
        decline_noun(
            &people,
            NounCell {
                case: Case::Nominative,
                number: Number::Singular,
                animacy: Animacy::Animate,
            },
            OrthographyProfile::Expanded,
        ),
        Err(Error::HistoricallyInvalidCell { .. })
    ));
}

#[test]
fn lexical_noun_animacy_rejects_syncretic_incompatible_requests() {
    let prince = NounLexeme::new(
        word("кнѧзь"),
        word("кнѧз"),
        Gender::Masculine,
        NounDeclension::FirstSoftMasculine,
    )
    .with_animacy_inventory(NounAnimacyInventory::AnimateOnly);
    assert!(
        decline_noun(
            &prince,
            NounCell {
                case: Case::Instrumental,
                number: Number::Singular,
                animacy: Animacy::Animate,
            },
            OrthographyProfile::Expanded,
        )
        .is_ok()
    );
    assert!(matches!(
        decline_noun(
            &prince,
            NounCell {
                case: Case::Instrumental,
                number: Number::Singular,
                animacy: Animacy::Inanimate,
            },
            OrthographyProfile::Expanded,
        ),
        Err(Error::HistoricallyInvalidCell { .. })
    ));
}

#[test]
fn alpy_8_33_37_mobile_e_ts_noun_is_complete() {
    let infant = NounLexeme::new(
        word("младенецъ"),
        word("младенц"),
        Gender::Masculine,
        NounDeclension::FirstMixedTsMasculine,
    );
    validate_noun_lexeme(&infant).expect("source-defined -ецъ : -ц- contract");
    assert_noun_paradigm(
        &infant,
        Animacy::Animate,
        &[
            &["младенецъ"],
            &["младенца"],
            &["младенцꙋ", "младенцеви"],
            &["младенца", "младенецъ"],
            &["младенцемъ"],
            &["младенци", "младенцѣ"],
            &["младенче"],
            &["младенца"],
            &["младенцꙋ"],
            &["младенцема"],
            &["младенца"],
            &["младенцема"],
            &["младенцꙋ"],
            &["младенца"],
            &["младенцы"],
            &["младенцєвъ", "младенецъ"],
            &["младенцємъ"],
            &["младенцы", "младенцєвъ"],
            &["младенцы", "младенцьми", "младенцами"],
            &["младенцѣхъ"],
            &["младенцы"],
        ],
    );
}

#[test]
fn alpy_8_33_37_transposed_e_ts_noun_is_complete() {
    let priest = NounLexeme::new(
        word("жрецъ"),
        word("жерц"),
        Gender::Masculine,
        NounDeclension::FirstMixedTsMasculine,
    )
    .with_animacy_inventory(NounAnimacyInventory::AnimateOnly);
    validate_noun_lexeme(&priest).expect("source-defined -рецъ : -ерц- contract");
    assert_noun_paradigm(
        &priest,
        Animacy::Animate,
        &[
            &["жрецъ"],
            &["жерца"],
            &["жерцꙋ", "жерцеви"],
            &["жерца", "жрецъ"],
            &["жерцемъ"],
            &["жерци", "жерцѣ"],
            &["жерче"],
            &["жерца"],
            &["жерцꙋ"],
            &["жерцема"],
            &["жерца"],
            &["жерцема"],
            &["жерцꙋ"],
            &["жерца"],
            &["жерцы"],
            &["жерцєвъ", "жрецъ"],
            &["жерцємъ"],
            &["жерцы", "жерцєвъ"],
            &["жерцы", "жерцьми", "жерцами"],
            &["жерцѣхъ"],
            &["жерцы"],
        ],
    );
    assert!(matches!(
        decline_noun(
            &priest,
            NounCell {
                case: Case::Instrumental,
                number: Number::Singular,
                animacy: Animacy::Inanimate,
            },
            OrthographyProfile::Expanded,
        ),
        Err(Error::HistoricallyInvalidCell { .. })
    ));
}

#[test]
fn alpy_37_44_remaining_productive_noun_profiles_are_bounded() {
    let ethnonym = NounLexeme::new(
        word("галїлеанинъ"),
        word("галїлеанин"),
        Gender::Masculine,
        NounDeclension::FirstHardMasculineInEthnonym,
    );
    assert_noun_paradigm(
        &ethnonym,
        Animacy::Animate,
        &[
            &["галїлеанинъ"],
            &["галїлеанина"],
            &["галїлеанинꙋ", "галїлеанинови"],
            &["галїлеанина", "галїлеанинъ"],
            &["галїлеаниномъ"],
            &["галїлеанинѣ"],
            &["галїлеанине"],
            &["галїлеанина"],
            &["галїлеанинꙋ"],
            &["галїлеанинома"],
            &["галїлеанина"],
            &["галїлеанинома"],
            &["галїлеанинꙋ"],
            &["галїлеанина"],
            &["галїлеане"],
            &["галїлеанъ"],
            &["галїлеаномъ"],
            &["галїлеане", "галїлеанъ"],
            &["галїлеаны"],
            &["галїлеанѣхъ"],
            &["галїлеане"],
        ],
    );

    let ud = NounLexeme::new(
        word("ꙋдъ"),
        word("ꙋдес"),
        Gender::Masculine,
        NounDeclension::FirstHardMasculineUdEs,
    );
    assert_noun_paradigm(
        &ud,
        Animacy::Inanimate,
        &[
            &["ꙋдъ"],
            &["ꙋда", "ꙋдесе"],
            &["ꙋдꙋ", "ꙋдови", "ꙋдеси"],
            &["ꙋдъ"],
            &["ꙋдомъ", "ꙋдесемъ"],
            &["ꙋдѣ", "ꙋдеси"],
            &["ꙋде"],
            &["ꙋда", "ꙋдєси"],
            &["ꙋдꙋ", "ꙋдесꙋ"],
            &["ꙋдома", "ꙋдесема"],
            &["ꙋда", "ꙋдєси"],
            &["ꙋдома", "ꙋдесема"],
            &["ꙋдꙋ", "ꙋдесꙋ"],
            &["ꙋда", "ꙋдєси"],
            &["ꙋди", "ꙋдеса"],
            &["ꙋдовъ", "ꙋдъ", "ꙋдесъ"],
            &["ꙋдомъ", "ꙋдесємъ"],
            &["ꙋды", "ꙋдеса"],
            &["ꙋды", "ꙋдми", "ꙋдами", "ꙋдесы"],
            &["ꙋдѣхъ", "ꙋдахъ", "ꙋдесѣхъ"],
            &["ꙋди", "ꙋдеса"],
        ],
    );

    let lord = NounLexeme::new(
        word("господь"),
        word("господ"),
        Gender::Masculine,
        NounDeclension::FirstSoftMasculineLord,
    );
    assert_noun_paradigm(
        &lord,
        Animacy::Animate,
        &[
            &["господь"],
            &["господа"],
            &["господꙋ", "господеви"],
            &["господа", "господь"],
            &["господомъ"],
            &["господѣ"],
            &["господи"],
            &["господи"],
            &["господїю", "господю"],
            &["господьма"],
            &["господи"],
            &["господьма"],
            &["господїю", "господю"],
            &["господи"],
            &["господїе"],
            &["господій", "господей"],
            &["господємъ"],
            &["господи", "господій"],
            &["господьми"],
            &["господехъ"],
            &["господїе"],
        ],
    );

    let alternating = NounLexeme::new(
        word("чꙋдо"),
        word("чꙋдес"),
        Gender::Neuter,
        NounDeclension::FourthNeuterEsAlternatingFirst,
    );
    assert_noun_paradigm(
        &alternating,
        Animacy::Inanimate,
        &[
            &["чꙋдо"],
            &["чꙋдесе", "чꙋда"],
            &["чꙋдеси", "чꙋдꙋ"],
            &["чꙋдо"],
            &["чꙋдесемъ", "чꙋдомъ"],
            &["чꙋдеси", "чꙋдѣ"],
            &["чꙋдо"],
            &["чꙋдєси", "чꙋда"],
            &["чꙋдесꙋ", "чꙋдꙋ"],
            &["чꙋдесема", "чꙋдома"],
            &["чꙋдєси", "чꙋда"],
            &["чꙋдесема", "чꙋдома"],
            &["чꙋдесꙋ", "чꙋдꙋ"],
            &["чꙋдєси", "чꙋда"],
            &["чꙋдеса", "чꙋда"],
            &["чꙋдесъ", "чꙋдъ"],
            &["чꙋдесємъ", "чꙋдомъ"],
            &["чꙋдеса", "чꙋда"],
            &["чꙋдесы", "чꙋды", "чꙋдами"],
            &["чꙋдесѣхъ", "чꙋдѣхъ", "чꙋдахъ"],
            &["чꙋдеса", "чꙋда"],
        ],
    );

    let day = NounLexeme::new(
        word("день"),
        word("дн"),
        Gender::Masculine,
        NounDeclension::FourthMasculineEnDay,
    );
    assert_noun_paradigm(
        &day,
        Animacy::Inanimate,
        &[
            &["день"],
            &["дне"],
            &["дни", "дневи"],
            &["день"],
            &["днемъ"],
            &["дни"],
            &["день"],
            &["дни"],
            &["днїю", "дню"],
            &["деньма"],
            &["дни"],
            &["деньма"],
            &["днїю", "дню"],
            &["дни"],
            &["дни", "дніе"],
            &["днїй", "дней"],
            &["днємъ"],
            &["дни"],
            &["деньми"],
            &["днехъ"],
            &["дни", "дніе"],
        ],
    );

    for (lexeme, case, number, expected) in [
        (
            NounLexeme::new(
                word("свидѣтель"),
                word("свидѣтел"),
                Gender::Masculine,
                NounDeclension::FirstSoftMasculineAgentTel,
            ),
            Case::Nominative,
            Number::Plural,
            vec!["свидѣтели", "свидѣтеле", "свидѣтелїе"],
        ),
        (
            NounLexeme::new(
                word("соборище"),
                word("соборищ"),
                Gender::Neuter,
                NounDeclension::FirstSoftNeuterIshche,
            ),
            Case::Locative,
            Number::Plural,
            vec!["соборищахъ", "соборищихъ", "соборищехъ"],
        ),
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
        .expect("bounded lexical subclass");
        assert_eq!(
            forms
                .variants()
                .iter()
                .map(|variant| variant.printed.as_str())
                .collect::<Vec<_>>(),
            expected
        );
    }

    let invariant = NounLexeme::new(
        word("адѡнаі"),
        word("адѡнаі"),
        Gender::Masculine,
        NounDeclension::Indeclinable,
    );
    for number in Number::ALL {
        for case in Case::ALL {
            assert_eq!(
                decline_noun(
                    &invariant,
                    NounCell {
                        case,
                        number,
                        animacy: Animacy::Animate,
                    },
                    OrthographyProfile::Expanded,
                )
                .expect("invariant noun cell")
                .primary_text(),
                "адѡнаі"
            );
        }
    }
}
