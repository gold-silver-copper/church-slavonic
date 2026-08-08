//! Rule-based adjective declension.

use crate::{
    AdjectiveCell, AdjectiveClass, AdjectiveForm, Animacy, Case, Gender, InflectionError, Number,
    PredictedForm, RuleId, RuleStep,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdjectiveLexeme {
    pub lemma: String,
    pub class: AdjectiveClass,
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
}
