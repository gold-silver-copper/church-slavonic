//! Rule-based noun declension.

use crate::{
    Animacy, Case, Gender, InflectionError, NounCell, NounClass, Number, NumberRestriction,
    PredictedForm, RuleId, RuleStep,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NounLexeme {
    pub lemma: String,
    pub class: NounClass,
    pub gender: Gender,
    pub animacy: Animacy,
    pub number_restriction: NumberRestriction,
}

pub fn decline(lexeme: &NounLexeme, cell: NounCell) -> Result<PredictedForm, InflectionError> {
    let normalized_lexeme = NounLexeme {
        lemma: crate::orthography::canonical_display(&lexeme.lemma)?,
        class: lexeme.class,
        gender: lexeme.gender,
        animacy: lexeme.animacy,
        number_restriction: lexeme.number_restriction,
    };
    let lexeme = &normalized_lexeme;
    enforce_number(&lexeme.lemma, lexeme.number_restriction, cell)?;
    match lexeme.class {
        NounClass::OMasculineHard => decline_o_masculine_hard(lexeme, cell),
        NounClass::ONeuterHard => decline_o_neuter_hard(lexeme, cell),
        NounClass::JoMasculineSoft => decline_jo_masculine_soft(lexeme, cell),
        NounClass::JoNeuterSoft => decline_jo_neuter_soft(lexeme, cell),
        NounClass::AHard => decline_a_hard(lexeme, cell),
        NounClass::JaSoft => decline_ja_soft(lexeme, cell),
        NounClass::IFeminine => decline_i_stem(lexeme, cell, false),
        NounClass::IMasculine => decline_i_stem(lexeme, cell, true),
        NounClass::UMasculine => decline_u_masculine(lexeme, cell),
        NounClass::NMasculine => decline_n_masculine(lexeme, cell),
        NounClass::NNeuter => decline_n_neuter(lexeme, cell),
        NounClass::NtNeuter => decline_nt_neuter(lexeme, cell),
        NounClass::RStem => decline_r_stem(lexeme, cell),
        NounClass::SNeuter => decline_s_neuter(lexeme, cell),
        NounClass::VFeminine => decline_v_feminine(lexeme, cell),
        NounClass::Indeclinable => Ok(predicted(
            &lexeme.lemma,
            &lexeme.lemma,
            RuleId::NounIndeclinable,
            "the lexeme is explicitly marked indeclinable",
        )),
    }
}

fn decline_jo_masculine_soft(
    lexeme: &NounLexeme,
    cell: NounCell,
) -> Result<PredictedForm, InflectionError> {
    let stem = strip_any_required(&lexeme.lemma, &["ь", "и"])?;
    let sibilant = ends_in_sibilant(stem);
    let (back, front) = if sibilant {
        ("а", "оу")
    } else {
        ("ꙗ", "ю")
    };
    let ending = match (cell.case, cell.number) {
        (Case::Nominative, Number::Singular) => "ь",
        (Case::Genitive, Number::Singular) => back,
        (Case::Dative, Number::Singular) => front,
        (Case::Accusative, Number::Singular) if lexeme.animacy == Animacy::Animate => back,
        (Case::Accusative, Number::Singular) => "ь",
        (Case::Instrumental, Number::Singular) => "емь",
        (Case::Locative, Number::Singular) => "и",
        (Case::Vocative, Number::Singular) => front,
        (Case::Nominative | Case::Accusative | Case::Vocative, Number::Dual) => back,
        (Case::Genitive | Case::Locative, Number::Dual) => front,
        (Case::Dative | Case::Instrumental, Number::Dual) => "ема",
        (Case::Nominative | Case::Vocative, Number::Plural) => "и",
        (Case::Genitive, Number::Plural) => "ь",
        (Case::Dative, Number::Plural) => "емъ",
        (Case::Accusative, Number::Plural) if lexeme.animacy == Animacy::Animate => "ь",
        (Case::Accusative, Number::Plural) => "ѧ",
        (Case::Instrumental, Number::Plural) => "и",
        (Case::Locative, Number::Plural) => "ихъ",
    };
    Ok(join(
        stem,
        ending,
        Mutation::None,
        RuleId::NounJoMasculineSoft,
    ))
}

fn decline_jo_neuter_soft(
    lexeme: &NounLexeme,
    cell: NounCell,
) -> Result<PredictedForm, InflectionError> {
    let stem = strip_any_required(&lexeme.lemma, &["е", "ѥ"])?;
    let ending = match (cell.case, cell.number) {
        (Case::Nominative | Case::Accusative | Case::Vocative, Number::Singular) => "ѥ",
        (Case::Genitive, Number::Singular) => "ꙗ",
        (Case::Dative, Number::Singular) => "ю",
        (Case::Instrumental, Number::Singular) => "емь",
        (Case::Locative, Number::Singular) => "и",
        (Case::Nominative | Case::Accusative | Case::Vocative, Number::Dual) => "и",
        (Case::Genitive | Case::Locative, Number::Dual) => "ю",
        (Case::Dative | Case::Instrumental, Number::Dual) => "ема",
        (Case::Nominative | Case::Accusative | Case::Vocative, Number::Plural) => "ꙗ",
        (Case::Genitive, Number::Plural) => "ь",
        (Case::Dative, Number::Plural) => "емъ",
        (Case::Instrumental, Number::Plural) => "и",
        (Case::Locative, Number::Plural) => "ихъ",
    };
    Ok(join(stem, ending, Mutation::None, RuleId::NounJoNeuterSoft))
}

fn decline_ja_soft(lexeme: &NounLexeme, cell: NounCell) -> Result<PredictedForm, InflectionError> {
    let stem = strip_any_required(&lexeme.lemma, &["ꙗ", "и"])?;
    let ending = match (cell.case, cell.number) {
        (Case::Nominative, Number::Singular) => "ꙗ",
        (Case::Genitive | Case::Dative | Case::Locative, Number::Singular) => "и",
        (Case::Accusative, Number::Singular) => "ѭ",
        (Case::Instrumental, Number::Singular) => "еѭ",
        (Case::Vocative, Number::Singular) => "е",
        (Case::Nominative | Case::Accusative | Case::Vocative, Number::Dual) => "и",
        (Case::Genitive | Case::Locative, Number::Dual) => "ю",
        (Case::Dative | Case::Instrumental, Number::Dual) => "ꙗма",
        (Case::Nominative | Case::Accusative | Case::Vocative, Number::Plural) => "ѩ",
        (Case::Genitive, Number::Plural) => "ь",
        (Case::Dative, Number::Plural) => "ꙗмъ",
        (Case::Instrumental, Number::Plural) => "ꙗми",
        (Case::Locative, Number::Plural) => "ꙗхъ",
    };
    Ok(join(stem, ending, Mutation::None, RuleId::NounJaSoft))
}

fn decline_i_stem(
    lexeme: &NounLexeme,
    cell: NounCell,
    masculine: bool,
) -> Result<PredictedForm, InflectionError> {
    let stem = strip_required(&lexeme.lemma, 'ь')?;
    let ending = match (cell.case, cell.number) {
        (Case::Nominative | Case::Accusative, Number::Singular) => "ь",
        (Case::Genitive | Case::Dative | Case::Locative | Case::Vocative, Number::Singular) => "и",
        (Case::Instrumental, Number::Singular) if masculine => "ьмь",
        (Case::Instrumental, Number::Singular) => "ьѭ",
        (Case::Nominative | Case::Accusative | Case::Vocative, Number::Dual) => "и",
        (Case::Genitive | Case::Locative, Number::Dual) => "ью",
        (Case::Dative | Case::Instrumental, Number::Dual) => "ьма",
        (Case::Nominative | Case::Vocative, Number::Plural) if masculine => "ьѥ",
        (Case::Nominative | Case::Vocative, Number::Plural) => "и",
        (Case::Genitive, Number::Plural) => "ьи",
        (Case::Dative, Number::Plural) => "ьмъ",
        (Case::Accusative, Number::Plural) => "и",
        (Case::Instrumental, Number::Plural) => "ьми",
        (Case::Locative, Number::Plural) => "ьхъ",
    };
    Ok(join(
        stem,
        ending,
        Mutation::None,
        if masculine {
            RuleId::NounIMasculine
        } else {
            RuleId::NounIFeminine
        },
    ))
}

fn decline_u_masculine(
    lexeme: &NounLexeme,
    cell: NounCell,
) -> Result<PredictedForm, InflectionError> {
    let stem = strip_required(&lexeme.lemma, 'ъ')?;
    let ending = match (cell.case, cell.number) {
        (Case::Nominative | Case::Accusative, Number::Singular) => "ъ",
        (Case::Genitive | Case::Locative | Case::Vocative, Number::Singular) => "оу",
        (Case::Dative, Number::Singular) => "ови",
        (Case::Instrumental, Number::Singular) => "ъмь",
        (Case::Nominative | Case::Accusative | Case::Vocative, Number::Dual) => "ꙑ",
        (Case::Genitive | Case::Locative, Number::Dual) => "овоу",
        (Case::Dative | Case::Instrumental, Number::Dual) => "ъма",
        (Case::Nominative | Case::Vocative, Number::Plural) => "ове",
        (Case::Genitive, Number::Plural) => "овъ",
        (Case::Dative, Number::Plural) => "ъмъ",
        (Case::Accusative, Number::Plural) => "ꙑ",
        (Case::Instrumental, Number::Plural) => "ъми",
        (Case::Locative, Number::Plural) => "ъхъ",
    };
    Ok(join(stem, ending, Mutation::None, RuleId::NounUMasculine))
}

fn decline_n_masculine(
    lexeme: &NounLexeme,
    cell: NounCell,
) -> Result<PredictedForm, InflectionError> {
    let stem = strip_required(&lexeme.lemma, 'ꙑ')?;
    let ending = consonant_ending(
        cell,
        "ꙑ",
        "ене",
        "ени",
        "ꙑ",
        "еньмь",
        "ене",
        "ꙑ",
        "ени",
        "еноу",
        "еньма",
        "ени",
        "еньма",
        "еноу",
        "ени",
        "ене",
        "енъ",
        "еньмъ",
        "ени",
        "еньми",
        "еньхъ",
        "ене",
    );
    Ok(join(stem, ending, Mutation::None, RuleId::NounNMasculine))
}

fn decline_n_neuter(lexeme: &NounLexeme, cell: NounCell) -> Result<PredictedForm, InflectionError> {
    let stem = strip_required(&lexeme.lemma, 'ѧ')?;
    let ending = consonant_ending(
        cell,
        "ѧ",
        "ене",
        "ени",
        "ѧ",
        "еньмь",
        "ене",
        "ѧ",
        "енѣ",
        "еноу",
        "еньма",
        "енѣ",
        "еньма",
        "еноу",
        "енѣ",
        "ена",
        "енъ",
        "еньмъ",
        "ена",
        "енꙑ",
        "еньхъ",
        "ена",
    );
    Ok(join(stem, ending, Mutation::None, RuleId::NounNNeuter))
}

fn decline_nt_neuter(
    lexeme: &NounLexeme,
    cell: NounCell,
) -> Result<PredictedForm, InflectionError> {
    let stem = strip_required(&lexeme.lemma, 'ѧ')?;
    let ending = consonant_ending(
        cell,
        "ѧ",
        "ѧте",
        "ѧти",
        "ѧ",
        "ѧтьмь",
        "ѧте",
        "ѧ",
        "ѧтѣ",
        "ѧтоу",
        "ѧтьма",
        "ѧтѣ",
        "ѧтьма",
        "ѧтоу",
        "ѧтѣ",
        "ѧта",
        "ѧтъ",
        "ѧтьмъ",
        "ѧта",
        "ѧтꙑ",
        "ѧтьхъ",
        "ѧта",
    );
    Ok(join(stem, ending, Mutation::None, RuleId::NounNtNeuter))
}

fn decline_r_stem(lexeme: &NounLexeme, cell: NounCell) -> Result<PredictedForm, InflectionError> {
    let stem = strip_required(&lexeme.lemma, 'и')?;
    let ending = consonant_ending(
        cell,
        "и",
        "ере",
        "ери",
        "ерь",
        "ерьѭ",
        "ери",
        "и",
        "ери",
        "ероу",
        "ерьма",
        "ери",
        "ерьма",
        "ероу",
        "ери",
        "ери",
        "еръ",
        "ерьмъ",
        "ери",
        "ерьми",
        "ерьхъ",
        "ери",
    );
    Ok(join(stem, ending, Mutation::None, RuleId::NounRStem))
}

fn decline_s_neuter(lexeme: &NounLexeme, cell: NounCell) -> Result<PredictedForm, InflectionError> {
    let stem = strip_required(&lexeme.lemma, 'о')?;
    let ending = consonant_ending(
        cell,
        "о",
        "есе",
        "еси",
        "о",
        "есьмь",
        "есе",
        "о",
        "есѣ",
        "есоу",
        "есьма",
        "есѣ",
        "есьма",
        "есоу",
        "есѣ",
        "еса",
        "есъ",
        "есьмъ",
        "еса",
        "есꙑ",
        "есьхъ",
        "еса",
    );
    Ok(join(stem, ending, Mutation::None, RuleId::NounSNeuter))
}

fn decline_v_feminine(
    lexeme: &NounLexeme,
    cell: NounCell,
) -> Result<PredictedForm, InflectionError> {
    let stem = strip_required(&lexeme.lemma, 'ꙑ')?;
    let ending = consonant_ending(
        cell,
        "ꙑ",
        "ъве",
        "ъви",
        "ъвь",
        "ъвьѭ",
        "ъве",
        "ꙑ",
        "ъви",
        "ъвоу",
        "ъвама",
        "ъви",
        "ъвама",
        "ъвоу",
        "ъви",
        "ъви",
        "ъвъ",
        "ъвамъ",
        "ъви",
        "ъвами",
        "ъвахъ",
        "ъви",
    );
    Ok(join(stem, ending, Mutation::None, RuleId::NounVFeminine))
}

fn enforce_number(
    lemma: &str,
    restriction: NumberRestriction,
    cell: NounCell,
) -> Result<(), InflectionError> {
    let supported = match restriction {
        NumberRestriction::All => true,
        NumberRestriction::SingularOnly => cell.number == Number::Singular,
        NumberRestriction::DualOnly => cell.number == Number::Dual,
        NumberRestriction::PluralOnly => cell.number == Number::Plural,
    };
    if supported {
        Ok(())
    } else {
        Err(InflectionError::unsupported(
            lemma,
            crate::RequestedCell::Noun(cell),
        ))
    }
}

fn decline_o_masculine_hard(
    lexeme: &NounLexeme,
    cell: NounCell,
) -> Result<PredictedForm, InflectionError> {
    let stem = strip_required(&lexeme.lemma, 'ъ')?;
    let (ending, mutation) = match (cell.case, cell.number) {
        (Case::Nominative, Number::Singular) => ("ъ", Mutation::None),
        (Case::Genitive, Number::Singular) => ("а", Mutation::None),
        (Case::Dative, Number::Singular) => ("оу", Mutation::None),
        (Case::Accusative, Number::Singular) if lexeme.animacy == Animacy::Animate => {
            ("а", Mutation::None)
        }
        (Case::Accusative, Number::Singular) => ("ъ", Mutation::None),
        (Case::Instrumental, Number::Singular) => ("омъ", Mutation::None),
        (Case::Locative, Number::Singular) => ("ѣ", Mutation::SecondPalatalization),
        (Case::Vocative, Number::Singular) => ("е", Mutation::FirstPalatalization),
        (Case::Nominative | Case::Accusative | Case::Vocative, Number::Dual) => {
            ("а", Mutation::None)
        }
        (Case::Genitive | Case::Locative, Number::Dual) => ("оу", Mutation::None),
        (Case::Dative | Case::Instrumental, Number::Dual) => ("ома", Mutation::None),
        (Case::Nominative | Case::Vocative, Number::Plural) => {
            ("и", Mutation::SecondPalatalization)
        }
        (Case::Genitive, Number::Plural) => ("ъ", Mutation::None),
        (Case::Dative, Number::Plural) => ("омъ", Mutation::None),
        (Case::Accusative, Number::Plural) if lexeme.animacy == Animacy::Animate => {
            ("ъ", Mutation::None)
        }
        (Case::Accusative | Case::Instrumental, Number::Plural) => ("ꙑ", Mutation::None),
        (Case::Locative, Number::Plural) => ("ѣхъ", Mutation::SecondPalatalization),
    };
    Ok(join(stem, ending, mutation, RuleId::NounOMasculineHard))
}

fn decline_o_neuter_hard(
    lexeme: &NounLexeme,
    cell: NounCell,
) -> Result<PredictedForm, InflectionError> {
    let stem = strip_required(&lexeme.lemma, 'о')?;
    let (ending, mutation) = match (cell.case, cell.number) {
        (Case::Nominative | Case::Accusative | Case::Vocative, Number::Singular) => {
            ("о", Mutation::None)
        }
        (Case::Genitive, Number::Singular) => ("а", Mutation::None),
        (Case::Dative, Number::Singular) => ("оу", Mutation::None),
        (Case::Instrumental, Number::Singular) => ("омъ", Mutation::None),
        (Case::Locative, Number::Singular) => ("ѣ", Mutation::SecondPalatalization),
        (Case::Nominative | Case::Accusative | Case::Vocative, Number::Dual) => {
            ("ѣ", Mutation::SecondPalatalization)
        }
        (Case::Genitive | Case::Locative, Number::Dual) => ("оу", Mutation::None),
        (Case::Dative | Case::Instrumental, Number::Dual) => ("ома", Mutation::None),
        (Case::Nominative | Case::Accusative | Case::Vocative, Number::Plural) => {
            ("а", Mutation::None)
        }
        (Case::Genitive, Number::Plural) => ("ъ", Mutation::None),
        (Case::Dative, Number::Plural) => ("омъ", Mutation::None),
        (Case::Instrumental, Number::Plural) => ("ꙑ", Mutation::None),
        (Case::Locative, Number::Plural) => ("ѣхъ", Mutation::SecondPalatalization),
    };
    Ok(join(stem, ending, mutation, RuleId::NounONeuterHard))
}

fn decline_a_hard(lexeme: &NounLexeme, cell: NounCell) -> Result<PredictedForm, InflectionError> {
    let stem = strip_required(&lexeme.lemma, 'а')?;
    let (ending, mutation) = match (cell.case, cell.number) {
        (Case::Nominative, Number::Singular) => ("а", Mutation::None),
        (Case::Genitive, Number::Singular) => ("ꙑ", Mutation::None),
        (Case::Dative | Case::Locative, Number::Singular) => ("ѣ", Mutation::SecondPalatalization),
        (Case::Accusative, Number::Singular) => ("ѫ", Mutation::None),
        (Case::Instrumental, Number::Singular) => ("оѭ", Mutation::None),
        (Case::Vocative, Number::Singular) => ("о", Mutation::None),
        (Case::Nominative | Case::Accusative | Case::Vocative, Number::Dual) => {
            ("ѣ", Mutation::SecondPalatalization)
        }
        (Case::Genitive | Case::Locative, Number::Dual) => ("оу", Mutation::None),
        (Case::Dative | Case::Instrumental, Number::Dual) => ("ама", Mutation::None),
        (Case::Nominative | Case::Accusative | Case::Vocative, Number::Plural) => {
            ("ꙑ", Mutation::None)
        }
        (Case::Genitive, Number::Plural) => ("ъ", Mutation::None),
        (Case::Dative, Number::Plural) => ("амъ", Mutation::None),
        (Case::Instrumental, Number::Plural) => ("ами", Mutation::None),
        (Case::Locative, Number::Plural) => ("ахъ", Mutation::None),
    };
    Ok(join(stem, ending, mutation, RuleId::NounAHard))
}

fn strip_required(lemma: &str, ending: char) -> Result<&str, InflectionError> {
    lemma
        .strip_suffix(ending)
        .filter(|stem| !stem.is_empty())
        .ok_or_else(|| InflectionError::InvalidInput {
            reason: format!("lemma does not have the ending required by its class: {ending}"),
        })
}

fn strip_any_required<'a>(lemma: &'a str, endings: &[&str]) -> Result<&'a str, InflectionError> {
    endings
        .iter()
        .find_map(|ending| lemma.strip_suffix(ending))
        .filter(|stem| !stem.is_empty())
        .ok_or_else(|| InflectionError::InvalidInput {
            reason: format!(
                "lemma does not have an ending required by its class: {}",
                endings.join(" or ")
            ),
        })
}

fn ends_in_sibilant(stem: &str) -> bool {
    stem.chars()
        .last()
        .is_some_and(|letter| matches!(letter, 'ш' | 'щ' | 'ч' | 'ж' | 'ѕ' | 'ꙃ' | 'ц'))
}

#[allow(clippy::too_many_arguments)]
fn consonant_ending(
    cell: NounCell,
    sg_nom: &'static str,
    sg_gen: &'static str,
    sg_dat: &'static str,
    sg_acc: &'static str,
    sg_ins: &'static str,
    sg_loc: &'static str,
    sg_voc: &'static str,
    du_nom: &'static str,
    du_gen: &'static str,
    du_dat: &'static str,
    du_acc: &'static str,
    du_ins: &'static str,
    du_loc: &'static str,
    du_voc: &'static str,
    pl_nom: &'static str,
    pl_gen: &'static str,
    pl_dat: &'static str,
    pl_acc: &'static str,
    pl_ins: &'static str,
    pl_loc: &'static str,
    pl_voc: &'static str,
) -> &'static str {
    let endings = [
        [sg_nom, sg_gen, sg_dat, sg_acc, sg_ins, sg_loc, sg_voc],
        [du_nom, du_gen, du_dat, du_acc, du_ins, du_loc, du_voc],
        [pl_nom, pl_gen, pl_dat, pl_acc, pl_ins, pl_loc, pl_voc],
    ];
    let number = match cell.number {
        Number::Singular => 0,
        Number::Dual => 1,
        Number::Plural => 2,
    };
    let case = match cell.case {
        Case::Nominative => 0,
        Case::Genitive => 1,
        Case::Dative => 2,
        Case::Accusative => 3,
        Case::Instrumental => 4,
        Case::Locative => 5,
        Case::Vocative => 6,
    };
    endings[number][case]
}

#[derive(Debug, Clone, Copy)]
enum Mutation {
    None,
    FirstPalatalization,
    SecondPalatalization,
}

fn join(stem: &str, ending: &str, mutation: Mutation, rule_id: RuleId) -> PredictedForm {
    let changed = match mutation {
        Mutation::None => stem.to_string(),
        Mutation::FirstPalatalization => replace_final(stem, [('к', "ч"), ('г', "ж"), ('х', "ш")]),
        Mutation::SecondPalatalization => replace_final(stem, [('к', "ц"), ('г', "ѕ"), ('х', "с")]),
    };
    let output = format!("{changed}{ending}");
    let reason = match mutation {
        Mutation::None => "attach the class ending to the citation stem",
        Mutation::FirstPalatalization => "apply first velar palatalization before the ending",
        Mutation::SecondPalatalization => "apply second velar palatalization before the ending",
    };
    predicted(stem, &output, rule_id, reason)
}

fn replace_final<const N: usize>(stem: &str, replacements: [(char, &str); N]) -> String {
    let Some(last) = stem.chars().last() else {
        return String::new();
    };
    let Some((_, replacement)) = replacements.iter().find(|(from, _)| *from == last) else {
        return stem.to_string();
    };
    let prefix_len = stem.len() - last.len_utf8();
    format!("{}{replacement}", &stem[..prefix_len])
}

fn predicted(before: &str, after: &str, rule_id: RuleId, reason: &'static str) -> PredictedForm {
    PredictedForm {
        text: after.to_string(),
        rule_id,
        trace: vec![RuleStep {
            rule_id,
            before: before.to_string(),
            after: after.to_string(),
            reason,
        }],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn obedu() -> NounLexeme {
        NounLexeme {
            lemma: "обѣдъ".to_string(),
            class: NounClass::OMasculineHard,
            gender: Gender::Masculine,
            animacy: Animacy::Inanimate,
            number_restriction: NumberRestriction::All,
        }
    }

    #[test]
    fn hard_o_masculine_has_dual_and_all_cases() {
        assert_eq!(
            decline(
                &obedu(),
                NounCell {
                    case: Case::Dative,
                    number: Number::Dual,
                }
            )
            .expect("supported")
            .text,
            "обѣдома"
        );
        assert_eq!(
            decline(
                &obedu(),
                NounCell {
                    case: Case::Locative,
                    number: Number::Plural,
                }
            )
            .expect("supported")
            .text,
            "обѣдѣхъ"
        );
    }

    #[test]
    fn velars_palatalize_at_the_documented_seam() {
        let drugu = NounLexeme {
            lemma: "дрꙋгъ".to_string(),
            ..obedu()
        };
        assert_eq!(
            decline(
                &drugu,
                NounCell {
                    case: Case::Vocative,
                    number: Number::Singular,
                }
            )
            .expect("supported")
            .text,
            "дрꙋже"
        );
    }

    #[test]
    fn every_documented_noun_class_matches_a_twenty_one_cell_golden() {
        // Audited against the template revisions recorded in docs/MORPHOLOGY_SPEC.md.
        // Order is singular, dual, plural; within each number, Case::ALL.
        let fixtures = [
            (
                "обѣдъ",
                NounClass::OMasculineHard,
                Gender::Masculine,
                "обѣдъ|обѣда|обѣдоу|обѣдъ|обѣдомъ|обѣдѣ|обѣде|обѣда|обѣдоу|обѣдома|обѣда|обѣдома|обѣдоу|обѣда|обѣди|обѣдъ|обѣдомъ|обѣдꙑ|обѣдꙑ|обѣдѣхъ|обѣди",
            ),
            (
                "слово",
                NounClass::ONeuterHard,
                Gender::Neuter,
                "слово|слова|словоу|слово|словомъ|словѣ|слово|словѣ|словоу|словома|словѣ|словома|словоу|словѣ|слова|словъ|словомъ|слова|словꙑ|словѣхъ|слова",
            ),
            (
                "конь",
                NounClass::JoMasculineSoft,
                Gender::Masculine,
                "конь|конꙗ|коню|конь|конемь|кони|коню|конꙗ|коню|конема|конꙗ|конема|коню|конꙗ|кони|конь|конемъ|конѧ|кони|конихъ|кони",
            ),
            (
                "полѥ",
                NounClass::JoNeuterSoft,
                Gender::Neuter,
                "полѥ|полꙗ|полю|полѥ|полемь|поли|полѥ|поли|полю|полема|поли|полема|полю|поли|полꙗ|поль|полемъ|полꙗ|поли|полихъ|полꙗ",
            ),
            (
                "жена",
                NounClass::AHard,
                Gender::Feminine,
                "жена|женꙑ|женѣ|женѫ|женоѭ|женѣ|жено|женѣ|женоу|женама|женѣ|женама|женоу|женѣ|женꙑ|женъ|женамъ|женꙑ|женами|женахъ|женꙑ",
            ),
            (
                "волꙗ",
                NounClass::JaSoft,
                Gender::Feminine,
                "волꙗ|воли|воли|волѭ|волеѭ|воли|воле|воли|волю|волꙗма|воли|волꙗма|волю|воли|волѩ|воль|волꙗмъ|волѩ|волꙗми|волꙗхъ|волѩ",
            ),
            (
                "кость",
                NounClass::IFeminine,
                Gender::Feminine,
                "кость|кости|кости|кость|костьѭ|кости|кости|кости|костью|костьма|кости|костьма|костью|кости|кости|костьи|костьмъ|кости|костьми|костьхъ|кости",
            ),
            (
                "пѫть",
                NounClass::IMasculine,
                Gender::Masculine,
                "пѫть|пѫти|пѫти|пѫть|пѫтьмь|пѫти|пѫти|пѫти|пѫтью|пѫтьма|пѫти|пѫтьма|пѫтью|пѫти|пѫтьѥ|пѫтьи|пѫтьмъ|пѫти|пѫтьми|пѫтьхъ|пѫтьѥ",
            ),
            (
                "сꙑнъ",
                NounClass::UMasculine,
                Gender::Masculine,
                "сꙑнъ|сꙑноу|сꙑнови|сꙑнъ|сꙑнъмь|сꙑноу|сꙑноу|сꙑнꙑ|сꙑновоу|сꙑнъма|сꙑнꙑ|сꙑнъма|сꙑновоу|сꙑнꙑ|сꙑнове|сꙑновъ|сꙑнъмъ|сꙑнꙑ|сꙑнъми|сꙑнъхъ|сꙑнове",
            ),
            (
                "камꙑ",
                NounClass::NMasculine,
                Gender::Masculine,
                "камꙑ|камене|камени|камꙑ|каменьмь|камене|камꙑ|камени|каменоу|каменьма|камени|каменьма|каменоу|камени|камене|каменъ|каменьмъ|камени|каменьми|каменьхъ|камене",
            ),
            (
                "имѧ",
                NounClass::NNeuter,
                Gender::Neuter,
                "имѧ|имене|имени|имѧ|именьмь|имене|имѧ|именѣ|именоу|именьма|именѣ|именьма|именоу|именѣ|имена|именъ|именьмъ|имена|именꙑ|именьхъ|имена",
            ),
            (
                "агнѧ",
                NounClass::NtNeuter,
                Gender::Neuter,
                "агнѧ|агнѧте|агнѧти|агнѧ|агнѧтьмь|агнѧте|агнѧ|агнѧтѣ|агнѧтоу|агнѧтьма|агнѧтѣ|агнѧтьма|агнѧтоу|агнѧтѣ|агнѧта|агнѧтъ|агнѧтьмъ|агнѧта|агнѧтꙑ|агнѧтьхъ|агнѧта",
            ),
            (
                "мати",
                NounClass::RStem,
                Gender::Feminine,
                "мати|матере|матери|матерь|матерьѭ|матери|мати|матери|матероу|матерьма|матери|матерьма|матероу|матери|матери|матеръ|матерьмъ|матери|матерьми|матерьхъ|матери",
            ),
            (
                "слово",
                NounClass::SNeuter,
                Gender::Neuter,
                "слово|словесе|словеси|слово|словесьмь|словесе|слово|словесѣ|словесоу|словесьма|словесѣ|словесьма|словесоу|словесѣ|словеса|словесъ|словесьмъ|словеса|словесꙑ|словесьхъ|словеса",
            ),
            (
                "црькꙑ",
                NounClass::VFeminine,
                Gender::Feminine,
                "црькꙑ|црькъве|црькъви|црькъвь|црькъвьѭ|црькъве|црькꙑ|црькъви|црькъвоу|црькъвама|црькъви|црькъвама|црькъвоу|црькъви|црькъви|црькъвъ|црькъвамъ|црькъви|црькъвами|црькъвахъ|црькъви",
            ),
            (
                "аминь",
                NounClass::Indeclinable,
                Gender::Masculine,
                "аминь|аминь|аминь|аминь|аминь|аминь|аминь|аминь|аминь|аминь|аминь|аминь|аминь|аминь|аминь|аминь|аминь|аминь|аминь|аминь|аминь",
            ),
        ];
        for (lemma, class, gender, expected) in fixtures {
            let lexeme = NounLexeme {
                lemma: lemma.to_string(),
                class,
                gender,
                animacy: Animacy::Inanimate,
                number_restriction: NumberRestriction::All,
            };
            let mut actual = Vec::with_capacity(21);
            for number in Number::ALL {
                for case in Case::ALL {
                    actual.push(
                        decline(&lexeme, NounCell { case, number })
                            .unwrap_or_else(|error| panic!("{lemma} {case:?} {number:?}: {error}"))
                            .text,
                    );
                }
            }
            assert_eq!(actual.join("|"), expected, "{lemma} {class:?}");
        }
    }

    #[test]
    fn consonant_stems_expose_the_documented_extended_stem() {
        let word = NounLexeme {
            lemma: "имѧ".to_string(),
            class: NounClass::NNeuter,
            gender: Gender::Neuter,
            animacy: Animacy::Inanimate,
            number_restriction: NumberRestriction::All,
        };
        assert_eq!(
            decline(
                &word,
                NounCell {
                    case: Case::Genitive,
                    number: Number::Singular,
                }
            )
            .expect("n-stem")
            .text,
            "имене"
        );
    }

    #[test]
    fn animacy_number_restrictions_and_hostile_lemmas_are_explicit() {
        let cell = NounCell {
            case: Case::Accusative,
            number: Number::Singular,
        };
        let mut lexeme = obedu();
        assert_eq!(decline(&lexeme, cell).expect("inanimate").text, "обѣдъ");
        lexeme.animacy = Animacy::Animate;
        assert_eq!(decline(&lexeme, cell).expect("animate").text, "обѣда");
        lexeme.number_restriction = NumberRestriction::PluralOnly;
        assert!(matches!(
            decline(&lexeme, cell),
            Err(InflectionError::UnsupportedCell { .. })
        ));

        for lemma in ["", "два слова", "слово\0"] {
            lexeme.lemma = lemma.to_string();
            assert!(matches!(
                decline(&lexeme, cell),
                Err(InflectionError::InvalidInput { .. })
            ));
        }
    }
}
