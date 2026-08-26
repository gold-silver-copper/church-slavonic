use super::*;

pub(super) fn compose_cardinal(
    value: u32,
    cell: CompoundNumeralCell,
    inflector: Inflector,
) -> Result<Vec<CardinalPhraseAnalysis>> {
    if value <= 999 {
        return sub_thousand_cardinal(value as u16, cell, inflector);
    }

    let mut ordinary = if value == MAX_CARDINAL_VALUE {
        magnitude_chunk(1_000, Magnitude::Thousand, cell, inflector)?
    } else {
        let thousands = value / 1_000;
        let remainder = (value % 1_000) as u16;
        let mut chunks = vec![magnitude_chunk(
            thousands,
            Magnitude::Thousand,
            cell,
            inflector,
        )?];
        if remainder != 0 {
            chunks.push(sub_thousand_cardinal(remainder, cell, inflector)?);
        }
        combine_chunks(chunks, inflector.orthography())?
    };

    if value < MAX_CARDINAL_VALUE {
        ordinary.extend(distributed_thousands_cardinal(value, cell, inflector)?);
    }

    let named = named_magnitude_cardinal(value, cell, inflector)?;
    if let Some(magnitude) = exact_magnitude(value) {
        let exact = single_cardinal_analysis(
            NumeralComposition::Magnitude,
            magnitude_form(magnitude, cell.case, Number::Singular, inflector)?,
        );
        ordinary.retain(|analysis| analysis.primary_text() != exact.primary_text());
        ordinary.insert(0, exact);
    } else {
        ordinary.extend(named);
    }
    deduplicate_analyses(&mut ordinary);
    Ok(ordinary)
}

/// Synodal biblical usage can repeat `тысѧща` after the hundreds and lower
/// parts of a multiplier: 603,000 is printed as `ше́сть сѡ́тъ ты́сѧщъ и҆ трѝ
/// ты́сѧщы`. Keep that analysis correlated rather than flattening it into a
/// list of unrelated word variants.
pub(super) fn distributed_thousands_cardinal(
    value: u32,
    cell: CompoundNumeralCell,
    inflector: Inflector,
) -> Result<Vec<CardinalPhraseAnalysis>> {
    let thousands = value / 1_000;
    if thousands < 101 {
        return Ok(Vec::new());
    }
    let high = (thousands / 100) * 100;
    let low = thousands % 100;
    if low == 0 {
        return Ok(Vec::new());
    }
    let mut chunks = vec![magnitude_chunk(high, Magnitude::Thousand, cell, inflector)?];
    chunks.push(magnitude_chunk(low, Magnitude::Thousand, cell, inflector)?);
    let remainder = (value % 1_000) as u16;
    if remainder != 0 {
        chunks.push(sub_thousand_cardinal(remainder, cell, inflector)?);
    }
    combine_chunks(chunks, inflector.orthography())
}

pub(super) fn sub_thousand_cardinal(
    value: u16,
    cell: CompoundNumeralCell,
    inflector: Inflector,
) -> Result<Vec<CardinalPhraseAnalysis>> {
    if value <= 99 {
        return lower_cardinal(value as u8, cell, inflector);
    }
    if value == 100 {
        return Ok(vec![single_cardinal_analysis(
            NumeralComposition::Magnitude,
            magnitude_form(Magnitude::Hundred, cell.case, Number::Singular, inflector)?,
        )]);
    }

    let mut chunks = Vec::new();
    let hundreds = u32::from(value / 100);
    let remainder = (value % 100) as u8;
    if hundreds != 0 {
        chunks.push(magnitude_chunk(
            hundreds,
            Magnitude::Hundred,
            cell,
            inflector,
        )?);
    }
    if remainder != 0 {
        chunks.push(lower_cardinal(remainder, cell, inflector)?);
    }
    combine_chunks(chunks, inflector.orthography())
}

/// Builds the source-listed named-magnitude analysis alongside ordinary
/// decimal thousands. Thus 54,000 is primarily `пѧтьдесѧтъ и четыре тысѧщы`,
/// but the semantically equivalent `пѧть темъ и четыре тысѧщы` remains
/// available under Alypy's тьма = 10,000 inventory.
pub(super) fn named_magnitude_cardinal(
    value: u32,
    cell: CompoundNumeralCell,
    inflector: Inflector,
) -> Result<Vec<CardinalPhraseAnalysis>> {
    if value < 10_000 || value == MAX_CARDINAL_VALUE {
        return Ok(Vec::new());
    }
    let mut chunks = Vec::new();
    let places = [
        (100_000, Magnitude::Legion),
        (10_000, Magnitude::Myriad),
        (1_000, Magnitude::Thousand),
        (100, Magnitude::Hundred),
    ];
    let mut remainder = value;
    for (place, magnitude) in places {
        let digit = remainder / place;
        if digit != 0 {
            chunks.push(magnitude_chunk(digit, magnitude, cell, inflector)?);
            remainder %= place;
        }
    }
    if remainder != 0 {
        chunks.push(lower_cardinal(remainder as u8, cell, inflector)?);
    }
    combine_chunks(chunks, inflector.orthography())
}

pub(super) fn lower_cardinal(
    value: u8,
    cell: CompoundNumeralCell,
    inflector: Inflector,
) -> Result<Vec<CardinalPhraseAnalysis>> {
    match value {
        1..=9 => Ok(vec![single_cardinal_analysis(
            NumeralComposition::Simple,
            digit_form(value, cell.case, cell.gender, cell.animacy, inflector)?,
        )]),
        10 => Ok(vec![single_cardinal_analysis(
            NumeralComposition::Simple,
            ten_form(cell.case, Number::Singular, cell.animacy, inflector)?,
        )]),
        11..=19 => teen_analyses(value - 10, cell, inflector),
        20..=99 => {
            let tens = value / 10;
            let unit = value % 10;
            let analyses = tens_analyses(tens, cell.case, cell.animacy, inflector)?;
            if unit == 0 {
                return Ok(analyses);
            }
            let unit = numeral_token(digit_form(
                unit,
                cell.case,
                cell.gender,
                cell.animacy,
                inflector,
            )?);
            let connector = PhraseToken {
                role: PhraseRole::Conjunction,
                forms: grammar_form(
                    "и",
                    Some("и҆"),
                    "SYN-NUMERAL-CARDINAL-ADDITIVE-ALYPY-63",
                    "Alypy (Gamanovich), §63 multi-component cardinals",
                    inflector.orthography(),
                )?,
            };
            let mut combined = Vec::with_capacity(analyses.len() * 2);
            for analysis in analyses {
                let mut with_i = analysis.tokens.clone();
                with_i.push(connector.clone());
                with_i.push(unit.clone());
                combined.push(CardinalPhraseAnalysis {
                    construction: NumeralComposition::AdditiveFinalConjunction,
                    tokens: with_i,
                });
                let mut asyndetic = analysis.tokens;
                asyndetic.push(unit.clone());
                combined.push(CardinalPhraseAnalysis {
                    construction: NumeralComposition::AdditiveAsyndetic,
                    tokens: asyndetic,
                });
            }
            Ok(combined)
        }
        _ => Err(Error::OutOfRange {
            value: u32::from(value),
            maximum: 99,
        }),
    }
}

pub(super) fn teen_analyses(
    unit: u8,
    cell: CompoundNumeralCell,
    inflector: Inflector,
) -> Result<Vec<CardinalPhraseAnalysis>> {
    let profile = inflector.orthography();
    let prefix_gender = if unit <= 4 { cell.gender } else { None };
    let declined_unit = digit_form(unit, cell.case, prefix_gender, cell.animacy, inflector)?;
    let citation_unit = digit_form(
        unit,
        Case::Nominative,
        if unit <= 4 {
            Some(match unit {
                1 => Gender::Neuter,
                3 => Gender::Feminine,
                _ => Gender::Masculine,
            })
        } else {
            None
        },
        Animacy::Inanimate,
        inflector,
    )?;
    let fixed_ten = fixed_ten_accusative(profile)?;
    let singular_ten = ten_form(cell.case, Number::Singular, cell.animacy, inflector)?;
    let plural_ten = ten_form(cell.case, Number::Plural, cell.animacy, inflector)?;
    let na = grammar_form(
        "на",
        Some("на́"),
        "SYN-NUMERAL-CARDINAL-TEEN-ALYPY-63-64",
        "Alypy (Gamanovich), §§63–64 teen formation and inflection",
        profile,
    )?;
    let mut analyses = Vec::new();
    push_fused_teen(
        &mut analyses,
        NumeralComposition::TeenFirstComponentDeclined,
        &declined_unit,
        &na,
        &fixed_ten,
        profile,
    )?;
    push_fused_teen(
        &mut analyses,
        NumeralComposition::TeenSecondComponentDeclined,
        &citation_unit,
        &na,
        &singular_ten,
        profile,
    )?;
    push_fused_teen(
        &mut analyses,
        NumeralComposition::TeenSecondComponentDeclined,
        &citation_unit,
        &na,
        &plural_ten,
        profile,
    )?;
    push_fused_teen(
        &mut analyses,
        NumeralComposition::TeenBothComponentsDeclined,
        &declined_unit,
        &na,
        &singular_ten,
        profile,
    )?;
    if unit == 2 {
        let dual_ten = ten_form(cell.case, Number::Dual, cell.animacy, inflector)?;
        push_fused_teen(
            &mut analyses,
            NumeralComposition::TeenBothComponentsDual,
            &declined_unit,
            &na,
            &dual_ten,
            profile,
        )?;
    }
    deduplicate_analyses(&mut analyses);
    Ok(analyses)
}

pub(super) fn push_fused_teen(
    analyses: &mut Vec<CardinalPhraseAnalysis>,
    construction: NumeralComposition,
    unit: &FormSet,
    na: &FormSet,
    ten: &FormSet,
    profile: OrthographyProfile,
) -> Result<()> {
    let fused = fuse_form_sets(
        &[unit, na, ten],
        1,
        "SYN-NUMERAL-CARDINAL-TEEN-ALYPY-63-64",
        "Alypy (Gamanovich), §§63–64 teen formation and inflection",
        profile,
    )?;
    analyses.push(single_cardinal_analysis(construction, fused));
    if construction != NumeralComposition::TeenSecondComponentDeclined {
        analyses.push(CardinalPhraseAnalysis {
            construction,
            tokens: vec![
                numeral_token(unit.clone()),
                PhraseToken {
                    role: PhraseRole::Preposition,
                    forms: na.clone(),
                },
                numeral_token(ten.clone()),
            ],
        });
    }
    Ok(())
}

pub(super) fn tens_analyses(
    multiplier: u8,
    case: Case,
    animacy: Animacy,
    inflector: Inflector,
) -> Result<Vec<CardinalPhraseAnalysis>> {
    let profile = inflector.orthography();
    let citation = digit_form(
        multiplier,
        Case::Nominative,
        if multiplier <= 4 {
            Some(if multiplier == 3 {
                Gender::Feminine
            } else {
                Gender::Masculine
            })
        } else {
            None
        },
        Animacy::Inanimate,
        inflector,
    )?;
    let mut results = Vec::new();
    if multiplier <= 4 {
        for number in [Number::Singular, Number::Plural] {
            let ten = ten_form(case, number, animacy, inflector)?;
            let fused = fuse_form_sets(
                &[&citation, &ten],
                0,
                "SYN-NUMERAL-CARDINAL-TENS-AGREEMENT-ALYPY-63-64",
                "Alypy (Gamanovich), §§63–64 twenty through forty",
                profile,
            )?;
            results.push(single_cardinal_analysis(
                NumeralComposition::TensAgreement,
                fused,
            ));
        }
    } else {
        let declined = digit_form(multiplier, case, None, animacy, inflector)?;
        let governed_ten = fixed_genitive_plural_ten(profile)?;
        let accent_component = tens_government_accent_component(multiplier, case);
        results.push(single_cardinal_analysis(
            NumeralComposition::TensGovernment,
            fuse_form_sets(
                &[&declined, &governed_ten],
                accent_component,
                "SYN-NUMERAL-CARDINAL-TENS-GOVERNMENT-ALYPY-63-64",
                "Alypy (Gamanovich), §§63–64 fifty through ninety",
                profile,
            )?,
        ));
        for (number, construction) in [
            (
                Number::Singular,
                NumeralComposition::TensBothComponentsSingular,
            ),
            (Number::Plural, NumeralComposition::TensBothComponentsPlural),
        ] {
            let ten = ten_form(case, number, animacy, inflector)?;
            results.push(single_cardinal_analysis(
                construction,
                fuse_form_sets(
                    &[&declined, &ten],
                    accent_component,
                    "SYN-NUMERAL-CARDINAL-TENS-BOTH-ALYPY-64",
                    "Alypy (Gamanovich), §64 both-component alternative for fifty through ninety",
                    profile,
                )?,
            ));
        }
    }
    deduplicate_analyses(&mut results);
    Ok(results)
}

/// Alypy §§62–64 preserve the declined first component's stress in oblique
/// forms. In the direct cases the reviewed Synodal inventory has governed
/// tail stress for fifty and sixty, but lexical first-component stress for
/// seventy through ninety.
pub(super) const fn tens_government_accent_component(multiplier: u8, case: Case) -> usize {
    if matches!(
        case,
        Case::Genitive | Case::Dative | Case::Instrumental | Case::Locative
    ) || multiplier >= 7
    {
        0
    } else {
        1
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum Magnitude {
    Hundred,
    Thousand,
    Myriad,
    Legion,
    Leodr,
}

impl Magnitude {
    pub(super) const fn id(self) -> &'static str {
        match self {
            Self::Hundred => "synodal:numeral:v06-sto",
            Self::Thousand => "synodal:numeral:tysiascha",
            Self::Myriad => "synodal:numeral:tma",
            Self::Legion => "synodal:numeral:legeon",
            Self::Leodr => "synodal:numeral:leodr",
        }
    }

    pub(super) const fn gender(self) -> Gender {
        match self {
            Self::Hundred => Gender::Neuter,
            Self::Thousand | Self::Myriad => Gender::Feminine,
            Self::Legion | Self::Leodr => Gender::Masculine,
        }
    }
}

pub(super) fn exact_magnitude(value: u32) -> Option<Magnitude> {
    match value {
        100 => Some(Magnitude::Hundred),
        1_000 => Some(Magnitude::Thousand),
        10_000 => Some(Magnitude::Myriad),
        100_000 => Some(Magnitude::Legion),
        1_000_000 => Some(Magnitude::Leodr),
        _ => None,
    }
}

pub(super) fn magnitude_chunk(
    multiplier: u32,
    magnitude: Magnitude,
    cell: CompoundNumeralCell,
    inflector: Inflector,
) -> Result<Vec<CardinalPhraseAnalysis>> {
    if multiplier == 1 {
        return Ok(vec![single_cardinal_analysis(
            NumeralComposition::Magnitude,
            magnitude_form(magnitude, cell.case, Number::Singular, inflector)?,
        )]);
    }
    if magnitude == Magnitude::Hundred && multiplier > 9 {
        return Err(Error::InvalidNumeral {
            reason: "a hundreds multiplier must be one through nine".into(),
        });
    }

    let multiplier_cell = CompoundNumeralCell {
        case: cell.case,
        gender: cardinal_requires_gender(multiplier).then_some(magnitude.gender()),
        animacy: cell.animacy,
    };
    let leading_analyses = if multiplier <= 9 {
        vec![single_cardinal_analysis(
            NumeralComposition::Simple,
            digit_form(
                multiplier as u8,
                multiplier_cell.case,
                multiplier_cell.gender,
                multiplier_cell.animacy,
                inflector,
            )?,
        )]
    } else {
        cardinal_with(multiplier, multiplier_cell, inflector)?.analyses
    };
    let magnitude_cells = following_government(multiplier, cell.case);
    let mut results = Vec::new();
    let (composition_rule, composition_citation) = if magnitude == Magnitude::Hundred {
        (
            "SYN-NUMERAL-CARDINAL-HUNDREDS-ALYPY-63-64",
            "Alypy (Gamanovich), §§63–64 hundreds formation and spelling",
        )
    } else {
        (
            "SYN-NUMERAL-CARDINAL-MAGNITUDE-COMPOSITION-ALYPY-63",
            "Alypy (Gamanovich), §63 separate magnitude composition by agreement or government",
        )
    };
    for leading in leading_analyses {
        let leading_tokens = tag_tokens(&leading.tokens, composition_rule, composition_citation)?;
        for government in &magnitude_cells {
            let (case, number, construction) = match government {
                NumeralGovernment::Agreement { number } => (
                    cell.case,
                    *number,
                    if magnitude == Magnitude::Hundred {
                        NumeralComposition::HundredsAgreement
                    } else {
                        NumeralComposition::MagnitudeAgreement
                    },
                ),
                NumeralGovernment::GenitivePlural => (
                    Case::Genitive,
                    Number::Plural,
                    if magnitude == Magnitude::Hundred {
                        NumeralComposition::HundredsGovernment
                    } else {
                        NumeralComposition::MagnitudeGovernment
                    },
                ),
                NumeralGovernment::ContextualNominativePlural => (
                    Case::Nominative,
                    Number::Plural,
                    if magnitude == Magnitude::Hundred {
                        NumeralComposition::HundredsAgreement
                    } else {
                        NumeralComposition::MagnitudeAgreement
                    },
                ),
            };
            let magnitude_forms = tag_form_set(
                &magnitude_form(magnitude, case, number, inflector)?,
                composition_rule,
                composition_citation,
            )?;
            let mut tokens = leading_tokens.clone();
            tokens.push(numeral_token(magnitude_forms.clone()));
            let spaced = CardinalPhraseAnalysis {
                construction,
                tokens,
            };
            if magnitude != Magnitude::Hundred || leading_tokens.len() != 1 {
                results.push(spaced);
                continue;
            }
            let fused = single_cardinal_analysis(
                construction,
                fuse_form_sets(
                    &[&leading_tokens[0].forms, &magnitude_forms],
                    usize::from(multiplier >= 5),
                    "SYN-NUMERAL-CARDINAL-HUNDREDS-ALYPY-63-64",
                    "Alypy (Gamanovich), §§63–64 hundreds formation and spelling",
                    inflector.orthography(),
                )?,
            );
            if cell.case == Case::Nominative && multiplier <= 4 {
                results.extend([fused, spaced]);
            } else {
                results.extend([spaced, fused]);
            }
        }
    }
    deduplicate_analyses(&mut results);
    Ok(results)
}

pub(super) fn combine_chunks(
    mut chunks: Vec<Vec<CardinalPhraseAnalysis>>,
    profile: OrthographyProfile,
) -> Result<Vec<CardinalPhraseAnalysis>> {
    if chunks.len() == 1 {
        return Ok(chunks.pop().unwrap_or_default());
    }
    let mut products = vec![Vec::<CardinalPhraseAnalysis>::new()];
    for chunk in chunks {
        let mut next = Vec::new();
        for prefix in &products {
            for suffix in &chunk {
                let mut combined = prefix.clone();
                combined.push(suffix.clone());
                next.push(combined);
            }
        }
        products = next;
    }

    let conjunction = PhraseToken {
        role: PhraseRole::Conjunction,
        forms: grammar_form(
            "и",
            Some("и҆"),
            "SYN-NUMERAL-CARDINAL-ADDITIVE-ALYPY-63",
            "Alypy (Gamanovich), §63 multi-component cardinal connectors",
            profile,
        )?,
    };
    let mut results = Vec::new();
    for product in products {
        for mode in [
            NumeralComposition::AdditiveFinalConjunction,
            NumeralComposition::AdditiveAllConjunctions,
            NumeralComposition::AdditiveAsyndetic,
        ] {
            let mut tokens = Vec::new();
            for (index, chunk) in product.iter().enumerate() {
                if index != 0
                    && (mode == NumeralComposition::AdditiveAllConjunctions
                        || (mode == NumeralComposition::AdditiveFinalConjunction
                            && index + 1 == product.len()))
                {
                    tokens.push(conjunction.clone());
                }
                tokens.extend(chunk.tokens.clone());
            }
            results.push(CardinalPhraseAnalysis {
                construction: mode,
                tokens,
            });
        }
    }
    deduplicate_analyses(&mut results);
    Ok(results)
}
