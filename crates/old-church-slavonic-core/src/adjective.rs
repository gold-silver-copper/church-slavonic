//! Rule-based adjective declension.

use crate::{
    AdjectiveCell, AdjectiveClass, AdjectiveForm, Animacy, Case, ComparativeFormation, Gender,
    InflectionError, Number, PredictedForm, RuleId, RuleStep,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdjectiveLexeme {
    pub lemma: String,
    pub class: AdjectiveClass,
}

/// The two principal parts needed to inflect one OCS comparative.
///
/// `syncopated_citation` is the short masculine nominative singular (`новѣи`,
/// `грѫбл҄ь`); `expanded_citation` is the short feminine nominative singular
/// (`новѣиши`, `грѫбл҄ьши`). Requiring both prevents the engine from guessing
/// the lexically restricted consonant alternations of old comparatives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComparativeLexeme {
    pub positive_lemma: String,
    pub syncopated_citation: String,
    pub expanded_citation: String,
    pub formation: ComparativeFormation,
}

/// Build the productive new comparative from an explicitly classified positive
/// adjective. Final velars undergo first palatalization and select surface
/// `-аи-`; all other bases select `-ѣи-`.
pub fn productive_new_comparative(
    positive: &AdjectiveLexeme,
) -> Result<ComparativeLexeme, InflectionError> {
    let lemma = crate::orthography::canonical_display(&positive.lemma)?;
    let stem = match positive.class {
        AdjectiveClass::Hard => strip_citation(&lemma, &["ъ"], "hard")?,
        AdjectiveClass::Soft => strip_citation(&lemma, &["ь", "и"], "soft")?,
    };
    let (base, suffix) = if stem.ends_with(['к', 'г', 'х']) {
        (palatalize(stem, [('к', "ч"), ('г', "ж"), ('х', "ш")]), "аи")
    } else {
        (stem.to_string(), "ѣи")
    };
    let syncopated_citation = format!("{base}{suffix}");
    Ok(ComparativeLexeme {
        positive_lemma: lemma,
        expanded_citation: format!("{syncopated_citation}ши"),
        syncopated_citation,
        formation: ComparativeFormation::New,
    })
}

/// Inflect a comparative from its independently supplied principal parts.
///
/// Comparatives use a syncopated stem in precisely three source-described
/// direct cells and an expanded soft-adjective stem elsewhere. Four expanded
/// cells take the alien endings `-и/-иꙗ` and `-е/-еи`.
pub fn decline_comparative(
    lexeme: &ComparativeLexeme,
    cell: AdjectiveCell,
) -> Result<PredictedForm, InflectionError> {
    let lexeme = validate_comparative(lexeme)?;
    let rule_id = match lexeme.formation {
        ComparativeFormation::New => RuleId::AdjectiveComparativeNew,
        ComparativeFormation::Old => RuleId::AdjectiveComparativeOld,
    };

    let text = if is_syncopated_comparative_cell(cell) {
        syncopated_comparative_form(&lexeme, cell)?
    } else {
        let expanded_stem = lexeme
            .expanded_citation
            .strip_suffix('и')
            .ok_or_else(|| contradictory_comparative(&lexeme))?;
        if let Some(ending) = comparative_alien_ending(cell) {
            format!("{expanded_stem}{ending}")
        } else {
            decline_validated_stem(
                expanded_stem,
                AdjectiveClass::Soft,
                cell,
                &lexeme.expanded_citation,
            )?
            .text
        }
    };

    Ok(PredictedForm {
        text: text.clone(),
        rule_id,
        trace: vec![RuleStep {
            rule_id,
            before: lexeme.positive_lemma,
            after: text,
            reason: "select the comparative principal-part stem and attach its agreement ending",
        }],
    })
}

fn validate_comparative(lexeme: &ComparativeLexeme) -> Result<ComparativeLexeme, InflectionError> {
    let normalized = ComparativeLexeme {
        positive_lemma: crate::orthography::canonical_display(&lexeme.positive_lemma)?,
        syncopated_citation: crate::orthography::canonical_display(&lexeme.syncopated_citation)?,
        expanded_citation: crate::orthography::canonical_display(&lexeme.expanded_citation)?,
        formation: lexeme.formation,
    };
    let valid_syncopated_ending = match normalized.formation {
        ComparativeFormation::New => normalized.syncopated_citation.ends_with('и'),
        ComparativeFormation::Old => normalized.syncopated_citation.ends_with('ь'),
    };
    if !valid_syncopated_ending
        || normalized.expanded_citation != format!("{}ши", normalized.syncopated_citation)
    {
        return Err(InflectionError::InvalidInput {
            reason: format!(
                "the {} comparative principal parts are contradictory",
                normalized.formation.code()
            ),
        });
    }
    Ok(normalized)
}

fn is_syncopated_comparative_cell(cell: AdjectiveCell) -> bool {
    if cell.number != Number::Singular {
        return false;
    }
    matches!(
        (cell.form, cell.case, cell.gender, cell.animacy),
        (
            AdjectiveForm::Short,
            Case::Nominative,
            Gender::Masculine | Gender::Neuter,
            _
        ) | (AdjectiveForm::Short, Case::Accusative, Gender::Neuter, _)
            | (
                AdjectiveForm::Short | AdjectiveForm::Long,
                Case::Accusative,
                Gender::Masculine,
                Animacy::Inanimate,
            )
            | (AdjectiveForm::Long, Case::Nominative, Gender::Masculine, _)
    )
}

fn syncopated_comparative_form(
    lexeme: &ComparativeLexeme,
    cell: AdjectiveCell,
) -> Result<String, InflectionError> {
    let text = match (lexeme.formation, cell.form, cell.gender) {
        (_, AdjectiveForm::Short, Gender::Masculine) => lexeme.syncopated_citation.clone(),
        (ComparativeFormation::New, AdjectiveForm::Short, Gender::Neuter) => {
            let stem = lexeme
                .syncopated_citation
                .strip_suffix('и')
                .ok_or_else(|| contradictory_comparative(lexeme))?;
            format!("{stem}ѥ")
        }
        (ComparativeFormation::Old, AdjectiveForm::Short, Gender::Neuter) => {
            let stem = lexeme
                .syncopated_citation
                .strip_suffix('ь')
                .ok_or_else(|| contradictory_comparative(lexeme))?;
            format!("{stem}е")
        }
        (ComparativeFormation::New, AdjectiveForm::Long, Gender::Masculine) => {
            format!("{}и", lexeme.syncopated_citation)
        }
        (ComparativeFormation::Old, AdjectiveForm::Long, Gender::Masculine) => {
            let stem = lexeme
                .syncopated_citation
                .strip_suffix('ь')
                .ok_or_else(|| contradictory_comparative(lexeme))?;
            format!("{stem}ии")
        }
        _ => return Err(contradictory_comparative(lexeme)),
    };
    Ok(text)
}

fn contradictory_comparative(lexeme: &ComparativeLexeme) -> InflectionError {
    InflectionError::InvalidInput {
        reason: format!(
            "the {} comparative principal parts are contradictory",
            lexeme.formation.code()
        ),
    }
}

fn comparative_alien_ending(cell: AdjectiveCell) -> Option<&'static str> {
    match (cell.form, cell.case, cell.number, cell.gender) {
        (
            AdjectiveForm::Short,
            Case::Nominative | Case::Vocative,
            Number::Singular,
            Gender::Feminine,
        ) => Some("и"),
        (
            AdjectiveForm::Long,
            Case::Nominative | Case::Vocative,
            Number::Singular,
            Gender::Feminine,
        ) => Some("иꙗ"),
        (
            AdjectiveForm::Short,
            Case::Nominative | Case::Vocative,
            Number::Plural,
            Gender::Masculine,
        ) => Some("е"),
        (
            AdjectiveForm::Long,
            Case::Nominative | Case::Vocative,
            Number::Plural,
            Gender::Masculine,
        ) => Some("еи"),
        _ => None,
    }
}

pub fn decline(
    lexeme: &AdjectiveLexeme,
    cell: AdjectiveCell,
) -> Result<PredictedForm, InflectionError> {
    let normalized_lexeme = AdjectiveLexeme {
        lemma: crate::orthography::canonical_display(&lexeme.lemma)?,
        class: lexeme.class,
    };
    let lexeme = &normalized_lexeme;
    let stem = match lexeme.class {
        AdjectiveClass::Hard => strip_citation(&lexeme.lemma, &["ъ"], "hard")?,
        AdjectiveClass::Soft => strip_citation(&lexeme.lemma, &["ь", "и"], "soft")?,
    };
    decline_validated_stem(stem, lexeme.class, cell, &lexeme.lemma)
}

/// Declines an already selected adjective stem. Participles use this entry point so
/// adjective agreement has one implementation without pretending the verbal stem is
/// itself a dictionary adjective citation.
pub fn decline_stem(
    stem: &str,
    class: AdjectiveClass,
    cell: AdjectiveCell,
) -> Result<PredictedForm, InflectionError> {
    let stem = crate::orthography::canonical_display(stem)?;
    if stem.is_empty() {
        return Err(InflectionError::InvalidInput {
            reason: "an adjective agreement stem cannot be empty".to_string(),
        });
    }
    decline_validated_stem(&stem, class, cell, &stem)
}

fn decline_validated_stem(
    stem: &str,
    class: AdjectiveClass,
    cell: AdjectiveCell,
    before: &str,
) -> Result<PredictedForm, InflectionError> {
    let (ending, rule_id) = match (class, cell.form) {
        (AdjectiveClass::Hard, AdjectiveForm::Short) => {
            (hard_short_ending(cell), RuleId::AdjectiveHardShort)
        }
        (AdjectiveClass::Hard, AdjectiveForm::Long) => {
            (hard_long_ending(cell), RuleId::AdjectiveHardLong)
        }
        (AdjectiveClass::Soft, AdjectiveForm::Short) => {
            (soft_short_ending(cell), RuleId::AdjectiveSoftShort)
        }
        (AdjectiveClass::Soft, AdjectiveForm::Long) => {
            (soft_long_ending(cell), RuleId::AdjectiveSoftLong)
        }
    };
    let changed_stem = if class == AdjectiveClass::Hard
        && (ending.starts_with('ѣ') || matches!(ending, "и" | "ии"))
    {
        palatalize(stem, [('к', "ц"), ('г', "ѕ"), ('х', "с")])
    } else if class == AdjectiveClass::Hard && ending == "е" {
        palatalize(stem, [('к', "ч"), ('г', "ж"), ('х', "ш")])
    } else {
        stem.to_string()
    };
    let text = format!("{changed_stem}{ending}");
    Ok(PredictedForm {
        text: text.clone(),
        rule_id,
        trace: vec![RuleStep {
            rule_id,
            before: before.to_string(),
            after: text,
            reason: "attach the class and form specific adjective agreement ending",
        }],
    })
}

fn strip_citation<'a>(
    lemma: &'a str,
    endings: &[&str],
    class: &str,
) -> Result<&'a str, InflectionError> {
    endings
        .iter()
        .find_map(|ending| lemma.strip_suffix(ending))
        .filter(|stem| !stem.is_empty())
        .ok_or_else(|| InflectionError::InvalidInput {
            reason: format!("a {class} adjective citation has an incompatible ending"),
        })
}

fn hard_long_ending(cell: AdjectiveCell) -> &'static str {
    use Case::*;
    use Gender::*;
    use Number::*;
    match (cell.case, cell.number, cell.gender) {
        (Nominative | Vocative, Singular, Masculine) => "ꙑи",
        (Nominative | Accusative | Vocative, Singular, Neuter) => "оѥ",
        (Nominative | Vocative, Singular, Feminine) => "аꙗ",
        (Genitive, Singular, Masculine | Neuter) => "аѥго",
        (Genitive, Singular, Feminine) => "ꙑѩ",
        (Dative, Singular, Masculine | Neuter) => "оуѥмоу",
        (Dative | Locative, Singular, Feminine) => "ѣи",
        (Accusative, Singular, Masculine) if cell.animacy == Animacy::Animate => "аѥго",
        (Accusative, Singular, Masculine) => "ꙑи",
        (Accusative | Instrumental, Singular, Feminine) => "ѫѭ",
        (Instrumental, Singular, Masculine | Neuter) => "ꙑимь",
        (Locative, Singular, Masculine | Neuter) => "ѣѥмь",
        (Nominative | Accusative | Vocative, Dual, Masculine) => "аꙗ",
        (Nominative | Accusative | Vocative, Dual, Feminine | Neuter) => "ѣи",
        (Genitive | Locative, Dual, _) => "оую",
        (Dative | Instrumental, Dual, _) => "ꙑима",
        (Nominative | Vocative, Plural, Masculine) => "ии",
        (Nominative | Accusative | Vocative, Plural, Feminine) => "ꙑѩ",
        (Nominative | Accusative | Vocative, Plural, Neuter) => "аꙗ",
        (Genitive | Locative, Plural, _) => "ꙑихъ",
        (Dative, Plural, _) => "ꙑимъ",
        (Accusative, Plural, Masculine) if cell.animacy == Animacy::Animate => "ꙑихъ",
        (Accusative, Plural, Masculine) => "ꙑѩ",
        (Instrumental, Plural, _) => "ꙑими",
    }
}

fn soft_short_ending(cell: AdjectiveCell) -> &'static str {
    use Case::*;
    use Gender::*;
    use Number::*;
    match (cell.case, cell.number, cell.gender) {
        (Nominative, Singular, Masculine) => "ь",
        (Nominative | Vocative, Singular, Feminine) => "а",
        (Nominative | Accusative | Vocative, Singular, Neuter) => "е",
        (Genitive, Singular, Masculine | Neuter) => "а",
        (Genitive, Singular, Feminine) => "ѧ",
        (Dative, Singular, Masculine | Neuter) => "оу",
        (Dative | Locative, Singular, Feminine) => "и",
        (Accusative, Singular, Masculine) if cell.animacy == Animacy::Animate => "а",
        (Accusative, Singular, Masculine) => "ь",
        (Accusative, Singular, Feminine) => "ѫ",
        (Instrumental, Singular, Masculine | Neuter) => "емь",
        (Instrumental, Singular, Feminine) => "еѭ",
        (Locative, Singular, Masculine | Neuter) => "и",
        (Vocative, Singular, Masculine) => "е",
        (Nominative | Accusative | Vocative, Dual, Masculine) => "а",
        (Nominative | Accusative | Vocative, Dual, Feminine | Neuter) => "и",
        (Genitive | Locative, Dual, _) => "оу",
        (Dative | Instrumental, Dual, Masculine | Neuter) => "ема",
        (Dative | Instrumental, Dual, Feminine) => "ама",
        (Nominative | Vocative, Plural, Masculine) => "и",
        (Nominative | Accusative | Vocative, Plural, Feminine) => "ѧ",
        (Nominative | Accusative | Vocative, Plural, Neuter) => "а",
        (Genitive, Plural, _) => "ь",
        (Dative, Plural, Masculine | Neuter) => "емъ",
        (Dative, Plural, Feminine) => "амъ",
        (Accusative, Plural, Masculine) if cell.animacy == Animacy::Animate => "ь",
        (Accusative, Plural, Masculine) => "ѧ",
        (Instrumental, Plural, Masculine | Neuter) => "и",
        (Instrumental, Plural, Feminine) => "ами",
        (Locative, Plural, Masculine | Neuter) => "ихъ",
        (Locative, Plural, Feminine) => "ахъ",
    }
}

fn soft_long_ending(cell: AdjectiveCell) -> &'static str {
    use Case::*;
    use Gender::*;
    use Number::*;
    match (cell.case, cell.number, cell.gender) {
        (Nominative | Vocative, Singular, Masculine) => "ии",
        (Nominative | Accusative | Vocative, Singular, Neuter) => "еѥ",
        (Nominative | Vocative, Singular, Feminine) => "аꙗ",
        (Genitive, Singular, Masculine | Neuter) => "аѥго",
        (Genitive, Singular, Feminine) => "ѧѩ",
        (Dative, Singular, Masculine | Neuter) => "оуѥмоу",
        (Dative | Locative, Singular, Feminine) => "ии",
        (Accusative, Singular, Masculine) if cell.animacy == Animacy::Animate => "аѥго",
        (Accusative, Singular, Masculine) => "ии",
        (Accusative, Singular, Feminine) => "ѫѭ",
        (Instrumental, Singular, Masculine | Neuter) => "иимь",
        (Instrumental, Singular, Feminine) => "еѭ",
        (Locative, Singular, Masculine | Neuter) => "иѥмь",
        (Nominative | Accusative | Vocative, Dual, Masculine) => "аꙗ",
        (Nominative | Accusative | Vocative, Dual, Feminine | Neuter) => "ии",
        (Genitive | Locative, Dual, _) => "оую",
        (Dative | Instrumental, Dual, _) => "иима",
        (Nominative | Vocative, Plural, Masculine) => "ии",
        (Nominative | Accusative | Vocative, Plural, Feminine) => "ѧѩ",
        (Nominative | Accusative | Vocative, Plural, Neuter) => "аꙗ",
        (Genitive | Locative, Plural, _) => "иихъ",
        (Dative, Plural, _) => "иимъ",
        (Accusative, Plural, Masculine) if cell.animacy == Animacy::Animate => "иихъ",
        (Accusative, Plural, Masculine) => "ѧѩ",
        (Instrumental, Plural, _) => "иими",
    }
}

fn hard_short_ending(cell: AdjectiveCell) -> &'static str {
    use Case::*;
    use Gender::*;
    use Number::*;
    match (cell.case, cell.number, cell.gender) {
        (Nominative, Singular, Masculine) => "ъ",
        (Nominative, Singular, Feminine) => "а",
        (Nominative | Accusative | Vocative, Singular, Neuter) => "о",
        (Genitive, Singular, Masculine | Neuter) => "а",
        (Genitive, Singular, Feminine) => "ꙑ",
        (Dative, Singular, Masculine | Neuter) => "оу",
        (Dative | Locative, Singular, Feminine) => "ѣ",
        (Accusative, Singular, Masculine) if cell.animacy == Animacy::Animate => "а",
        (Accusative, Singular, Masculine) => "ъ",
        (Accusative, Singular, Feminine) => "ѫ",
        (Instrumental, Singular, Masculine | Neuter) => "омь",
        (Instrumental, Singular, Feminine) => "оѭ",
        (Locative, Singular, Masculine | Neuter) => "ѣ",
        (Vocative, Singular, Masculine) => "е",
        (Vocative, Singular, Feminine) => "о",
        (Nominative | Accusative | Vocative, Dual, Masculine) => "а",
        (Nominative | Accusative | Vocative, Dual, Feminine | Neuter) => "ѣ",
        (Genitive | Locative, Dual, _) => "оу",
        (Dative | Instrumental, Dual, Masculine | Neuter) => "ома",
        (Dative | Instrumental, Dual, Feminine) => "ама",
        (Nominative | Vocative, Plural, Masculine) => "и",
        (Nominative | Accusative | Vocative, Plural, Feminine) => "ꙑ",
        (Nominative | Accusative | Vocative, Plural, Neuter) => "а",
        (Genitive, Plural, _) => "ъ",
        (Dative, Plural, Masculine | Neuter) => "омъ",
        (Dative, Plural, Feminine) => "амъ",
        (Accusative, Plural, Masculine) if cell.animacy == Animacy::Animate => "ъ",
        (Accusative, Plural, Masculine) => "ꙑ",
        (Instrumental, Plural, Masculine | Neuter) => "ꙑ",
        (Instrumental, Plural, Feminine) => "ами",
        (Locative, Plural, Masculine | Neuter) => "ѣхъ",
        (Locative, Plural, Feminine) => "ахъ",
    }
}

fn palatalize<const N: usize>(stem: &str, replacements: [(char, &str); N]) -> String {
    let Some(last) = stem.chars().last() else {
        return String::new();
    };
    let Some((_, replacement)) = replacements.iter().find(|(from, _)| *from == last) else {
        return stem.to_string();
    };
    let prefix_len = stem.len() - last.len_utf8();
    format!("{}{replacement}", &stem[..prefix_len])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hard_short_agreement_includes_dual() {
        let mal = AdjectiveLexeme {
            lemma: "малъ".to_string(),
            class: AdjectiveClass::Hard,
        };
        let form = decline(
            &mal,
            AdjectiveCell {
                case: Case::Nominative,
                number: Number::Dual,
                gender: Gender::Feminine,
                animacy: Animacy::Inanimate,
                form: AdjectiveForm::Short,
            },
        )
        .expect("supported");
        assert_eq!(form.text, "малѣ");
    }

    #[test]
    fn hard_short_velar_seams_palatalize() {
        let drug = AdjectiveLexeme {
            lemma: "дроугъ".to_string(),
            class: AdjectiveClass::Hard,
        };
        let form = decline(
            &drug,
            AdjectiveCell {
                case: Case::Dative,
                number: Number::Singular,
                gender: Gender::Feminine,
                animacy: Animacy::Inanimate,
                form: AdjectiveForm::Short,
            },
        )
        .expect("supported");
        assert_eq!(form.text, "дроуѕѣ");
    }

    #[test]
    fn long_and_soft_paradigms_are_distinct() {
        let hard = AdjectiveLexeme {
            lemma: "добръ".to_string(),
            class: AdjectiveClass::Hard,
        };
        let long = decline(
            &hard,
            AdjectiveCell {
                case: Case::Nominative,
                number: Number::Singular,
                gender: Gender::Masculine,
                animacy: Animacy::Inanimate,
                form: AdjectiveForm::Long,
            },
        )
        .expect("hard long");
        assert_eq!(long.text, "добрꙑи");

        let soft = AdjectiveLexeme {
            lemma: "синь".to_string(),
            class: AdjectiveClass::Soft,
        };
        let soft_long = decline(
            &soft,
            AdjectiveCell {
                case: Case::Nominative,
                number: Number::Singular,
                gender: Gender::Masculine,
                animacy: Animacy::Inanimate,
                form: AdjectiveForm::Long,
            },
        )
        .expect("soft long");
        assert_eq!(soft_long.text, "синии");
    }

    #[test]
    fn invalid_citations_and_hostile_lemmas_are_typed() {
        let cell = AdjectiveCell {
            case: Case::Nominative,
            number: Number::Singular,
            gender: Gender::Masculine,
            animacy: Animacy::Inanimate,
            form: AdjectiveForm::Short,
        };
        for lemma in ["", "добр ъ", "добръ\0", "ъ"] {
            let result = decline(
                &AdjectiveLexeme {
                    lemma: lemma.to_string(),
                    class: AdjectiveClass::Hard,
                },
                cell,
            );
            assert!(matches!(result, Err(InflectionError::InvalidInput { .. })));
        }
    }

    #[test]
    fn masculine_accusative_animacy_is_not_collapsed() {
        let adjective = AdjectiveLexeme {
            lemma: "добръ".to_string(),
            class: AdjectiveClass::Hard,
        };
        let cell = AdjectiveCell {
            case: Case::Accusative,
            number: Number::Singular,
            gender: Gender::Masculine,
            animacy: Animacy::Inanimate,
            form: AdjectiveForm::Short,
        };
        assert_eq!(decline(&adjective, cell).expect("inanimate").text, "добръ");
        assert_eq!(
            decline(
                &adjective,
                AdjectiveCell {
                    animacy: Animacy::Animate,
                    ..cell
                }
            )
            .expect("animate")
            .text,
            "добра"
        );
    }

    fn cell(
        form: AdjectiveForm,
        case: Case,
        number: Number,
        gender: Gender,
        animacy: Animacy,
    ) -> AdjectiveCell {
        AdjectiveCell {
            case,
            number,
            gender,
            animacy,
            form,
        }
    }

    #[test]
    fn productive_new_comparative_forms_velar_and_nonvelar_principal_parts() {
        for (lemma, expected_syncopated) in [
            ("новъ", "новѣи"),
            ("горькъ", "горьчаи"),
            ("драгъ", "дражаи"),
            ("тихъ", "тишаи"),
        ] {
            let comparative = productive_new_comparative(&AdjectiveLexeme {
                lemma: lemma.to_string(),
                class: AdjectiveClass::Hard,
            })
            .expect("productive new comparative");
            assert_eq!(comparative.syncopated_citation, expected_syncopated);
            assert_eq!(
                comparative.expanded_citation,
                format!("{expected_syncopated}ши")
            );
        }
    }

    #[test]
    fn new_comparative_has_all_syncopated_and_alien_terminal_cells() {
        let new = productive_new_comparative(&AdjectiveLexeme {
            lemma: "новъ".to_string(),
            class: AdjectiveClass::Hard,
        })
        .expect("productive new comparative");
        let examples = [
            (
                cell(
                    AdjectiveForm::Short,
                    Case::Nominative,
                    Number::Singular,
                    Gender::Masculine,
                    Animacy::Inanimate,
                ),
                "новѣи",
            ),
            (
                cell(
                    AdjectiveForm::Short,
                    Case::Nominative,
                    Number::Singular,
                    Gender::Neuter,
                    Animacy::Inanimate,
                ),
                "новѣѥ",
            ),
            (
                cell(
                    AdjectiveForm::Long,
                    Case::Nominative,
                    Number::Singular,
                    Gender::Masculine,
                    Animacy::Inanimate,
                ),
                "новѣии",
            ),
            (
                cell(
                    AdjectiveForm::Short,
                    Case::Nominative,
                    Number::Singular,
                    Gender::Feminine,
                    Animacy::Inanimate,
                ),
                "новѣиши",
            ),
            (
                cell(
                    AdjectiveForm::Long,
                    Case::Nominative,
                    Number::Singular,
                    Gender::Feminine,
                    Animacy::Inanimate,
                ),
                "новѣишиꙗ",
            ),
            (
                cell(
                    AdjectiveForm::Short,
                    Case::Nominative,
                    Number::Plural,
                    Gender::Masculine,
                    Animacy::Inanimate,
                ),
                "новѣише",
            ),
            (
                cell(
                    AdjectiveForm::Long,
                    Case::Nominative,
                    Number::Plural,
                    Gender::Masculine,
                    Animacy::Inanimate,
                ),
                "новѣишеи",
            ),
        ];
        for (cell, expected) in examples {
            assert_eq!(
                decline_comparative(&new, cell)
                    .expect("source-described comparative cell")
                    .text,
                expected
            );
        }
    }

    #[test]
    fn old_comparative_uses_its_independent_softened_principal_parts() {
        let old = ComparativeLexeme {
            positive_lemma: "грѫбъ".to_string(),
            syncopated_citation: "грѫбл҄ь".to_string(),
            expanded_citation: "грѫбл҄ьши".to_string(),
            formation: ComparativeFormation::Old,
        };
        for (cell, expected) in [
            (
                cell(
                    AdjectiveForm::Short,
                    Case::Nominative,
                    Number::Singular,
                    Gender::Masculine,
                    Animacy::Inanimate,
                ),
                "грѫбл҄ь",
            ),
            (
                cell(
                    AdjectiveForm::Short,
                    Case::Nominative,
                    Number::Singular,
                    Gender::Neuter,
                    Animacy::Inanimate,
                ),
                "грѫбл҄е",
            ),
            (
                cell(
                    AdjectiveForm::Long,
                    Case::Nominative,
                    Number::Singular,
                    Gender::Masculine,
                    Animacy::Inanimate,
                ),
                "грѫбл҄ии",
            ),
            (
                cell(
                    AdjectiveForm::Short,
                    Case::Genitive,
                    Number::Singular,
                    Gender::Masculine,
                    Animacy::Inanimate,
                ),
                "грѫбл҄ьша",
            ),
        ] {
            let predicted = decline_comparative(&old, cell).expect("old comparative");
            assert_eq!(predicted.text, expected);
            assert_eq!(predicted.rule_id, RuleId::AdjectiveComparativeOld);
        }
    }

    #[test]
    fn comparative_inventory_is_exhaustive_and_keeps_accusative_animacy() {
        let comparative = productive_new_comparative(&AdjectiveLexeme {
            lemma: "новъ".to_string(),
            class: AdjectiveClass::Hard,
        })
        .expect("new comparative");
        let forms = AdjectiveCell::all()
            .map(|cell| decline_comparative(&comparative, cell).expect("complete cell"))
            .collect::<Vec<_>>();
        assert_eq!(forms.len(), 252);

        let inanimate = decline_comparative(
            &comparative,
            cell(
                AdjectiveForm::Short,
                Case::Accusative,
                Number::Singular,
                Gender::Masculine,
                Animacy::Inanimate,
            ),
        )
        .expect("inanimate accusative");
        let animate = decline_comparative(
            &comparative,
            cell(
                AdjectiveForm::Short,
                Case::Accusative,
                Number::Singular,
                Gender::Masculine,
                Animacy::Animate,
            ),
        )
        .expect("animate accusative");
        assert_eq!(inanimate.text, "новѣи");
        assert_eq!(animate.text, "новѣиша");
    }

    #[test]
    fn contradictory_comparative_principal_parts_are_rejected() {
        for lexeme in [
            ComparativeLexeme {
                positive_lemma: "новъ".to_string(),
                syncopated_citation: "новѣь".to_string(),
                expanded_citation: "новѣиши".to_string(),
                formation: ComparativeFormation::New,
            },
            ComparativeLexeme {
                positive_lemma: "грѫбъ".to_string(),
                syncopated_citation: "грѫбл҄ь".to_string(),
                expanded_citation: "грѫбьши".to_string(),
                formation: ComparativeFormation::Old,
            },
        ] {
            assert!(matches!(
                decline_comparative(
                    &lexeme,
                    cell(
                        AdjectiveForm::Short,
                        Case::Nominative,
                        Number::Singular,
                        Gender::Masculine,
                        Animacy::Inanimate,
                    ),
                ),
                Err(InflectionError::InvalidInput { .. })
            ));
        }
    }
}
