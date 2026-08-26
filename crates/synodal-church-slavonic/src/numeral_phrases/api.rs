use super::*;

/// Realizes every source-licensed structural strategy for a cardinal from one
/// through the largest exact simple value listed by Alypy, one million.
pub fn cardinal(value: u32, cell: CompoundNumeralCell) -> Result<RealizedCardinal> {
    cardinal_with(value, cell, Inflector::default())
}

pub fn cardinal_with(
    value: u32,
    cell: CompoundNumeralCell,
    inflector: Inflector,
) -> Result<RealizedCardinal> {
    if !(MIN_CARDINAL_VALUE..=MAX_CARDINAL_VALUE).contains(&value) {
        return Err(Error::OutOfRange {
            value,
            maximum: MAX_CARDINAL_VALUE,
        });
    }
    if cell.case == Case::Vocative {
        return Err(Error::HistoricallyInvalidCell {
            reason: "Alypy §§62–64 do not license a cardinal vocative".into(),
        });
    }
    let needs_gender = cardinal_requires_gender(value);
    if cell.gender.is_some() != needs_gender {
        return Err(Error::HistoricallyInvalidCell {
            reason: if needs_gender {
                "the final agreeing cardinal component requires gender"
            } else {
                "a substantival or magnitude-final cardinal has no agreement-gender dimension"
            }
            .into(),
        });
    }

    let analyses = compose_cardinal(value, cell, inflector)?;
    RealizedCardinal::new(
        value,
        cell,
        following_government(value, cell.case),
        preceding_government(value, cell.case),
        vec![numeral_evidence(
            "SYN-NUMERAL-GOVERNMENT-ALYPY-65-67",
            "Alypy (Gamanovich), §§65–67 numeral agreement, government, position, and contextual nominative",
        )],
        analyses,
    )
}

/// Realizes simple and compound ordinals through Alypy's last explicitly
/// supplied ordinal head, `тысѧщный` “thousandth”.
pub fn ordinal(value: u16, cell: NumeralCell) -> Result<RealizedOrdinal> {
    ordinal_with(value, cell, Inflector::default())
}

pub fn ordinal_with(
    value: u16,
    cell: NumeralCell,
    inflector: Inflector,
) -> Result<RealizedOrdinal> {
    if value == 0 || value > MAX_COMPOUND_ORDINAL_VALUE {
        return Err(Error::OutOfRange {
            value: u32::from(value),
            maximum: u32::from(MAX_COMPOUND_ORDINAL_VALUE),
        });
    }
    if cell.kind != NumeralKind::Ordinal || cell.gender.is_none() {
        return Err(Error::HistoricallyInvalidCell {
            reason: "a compound ordinal requires an ordinal agreement cell with gender".into(),
        });
    }
    let analyses = compose_ordinal(value, cell, inflector)?;
    RealizedOrdinal::new(value, cell, analyses)
}

/// Repeats a fully inflected cardinal as a distributive expression (`два
/// два`, Alypy §61). The cited value two is exact construction evidence; the
/// same transparent repetition is available productively for every cardinal
/// in the source-bounded range without labeling an unattested phrase attested.
pub fn repeated_distributive(value: u32, cell: CompoundNumeralCell) -> Result<Vec<RealizedPhrase>> {
    repeated_distributive_with(value, cell, Inflector::default())
}

pub fn repeated_distributive_with(
    value: u32,
    cell: CompoundNumeralCell,
    inflector: Inflector,
) -> Result<Vec<RealizedPhrase>> {
    let cardinal = cardinal_with(value, cell, inflector)?;
    let mut phrases = Vec::new();
    for analysis in cardinal.analyses() {
        let first = tag_tokens(
            &analysis.tokens,
            "SYN-NUMERAL-DISTRIBUTIVE-REPETITION-ALYPY-61",
            "Alypy (Gamanovich), §61; Mark 6:7 два два",
        )?;
        let mut tokens = first.clone();
        tokens.extend(first);
        phrases.push(RealizedPhrase::new(
            AnalyticConstruction::RepeatedDistributive,
            tokens,
        )?);
    }
    deduplicate_phrases(&mut phrases);
    Ok(phrases)
}

/// Realizes a quantitative multiplicative: an inflected cardinal followed by
/// invariant `кратъ` (Alypy §70).
pub fn multiplicative_krat(value: u32, cell: CompoundNumeralCell) -> Result<Vec<RealizedPhrase>> {
    multiplicative_krat_with(value, cell, Inflector::default())
}

pub fn multiplicative_krat_with(
    value: u32,
    cell: CompoundNumeralCell,
    inflector: Inflector,
) -> Result<Vec<RealizedPhrase>> {
    let cardinal = cardinal_with(value, cell, inflector)?;
    let krat = PhraseToken {
        role: PhraseRole::MultiplicativeUnit,
        forms: grammar_form(
            "кратъ",
            Some("кра́тъ"),
            "SYN-NUMERAL-MULTIPLICATIVE-KRAT-ALYPY-70",
            "Alypy (Gamanovich), §70 invariant кратъ multiplicatives",
            inflector.orthography(),
        )?,
    };
    let mut phrases = Vec::new();
    for analysis in cardinal.analyses() {
        let mut tokens = tag_tokens(
            &analysis.tokens,
            "SYN-NUMERAL-MULTIPLICATIVE-KRAT-ALYPY-70",
            "Alypy (Gamanovich), §70 invariant кратъ multiplicatives",
        )?;
        tokens.push(krat.clone());
        phrases.push(RealizedPhrase::new(
            AnalyticConstruction::MultiplicativeKrat,
            tokens,
        )?);
    }
    deduplicate_phrases(&mut phrases);
    Ok(phrases)
}

/// Realizes cardinal expressions with inflected `часть`: `єдина часть`,
/// `двѣ части`, `три части`, and their productive case/number extensions.
pub fn fractional_cardinal_parts(
    count: u32,
    case: Case,
    animacy: Animacy,
) -> Result<Vec<RealizedPhrase>> {
    fractional_cardinal_parts_with(count, case, animacy, Inflector::default())
}

pub fn fractional_cardinal_parts_with(
    count: u32,
    case: Case,
    animacy: Animacy,
    inflector: Inflector,
) -> Result<Vec<RealizedPhrase>> {
    let cell = CompoundNumeralCell {
        case,
        gender: cardinal_requires_gender(count).then_some(Gender::Feminine),
        animacy,
    };
    let cardinal = cardinal_with(count, cell, inflector)?;
    fractional_cardinal_phrases(&cardinal, animacy, inflector)
}

/// Realizes an ordinal denominator agreeing with inflected `часть`, such as
/// `десѧтаѧ часть` (Alypy §70).
pub fn fractional_ordinal_parts(denominator: u16, cell: NounCell) -> Result<Vec<RealizedPhrase>> {
    fractional_ordinal_parts_with(denominator, cell, Inflector::default())
}

pub fn fractional_ordinal_parts_with(
    denominator: u16,
    cell: NounCell,
    inflector: Inflector,
) -> Result<Vec<RealizedPhrase>> {
    let ordinal = ordinal_with(
        denominator,
        NumeralCell {
            kind: NumeralKind::Ordinal,
            case: cell.case,
            number: cell.number,
            gender: Some(Gender::Feminine),
            animacy: cell.animacy,
        },
        inflector,
    )?;
    let part = fraction_noun_form(cell, inflector)?;
    let mut phrases = Vec::new();
    for analysis in ordinal.analyses() {
        let mut tokens = tag_tokens(
            &analysis.tokens,
            "SYN-NUMERAL-FRACTION-ORDINAL-PART-ALYPY-70",
            "Alypy (Gamanovich), §70 ordinal + inflected часть fractions",
        )?;
        tokens.push(PhraseToken {
            role: PhraseRole::FractionNoun,
            forms: tag_form_set(
                &part,
                "SYN-NUMERAL-FRACTION-ORDINAL-PART-ALYPY-70",
                "Alypy (Gamanovich), §70 ordinal + inflected часть fractions",
            )?,
        });
        phrases.push(RealizedPhrase::new(
            AnalyticConstruction::FractionalPart,
            tokens,
        )?);
    }
    deduplicate_phrases(&mut phrases);
    Ok(phrases)
}

/// Realizes the directly Synodal fractional adjective `полдесѧтый` with
/// inflected `часть`. III Esdras 14:11–12 directly supplies the feminine
/// genitive singular; the remaining agreement cells are transparent
/// applications of the ordinary hard-adjective paradigm.
pub fn fractional_half_tenth_parts(cell: NounCell) -> Result<RealizedPhrase> {
    fractional_half_tenth_parts_with(cell, Inflector::default())
}

pub fn fractional_half_tenth_parts_with(
    cell: NounCell,
    inflector: Inflector,
) -> Result<RealizedPhrase> {
    let fractional = inflector.form_by_id(
        &LexemeId::from("synodal:numeral:fractional-poludesyatyi"),
        GrammarCell::Numeral(NumeralCell {
            kind: NumeralKind::Fractional,
            case: cell.case,
            number: cell.number,
            gender: Some(Gender::Feminine),
            animacy: cell.animacy,
        }),
    )?;
    let part = fraction_noun_form(cell, inflector)?;
    RealizedPhrase::new(
        AnalyticConstruction::FractionalPart,
        vec![
            numeral_token(fractional),
            PhraseToken {
                role: PhraseRole::FractionNoun,
                forms: part,
            },
        ],
    )
}

/// Realizes a rational expression whose numerator governs an ordinally
/// qualified `часть`. For example, two fifth parts use dual agreement, while
/// five fifth parts use the source-licensed genitive-plural construction.
pub fn fraction(
    numerator: u32,
    denominator: u16,
    case: Case,
    animacy: Animacy,
) -> Result<Vec<RealizedPhrase>> {
    fraction_with(numerator, denominator, case, animacy, Inflector::default())
}

pub fn fraction_with(
    numerator: u32,
    denominator: u16,
    case: Case,
    animacy: Animacy,
    inflector: Inflector,
) -> Result<Vec<RealizedPhrase>> {
    let cardinal = cardinal_with(
        numerator,
        CompoundNumeralCell {
            case,
            gender: cardinal_requires_gender(numerator).then_some(Gender::Feminine),
            animacy,
        },
        inflector,
    )?;
    let noun_cells = governed_fraction_noun_cells(&cardinal, animacy);
    let mut phrases = Vec::new();
    for noun_cell in noun_cells {
        let ordinals = ordinal_with(
            denominator,
            NumeralCell {
                kind: NumeralKind::Ordinal,
                case: noun_cell.case,
                number: noun_cell.number,
                gender: Some(Gender::Feminine),
                animacy: noun_cell.animacy,
            },
            inflector,
        )?;
        let part = fraction_noun_form(noun_cell, inflector)?;
        for cardinal_analysis in cardinal.analyses() {
            for ordinal_analysis in ordinals.analyses() {
                let mut tokens = tag_tokens(
                    &cardinal_analysis.tokens,
                    "SYN-NUMERAL-FRACTION-CARDINAL-ORDINAL-PART-ALYPY-70",
                    "Alypy (Gamanovich), §70 cardinal/ordinal + часть fractions",
                )?;
                tokens.extend(tag_tokens(
                    &ordinal_analysis.tokens,
                    "SYN-NUMERAL-FRACTION-CARDINAL-ORDINAL-PART-ALYPY-70",
                    "Alypy (Gamanovich), §70 cardinal/ordinal + часть fractions",
                )?);
                tokens.push(PhraseToken {
                    role: PhraseRole::FractionNoun,
                    forms: tag_form_set(
                        &part,
                        "SYN-NUMERAL-FRACTION-CARDINAL-ORDINAL-PART-ALYPY-70",
                        "Alypy (Gamanovich), §70 cardinal/ordinal + часть fractions",
                    )?,
                });
                phrases.push(RealizedPhrase::new(
                    AnalyticConstruction::FractionalPart,
                    tokens,
                )?);
            }
        }
    }
    deduplicate_phrases(&mut phrases);
    Ok(phrases)
}

pub(super) fn fractional_cardinal_phrases(
    cardinal: &RealizedCardinal,
    animacy: Animacy,
    inflector: Inflector,
) -> Result<Vec<RealizedPhrase>> {
    let noun_cells = governed_fraction_noun_cells(cardinal, animacy);
    let mut phrases = Vec::new();
    for noun_cell in noun_cells {
        let part = fraction_noun_form(noun_cell, inflector)?;
        for analysis in cardinal.analyses() {
            let mut tokens = tag_tokens(
                &analysis.tokens,
                "SYN-NUMERAL-FRACTION-CARDINAL-PART-ALYPY-70",
                "Alypy (Gamanovich), §70 cardinal + inflected часть fractions",
            )?;
            tokens.push(PhraseToken {
                role: PhraseRole::FractionNoun,
                forms: tag_form_set(
                    &part,
                    "SYN-NUMERAL-FRACTION-CARDINAL-PART-ALYPY-70",
                    "Alypy (Gamanovich), §70 cardinal + inflected часть fractions",
                )?,
            });
            phrases.push(RealizedPhrase::new(
                AnalyticConstruction::FractionalPart,
                tokens,
            )?);
        }
    }
    deduplicate_phrases(&mut phrases);
    Ok(phrases)
}

pub(super) fn governed_fraction_noun_cells(
    cardinal: &RealizedCardinal,
    animacy: Animacy,
) -> Vec<NounCell> {
    let mut cells = Vec::new();
    for government in cardinal.government(NumeralNounPosition::Following) {
        let (case, number) = match government {
            NumeralGovernment::Agreement { number } => (cardinal.cell().case, *number),
            NumeralGovernment::GenitivePlural => (Case::Genitive, Number::Plural),
            NumeralGovernment::ContextualNominativePlural => (Case::Nominative, Number::Plural),
        };
        let cell = NounCell {
            case,
            number,
            animacy,
        };
        if !cells.contains(&cell) {
            cells.push(cell);
        }
    }
    cells
}

pub(super) fn fraction_noun_form(cell: NounCell, inflector: Inflector) -> Result<FormSet> {
    inflector.form_by_id(
        &LexemeId::from("synodal:noun:v07-6ef4c1b12b34ac8c"),
        GrammarCell::Noun(cell),
    )
}
