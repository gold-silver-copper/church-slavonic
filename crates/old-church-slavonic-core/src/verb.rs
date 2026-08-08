//! Conservative rule-based verb forms.

use crate::{
    FiniteTense, FiniteVerbCell, Gender, InflectionError, LParticipleCell, Number, Person,
    PredictedForm, RuleId, RuleStep, VerbClass,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerbLexeme {
    pub lemma: String,
    pub class: VerbClass,
    pub present_stem: Option<String>,
    pub aorist_stem: Option<String>,
}

pub fn finite(lexeme: &VerbLexeme, cell: FiniteVerbCell) -> Result<PredictedForm, InflectionError> {
    crate::orthography::canonical_display(&lexeme.lemma)?;
    if cell.tense != FiniteTense::Present {
        return Err(InflectionError::UnsupportedCell);
    }
    let raw_stem =
        lexeme
            .present_stem
            .as_deref()
            .ok_or_else(|| InflectionError::MissingLexicalMetadata {
                needed: vec![crate::MetadataField::PresentStem],
            })?;
    let stem = crate::orthography::canonical_display(raw_stem)?;
    let (ending, rule_id) = present_ending(lexeme.class, cell)?;
    let changed_stem = if matches!(
        (lexeme.class, cell.person, cell.number),
        (
            VerbClass::II1 | VerbClass::II2 | VerbClass::II3,
            Person::First,
            Number::Singular
        )
    ) {
        mutate_second_first_singular(&stem)
    } else {
        stem.clone()
    };
    let text = format!("{changed_stem}{ending}");
    Ok(PredictedForm {
        text: text.clone(),
        rule_id,
        trace: vec![RuleStep {
            rule_id,
            before: stem,
            after: text,
            reason: "attach the class-specific present ending to the supplied present stem",
        }],
    })
}

pub fn infinitive(lexeme: &VerbLexeme) -> Result<PredictedForm, InflectionError> {
    let lemma = crate::orthography::canonical_display(&lexeme.lemma)?;
    if !lemma.ends_with("ти") || lemma.len() <= "ти".len() {
        return Err(InflectionError::InvalidInput {
            reason: "an OCS infinitive citation must end in ти".to_string(),
        });
    }
    Ok(single_step(
        &lemma,
        &lemma,
        RuleId::VerbInfinitive,
        "return the supplied infinitive citation form",
    ))
}

pub fn supine(lexeme: &VerbLexeme) -> Result<PredictedForm, InflectionError> {
    let lemma = crate::orthography::canonical_display(&lexeme.lemma)?;
    let stem = lemma
        .strip_suffix("ти")
        .filter(|stem| !stem.is_empty())
        .ok_or_else(|| InflectionError::InvalidInput {
            reason: "a regularly derived supine needs an infinitive ending in ти".to_string(),
        })?;
    let text = format!("{stem}тъ");
    Ok(single_step(
        &lemma,
        &text,
        RuleId::VerbSupine,
        "replace the regular infinitive ending ти with the supine ending тъ",
    ))
}

pub fn l_participle(
    lexeme: &VerbLexeme,
    cell: LParticipleCell,
) -> Result<PredictedForm, InflectionError> {
    crate::orthography::canonical_display(&lexeme.lemma)?;
    let raw_stem =
        lexeme
            .aorist_stem
            .as_deref()
            .ok_or_else(|| InflectionError::MissingLexicalMetadata {
                needed: vec![crate::MetadataField::AoristStem],
            })?;
    let stem = crate::orthography::canonical_display(raw_stem)?;
    let ending = match (cell.gender, cell.number) {
        (Gender::Masculine, Number::Singular) => "лъ",
        (Gender::Feminine, Number::Singular) => "ла",
        (Gender::Neuter, Number::Singular) => "ло",
        (Gender::Masculine, Number::Dual) => "ла",
        (Gender::Feminine | Gender::Neuter, Number::Dual) => "лѣ",
        (Gender::Masculine, Number::Plural) => "ли",
        (Gender::Feminine, Number::Plural) => "лꙑ",
        (Gender::Neuter, Number::Plural) => "ла",
    };
    let text = format!("{stem}{ending}");
    Ok(single_step(
        &stem,
        &text,
        RuleId::VerbLParticiple,
        "attach the l-participle agreement ending to the explicitly supplied aorist stem",
    ))
}

fn single_step(before: &str, after: &str, rule_id: RuleId, reason: &'static str) -> PredictedForm {
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

fn mutate_second_first_singular(stem: &str) -> String {
    const CLUSTERS: [(&str, &str); 6] = [
        ("зд", "жд"),
        ("ск", "щ"),
        ("ст", "щ"),
        ("сл", "шл"),
        ("сн", "шн"),
        ("зн", "жн"),
    ];
    for (from, to) in CLUSTERS {
        if let Some(prefix) = stem.strip_suffix(from) {
            return format!("{prefix}{to}");
        }
    }
    let Some(last) = stem.chars().last() else {
        return String::new();
    };
    let replacement = match last {
        'г' | 'ѕ' | 'з' => "ж",
        'к' => "ч",
        'с' => "ш",
        'х' => "ш",
        'т' => "щ",
        'д' => "жд",
        'б' | 'в' | 'м' | 'п' => return format!("{stem}л"),
        'j' => "",
        _ => return stem.to_string(),
    };
    let prefix_len = stem.len() - last.len_utf8();
    format!("{}{replacement}", &stem[..prefix_len])
}

fn present_ending(
    class: VerbClass,
    cell: FiniteVerbCell,
) -> Result<(&'static str, RuleId), InflectionError> {
    let first = matches!(class, VerbClass::IA1 | VerbClass::IA2);
    let second = matches!(class, VerbClass::II1 | VerbClass::II2 | VerbClass::II3);
    let rule = match class {
        VerbClass::IA1 => RuleId::VerbIA1,
        VerbClass::IA2 => RuleId::VerbIA2,
        VerbClass::II1 => RuleId::VerbII1,
        VerbClass::II2 => RuleId::VerbII2,
        VerbClass::II3 => RuleId::VerbII3,
        _ => return Err(InflectionError::UnsupportedCell),
    };
    let ending = match (first, second, cell.person, cell.number) {
        (true, _, Person::First, Number::Singular) => "ѫ",
        (true, _, Person::Second, Number::Singular) => "еши",
        (true, _, Person::Third, Number::Singular) => "етъ",
        (true, _, Person::First, Number::Dual) => "евѣ",
        (true, _, Person::Second, Number::Dual) => "ета",
        (true, _, Person::Third, Number::Dual) => "ете",
        (true, _, Person::First, Number::Plural) => "емъ",
        (true, _, Person::Second, Number::Plural) => "ете",
        (true, _, Person::Third, Number::Plural) => "ѫтъ",
        (_, true, Person::First, Number::Singular) => "ѭ",
        (_, true, Person::Second, Number::Singular) => "иши",
        (_, true, Person::Third, Number::Singular) => "итъ",
        (_, true, Person::First, Number::Dual) => "ивѣ",
        (_, true, Person::Second, Number::Dual) => "ита",
        (_, true, Person::Third, Number::Dual) => "ите",
        (_, true, Person::First, Number::Plural) => "имъ",
        (_, true, Person::Second, Number::Plural) => "ите",
        (_, true, Person::Third, Number::Plural) => "ѧтъ",
        _ => return Err(InflectionError::UnsupportedCell),
    };
    Ok((ending, rule))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_productive_present_class_uses_its_stable_rule() {
        for (class, expected) in [
            (VerbClass::IA1, RuleId::VerbIA1),
            (VerbClass::IA2, RuleId::VerbIA2),
            (VerbClass::II1, RuleId::VerbII1),
            (VerbClass::II2, RuleId::VerbII2),
            (VerbClass::II3, RuleId::VerbII3),
        ] {
            let verb = VerbLexeme {
                lemma: "правити".to_string(),
                class,
                present_stem: Some("прав".to_string()),
                aorist_stem: None,
            };
            let predicted = finite(
                &verb,
                FiniteVerbCell {
                    tense: FiniteTense::Present,
                    person: Person::Third,
                    number: Number::Dual,
                },
            )
            .expect("productive present class");
            assert_eq!(predicted.rule_id, expected);
        }
    }

    #[test]
    fn second_conjugation_first_singular_mutates_labial() {
        let verb = VerbLexeme {
            lemma: "правити".to_string(),
            class: VerbClass::II1,
            present_stem: Some("прав".to_string()),
            aorist_stem: None,
        };
        let form = finite(
            &verb,
            FiniteVerbCell {
                tense: FiniteTense::Present,
                person: Person::First,
                number: Number::Singular,
            },
        )
        .expect("supported");
        assert_eq!(form.text, "правлѭ");
    }

    #[test]
    fn explicit_stems_generate_nonfinite_components() {
        let verb = VerbLexeme {
            lemma: "правити".to_string(),
            class: VerbClass::II1,
            present_stem: Some("прав".to_string()),
            aorist_stem: Some("прави".to_string()),
        };
        assert_eq!(supine(&verb).expect("regular supine").text, "правитъ");
        assert_eq!(
            l_participle(
                &verb,
                LParticipleCell {
                    gender: Gender::Feminine,
                    number: Number::Dual,
                }
            )
            .expect("explicit aorist stem")
            .text,
            "правилѣ"
        );
    }

    #[test]
    fn missing_empty_and_hostile_metadata_fail_without_panicking() {
        let cell = FiniteVerbCell {
            tense: FiniteTense::Present,
            person: Person::First,
            number: Number::Singular,
        };
        let mut verb = VerbLexeme {
            lemma: "правити".to_string(),
            class: VerbClass::II1,
            present_stem: None,
            aorist_stem: None,
        };
        assert!(matches!(
            finite(&verb, cell),
            Err(InflectionError::MissingLexicalMetadata { .. })
        ));
        verb.present_stem = Some(String::new());
        assert!(matches!(
            finite(&verb, cell),
            Err(InflectionError::InvalidInput { .. })
        ));
        verb.aorist_stem = Some(String::new());
        assert!(matches!(
            l_participle(
                &verb,
                LParticipleCell {
                    gender: Gender::Masculine,
                    number: Number::Singular,
                }
            ),
            Err(InflectionError::InvalidInput { .. })
        ));
        verb.lemma = "two words".to_string();
        assert!(matches!(
            finite(&verb, cell),
            Err(InflectionError::InvalidInput { .. })
        ));
    }
}
