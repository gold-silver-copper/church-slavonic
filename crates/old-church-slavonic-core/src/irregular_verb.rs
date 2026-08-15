//! Reusable irregular-workstem groups in Polivanova 2023 §§434–454.
//!
//! These thirteen groups are distinct from the nineteen whole-lexeme unique
//! profiles in §§516–605. Their irregularity is confined to the segmental
//! content and subparadigm distribution of the present and infinitive
//! workstems, so each group remains a reusable lexical class.

use crate::verb::VerbLexeme;
use crate::{
    AoristFormation, FiniteTense, FiniteVerbCell, ImperativeCell, ImperativeFormation,
    ImperfectFormation, ImperfectVariantPolicy, Number, PastActiveParticipleFormation,
    PastPassiveParticipleFormation, Person, PresentActiveParticipleFormation,
    PresentPassiveParticipleFormation, VerbAspect, VerbClass, VerbMorphologyCell,
};

/// The exhaustive reusable irregular-workstem group inventory of Table 440.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IrregularVerbGroup {
    Metati3NoSoftening,
    Pisati3UnstableVocalism,
    Birati3NoSofteningUnstableVocalism,
    Plivati3LabileArrested,
    Kleti4Labile,
    Brati4LabileSoftened,
    Pluti4InfStemExpanded,
    Biti4UnstableInfStemExpanded,
    Kryti4UnstableArrestedInfStemExpanded,
    Peti4InfStemExpanded,
    Mreti4UnstablePresentStemExpanded,
    Vleshti4UnstablePresentStemExpanded,
    Chisti4UnstablePresentStemExpanded,
}

impl IrregularVerbGroup {
    pub const ALL: [Self; 13] = [
        Self::Metati3NoSoftening,
        Self::Pisati3UnstableVocalism,
        Self::Birati3NoSofteningUnstableVocalism,
        Self::Plivati3LabileArrested,
        Self::Kleti4Labile,
        Self::Brati4LabileSoftened,
        Self::Pluti4InfStemExpanded,
        Self::Biti4UnstableInfStemExpanded,
        Self::Kryti4UnstableArrestedInfStemExpanded,
        Self::Peti4InfStemExpanded,
        Self::Mreti4UnstablePresentStemExpanded,
        Self::Vleshti4UnstablePresentStemExpanded,
        Self::Chisti4UnstablePresentStemExpanded,
    ];

    pub const fn representative(self) -> &'static str {
        match self {
            Self::Metati3NoSoftening => "метати",
            Self::Pisati3UnstableVocalism => "пьсати",
            Self::Birati3NoSofteningUnstableVocalism => "бьрати",
            Self::Plivati3LabileArrested => "пльвати",
            Self::Kleti4Labile => "клѧти",
            Self::Brati4LabileSoftened => "брати",
            Self::Pluti4InfStemExpanded => "плути",
            Self::Biti4UnstableInfStemExpanded => "бити",
            Self::Kryti4UnstableArrestedInfStemExpanded => "крꙑти",
            Self::Peti4InfStemExpanded => "пѣти",
            Self::Mreti4UnstablePresentStemExpanded => "мрѣти",
            Self::Vleshti4UnstablePresentStemExpanded => "влѣщи",
            Self::Chisti4UnstablePresentStemExpanded => "чисти",
        }
    }

    pub const fn source_section(self) -> &'static str {
        match self {
            Self::Metati3NoSoftening => "§441",
            Self::Pisati3UnstableVocalism => "§442",
            Self::Birati3NoSofteningUnstableVocalism => "§443",
            Self::Plivati3LabileArrested => "§444",
            Self::Kleti4Labile => "§445",
            Self::Brati4LabileSoftened => "§446",
            Self::Pluti4InfStemExpanded => "§447",
            Self::Biti4UnstableInfStemExpanded => "§448",
            Self::Kryti4UnstableArrestedInfStemExpanded => "§449",
            Self::Peti4InfStemExpanded => "§450",
            Self::Mreti4UnstablePresentStemExpanded => "§451",
            Self::Vleshti4UnstablePresentStemExpanded => "§452",
            Self::Chisti4UnstablePresentStemExpanded => "§453",
        }
    }

    /// Source-listed family anchors in Table 434, in source order.
    ///
    /// Dots marking synchronic prefix boundaries are removed, and ordinary
    /// source spellings such as `распѧти` and project-wide `ꙑ` normalization
    /// are used for public lemmas.
    pub const fn family_anchors(self) -> &'static [&'static str] {
        match self {
            Self::Metati3NoSoftening => &[
                "искати",
                "ковати",
                "метати",
                "уръвати",
                "обсновати",
                "съсати",
                "тъкати",
            ],
            Self::Pisati3UnstableVocalism => &[
                "дъхати",
                "зиꙗти",
                "зьдати",
                "лиꙗти",
                "пльзати",
                "пьсати",
                "възсльпати",
                "смиꙗти",
                "стръгати",
                "стьлати",
                "трьѕати",
                "чрьпати",
                "възьмати",
            ],
            Self::Birati3NoSofteningUnstableVocalism => {
                &["бьрати", "дьрати", "жьдати", "зъвати", "пьрати"]
            }
            Self::Plivati3LabileArrested => &["бльвати", "пльвати"],
            Self::Kleti4Labile => &[
                "дѫти",
                "жити",
                "съжѧти",
                "клѧти",
                "начѧти",
                "распѧти",
                "възѧти",
            ],
            Self::Brati4LabileSoftened => &["брати", "жрьти", "жѧти", "клати", "млѣти", "трьти"],
            Self::Pluti4InfStemExpanded => &["плути", "рути", "слути", "натрути"],
            Self::Biti4UnstableInfStemExpanded => &["бити", "съвити", "гнити", "вълити", "пити"],
            Self::Kryti4UnstableArrestedInfStemExpanded => &["крꙑти", "мꙑти", "рꙑти", "шити"],
            Self::Peti4InfStemExpanded => &["пѣти"],
            Self::Mreti4UnstablePresentStemExpanded => &[
                "заврѣти",
                "пожрѣти",
                "мрѣти",
                "обпрѣти",
                "разскврѣти",
                "прострѣти",
            ],
            Self::Vleshti4UnstablePresentStemExpanded => &["влѣщи", "брѣщи"],
            Self::Chisti4UnstablePresentStemExpanded => &[
                "отъврѣсти",
                "врѣщи",
                "вънисти",
                "стрѣщи",
                "тлѣщи",
                "цвисти",
                "чисти",
                "почрѣти",
            ],
        }
    }

    pub fn classify_representative(lemma: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|group| group.representative() == lemma)
    }

    /// Assemble the complete Table 454 profile of this group's representative.
    pub fn representative_lexeme(self) -> VerbLexeme {
        let mut lexeme = VerbLexeme::new(self.representative(), VerbClass::Irregular);
        lexeme.aspect = Some(self.representative_aspect());

        match self {
            Self::Metati3NoSoftening => {
                insert_hard_present(&mut lexeme, "мет");
                configure_class_three(&mut lexeme, "мета", "мет", PresentShape::Hard);
            }
            Self::Pisati3UnstableVocalism => {
                insert_hard_present(&mut lexeme, "пиш");
                configure_class_three(&mut lexeme, "пьса", "пиш", PresentShape::MixedSoft);
            }
            Self::Birati3NoSofteningUnstableVocalism => {
                insert_hard_present(&mut lexeme, "бер");
                configure_class_three(&mut lexeme, "бьра", "бер", PresentShape::Hard);
            }
            Self::Plivati3LabileArrested => {
                insert_iotated_present(&mut lexeme, "плю");
                configure_class_three(&mut lexeme, "пльва", "плю", PresentShape::Iotated);
            }
            Self::Kleti4Labile => {
                insert_hard_present(&mut lexeme, "кльн");
                set_imperfect(&mut lexeme, "кльн", ImperfectFormation::PresentYatA);
                set_vowel_aorist(&mut lexeme, "клѧ");
                set_imperative(&mut lexeme, "кльн", ImperativeFormation::YatSeries);
                set_l_participle(&mut lexeme, "клѧ");
                set_present_active(
                    &mut lexeme,
                    "кльн",
                    PresentActiveParticipleFormation::YushtHard,
                );
                set_present_passive(&mut lexeme, "кльн", PresentPassiveParticipleFormation::Om);
                set_past_active(&mut lexeme, "кльн", PastActiveParticipleFormation::Ush);
                set_past_passive(&mut lexeme, "клѧ", PastPassiveParticipleFormation::T);
            }
            Self::Brati4LabileSoftened => {
                insert_soft_present(&mut lexeme, "бор");
                set_imperfect(&mut lexeme, "бор", ImperfectFormation::PresentYatA);
                set_vowel_aorist(&mut lexeme, "бра");
                set_imperative(&mut lexeme, "бор", ImperativeFormation::ISeries);
                set_l_participle(&mut lexeme, "бра");
                set_present_active(
                    &mut lexeme,
                    "бор",
                    PresentActiveParticipleFormation::MixedYushtSoft,
                );
                set_present_passive(&mut lexeme, "бор", PresentPassiveParticipleFormation::Om);
                set_past_active(&mut lexeme, "бор", PastActiveParticipleFormation::Ush);
                set_past_passive(&mut lexeme, "бор", PastPassiveParticipleFormation::En);
            }
            Self::Pluti4InfStemExpanded => {
                insert_hard_present(&mut lexeme, "плов");
                set_imperfect(&mut lexeme, "плов", ImperfectFormation::PresentYatA);
                set_vowel_aorist(&mut lexeme, "плу");
                set_imperative(&mut lexeme, "плов", ImperativeFormation::YatSeries);
                set_l_participle(&mut lexeme, "плу");
                set_present_active(
                    &mut lexeme,
                    "плов",
                    PresentActiveParticipleFormation::YushtHard,
                );
                set_present_passive(&mut lexeme, "плов", PresentPassiveParticipleFormation::Om);
                set_past_active(&mut lexeme, "плу", PastActiveParticipleFormation::Vush);
                set_past_passive(&mut lexeme, "плов", PastPassiveParticipleFormation::En);
            }
            Self::Biti4UnstableInfStemExpanded => {
                insert_iotated_present(&mut lexeme, "би");
                set_imperfect(&mut lexeme, "би", ImperfectFormation::PresentYatA);
                set_vowel_aorist(&mut lexeme, "би");
                set_imperative(&mut lexeme, "би", ImperativeFormation::ISeries);
                insert_imperative_singular(&mut lexeme, "би");
                set_l_participle(&mut lexeme, "би");
                configure_iotated_present_participles(&mut lexeme, "би");
                set_past_active(&mut lexeme, "би", PastActiveParticipleFormation::Vush);
                // Table 454 licenses both бьјенъ and битъ; the productive core
                // records the independently reconstructable t-variant here.
                set_past_passive(&mut lexeme, "би", PastPassiveParticipleFormation::T);
            }
            Self::Kryti4UnstableArrestedInfStemExpanded => {
                insert_iotated_present(&mut lexeme, "крꙑ");
                set_imperfect(&mut lexeme, "крꙑ", ImperfectFormation::PresentYatA);
                set_vowel_aorist(&mut lexeme, "крꙑ");
                set_imperative(&mut lexeme, "крꙑ", ImperativeFormation::ISeries);
                set_l_participle(&mut lexeme, "крꙑ");
                configure_iotated_present_participles(&mut lexeme, "крꙑ");
                set_past_active(&mut lexeme, "крꙑ", PastActiveParticipleFormation::Vush);
                set_past_passive(&mut lexeme, "кръв", PastPassiveParticipleFormation::En);
            }
            Self::Peti4InfStemExpanded => {
                insert_iotated_present(&mut lexeme, "по");
                set_imperfect(&mut lexeme, "по", ImperfectFormation::PresentYatA);
                set_vowel_aorist(&mut lexeme, "пѣ");
                set_imperative(&mut lexeme, "по", ImperativeFormation::ISeries);
                insert_imperative_singular(&mut lexeme, "пѣи");
                set_l_participle(&mut lexeme, "пѣ");
                configure_iotated_present_participles(&mut lexeme, "по");
                set_past_active(&mut lexeme, "пѣ", PastActiveParticipleFormation::Vush);
                set_past_passive(&mut lexeme, "пѣ", PastPassiveParticipleFormation::N);
            }
            Self::Mreti4UnstablePresentStemExpanded => {
                insert_hard_present(&mut lexeme, "мьр");
                set_imperfect(&mut lexeme, "мьр", ImperfectFormation::PresentYatA);
                set_vowel_aorist(&mut lexeme, "мрѣ");
                set_imperative(&mut lexeme, "мьр", ImperativeFormation::YatSeries);
                set_l_participle(&mut lexeme, "мрь");
                set_present_active(
                    &mut lexeme,
                    "мьр",
                    PresentActiveParticipleFormation::YushtHard,
                );
                set_present_passive(&mut lexeme, "мьр", PresentPassiveParticipleFormation::Om);
                set_past_active(&mut lexeme, "мьр", PastActiveParticipleFormation::Ush);
                set_past_passive(&mut lexeme, "мрь", PastPassiveParticipleFormation::T);
            }
            Self::Vleshti4UnstablePresentStemExpanded => {
                insert_velar_present(&mut lexeme, "влѣк", "влѣч");
                lexeme
                    .exact_forms
                    .insert(VerbMorphologyCell::Infinitive, "влѣщи".to_string());
                lexeme
                    .exact_forms
                    .insert(VerbMorphologyCell::Supine, "влѣщь".to_string());
                set_imperfect(&mut lexeme, "влѣк", ImperfectFormation::PresentYatA);
                set_new_aorist(&mut lexeme, "влѣк");
                set_imperative(&mut lexeme, "влѣц", ImperativeFormation::YatSeries);
                set_l_participle(&mut lexeme, "вльк");
                set_present_active(
                    &mut lexeme,
                    "влѣк",
                    PresentActiveParticipleFormation::YushtHard,
                );
                set_present_passive(&mut lexeme, "влѣк", PresentPassiveParticipleFormation::Om);
                set_past_active(&mut lexeme, "вльк", PastActiveParticipleFormation::Ush);
                set_past_passive(&mut lexeme, "вльч", PastPassiveParticipleFormation::En);
            }
            Self::Chisti4UnstablePresentStemExpanded => {
                insert_hard_present(&mut lexeme, "чьт");
                set_imperfect(&mut lexeme, "чьт", ImperfectFormation::PresentYatA);
                set_new_aorist(&mut lexeme, "чьт");
                set_imperative(&mut lexeme, "чьт", ImperativeFormation::YatSeries);
                set_l_participle(&mut lexeme, "чьт");
                set_present_active(
                    &mut lexeme,
                    "чьт",
                    PresentActiveParticipleFormation::YushtHard,
                );
                set_present_passive(&mut lexeme, "чьт", PresentPassiveParticipleFormation::Om);
                set_past_active(&mut lexeme, "чьт", PastActiveParticipleFormation::Ush);
                set_past_passive(&mut lexeme, "чьт", PastPassiveParticipleFormation::En);
            }
        }
        lexeme
    }

    const fn representative_aspect(self) -> VerbAspect {
        match self {
            Self::Pisati3UnstableVocalism
            | Self::Biti4UnstableInfStemExpanded
            | Self::Peti4InfStemExpanded => VerbAspect::Biaspectual,
            _ => VerbAspect::Imperfective,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum PresentShape {
    Hard,
    MixedSoft,
    Iotated,
}

fn configure_class_three(
    lexeme: &mut VerbLexeme,
    infinitive_stem: &str,
    present_stem: &str,
    shape: PresentShape,
) {
    set_imperfect(lexeme, infinitive_stem, ImperfectFormation::A);
    set_vowel_aorist(lexeme, infinitive_stem);
    set_imperative(
        lexeme,
        present_stem,
        match shape {
            PresentShape::Hard => ImperativeFormation::YatSeries,
            PresentShape::MixedSoft | PresentShape::Iotated => ImperativeFormation::ISeries,
        },
    );
    set_l_participle(lexeme, infinitive_stem);
    match shape {
        PresentShape::Hard => {
            set_present_active(
                lexeme,
                present_stem,
                PresentActiveParticipleFormation::YushtHard,
            );
            set_present_passive(lexeme, present_stem, PresentPassiveParticipleFormation::Om);
        }
        PresentShape::MixedSoft => {
            set_present_active(
                lexeme,
                present_stem,
                PresentActiveParticipleFormation::MixedYushtSoft,
            );
            set_present_passive(lexeme, present_stem, PresentPassiveParticipleFormation::Em);
        }
        PresentShape::Iotated => configure_iotated_present_participles(lexeme, present_stem),
    }
    set_past_active(lexeme, infinitive_stem, PastActiveParticipleFormation::Vush);
    set_past_passive(lexeme, infinitive_stem, PastPassiveParticipleFormation::N);
}

fn configure_iotated_present_participles(lexeme: &mut VerbLexeme, stem: &str) {
    set_present_active(
        lexeme,
        stem,
        PresentActiveParticipleFormation::IotatedYushtSoft,
    );
    set_present_passive(lexeme, stem, PresentPassiveParticipleFormation::IotatedEm);
}

fn insert_hard_present(lexeme: &mut VerbLexeme, stem: &str) {
    insert_present_with(lexeme, stem, stem, "ѫ", "ѫтъ");
}

fn insert_soft_present(lexeme: &mut VerbLexeme, stem: &str) {
    insert_present_with(lexeme, stem, stem, "ѭ", "ѫтъ");
}

fn insert_iotated_present(lexeme: &mut VerbLexeme, stem: &str) {
    let forms = [
        format!("{stem}ѭ"),
        format!("{stem}ѥши"),
        format!("{stem}ѥтъ"),
        format!("{stem}ѥвѣ"),
        format!("{stem}ѥта"),
        format!("{stem}ѥте"),
        format!("{stem}ѥмъ"),
        format!("{stem}ѥте"),
        format!("{stem}ѭтъ"),
    ];
    insert_present_forms(lexeme, forms);
}

fn insert_velar_present(lexeme: &mut VerbLexeme, first_stem: &str, other_stem: &str) {
    insert_present_with(lexeme, first_stem, other_stem, "ѫ", "ѫтъ");
}

fn insert_present_with(
    lexeme: &mut VerbLexeme,
    first_stem: &str,
    other_stem: &str,
    first_ending: &str,
    third_plural_ending: &str,
) {
    let forms = [
        format!("{first_stem}{first_ending}"),
        format!("{other_stem}еши"),
        format!("{other_stem}етъ"),
        format!("{other_stem}евѣ"),
        format!("{other_stem}ета"),
        format!("{other_stem}ете"),
        format!("{other_stem}емъ"),
        format!("{other_stem}ете"),
        format!("{first_stem}{third_plural_ending}"),
    ];
    insert_present_forms(lexeme, forms);
}

fn insert_present_forms(lexeme: &mut VerbLexeme, forms: [String; 9]) {
    for (cell, form) in FiniteVerbCell::for_tense(FiniteTense::Present).zip(forms) {
        lexeme
            .exact_forms
            .insert(VerbMorphologyCell::Finite(cell), form);
    }
}

fn insert_imperative_singular(lexeme: &mut VerbLexeme, form: &str) {
    for person in [Person::Second, Person::Third] {
        lexeme.exact_forms.insert(
            VerbMorphologyCell::Imperative(ImperativeCell {
                person,
                number: Number::Singular,
            }),
            form.to_string(),
        );
    }
}

fn set_imperfect(lexeme: &mut VerbLexeme, stem: &str, formation: ImperfectFormation) {
    lexeme.stems.imperfect = Some(stem.to_string());
    lexeme.formations.imperfect = Some(formation);
    lexeme.formations.imperfect_variant_policy = Some(ImperfectVariantPolicy::UncontractedOnly);
}

fn set_vowel_aorist(lexeme: &mut VerbLexeme, stem: &str) {
    lexeme.stems.aorist = Some(stem.to_string());
    lexeme.stems.aorist_second_third_singular = Some(stem.to_string());
    lexeme.formations.aorist = Some(AoristFormation::SigmaticVowel);
}

fn set_new_aorist(lexeme: &mut VerbLexeme, stem: &str) {
    lexeme.stems.aorist = Some(stem.to_string());
    lexeme.formations.aorist = Some(AoristFormation::New);
}

fn set_imperative(lexeme: &mut VerbLexeme, stem: &str, formation: ImperativeFormation) {
    lexeme.stems.imperative = Some(stem.to_string());
    lexeme.formations.imperative = Some(formation);
}

fn set_l_participle(lexeme: &mut VerbLexeme, stem: &str) {
    lexeme.stems.l_participle = Some(stem.to_string());
}

fn set_present_active(
    lexeme: &mut VerbLexeme,
    stem: &str,
    formation: PresentActiveParticipleFormation,
) {
    lexeme.stems.present_active_participle = Some(stem.to_string());
    lexeme.formations.present_active_participle = Some(formation);
}

fn set_present_passive(
    lexeme: &mut VerbLexeme,
    stem: &str,
    formation: PresentPassiveParticipleFormation,
) {
    lexeme.stems.present_passive_participle = Some(stem.to_string());
    lexeme.formations.present_passive_participle = Some(formation);
}

fn set_past_active(lexeme: &mut VerbLexeme, stem: &str, formation: PastActiveParticipleFormation) {
    lexeme.stems.past_active_participle = Some(stem.to_string());
    lexeme.formations.past_active_participle = Some(formation);
}

fn set_past_passive(
    lexeme: &mut VerbLexeme,
    stem: &str,
    formation: PastPassiveParticipleFormation,
) {
    lexeme.stems.past_passive_participle = Some(stem.to_string());
    lexeme.formations.past_passive_participle = Some(formation);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::verb::{finite, imperative, infinitive, l_participle, participle, supine};
    use crate::{
        AdjectiveCell, AdjectiveForm, Animacy, Case, Gender, LParticipleCell, ParticipleCell,
        ParticipleKind,
    };
    use std::collections::BTreeSet;

    fn finite_cell(tense: FiniteTense, person: Person, number: Number) -> FiniteVerbCell {
        FiniteVerbCell {
            tense,
            person,
            number,
        }
    }

    fn short_nominative(kind: ParticipleKind) -> ParticipleCell {
        ParticipleCell {
            kind,
            adjective: AdjectiveCell {
                case: Case::Nominative,
                number: Number::Singular,
                gender: Gender::Masculine,
                animacy: Animacy::Inanimate,
                form: AdjectiveForm::Short,
            },
        }
    }

    #[test]
    fn group_and_family_anchor_inventories_are_closed_and_nonoverlapping() {
        let expected_counts = [7, 13, 5, 2, 7, 6, 4, 5, 4, 1, 6, 2, 8];
        let mut anchors = BTreeSet::new();
        for (group, count) in IrregularVerbGroup::ALL.into_iter().zip(expected_counts) {
            assert_eq!(group.family_anchors().len(), count, "{group:?}");
            assert!(group.family_anchors().contains(&group.representative()));
            assert_eq!(
                IrregularVerbGroup::classify_representative(group.representative()),
                Some(group)
            );
            assert!(group.source_section().starts_with('§'));
            for lemma in group.family_anchors() {
                crate::Lemma::parse(lemma).expect("source family anchor");
                assert!(anchors.insert(*lemma), "duplicate family anchor {lemma}");
            }
        }
        assert_eq!(anchors.len(), 70);
    }

    #[test]
    fn representative_profiles_match_table_454_key_forms() {
        let first_present = finite_cell(FiniteTense::Present, Person::First, Number::Singular);
        let second_present = finite_cell(FiniteTense::Present, Person::Second, Number::Singular);
        let first_imperfect = finite_cell(FiniteTense::Imperfect, Person::First, Number::Singular);
        let first_aorist = finite_cell(FiniteTense::Aorist, Person::First, Number::Singular);
        let second_aorist = finite_cell(FiniteTense::Aorist, Person::Second, Number::Singular);
        let second_plural_imperative = ImperativeCell {
            person: Person::Second,
            number: Number::Plural,
        };
        let goldens = [
            (
                IrregularVerbGroup::Metati3NoSoftening,
                "метѫ",
                "метеши",
                "метѣте",
                "метаахъ",
                "метахъ",
                "мета",
            ),
            (
                IrregularVerbGroup::Pisati3UnstableVocalism,
                "пишѫ",
                "пишеши",
                "пишите",
                "пьсаахъ",
                "пьсахъ",
                "пьса",
            ),
            (
                IrregularVerbGroup::Birati3NoSofteningUnstableVocalism,
                "берѫ",
                "береши",
                "берѣте",
                "бьраахъ",
                "бьрахъ",
                "бьра",
            ),
            (
                IrregularVerbGroup::Plivati3LabileArrested,
                "плюѭ",
                "плюѥши",
                "плюите",
                "пльваахъ",
                "пльвахъ",
                "пльва",
            ),
            (
                IrregularVerbGroup::Kleti4Labile,
                "кльнѫ",
                "кльнеши",
                "кльнѣте",
                "кльнѣахъ",
                "клѧхъ",
                "клѧ",
            ),
            (
                IrregularVerbGroup::Brati4LabileSoftened,
                "борѭ",
                "бореши",
                "борите",
                "борѣахъ",
                "брахъ",
                "бра",
            ),
            (
                IrregularVerbGroup::Pluti4InfStemExpanded,
                "пловѫ",
                "пловеши",
                "пловѣте",
                "пловѣахъ",
                "плухъ",
                "плу",
            ),
            (
                IrregularVerbGroup::Biti4UnstableInfStemExpanded,
                "биѭ",
                "биѥши",
                "биите",
                "биѣахъ",
                "бихъ",
                "би",
            ),
            (
                IrregularVerbGroup::Kryti4UnstableArrestedInfStemExpanded,
                "крꙑѭ",
                "крꙑѥши",
                "крꙑите",
                "крꙑѣахъ",
                "крꙑхъ",
                "крꙑ",
            ),
            (
                IrregularVerbGroup::Peti4InfStemExpanded,
                "поѭ",
                "поѥши",
                "поите",
                "поѣахъ",
                "пѣхъ",
                "пѣ",
            ),
            (
                IrregularVerbGroup::Mreti4UnstablePresentStemExpanded,
                "мьрѫ",
                "мьреши",
                "мьрѣте",
                "мьрѣахъ",
                "мрѣхъ",
                "мрѣ",
            ),
            (
                IrregularVerbGroup::Vleshti4UnstablePresentStemExpanded,
                "влѣкѫ",
                "влѣчеши",
                "влѣцѣте",
                "влѣкѣахъ",
                "влѣкохъ",
                "влѣче",
            ),
            (
                IrregularVerbGroup::Chisti4UnstablePresentStemExpanded,
                "чьтѫ",
                "чьтеши",
                "чьтѣте",
                "чьтѣахъ",
                "чьтохъ",
                "чьте",
            ),
        ];
        for (group, present_1, present_2, imperative_2pl, imperfect_1, aorist_1, aorist_2) in
            goldens
        {
            let lexeme = group.representative_lexeme();
            assert_eq!(
                finite(&lexeme, first_present)
                    .expect("table present 1sg")
                    .text,
                present_1,
                "{group:?}"
            );
            assert_eq!(
                finite(&lexeme, second_present)
                    .expect("table present 2sg")
                    .text,
                present_2,
                "{group:?}"
            );
            assert_eq!(
                imperative(&lexeme, second_plural_imperative)
                    .expect("table imperative 2pl")
                    .text,
                imperative_2pl,
                "{group:?}"
            );
            assert_eq!(
                finite(&lexeme, first_imperfect)
                    .expect("table imperfect 1sg")
                    .text,
                imperfect_1,
                "{group:?}"
            );
            assert_eq!(
                finite(&lexeme, first_aorist)
                    .expect("table aorist 1sg")
                    .text,
                aorist_1,
                "{group:?}"
            );
            assert_eq!(
                finite(&lexeme, second_aorist)
                    .expect("table aorist 2sg")
                    .text,
                aorist_2,
                "{group:?}"
            );
        }
    }

    #[test]
    fn representative_workstem_distribution_reaches_every_cell() {
        for group in IrregularVerbGroup::ALL {
            let lexeme = group.representative_lexeme();
            for cell in FiniteVerbCell::all() {
                finite(&lexeme, cell)
                    .unwrap_or_else(|error| panic!("{group:?} {cell:?}: {error:?}"));
            }
            for cell in ImperativeCell::SUPPORTED {
                imperative(&lexeme, cell)
                    .unwrap_or_else(|error| panic!("{group:?} {cell:?}: {error:?}"));
            }
            infinitive(&lexeme).unwrap_or_else(|error| panic!("{group:?}: {error:?}"));
            supine(&lexeme).unwrap_or_else(|error| panic!("{group:?}: {error:?}"));
            for cell in LParticipleCell::all() {
                l_participle(&lexeme, cell)
                    .unwrap_or_else(|error| panic!("{group:?} {cell:?}: {error:?}"));
            }
            for kind in ParticipleKind::ALL {
                for cell in ParticipleCell::for_kind(kind) {
                    participle(&lexeme, cell)
                        .unwrap_or_else(|error| panic!("{group:?} {cell:?}: {error:?}"));
                }
            }
        }
    }

    #[test]
    fn representative_participial_allomorphs_remain_independent() {
        let present_active = short_nominative(ParticipleKind::PresentActive);
        let present_passive = short_nominative(ParticipleKind::PresentPassive);
        let past_active = short_nominative(ParticipleKind::PastActive);
        let past_passive = short_nominative(ParticipleKind::PastPassive);
        let masculine_singular = LParticipleCell {
            gender: Gender::Masculine,
            number: Number::Singular,
        };

        let plivati = IrregularVerbGroup::Plivati3LabileArrested.representative_lexeme();
        assert_eq!(
            participle(&plivati, present_active)
                .expect("iotated present active")
                .text,
            "плюѩ"
        );
        assert_eq!(
            participle(&plivati, present_passive)
                .expect("iotated present passive")
                .text,
            "плюѥмъ"
        );

        let kryti =
            IrregularVerbGroup::Kryti4UnstableArrestedInfStemExpanded.representative_lexeme();
        assert_eq!(
            participle(&kryti, present_passive)
                .expect("kryti present passive")
                .text,
            "крꙑѥмъ"
        );
        assert_eq!(
            participle(&kryti, past_active)
                .expect("kryti past active")
                .text,
            "крꙑвъ"
        );
        assert_eq!(
            participle(&kryti, past_passive)
                .expect("kryti past passive")
                .text,
            "кръвенъ"
        );

        let mreti = IrregularVerbGroup::Mreti4UnstablePresentStemExpanded.representative_lexeme();
        assert_eq!(
            participle(&mreti, past_active)
                .expect("mreti past active")
                .text,
            "мьръ"
        );
        assert_eq!(
            participle(&mreti, past_passive)
                .expect("mreti past passive")
                .text,
            "мрьтъ"
        );

        let vleshti =
            IrregularVerbGroup::Vleshti4UnstablePresentStemExpanded.representative_lexeme();
        assert_eq!(
            l_participle(&vleshti, masculine_singular)
                .expect("vleshti l-participle")
                .text,
            "вльклъ"
        );
        assert_eq!(
            participle(&vleshti, past_passive)
                .expect("vleshti past passive")
                .text,
            "вльченъ"
        );
        assert_eq!(supine(&vleshti).expect("vleshti supine").text, "влѣщь");
    }
}
