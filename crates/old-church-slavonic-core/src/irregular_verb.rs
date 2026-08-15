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

/// Independently sourced lexical analyses within an irregular family.
///
/// Most Table 434 members have one reviewed analysis. `метати` is the
/// exception: the official LOVe record lists competing `je`- and `aje`-stem
/// presents alongside Polivanova's unsoftened `мет-` analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IrregularVerbAnalysis {
    PolivanovaTable434,
    LoveMetatiJePresent,
    LoveMetatiAjePresent,
}

impl IrregularVerbAnalysis {
    pub const fn code(self) -> &'static str {
        match self {
            Self::PolivanovaTable434 => "polivanova-table-434",
            Self::LoveMetatiJePresent => "love-metati-je-present",
            Self::LoveMetatiAjePresent => "love-metati-aje-present",
        }
    }

    pub const fn authority(self) -> &'static str {
        match self {
            Self::PolivanovaTable434 => "Polivanova 2023 Tables 434 and 440",
            Self::LoveMetatiJePresent | Self::LoveMetatiAjePresent => {
                "LMU Lexicon of Old Church Slavonic Verbs, metati record, reviewed 2026-08-15"
            }
        }
    }
}

const POLIVANOVA_ANALYSIS: [IrregularVerbAnalysis; 1] = [IrregularVerbAnalysis::PolivanovaTable434];
const METATI_ANALYSES: [IrregularVerbAnalysis; 3] = [
    IrregularVerbAnalysis::PolivanovaTable434,
    IrregularVerbAnalysis::LoveMetatiJePresent,
    IrregularVerbAnalysis::LoveMetatiAjePresent,
];

/// One source-listed member of the reusable irregular-workstem groups.
///
/// The identity is closed over Table 434. Each value therefore carries an
/// explicit allomorph map; callers never have to infer a prefix boundary or a
/// present stem from the infinitive spelling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IrregularVerbFamilyMember {
    group: IrregularVerbGroup,
    lemma: &'static str,
}

impl IrregularVerbFamilyMember {
    pub const COUNT: usize = 70;

    pub fn all() -> impl Iterator<Item = Self> {
        IrregularVerbGroup::ALL.into_iter().flat_map(|group| {
            group
                .family_anchors()
                .iter()
                .copied()
                .map(move |lemma| Self { group, lemma })
        })
    }

    pub fn classify_source_lemma(lemma: &str) -> Option<Self> {
        Self::all().find(|member| member.lemma == lemma)
    }

    pub const fn group(self) -> IrregularVerbGroup {
        self.group
    }

    pub const fn canonical_lemma(self) -> &'static str {
        self.lemma
    }

    pub const fn source_section(self) -> &'static str {
        self.group.source_section()
    }

    /// Lexical aspect cross-checked against the August 2026 LOVe export.
    ///
    /// Polivanova's group tables describe workstem distribution rather than
    /// aspect. The few prefixed imperfectives and unprefixed perfective or
    /// biaspectual members are therefore listed instead of inferred.
    pub fn aspect(self) -> VerbAspect {
        match self.lemma {
            "пьсати" | "чрьпати" | "бити" | "пити" | "пѣти" => {
                VerbAspect::Biaspectual
            }
            "уръвати"
            | "обсновати"
            | "възсльпати"
            | "съжѧти"
            | "начѧти"
            | "распѧти"
            | "възѧти"
            | "натрути"
            | "съвити"
            | "вълити"
            | "шити"
            | "трьти"
            | "заврѣти"
            | "пожрѣти"
            | "обпрѣти"
            | "разскврѣти"
            | "прострѣти"
            | "отъврѣсти"
            | "вънисти"
            | "почрѣти" => VerbAspect::Perfective,
            _ => VerbAspect::Imperfective,
        }
    }

    /// Assemble every finite and nonfinite system from the member's explicit
    /// Table 434 principal parts and its Table 440 group distribution.
    pub fn lexeme(self) -> VerbLexeme {
        assemble_family_member(self)
    }

    /// Every reviewed lexical analysis, in deterministic source order.
    pub fn analyses(self) -> &'static [IrregularVerbAnalysis] {
        if self.lemma == "метати" {
            &METATI_ANALYSES
        } else {
            &POLIVANOVA_ANALYSIS
        }
    }

    /// Assemble one explicitly selected source analysis.
    pub fn lexeme_for_analysis(self, analysis: IrregularVerbAnalysis) -> Option<VerbLexeme> {
        match analysis {
            IrregularVerbAnalysis::PolivanovaTable434 => Some(self.lexeme()),
            IrregularVerbAnalysis::LoveMetatiJePresent if self.lemma == "метати" => {
                Some(build_class_three_member(
                    self.lemma,
                    self.aspect(),
                    "мета",
                    "мещ",
                    PresentShape::MixedSoft,
                ))
            }
            IrregularVerbAnalysis::LoveMetatiAjePresent if self.lemma == "метати" => {
                Some(build_class_three_member(
                    self.lemma,
                    self.aspect(),
                    "мета",
                    "мета",
                    PresentShape::Iotated,
                ))
            }
            IrregularVerbAnalysis::LoveMetatiJePresent
            | IrregularVerbAnalysis::LoveMetatiAjePresent => None,
        }
    }
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
        assemble_family_member(IrregularVerbFamilyMember {
            group: self,
            lemma: self.representative(),
        })
    }
}

fn assemble_family_member(member: IrregularVerbFamilyMember) -> VerbLexeme {
    let lemma = member.canonical_lemma();
    let aspect = member.aspect();
    match member.group() {
        IrregularVerbGroup::Metati3NoSoftening => {
            let (infinitive_stem, present_stem) = match lemma {
                "искати" => ("иска", "иск"),
                "ковати" => ("кова", "ков"),
                "метати" => ("мета", "мет"),
                "уръвати" => ("уръва", "уръв"),
                "обсновати" => ("обснова", "обснов"),
                "съсати" => ("съса", "със"),
                "тъкати" => ("тъка", "тък"),
                _ => ("мета", "мет"),
            };
            build_class_three_member(
                lemma,
                aspect,
                infinitive_stem,
                present_stem,
                PresentShape::Hard,
            )
        }
        IrregularVerbGroup::Pisati3UnstableVocalism => {
            let (infinitive_stem, present_stem, shape) = match lemma {
                "дъхати" => ("дъха", "душ", PresentShape::MixedSoft),
                "зиꙗти" => ("зиꙗ", "зѣ", PresentShape::Iotated),
                "зьдати" => ("зьда", "зижд", PresentShape::MixedSoft),
                "лиꙗти" => ("лиꙗ", "лѣ", PresentShape::Iotated),
                "пльзати" => ("пльза", "плѣж", PresentShape::MixedSoft),
                "пьсати" => ("пьса", "пиш", PresentShape::MixedSoft),
                "възсльпати" => ("възсльпа", "възслѣпл", PresentShape::Soft),
                "смиꙗти" => ("смиꙗ", "смѣ", PresentShape::Iotated),
                "стръгати" => ("стръга", "струж", PresentShape::MixedSoft),
                "стьлати" => ("стьла", "стел", PresentShape::Soft),
                "трьѕати" => ("трьѕа", "трѣж", PresentShape::MixedSoft),
                "чрьпати" => ("чрьпа", "чрѣпл", PresentShape::Soft),
                "възьмати" => ("възьма", "въземл", PresentShape::Soft),
                _ => ("пьса", "пиш", PresentShape::MixedSoft),
            };
            build_class_three_member(lemma, aspect, infinitive_stem, present_stem, shape)
        }
        IrregularVerbGroup::Birati3NoSofteningUnstableVocalism => {
            let (infinitive_stem, present_stem) = match lemma {
                "бьрати" => ("бьра", "бер"),
                "дьрати" => ("дьра", "дер"),
                "жьдати" => ("жьда", "жид"),
                "зъвати" => ("зъва", "зов"),
                "пьрати" => ("пьра", "пер"),
                _ => ("бьра", "бер"),
            };
            build_class_three_member(
                lemma,
                aspect,
                infinitive_stem,
                present_stem,
                PresentShape::Hard,
            )
        }
        IrregularVerbGroup::Plivati3LabileArrested => {
            let (infinitive_stem, present_stem) = match lemma {
                "бльвати" => ("бльва", "блю"),
                "пльвати" => ("пльва", "плю"),
                _ => ("пльва", "плю"),
            };
            build_class_three_member(
                lemma,
                aspect,
                infinitive_stem,
                present_stem,
                PresentShape::Iotated,
            )
        }
        IrregularVerbGroup::Kleti4Labile => {
            let (present_stem, infinitive_stem) = match lemma {
                "дѫти" => ("дъм", "дѫ"),
                "жити" => ("жив", "жи"),
                "съжѧти" => ("съжьм", "съжѧ"),
                "клѧти" => ("кльн", "клѧ"),
                "начѧти" => ("начьн", "начѧ"),
                "распѧти" => ("распьн", "распѧ"),
                "възѧти" => ("възьм", "възѧ"),
                _ => ("кльн", "клѧ"),
            };
            build_kleti_member(lemma, aspect, present_stem, infinitive_stem)
        }
        IrregularVerbGroup::Brati4LabileSoftened => {
            let (present_stem, infinitive_stem) = match lemma {
                "брати" => ("бор", "бра"),
                "жрьти" => ("жьр", "жрь"),
                "жѧти" => ("жьн", "жѧ"),
                "клати" => ("кол", "кла"),
                "млѣти" => ("мел", "млѣ"),
                "трьти" => ("тьр", "трь"),
                _ => ("бор", "бра"),
            };
            build_brati_member(lemma, aspect, present_stem, infinitive_stem)
        }
        IrregularVerbGroup::Pluti4InfStemExpanded => {
            let (present_stem, infinitive_stem) = match lemma {
                "плути" => ("плов", "плу"),
                "рути" => ("ров", "ру"),
                "слути" => ("слов", "слу"),
                "натрути" => ("натров", "натру"),
                _ => ("плов", "плу"),
            };
            build_pluti_member(lemma, aspect, present_stem, infinitive_stem)
        }
        IrregularVerbGroup::Biti4UnstableInfStemExpanded => {
            let stem = match lemma {
                "бити" => "би",
                "съвити" => "съви",
                "гнити" => "гни",
                "вълити" => "въли",
                "пити" => "пи",
                _ => "би",
            };
            build_biti_member(lemma, aspect, stem)
        }
        IrregularVerbGroup::Kryti4UnstableArrestedInfStemExpanded => {
            let (stem, past_passive_stem) = match lemma {
                "крꙑти" => ("крꙑ", "кръв"),
                "мꙑти" => ("мꙑ", "мъв"),
                "рꙑти" => ("рꙑ", "ръв"),
                "шити" => ("ши", "шьв"),
                _ => ("крꙑ", "кръв"),
            };
            build_kryti_member(lemma, aspect, stem, past_passive_stem)
        }
        IrregularVerbGroup::Peti4InfStemExpanded => build_peti_member(lemma, aspect),
        IrregularVerbGroup::Mreti4UnstablePresentStemExpanded => {
            let (present_stem, infinitive_stem, past_stem) = match lemma {
                "заврѣти" => ("завьр", "заврѣ", "заврь"),
                "пожрѣти" => ("пожьр", "пожрѣ", "пожрь"),
                "мрѣти" => ("мьр", "мрѣ", "мрь"),
                "обпрѣти" => ("обпьр", "обпрѣ", "обпрь"),
                "разскврѣти" => ("разсквьр", "разскврѣ", "разскврь"),
                "прострѣти" => ("простьр", "прострѣ", "прострь"),
                _ => ("мьр", "мрѣ", "мрь"),
            };
            build_mreti_member(lemma, aspect, present_stem, infinitive_stem, past_stem)
        }
        IrregularVerbGroup::Vleshti4UnstablePresentStemExpanded => {
            let (first_present, other_present, imperative_stem, past_stem, passive_stem) =
                match lemma {
                    "влѣщи" => ("влѣк", "влѣч", "влѣц", "вльк", "вльч"),
                    "брѣщи" => ("брѣг", "брѣж", "брѣѕ", "брьг", "брьж"),
                    _ => ("влѣк", "влѣч", "влѣц", "вльк", "вльч"),
                };
            build_velar_shti_member(
                lemma,
                aspect,
                first_present,
                other_present,
                imperative_stem,
                past_stem,
                passive_stem,
            )
        }
        IrregularVerbGroup::Chisti4UnstablePresentStemExpanded => {
            let (first_present, other_present, imperative_stem, past_stem, passive_stem) =
                match lemma {
                    "отъврѣсти" => ("отъврьз", "отъврьз", "отъврьз", "отъврьз", "отъврьз"),
                    "врѣщи" => ("врьг", "врьж", "врьѕ", "врьг", "врьж"),
                    "вънисти" => ("въньз", "въньз", "въньз", "въньз", "въньз"),
                    "стрѣщи" => ("стриг", "стриж", "стриѕ", "стриг", "стриж"),
                    "тлѣщи" => ("тльк", "тльч", "тльц", "тльк", "тльч"),
                    "цвисти" => ("цвьт", "цвьт", "цвьт", "цвьт", "цвьт"),
                    "чисти" => ("чьт", "чьт", "чьт", "чьт", "чьт"),
                    "почрѣти" => ("почрьп", "почрьп", "почрьп", "почрьп", "почрьп"),
                    _ => ("чьт", "чьт", "чьт", "чьт", "чьт"),
                };
            build_chisti_member(
                lemma,
                aspect,
                first_present,
                other_present,
                imperative_stem,
                past_stem,
                passive_stem,
            )
        }
    }
}

fn new_family_lexeme(lemma: &str, aspect: VerbAspect) -> VerbLexeme {
    let mut lexeme = VerbLexeme::new(lemma, VerbClass::Irregular);
    lexeme.aspect = Some(aspect);
    lexeme
}

fn build_class_three_member(
    lemma: &str,
    aspect: VerbAspect,
    infinitive_stem: &str,
    present_stem: &str,
    shape: PresentShape,
) -> VerbLexeme {
    let mut lexeme = new_family_lexeme(lemma, aspect);
    insert_present_shape(&mut lexeme, present_stem, shape);
    configure_class_three(&mut lexeme, infinitive_stem, present_stem, shape);
    lexeme
}

fn build_kleti_member(
    lemma: &str,
    aspect: VerbAspect,
    present_stem: &str,
    infinitive_stem: &str,
) -> VerbLexeme {
    let mut lexeme = new_family_lexeme(lemma, aspect);
    insert_hard_present(&mut lexeme, present_stem);
    set_imperfect(&mut lexeme, present_stem, ImperfectFormation::PresentYatA);
    set_vowel_aorist(&mut lexeme, infinitive_stem);
    set_imperative(&mut lexeme, present_stem, ImperativeFormation::YatSeries);
    set_l_participle(&mut lexeme, infinitive_stem);
    set_present_active(
        &mut lexeme,
        present_stem,
        PresentActiveParticipleFormation::YushtHard,
    );
    set_present_passive(
        &mut lexeme,
        present_stem,
        PresentPassiveParticipleFormation::Om,
    );
    set_past_active(
        &mut lexeme,
        present_stem,
        PastActiveParticipleFormation::Ush,
    );
    set_past_passive(
        &mut lexeme,
        infinitive_stem,
        PastPassiveParticipleFormation::T,
    );
    lexeme
}

fn build_brati_member(
    lemma: &str,
    aspect: VerbAspect,
    present_stem: &str,
    infinitive_stem: &str,
) -> VerbLexeme {
    let mut lexeme = new_family_lexeme(lemma, aspect);
    insert_soft_present(&mut lexeme, present_stem);
    set_imperfect(&mut lexeme, present_stem, ImperfectFormation::PresentYatA);
    set_vowel_aorist(&mut lexeme, infinitive_stem);
    set_imperative(&mut lexeme, present_stem, ImperativeFormation::ISeries);
    set_l_participle(&mut lexeme, infinitive_stem);
    set_present_active(
        &mut lexeme,
        present_stem,
        PresentActiveParticipleFormation::MixedYushtSoft,
    );
    set_present_passive(
        &mut lexeme,
        present_stem,
        PresentPassiveParticipleFormation::Om,
    );
    set_past_active(
        &mut lexeme,
        present_stem,
        PastActiveParticipleFormation::Ush,
    );
    set_past_passive(
        &mut lexeme,
        present_stem,
        PastPassiveParticipleFormation::En,
    );
    lexeme
}

fn build_pluti_member(
    lemma: &str,
    aspect: VerbAspect,
    present_stem: &str,
    infinitive_stem: &str,
) -> VerbLexeme {
    let mut lexeme = new_family_lexeme(lemma, aspect);
    insert_hard_present(&mut lexeme, present_stem);
    set_imperfect(&mut lexeme, present_stem, ImperfectFormation::PresentYatA);
    set_vowel_aorist(&mut lexeme, infinitive_stem);
    set_imperative(&mut lexeme, present_stem, ImperativeFormation::YatSeries);
    set_l_participle(&mut lexeme, infinitive_stem);
    set_present_active(
        &mut lexeme,
        present_stem,
        PresentActiveParticipleFormation::YushtHard,
    );
    set_present_passive(
        &mut lexeme,
        present_stem,
        PresentPassiveParticipleFormation::Om,
    );
    set_past_active(
        &mut lexeme,
        infinitive_stem,
        PastActiveParticipleFormation::Vush,
    );
    set_past_passive(
        &mut lexeme,
        present_stem,
        PastPassiveParticipleFormation::En,
    );
    lexeme
}

fn build_biti_member(lemma: &str, aspect: VerbAspect, stem: &str) -> VerbLexeme {
    let mut lexeme = new_family_lexeme(lemma, aspect);
    insert_iotated_present(&mut lexeme, stem);
    set_imperfect(&mut lexeme, stem, ImperfectFormation::PresentYatA);
    set_vowel_aorist(&mut lexeme, stem);
    set_imperative(&mut lexeme, stem, ImperativeFormation::ISeries);
    insert_imperative_singular(&mut lexeme, stem);
    set_l_participle(&mut lexeme, stem);
    configure_iotated_present_participles(&mut lexeme, stem);
    set_past_active(&mut lexeme, stem, PastActiveParticipleFormation::Vush);
    // Table 454 licenses both бьјенъ and битъ for the representative. This
    // productive family profile retains the independently reconstructable
    // t-series; the facade must preserve source-listed en-variants separately.
    set_past_passive(&mut lexeme, stem, PastPassiveParticipleFormation::T);
    lexeme
}

fn build_kryti_member(
    lemma: &str,
    aspect: VerbAspect,
    stem: &str,
    past_passive_stem: &str,
) -> VerbLexeme {
    let mut lexeme = new_family_lexeme(lemma, aspect);
    insert_iotated_present(&mut lexeme, stem);
    set_imperfect(&mut lexeme, stem, ImperfectFormation::PresentYatA);
    set_vowel_aorist(&mut lexeme, stem);
    set_imperative(&mut lexeme, stem, ImperativeFormation::ISeries);
    set_l_participle(&mut lexeme, stem);
    configure_iotated_present_participles(&mut lexeme, stem);
    set_past_active(&mut lexeme, stem, PastActiveParticipleFormation::Vush);
    set_past_passive(
        &mut lexeme,
        past_passive_stem,
        PastPassiveParticipleFormation::En,
    );
    lexeme
}

fn build_peti_member(lemma: &str, aspect: VerbAspect) -> VerbLexeme {
    let mut lexeme = new_family_lexeme(lemma, aspect);
    insert_iotated_present(&mut lexeme, "по");
    set_imperfect(&mut lexeme, "по", ImperfectFormation::PresentYatA);
    set_vowel_aorist(&mut lexeme, "пѣ");
    set_imperative(&mut lexeme, "по", ImperativeFormation::ISeries);
    insert_imperative_singular(&mut lexeme, "пѣи");
    set_l_participle(&mut lexeme, "пѣ");
    configure_iotated_present_participles(&mut lexeme, "по");
    set_past_active(&mut lexeme, "пѣ", PastActiveParticipleFormation::Vush);
    set_past_passive(&mut lexeme, "пѣ", PastPassiveParticipleFormation::N);
    lexeme
}

fn build_mreti_member(
    lemma: &str,
    aspect: VerbAspect,
    present_stem: &str,
    infinitive_stem: &str,
    past_stem: &str,
) -> VerbLexeme {
    let mut lexeme = new_family_lexeme(lemma, aspect);
    insert_hard_present(&mut lexeme, present_stem);
    set_imperfect(&mut lexeme, present_stem, ImperfectFormation::PresentYatA);
    set_vowel_aorist(&mut lexeme, infinitive_stem);
    set_imperative(&mut lexeme, present_stem, ImperativeFormation::YatSeries);
    set_l_participle(&mut lexeme, past_stem);
    set_present_active(
        &mut lexeme,
        present_stem,
        PresentActiveParticipleFormation::YushtHard,
    );
    set_present_passive(
        &mut lexeme,
        present_stem,
        PresentPassiveParticipleFormation::Om,
    );
    set_past_active(
        &mut lexeme,
        present_stem,
        PastActiveParticipleFormation::Ush,
    );
    set_past_passive(&mut lexeme, past_stem, PastPassiveParticipleFormation::T);
    lexeme
}

#[allow(clippy::too_many_arguments)]
fn build_velar_shti_member(
    lemma: &str,
    aspect: VerbAspect,
    first_present: &str,
    other_present: &str,
    imperative_stem: &str,
    past_stem: &str,
    passive_stem: &str,
) -> VerbLexeme {
    let mut lexeme = new_family_lexeme(lemma, aspect);
    insert_velar_present(&mut lexeme, first_present, other_present);
    insert_shti_infinitive_and_supine(&mut lexeme, lemma);
    set_imperfect(&mut lexeme, other_present, ImperfectFormation::PresentYatA);
    set_new_aorist(&mut lexeme, first_present);
    set_imperative(&mut lexeme, imperative_stem, ImperativeFormation::YatSeries);
    set_l_participle(&mut lexeme, past_stem);
    set_present_active(
        &mut lexeme,
        first_present,
        PresentActiveParticipleFormation::YushtHard,
    );
    set_present_passive(
        &mut lexeme,
        first_present,
        PresentPassiveParticipleFormation::Om,
    );
    set_past_active(&mut lexeme, past_stem, PastActiveParticipleFormation::Ush);
    set_past_passive(
        &mut lexeme,
        passive_stem,
        PastPassiveParticipleFormation::En,
    );
    lexeme
}

#[allow(clippy::too_many_arguments)]
fn build_chisti_member(
    lemma: &str,
    aspect: VerbAspect,
    first_present: &str,
    other_present: &str,
    imperative_stem: &str,
    past_stem: &str,
    passive_stem: &str,
) -> VerbLexeme {
    let mut lexeme = new_family_lexeme(lemma, aspect);
    if first_present == other_present {
        insert_hard_present(&mut lexeme, first_present);
    } else {
        insert_velar_present(&mut lexeme, first_present, other_present);
    }
    if lemma.ends_with("щи") {
        insert_shti_infinitive_and_supine(&mut lexeme, lemma);
    }
    set_imperfect(&mut lexeme, other_present, ImperfectFormation::PresentYatA);
    set_new_aorist(&mut lexeme, first_present);
    set_imperative(&mut lexeme, imperative_stem, ImperativeFormation::YatSeries);
    set_l_participle(&mut lexeme, past_stem);
    set_present_active(
        &mut lexeme,
        first_present,
        PresentActiveParticipleFormation::YushtHard,
    );
    set_present_passive(
        &mut lexeme,
        first_present,
        PresentPassiveParticipleFormation::Om,
    );
    set_past_active(&mut lexeme, past_stem, PastActiveParticipleFormation::Ush);
    set_past_passive(
        &mut lexeme,
        passive_stem,
        PastPassiveParticipleFormation::En,
    );
    lexeme
}

fn insert_shti_infinitive_and_supine(lexeme: &mut VerbLexeme, lemma: &str) {
    lexeme
        .exact_forms
        .insert(VerbMorphologyCell::Infinitive, lemma.to_string());
    if let Some(stem) = lemma.strip_suffix("щи") {
        lexeme
            .exact_forms
            .insert(VerbMorphologyCell::Supine, format!("{stem}щь"));
    }
}

#[derive(Debug, Clone, Copy)]
enum PresentShape {
    Hard,
    Soft,
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
            PresentShape::Soft | PresentShape::MixedSoft | PresentShape::Iotated => {
                ImperativeFormation::ISeries
            }
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
        PresentShape::Soft | PresentShape::MixedSoft => {
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

fn insert_present_shape(lexeme: &mut VerbLexeme, stem: &str, shape: PresentShape) {
    match shape {
        PresentShape::Hard | PresentShape::MixedSoft => insert_hard_present(lexeme, stem),
        PresentShape::Soft => insert_soft_present(lexeme, stem),
        PresentShape::Iotated => insert_iotated_present(lexeme, stem),
    }
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
                "влѣчѣахъ",
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
    fn family_member_inventory_is_closed_and_source_ordered() {
        let members = IrregularVerbFamilyMember::all().collect::<Vec<_>>();
        assert_eq!(members.len(), IrregularVerbFamilyMember::COUNT);
        let lemmas = members
            .iter()
            .map(|member| member.canonical_lemma())
            .collect::<BTreeSet<_>>();
        assert_eq!(lemmas.len(), IrregularVerbFamilyMember::COUNT);

        for member in members {
            assert_eq!(
                IrregularVerbFamilyMember::classify_source_lemma(member.canonical_lemma()),
                Some(member)
            );
            assert!(
                member
                    .group()
                    .family_anchors()
                    .contains(&member.canonical_lemma()),
                "{:?}",
                member
            );
            assert_eq!(member.source_section(), member.group().source_section());
            assert_eq!(member.lexeme().aspect, Some(member.aspect()));
        }

        assert_eq!(
            IrregularVerbFamilyMember::classify_source_lemma("възьмати")
                .expect("source-listed imperfective")
                .aspect(),
            VerbAspect::Imperfective
        );
        assert_eq!(
            IrregularVerbFamilyMember::classify_source_lemma("шити")
                .expect("LOVe perfective")
                .aspect(),
            VerbAspect::Perfective
        );
        assert_eq!(
            IrregularVerbFamilyMember::classify_source_lemma("чрьпати")
                .expect("LOVe biaspectual")
                .aspect(),
            VerbAspect::Biaspectual
        );
    }

    #[test]
    fn every_family_member_matches_its_table_434_present_key() {
        let first_present = finite_cell(FiniteTense::Present, Person::First, Number::Singular);
        let goldens = [
            ("искати", "искѫ"),
            ("ковати", "ковѫ"),
            ("метати", "метѫ"),
            ("уръвати", "уръвѫ"),
            ("обсновати", "обсновѫ"),
            ("съсати", "съсѫ"),
            ("тъкати", "тъкѫ"),
            ("дъхати", "душѫ"),
            ("зиꙗти", "зѣѭ"),
            ("зьдати", "зиждѫ"),
            ("лиꙗти", "лѣѭ"),
            ("пльзати", "плѣжѫ"),
            ("пьсати", "пишѫ"),
            ("възсльпати", "възслѣплѭ"),
            ("смиꙗти", "смѣѭ"),
            ("стръгати", "стружѫ"),
            ("стьлати", "стелѭ"),
            ("трьѕати", "трѣжѫ"),
            ("чрьпати", "чрѣплѭ"),
            ("възьмати", "въземлѭ"),
            ("бьрати", "берѫ"),
            ("дьрати", "дерѫ"),
            ("жьдати", "жидѫ"),
            ("зъвати", "зовѫ"),
            ("пьрати", "перѫ"),
            ("бльвати", "блюѭ"),
            ("пльвати", "плюѭ"),
            ("дѫти", "дъмѫ"),
            ("жити", "живѫ"),
            ("съжѧти", "съжьмѫ"),
            ("клѧти", "кльнѫ"),
            ("начѧти", "начьнѫ"),
            ("распѧти", "распьнѫ"),
            ("възѧти", "възьмѫ"),
            ("брати", "борѭ"),
            ("жрьти", "жьрѭ"),
            ("жѧти", "жьнѭ"),
            ("клати", "колѭ"),
            ("млѣти", "мелѭ"),
            ("трьти", "тьрѭ"),
            ("плути", "пловѫ"),
            ("рути", "ровѫ"),
            ("слути", "словѫ"),
            ("натрути", "натровѫ"),
            ("бити", "биѭ"),
            ("съвити", "съвиѭ"),
            ("гнити", "гниѭ"),
            ("вълити", "вълиѭ"),
            ("пити", "пиѭ"),
            ("крꙑти", "крꙑѭ"),
            ("мꙑти", "мꙑѭ"),
            ("рꙑти", "рꙑѭ"),
            ("шити", "шиѭ"),
            ("пѣти", "поѭ"),
            ("заврѣти", "завьрѫ"),
            ("пожрѣти", "пожьрѫ"),
            ("мрѣти", "мьрѫ"),
            ("обпрѣти", "обпьрѫ"),
            ("разскврѣти", "разсквьрѫ"),
            ("прострѣти", "простьрѫ"),
            ("влѣщи", "влѣкѫ"),
            ("брѣщи", "брѣгѫ"),
            ("отъврѣсти", "отъврьзѫ"),
            ("врѣщи", "врьгѫ"),
            ("вънисти", "въньзѫ"),
            ("стрѣщи", "стригѫ"),
            ("тлѣщи", "тлькѫ"),
            ("цвисти", "цвьтѫ"),
            ("чисти", "чьтѫ"),
            ("почрѣти", "почрьпѫ"),
        ];
        assert_eq!(goldens.len(), IrregularVerbFamilyMember::COUNT);

        for (lemma, expected) in goldens {
            let member = IrregularVerbFamilyMember::classify_source_lemma(lemma)
                .unwrap_or_else(|| panic!("missing source member {lemma}"));
            assert_eq!(
                finite(&member.lexeme(), first_present)
                    .unwrap_or_else(|error| panic!("{lemma}: {error:?}"))
                    .text,
                expected,
                "{lemma}"
            );
        }
    }

    #[test]
    fn family_members_match_the_additional_table_434_principal_parts() {
        let first_imperfect = finite_cell(FiniteTense::Imperfect, Person::First, Number::Singular);
        for (lemma, expected) in [
            ("брати", "борѣахъ"),
            ("жрьти", "жьрѣахъ"),
            ("жѧти", "жьнѣахъ"),
            ("клати", "колѣахъ"),
            ("млѣти", "мелѣахъ"),
            ("трьти", "тьрѣахъ"),
        ] {
            let member = IrregularVerbFamilyMember::classify_source_lemma(lemma)
                .unwrap_or_else(|| panic!("missing source member {lemma}"));
            assert_eq!(
                finite(&member.lexeme(), first_imperfect)
                    .expect("Table 434 imperfect key")
                    .text,
                expected,
                "{lemma}"
            );
        }

        let past_passive = short_nominative(ParticipleKind::PastPassive);
        for (lemma, expected) in [
            ("крꙑти", "кръвенъ"),
            ("мꙑти", "мъвенъ"),
            ("рꙑти", "ръвенъ"),
            ("шити", "шьвенъ"),
            ("влѣщи", "вльченъ"),
            ("брѣщи", "брьженъ"),
        ] {
            let member = IrregularVerbFamilyMember::classify_source_lemma(lemma)
                .unwrap_or_else(|| panic!("missing source member {lemma}"));
            assert_eq!(
                participle(&member.lexeme(), past_passive)
                    .expect("Table 434 past-passive key")
                    .text,
                expected,
                "{lemma}"
            );
        }

        let masculine_singular = LParticipleCell {
            gender: Gender::Masculine,
            number: Number::Singular,
        };
        // Table 434 transposes the мрѣти and разскврѣти past-stem labels.
        // Their own roots and the uniform Table 440 distribution determine the
        // corrected alignments used here.
        for (lemma, expected_l, expected_passive) in [
            ("заврѣти", "заврьлъ", "заврьтъ"),
            ("пожрѣти", "пожрьлъ", "пожрьтъ"),
            ("мрѣти", "мрьлъ", "мрьтъ"),
            ("обпрѣти", "обпрьлъ", "обпрьтъ"),
            ("разскврѣти", "разскврьлъ", "разскврьтъ"),
            ("прострѣти", "прострьлъ", "прострьтъ"),
        ] {
            let member = IrregularVerbFamilyMember::classify_source_lemma(lemma)
                .unwrap_or_else(|| panic!("missing source member {lemma}"));
            let lexeme = member.lexeme();
            assert_eq!(
                l_participle(&lexeme, masculine_singular)
                    .expect("Table 434 l-participle key")
                    .text,
                expected_l,
                "{lemma}"
            );
            assert_eq!(
                participle(&lexeme, past_passive)
                    .expect("Table 434 past-passive key")
                    .text,
                expected_passive,
                "{lemma}"
            );
        }
    }

    #[test]
    fn every_family_member_reaches_every_licensed_cell() {
        for member in IrregularVerbFamilyMember::all() {
            let lexeme = member.lexeme();
            for cell in FiniteVerbCell::all() {
                finite(&lexeme, cell)
                    .unwrap_or_else(|error| panic!("{member:?} {cell:?}: {error:?}"));
            }
            for cell in ImperativeCell::SUPPORTED {
                imperative(&lexeme, cell)
                    .unwrap_or_else(|error| panic!("{member:?} {cell:?}: {error:?}"));
            }
            infinitive(&lexeme).unwrap_or_else(|error| panic!("{member:?}: {error:?}"));
            supine(&lexeme).unwrap_or_else(|error| panic!("{member:?}: {error:?}"));
            for cell in LParticipleCell::all() {
                l_participle(&lexeme, cell)
                    .unwrap_or_else(|error| panic!("{member:?} {cell:?}: {error:?}"));
            }
            for kind in ParticipleKind::ALL {
                for cell in ParticipleCell::for_kind(kind) {
                    participle(&lexeme, cell)
                        .unwrap_or_else(|error| panic!("{member:?} {cell:?}: {error:?}"));
                }
            }
        }
    }

    #[test]
    fn metati_keeps_all_three_reviewed_present_analyses() {
        let member = IrregularVerbFamilyMember::classify_source_lemma("метати")
            .expect("Table 434 metati member");
        assert_eq!(member.analyses(), &METATI_ANALYSES);
        let third_singular = finite_cell(FiniteTense::Present, Person::Third, Number::Singular);
        let first_singular = finite_cell(FiniteTense::Present, Person::First, Number::Singular);
        let actual = member
            .analyses()
            .iter()
            .map(|analysis| {
                let lexeme = member
                    .lexeme_for_analysis(*analysis)
                    .expect("analysis belongs to metati");
                (
                    finite(&lexeme, first_singular)
                        .expect("first-singular analysis")
                        .text,
                    finite(&lexeme, third_singular)
                        .expect("third-singular analysis")
                        .text,
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            actual,
            [
                ("метѫ".to_string(), "мететъ".to_string()),
                ("мещѫ".to_string(), "мещетъ".to_string()),
                ("метаѭ".to_string(), "метаѥтъ".to_string()),
            ]
        );

        for other in IrregularVerbFamilyMember::all()
            .filter(|candidate| candidate.canonical_lemma() != "метати")
        {
            assert_eq!(other.analyses(), &POLIVANOVA_ANALYSIS, "{other:?}");
            assert!(
                other
                    .lexeme_for_analysis(IrregularVerbAnalysis::LoveMetatiJePresent)
                    .is_none(),
                "{other:?}"
            );
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
