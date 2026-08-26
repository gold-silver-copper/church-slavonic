use super::*;

pub(super) fn compose_ordinal(
    value: u16,
    cell: NumeralCell,
    inflector: Inflector,
) -> Result<Vec<OrdinalPhraseAnalysis>> {
    if value <= 10 {
        return Ok(vec![OrdinalPhraseAnalysis {
            construction: NumeralComposition::Simple,
            tokens: vec![numeral_token(simple_ordinal_form(
                value as u8,
                cell,
                inflector,
            )?)],
        }]);
    }
    if (11..=19).contains(&value) {
        let unit = (value - 10) as u8;
        let ordinal_unit = simple_ordinal_form(unit, cell, inflector)?;
        let tail = grammar_form(
            "надесѧть",
            Some("на́десѧть"),
            "SYN-NUMERAL-ORDINAL-TEEN-ALYPY-68",
            "Alypy (Gamanovich), §68 and appendix, ordinal teens",
            inflector.orthography(),
        )?;
        let analytic = fuse_form_sets(
            &[&ordinal_unit, &tail],
            1,
            "SYN-NUMERAL-ORDINAL-TEEN-ALYPY-68",
            "Alypy (Gamanovich), §68 ordinal ending on the first teen component",
            inflector.orthography(),
        )?;
        let (lemma, stem) = ordinal_head(value)?;
        return Ok(vec![
            OrdinalPhraseAnalysis {
                construction: NumeralComposition::CompoundOrdinalAnalyticTeen,
                tokens: vec![numeral_token(analytic)],
            },
            OrdinalPhraseAnalysis {
                construction: NumeralComposition::CompoundOrdinalFused,
                tokens: vec![numeral_token(dynamic_ordinal_form(
                    lemma, stem, cell, inflector,
                )?)],
            },
        ]);
    }
    if let Ok((lemma, stem)) = ordinal_head(value) {
        return Ok(vec![OrdinalPhraseAnalysis {
            construction: NumeralComposition::CompoundOrdinalFused,
            tokens: vec![numeral_token(dynamic_ordinal_form(
                lemma, stem, cell, inflector,
            )?)],
        }]);
    }

    let (prefix, final_value) = ordinal_prefix_and_head(value)?;
    let final_forms = if final_value <= 10 {
        simple_ordinal_form(final_value as u8, cell, inflector)?
    } else {
        let (lemma, stem) = ordinal_head(final_value)?;
        dynamic_ordinal_form(lemma, stem, cell, inflector)?
    };
    let final_token = numeral_token(final_forms);
    let prefix_cell = CompoundNumeralCell {
        case: Case::Nominative,
        gender: None,
        animacy: Animacy::Inanimate,
    };
    let cardinal_prefix = cardinal_with(u32::from(prefix), prefix_cell, inflector)?;
    let conjunction = PhraseToken {
        role: PhraseRole::Conjunction,
        forms: grammar_form(
            "и",
            Some("и҆"),
            "SYN-NUMERAL-ORDINAL-COMPOUND-ALYPY-68",
            "Alypy (Gamanovich), §68 multi-component ordinals",
            inflector.orthography(),
        )?,
    };
    let mut analyses = Vec::new();
    for prefix_analysis in cardinal_prefix.analyses() {
        let mut asyndetic = prefix_analysis.tokens.clone();
        asyndetic.push(final_token.clone());
        analyses.push(OrdinalPhraseAnalysis {
            construction: NumeralComposition::CompoundOrdinalAsyndetic,
            tokens: asyndetic,
        });
        let mut connected = prefix_analysis.tokens.clone();
        connected.push(conjunction.clone());
        connected.push(final_token.clone());
        analyses.push(OrdinalPhraseAnalysis {
            construction: NumeralComposition::CompoundOrdinalConjunction,
            tokens: connected,
        });
    }
    deduplicate_ordinal_analyses(&mut analyses);
    Ok(analyses)
}

pub(super) fn ordinal_prefix_and_head(value: u16) -> Result<(u16, u16)> {
    let final_value = if value % 10 != 0 {
        value % 10
    } else if value % 100 != 0 {
        value % 100
    } else if value % 1_000 != 0 {
        value % 1_000
    } else {
        value
    };
    let prefix = value - final_value;
    if prefix == 0 || (final_value > 10 && ordinal_head(final_value).is_err()) {
        return Err(Error::UnsupportedFormation {
            formation: format!("compound ordinal {value}"),
        });
    }
    Ok((prefix, final_value))
}

pub(super) fn ordinal_head(value: u16) -> Result<(&'static str, &'static str)> {
    let head = match value {
        11 => ("єдинонадесѧтый", "єдинонадесѧт"),
        12 => ("дванадесѧтый", "дванадесѧт"),
        13 => ("тринадесѧтый", "тринадесѧт"),
        14 => ("четыренадесѧтый", "четыренадесѧт"),
        15 => ("пѧтьнадесѧтый", "пѧтьнадесѧт"),
        16 => ("шестьнадесѧтый", "шестьнадесѧт"),
        17 => ("седмьнадесѧтый", "седмьнадесѧт"),
        18 => ("осмьнадесѧтый", "осмьнадесѧт"),
        19 => ("девѧтьнадесѧтый", "девѧтьнадесѧт"),
        20 => ("двадесѧтый", "двадесѧт"),
        30 => ("тридесѧтый", "тридесѧт"),
        40 => ("четыредесѧтый", "четыредесѧт"),
        50 => ("пѧтьдесѧтый", "пѧтьдесѧт"),
        60 => ("шестьдесѧтый", "шестьдесѧт"),
        70 => ("седмьдесѧтый", "седмьдесѧт"),
        80 => ("осмьдесѧтый", "осмьдесѧт"),
        90 => ("девѧтьдесѧтый", "девѧтьдесѧт"),
        100 => ("сотный", "сотн"),
        200 => ("двосотный", "двосотн"),
        300 => ("трисотный", "трисотн"),
        400 => ("четвертосотный", "четвертосотн"),
        500 => ("пѧтьсотный", "пѧтьсотн"),
        600 => ("шестьсотный", "шестьсотн"),
        700 => ("седмьсотный", "седмьсотн"),
        800 => ("осмьсотный", "осмьсотн"),
        900 => ("девѧтьсотный", "девѧтьсотн"),
        1_000 => ("тысѧщный", "тысѧщн"),
        _ => {
            return Err(Error::UnsupportedFormation {
                formation: format!("ordinal head {value}"),
            });
        }
    };
    Ok(head)
}

pub(super) fn simple_ordinal_form(
    value: u8,
    cell: NumeralCell,
    inflector: Inflector,
) -> Result<FormSet> {
    let id = match value {
        1 => "synodal:numeral:pervyi",
        2 => "synodal:numeral:vtoryi",
        3 => "synodal:numeral:tretii",
        4 => "synodal:numeral:chetvertyi",
        5 => "synodal:numeral:pyatyi",
        6 => "synodal:numeral:shestyi",
        7 => "synodal:numeral:sedmyi",
        8 => "synodal:numeral:osmyi",
        9 => "synodal:numeral:devyatyi",
        10 => "synodal:numeral:desyatyi",
        _ => {
            return Err(Error::OutOfRange {
                value: u32::from(value),
                maximum: 10,
            });
        }
    };
    inflector.form_by_id(&LexemeId::from(id), GrammarCell::Numeral(cell))
}

pub(super) fn dynamic_ordinal_form(
    lemma: &str,
    stem: &str,
    cell: NumeralCell,
    inflector: Inflector,
) -> Result<FormSet> {
    let lexeme = NumeralLexeme::new(
        SynodalWord::parse(lemma)?,
        SynodalWord::parse(stem)?,
        NumeralDeclension::OrdinalHard,
    );
    decline_numeral(&lexeme, cell, inflector.orthography())
}
