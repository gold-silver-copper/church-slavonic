//! Rule-based noun declension.
//!
//! Since the phase-4 noun merge (docs/UNIFIED_LANGUAGE_PROMPT.md) the shared
//! vocalic and consonant-stem ending tables live in the merged kernel
//! (`church_slavonic_core::noun` and `church_slavonic_core::noun_consonant`);
//! this module is the family adapter that keeps the public API, the citation
//! parsing, the palatalization seams, the iotation/glide respelling of the
//! canonical soft columns, and the class-0 dispatch.

use crate::{
    Animacy, Case, Gender, InflectionError, NounCell, NounClass, Number, NumberRestriction,
    PredictedForm, RuleId, RuleStep,
};
use church_slavonic_core::{Recension, noun as kernel, noun_consonant as kernel_consonant};

const OCS: Recension = Recension::OldChurchSlavonic;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NounLexeme {
    pub lemma: String,
    pub class: NounClass,
    pub gender: Gender,
    pub animacy: Animacy,
    pub number_restriction: NumberRestriction,
}

/// Read one OCS vocalic column of the merged kernel. The kernel's totality
/// test guarantees a single non-empty OCS ending per cell.
fn kernel_ending(
    class: kernel::VocalicNounClass,
    cell: NounCell,
    animacy: Animacy,
) -> &'static str {
    kernel::vocalic_ending(class, cell.case, cell.number, animacy, OCS)[0]
}

/// Respell one canonical iotated soft-column ending as the family prints it
/// after the given stem shape (Polivanova's positional norm): after a vowel
/// glide the iotated letters stand; after a plain consonant ѥ/ѩ print as
/// е/ѧ; after a sibilant ꙗ/ю/ѭ additionally print as а/оу/ѫ.
fn soft_surface(ending: &'static str, glide: bool, sibilant: bool) -> String {
    if glide {
        return ending.to_string();
    }
    let mut chars = ending.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    let rest = chars.as_str();
    let replaced = match first {
        'ѥ' => Some("е"),
        'ѩ' => Some("ѧ"),
        'ꙗ' if sibilant => Some("а"),
        'ю' if sibilant => Some("оу"),
        'ѭ' if sibilant => Some("ѫ"),
        _ => None,
    };
    replaced.map_or_else(|| ending.to_string(), |first| format!("{first}{rest}"))
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
    if !lexeme.class.accepts_gender(lexeme.gender) {
        return Err(InflectionError::InvalidInput {
            reason: format!(
                "noun class {} is incompatible with {:?} gender",
                lexeme.class.code(),
                lexeme.gender
            ),
        });
    }
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
        NounClass::NMasculine => decline_consonant_stem(
            lexeme,
            cell,
            kernel_consonant::ConsonantNounClass::NMasculine,
            'ꙑ',
            "ен",
            RuleId::NounNMasculine,
        ),
        NounClass::NNeuter => decline_consonant_stem(
            lexeme,
            cell,
            kernel_consonant::ConsonantNounClass::NNeuter,
            'ѧ',
            "ен",
            RuleId::NounNNeuter,
        ),
        NounClass::NtNeuter => decline_consonant_stem(
            lexeme,
            cell,
            kernel_consonant::ConsonantNounClass::NtNeuter,
            'ѧ',
            "ѧт",
            RuleId::NounNtNeuter,
        ),
        NounClass::RStem => decline_consonant_stem(
            lexeme,
            cell,
            kernel_consonant::ConsonantNounClass::RFeminine,
            'и',
            "ер",
            RuleId::NounRStem,
        ),
        NounClass::SNeuter => decline_consonant_stem(
            lexeme,
            cell,
            kernel_consonant::ConsonantNounClass::SNeuter,
            'о',
            "ес",
            RuleId::NounSNeuter,
        ),
        NounClass::VFeminine => decline_consonant_stem(
            lexeme,
            cell,
            kernel_consonant::ConsonantNounClass::VFeminine,
            'ꙑ',
            "ъв",
            RuleId::NounVFeminine,
        ),
        NounClass::TwofoldAgentMasculine => decline_twofold_agent_masculine(lexeme, cell),
        NounClass::TwofoldInMasculine => decline_twofold_in_masculine(lexeme, cell),
        NounClass::TwofoldFeminineI => decline_twofold_feminine_i(lexeme, cell),
        NounClass::UniqueMixed => decline_unique_mixed(lexeme, cell),
        NounClass::Indeclinable => Ok(predicted(
            &lexeme.lemma,
            &lexeme.lemma,
            RuleId::NounIndeclinable,
            "the lexeme is explicitly marked indeclinable",
        )),
    }
}

fn require_gender(
    lexeme: &NounLexeme,
    expected: Gender,
    class_name: &str,
) -> Result<(), InflectionError> {
    if lexeme.gender == expected {
        Ok(())
    } else {
        Err(InflectionError::InvalidInput {
            reason: format!("{class_name} requires {expected:?} gender"),
        })
    }
}

fn decline_twofold_agent_masculine(
    lexeme: &NounLexeme,
    cell: NounCell,
) -> Result<PredictedForm, InflectionError> {
    require_gender(lexeme, Gender::Masculine, "Polivanova class 2/m*")?;
    let stem = strip_required(&lexeme.lemma, 'ь')?;
    let base = decline_jo_masculine_soft(lexeme, cell)?;
    if cell.number == Number::Plural {
        // Merged kernel: the agent direct-plural overrides (divergence
        // noun:agent-plural-reinventory).
        let overriding =
            kernel_consonant::agent_direct_plural_ending(cell.case, lexeme.animacy, OCS);
        if let Some(ending) = overriding.first() {
            return Ok(join(
                stem,
                ending,
                Mutation::None,
                RuleId::NounTwofoldAgentMasculine,
            ));
        }
    }
    Ok(relabel(base, RuleId::NounTwofoldAgentMasculine))
}

fn decline_twofold_in_masculine(
    lexeme: &NounLexeme,
    cell: NounCell,
) -> Result<PredictedForm, InflectionError> {
    require_gender(lexeme, Gender::Masculine, "Polivanova class 2/m**")?;
    let expanded_stem = strip_required(&lexeme.lemma, 'ъ')?;
    let syncopated_stem = expanded_stem
        .strip_suffix("ин")
        .filter(|stem| !stem.is_empty())
        .ok_or_else(|| InflectionError::InvalidInput {
            reason: "Polivanova class 2/m** requires a citation in -инъ".to_string(),
        })?;
    let (stem, ending, mutation) = if cell.number == Number::Plural {
        // Merged kernel: the shared singulative plural on the syncopated stem.
        let ending =
            kernel_consonant::in_singulative_plural_ending(cell.case, lexeme.animacy, OCS)[0];
        let mutation = if cell.case == Case::Locative {
            Mutation::SecondPalatalization
        } else {
            Mutation::None
        };
        (syncopated_stem, ending, mutation)
    } else {
        // Merged kernel: the shared hard o-stem singular and dual on the
        // expanded -ин- stem.
        let ending = kernel_ending(
            kernel::VocalicNounClass::OHardMasculine,
            cell,
            lexeme.animacy,
        );
        (expanded_stem, ending, hard_o_mutation(cell))
    };
    Ok(join(stem, ending, mutation, RuleId::NounTwofoldInMasculine))
}

fn decline_twofold_feminine_i(
    lexeme: &NounLexeme,
    cell: NounCell,
) -> Result<PredictedForm, InflectionError> {
    require_gender(lexeme, Gender::Feminine, "Polivanova class 2/f*")?;
    let stem = strip_required(&lexeme.lemma, 'и')?;
    if cell.case == Case::Nominative && cell.number == Number::Singular {
        return Ok(join(
            stem,
            "и",
            Mutation::None,
            RuleId::NounTwofoldFeminineI,
        ));
    }
    Ok(relabel(
        decline_ja_soft(lexeme, cell)?,
        RuleId::NounTwofoldFeminineI,
    ))
}

fn relabel(mut form: PredictedForm, rule_id: RuleId) -> PredictedForm {
    form.rule_id = rule_id;
    for step in &mut form.trace {
        step.rule_id = rule_id;
    }
    form
}

fn decline_unique_mixed(
    lexeme: &NounLexeme,
    cell: NounCell,
) -> Result<PredictedForm, InflectionError> {
    let member =
        crate::UniqueNounFamilyMember::classify_source_lemma(&lexeme.lemma).ok_or_else(|| {
            InflectionError::InvalidInput {
                reason: "the unique-mixed noun class requires a reviewed class-0 substantive"
                    .to_string(),
            }
        })?;
    require_gender(
        lexeme,
        member.gender(),
        "the reviewed class-0 substantive profile",
    )?;
    if lexeme.number_restriction != member.number_restriction() {
        return Err(InflectionError::InvalidInput {
            reason: format!(
                "class-0 noun {} requires the reviewed {:?} number restriction",
                member.canonical_lemma(),
                member.number_restriction()
            ),
        });
    }
    member.decline_primary(cell)
}

fn decline_jo_masculine_soft(
    lexeme: &NounLexeme,
    cell: NounCell,
) -> Result<PredictedForm, InflectionError> {
    let glide_citation = lexeme.lemma.ends_with('и');
    let stem = strip_any_required(&lexeme.lemma, &["ь", "и"])?;
    let sibilant = ends_in_sibilant(stem);
    if cell.case == Case::Vocative && cell.number == Number::Singular && stem.ends_with('ц') {
        let stem = stem.strip_suffix('ц').unwrap_or(stem);
        return Ok(join(
            stem,
            "че",
            Mutation::None,
            RuleId::NounJoMasculineSoft,
        ));
    }
    // Family citation overrides over the merged kernel column: the vowel-
    // glide citation prints -и in the direct singular and the genitive
    // plural.
    let ending = match (cell.case, cell.number) {
        (Case::Nominative, Number::Singular) if glide_citation => "и".to_string(),
        (Case::Accusative, Number::Singular)
            if glide_citation && lexeme.animacy != Animacy::Animate =>
        {
            "и".to_string()
        }
        (Case::Genitive, Number::Plural) if glide_citation => "и".to_string(),
        _ => soft_surface(
            kernel_ending(
                kernel::VocalicNounClass::JoSoftMasculine,
                cell,
                lexeme.animacy,
            ),
            glide_citation,
            sibilant,
        ),
    };
    Ok(join(
        stem,
        &ending,
        Mutation::None,
        RuleId::NounJoMasculineSoft,
    ))
}

fn decline_jo_neuter_soft(
    lexeme: &NounLexeme,
    cell: NounCell,
) -> Result<PredictedForm, InflectionError> {
    let iotated = lexeme.lemma.ends_with('ѥ');
    let stem = strip_any_required(&lexeme.lemma, &["е", "ѥ"])?;
    let yer_j_stem = iotated && stem.ends_with('и');
    let canonical = kernel_ending(kernel::VocalicNounClass::JoSoftNeuter, cell, lexeme.animacy);
    let ending = if cell.case == Case::Genitive && cell.number == Number::Plural && yer_j_stem {
        // Family citation override: the ьj workstem genitive plural -ии.
        "и".to_string()
    } else if !iotated {
        soft_surface(canonical, false, true)
    } else if yer_j_stem {
        canonical.to_string()
    } else if let Some(rest) = canonical.strip_prefix("ѥм") {
        // An iotated citation after a plain consonant keeps ѥ/ꙗ/ю in the
        // direct and genitive cells but prints the ѥм- obliques with plain е.
        format!("ем{rest}")
    } else {
        canonical.to_string()
    };
    Ok(join(
        stem,
        &ending,
        Mutation::None,
        RuleId::NounJoNeuterSoft,
    ))
}

fn decline_ja_soft(lexeme: &NounLexeme, cell: NounCell) -> Result<PredictedForm, InflectionError> {
    let (stem, iotated) = if let Some(stem) = lexeme.lemma.strip_suffix('ꙗ') {
        (stem, true)
    } else if let Some(stem) = lexeme.lemma.strip_suffix('и') {
        (stem, true)
    } else {
        (strip_required(&lexeme.lemma, 'а')?, false)
    };
    let canonical = kernel_ending(kernel::VocalicNounClass::JaSoft, cell, lexeme.animacy);
    let ending = if cell.case == Case::Genitive
        && cell.number == Number::Plural
        && iotated
        && stem.ends_with('и')
    {
        // Family citation override: the ьj workstem genitive plural -ии.
        "и".to_string()
    } else if iotated {
        canonical.to_string()
    } else {
        soft_surface(canonical, false, true)
    };
    Ok(join(stem, &ending, Mutation::None, RuleId::NounJaSoft))
}

fn decline_i_stem(
    lexeme: &NounLexeme,
    cell: NounCell,
    masculine: bool,
) -> Result<PredictedForm, InflectionError> {
    let stem = strip_required(&lexeme.lemma, 'ь')?;
    let (class, rule) = if masculine {
        (kernel::VocalicNounClass::IMasculine, RuleId::NounIMasculine)
    } else {
        (kernel::VocalicNounClass::IFeminine, RuleId::NounIFeminine)
    };
    let ending = kernel_ending(class, cell, lexeme.animacy);
    Ok(join(stem, ending, Mutation::None, rule))
}

fn decline_u_masculine(
    lexeme: &NounLexeme,
    cell: NounCell,
) -> Result<PredictedForm, InflectionError> {
    let stem = strip_required(&lexeme.lemma, 'ъ')?;
    let ending = kernel_ending(
        kernel::VocalicNounClass::UStemMasculine,
        cell,
        lexeme.animacy,
    );
    Ok(join(stem, ending, Mutation::None, RuleId::NounUMasculine))
}

fn decline_consonant_stem(
    lexeme: &NounLexeme,
    cell: NounCell,
    class: kernel_consonant::ConsonantNounClass,
    citation_ending: char,
    extension: &str,
    rule: RuleId,
) -> Result<PredictedForm, InflectionError> {
    let stem = strip_required(&lexeme.lemma, citation_ending)?;
    // Merged kernel: consonant-stem endings after the extended oblique stem;
    // an empty column marks a family-owned citation cell.
    let endings =
        kernel_consonant::consonant_ending(class, cell.case, cell.number, lexeme.animacy, OCS);
    let ending = endings.first().map_or_else(
        || citation_ending.to_string(),
        |ending| format!("{extension}{ending}"),
    );
    Ok(join(stem, &ending, Mutation::None, rule))
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

/// The hard o-stem palatalization seams (Polivanova table 327), applied over
/// the merged kernel's ending column.
fn hard_o_mutation(cell: NounCell) -> Mutation {
    match (cell.case, cell.number) {
        (Case::Locative, Number::Singular | Number::Plural)
        | (Case::Nominative | Case::Vocative, Number::Plural) => Mutation::SecondPalatalization,
        (Case::Vocative, Number::Singular) => Mutation::FirstPalatalization,
        _ => Mutation::None,
    }
}

fn decline_o_masculine_hard(
    lexeme: &NounLexeme,
    cell: NounCell,
) -> Result<PredictedForm, InflectionError> {
    let stem = strip_required(&lexeme.lemma, 'ъ')?;
    let ending = kernel_ending(
        kernel::VocalicNounClass::OHardMasculine,
        cell,
        lexeme.animacy,
    );
    Ok(join(
        stem,
        ending,
        hard_o_mutation(cell),
        RuleId::NounOMasculineHard,
    ))
}

fn decline_o_neuter_hard(
    lexeme: &NounLexeme,
    cell: NounCell,
) -> Result<PredictedForm, InflectionError> {
    let stem = strip_required(&lexeme.lemma, 'о')?;
    let ending = kernel_ending(kernel::VocalicNounClass::OHardNeuter, cell, lexeme.animacy);
    let mutation = match (cell.case, cell.number) {
        (Case::Locative, Number::Singular | Number::Plural)
        | (Case::Nominative | Case::Accusative | Case::Vocative, Number::Dual) => {
            Mutation::SecondPalatalization
        }
        _ => Mutation::None,
    };
    Ok(join(stem, ending, mutation, RuleId::NounONeuterHard))
}

fn decline_a_hard(lexeme: &NounLexeme, cell: NounCell) -> Result<PredictedForm, InflectionError> {
    let stem = strip_required(&lexeme.lemma, 'а')?;
    let ending = kernel_ending(kernel::VocalicNounClass::AHard, cell, lexeme.animacy);
    let mutation = match (cell.case, cell.number) {
        (Case::Dative | Case::Locative, Number::Singular)
        | (Case::Nominative | Case::Accusative | Case::Vocative, Number::Dual) => {
            Mutation::SecondPalatalization
        }
        _ => Mutation::None,
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
    stem.ends_with("жд")
        || stem
            .chars()
            .last()
            .is_some_and(|letter| matches!(letter, 'ш' | 'щ' | 'ч' | 'ж' | 'ѕ' | 'ꙃ' | 'ц'))
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
    fn twofold_glide_and_soft_consonant_seams_follow_polivanova_normalization() {
        let form = |lemma: &str, class, gender, case, number| {
            decline(
                &NounLexeme {
                    lemma: lemma.to_string(),
                    class,
                    gender,
                    animacy: Animacy::Inanimate,
                    number_restriction: NumberRestriction::All,
                },
                NounCell { case, number },
            )
            .expect("licensed twofold cell")
            .text
        };

        // §§326–328 and the canonical -j-e- spelling illustrated in §241.
        assert_eq!(
            form(
                "гнои",
                NounClass::JoMasculineSoft,
                Gender::Masculine,
                Case::Instrumental,
                Number::Singular,
            ),
            "гноѥмь"
        );
        assert_eq!(
            form(
                "гнои",
                NounClass::JoMasculineSoft,
                Gender::Masculine,
                Case::Accusative,
                Number::Plural,
            ),
            "гноѩ"
        );
        assert_eq!(
            form(
                "змии",
                NounClass::JoMasculineSoft,
                Gender::Masculine,
                Case::Genitive,
                Number::Plural,
            ),
            "змии"
        );
        assert_eq!(
            form(
                "вождь",
                NounClass::JoMasculineSoft,
                Gender::Masculine,
                Case::Genitive,
                Number::Singular,
            ),
            "вожда"
        );

        // §§338–340: the ьj seam coalesces before ь but remains before е.
        assert_eq!(
            form(
                "знаниѥ",
                NounClass::JoNeuterSoft,
                Gender::Neuter,
                Case::Instrumental,
                Number::Singular,
            ),
            "знаниѥмь"
        );
        assert_eq!(
            form(
                "знаниѥ",
                NounClass::JoNeuterSoft,
                Gender::Neuter,
                Case::Genitive,
                Number::Plural,
            ),
            "знании"
        );

        // §§342–344: ц selects the soft twofold cells, while invariant
        // accusative/dual terminals keep their back vowel.
        for (case, number, expected) in [
            (Case::Genitive, Number::Singular, "овьцѧ"),
            (Case::Accusative, Number::Singular, "овьцѫ"),
            (Case::Dative, Number::Singular, "овьци"),
            (Case::Genitive, Number::Dual, "овьцоу"),
            (Case::Dative, Number::Dual, "овьцама"),
            (Case::Nominative, Number::Plural, "овьцѧ"),
        ] {
            assert_eq!(
                form("овьца", NounClass::JaSoft, Gender::Feminine, case, number,),
                expected
            );
        }
        assert_eq!(
            form(
                "змиꙗ",
                NounClass::JaSoft,
                Gender::Feminine,
                Case::Genitive,
                Number::Plural,
            ),
            "змии"
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
                "волꙗ|волѩ|воли|волѭ|волеѭ|воли|воле|воли|волю|волꙗма|воли|волꙗма|волю|воли|волѩ|воль|волꙗмъ|волѩ|волꙗми|волꙗхъ|волѩ",
            ),
            (
                "кость",
                NounClass::IFeminine,
                Gender::Feminine,
                "кость|кости|кости|кость|костьѭ|кости|кости|кости|костию|костьма|кости|костьма|костию|кости|кости|костии|костьмъ|кости|костьми|костьхъ|кости",
            ),
            (
                "пѫть",
                NounClass::IMasculine,
                Gender::Masculine,
                "пѫть|пѫти|пѫти|пѫть|пѫтьмь|пѫти|пѫти|пѫти|пѫтию|пѫтьма|пѫти|пѫтьма|пѫтию|пѫти|пѫтиѥ|пѫтии|пѫтьмъ|пѫти|пѫтьми|пѫтьхъ|пѫтиѥ",
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
            (
                "дѣлател҄ь",
                NounClass::TwofoldAgentMasculine,
                Gender::Masculine,
                "дѣлател҄ь|дѣлател҄ꙗ|дѣлател҄ю|дѣлател҄ь|дѣлател҄емь|дѣлател҄и|дѣлател҄ю|дѣлател҄ꙗ|дѣлател҄ю|дѣлател҄ема|дѣлател҄ꙗ|дѣлател҄ема|дѣлател҄ю|дѣлател҄ꙗ|дѣлател҄ѥ|дѣлател҄ь|дѣлател҄емъ|дѣлател҄ѩ|дѣлател҄и|дѣлател҄ихъ|дѣлател҄ѥ",
            ),
            (
                "гражданинъ",
                NounClass::TwofoldInMasculine,
                Gender::Masculine,
                "гражданинъ|гражданина|гражданиноу|гражданинъ|гражданиномъ|гражданинѣ|гражданине|гражданина|гражданиноу|гражданинома|гражданина|гражданинома|гражданиноу|гражданина|граждане|гражданъ|гражданомъ|гражданꙑ|гражданꙑ|гражданѣхъ|граждане",
            ),
            (
                "рабын҄и",
                NounClass::TwofoldFeminineI,
                Gender::Feminine,
                "рабын҄и|рабын҄ѩ|рабын҄и|рабын҄ѭ|рабын҄еѭ|рабын҄и|рабын҄е|рабын҄и|рабын҄ю|рабын҄ꙗма|рабын҄и|рабын҄ꙗма|рабын҄ю|рабын҄и|рабын҄ѩ|рабын҄ь|рабын҄ꙗмъ|рабын҄ѩ|рабын҄ꙗми|рабын҄ꙗхъ|рабын҄ѩ",
            ),
            (
                "имѧ",
                NounClass::UniqueMixed,
                Gender::Neuter,
                "имѧ|имене|имени|имѧ|именемь|имени|имѧ|именѣ|именоу|именьма|именѣ|именьма|именоу|именѣ|имена|именъ|именемъ|имена|имены|именехъ|имена",
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

    #[test]
    fn productive_classes_reject_contradictory_gender_metadata() {
        let cell = NounCell {
            case: Case::Nominative,
            number: Number::Singular,
        };
        for (lemma, class, gender) in [
            ("градъ", NounClass::OMasculineHard, Gender::Neuter),
            ("село", NounClass::ONeuterHard, Gender::Masculine),
            ("жена", NounClass::AHard, Gender::Neuter),
            ("душа", NounClass::JaSoft, Gender::Neuter),
        ] {
            let error = decline(
                &NounLexeme {
                    lemma: lemma.to_string(),
                    class,
                    gender,
                    animacy: Animacy::Inanimate,
                    number_restriction: NumberRestriction::All,
                },
                cell,
            )
            .expect_err("contradictory class and gender must be rejected");
            assert!(
                matches!(error, InflectionError::InvalidInput { .. }),
                "{lemma}: {error:?}"
            );
        }
    }
}
