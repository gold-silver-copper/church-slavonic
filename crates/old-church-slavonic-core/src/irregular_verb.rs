//! Listed irregular verb profiles in Polivanova 2023 §§434–464 and §509.
//!
//! Thirteen groups are reusable workstem classes distinct from the nineteen
//! whole-lexeme unique profiles in §§516–605. The remaining three profiles
//! preserve a bounded anomalous imperative or past-passive cell. Section 421
//! requires every simple and prefixed member of one root family to have the
//! same paradigmatic behavior, so the inventory includes both Table 434's
//! compact anchors and every additional marked OSD dictionary member.

use crate::verb::{
    VerbLexeme, insert_imperative_singular, set_imperative, set_imperfect, set_l_participle,
    set_new_aorist, set_past_active, set_past_passive, set_present_active, set_present_passive,
    set_sigmatic_vowel_aorist,
};
use crate::{
    FiniteTense, FiniteVerbCell, ImperativeFormation, ImperfectFormation, ImperfectVariantPolicy,
    PastActiveParticipleFormation, PastPassiveParticipleFormation,
    PresentActiveParticipleFormation, PresentPassiveParticipleFormation, VerbAspect, VerbClass,
    VerbMorphologyCell,
};

/// The exhaustive listed irregular-profile inventory of §§434–464 and §509.
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
    Rekti4cAnomalousImperative,
    Videti2AnomalousImperative,
    Obuti4vAnomalousPastPassive,
}

/// Independently sourced lexical analyses within an irregular family.
///
/// Most listed members have one reviewed analysis. `метати` is the exception:
/// the official LOVe record lists competing `je`- and `aje`-stem presents
/// alongside Polivanova's unsoftened `мет-` analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IrregularVerbAnalysis {
    PolivanovaListedIrregularProfile,
    LoveMetatiJePresent,
    LoveMetatiAjePresent,
}

impl IrregularVerbAnalysis {
    pub const fn code(self) -> &'static str {
        match self {
            Self::PolivanovaListedIrregularProfile => "polivanova-listed-irregular-profile",
            Self::LoveMetatiJePresent => "love-metati-je-present",
            Self::LoveMetatiAjePresent => "love-metati-aje-present",
        }
    }

    pub const fn authority(self) -> &'static str {
        match self {
            Self::PolivanovaListedIrregularProfile => {
                "Polivanova 2023 §§421, 434–464, and 509 with the marked OSD dictionary inventory"
            }
            Self::LoveMetatiJePresent | Self::LoveMetatiAjePresent => {
                "LMU Lexicon of Old Church Slavonic Verbs, metati record, reviewed 2026-08-15"
            }
        }
    }
}

const POLIVANOVA_ANALYSIS: [IrregularVerbAnalysis; 1] =
    [IrregularVerbAnalysis::PolivanovaListedIrregularProfile];
const METATI_ANALYSES: [IrregularVerbAnalysis; 3] = [
    IrregularVerbAnalysis::PolivanovaListedIrregularProfile,
    IrregularVerbAnalysis::LoveMetatiJePresent,
    IrregularVerbAnalysis::LoveMetatiAjePresent,
];

macro_rules! additional_family_members {
    ($( $group:ident => [$($lemma:literal),* $(,)?] ),* $(,)?) => {
        const OSD_ADDITIONAL_FAMILY_MEMBERS: [IrregularVerbFamilyMember; 250] = [
            $($(
                IrregularVerbFamilyMember {
                    group: IrregularVerbGroup::$group,
                    lemma: $lemma,
                },
            )*)*
        ];
    };
}

// Polivanova OSD dictionary rows whose marked profile is not one of the
// seventy compact Table 434 anchors. The list is exhaustive and source-keyed;
// §421, not surface similarity, licenses the shared family behavior.
additional_family_members! {
    Birati3NoSofteningUnstableVocalism => [
        "одьрати", "избьрати", "събьрати", "издьрати", "дожьдати", "пожьдати",
        "позъвати", "съзъвати", "възъвати", "запьрати", "попьрати", "прибьрати",
        "възбьрати", "прѣдьрати", "раздьрати", "въздьрати", "призъвати",
        "прозъвати", "распьрати",
    ],
    Biti4UnstableInfStemExpanded => [
        "убити", "обити", "упити", "избити", "побити", "събити", "завити",
        "извити", "повити", "излити", "испити", "прибити", "пробити", "прѣбити",
        "разбити", "възбити", "прѣвити", "изгнити", "съгнити", "пролити",
        "низъбити",
    ],
    Brati4LabileSoftened => [
        "пожѧти", "отрьти", "пожрьти", "заклати", "исклати", "съмлѣти", "сътрьти",
        "прѣбрати", "прѣтрьти", "съпобрати",
    ],
    Chisti4UnstablePresentStemExpanded => [
        "ичисти", "изврѣщи", "поврѣщи", "съврѣщи", "въврѣщи", "выврѣщи",
        "уврѣсти", "сътлѣщи", "почисти", "ращисти", "въчисти", "отъврѣщи",
        "приврѣщи", "разврѣщи", "възврѣщи", "отврѣсти", "поврѣсти",
        "пострѣщи", "отъчисти", "причисти", "низъврѣщи", "опроврѣщи",
        "разврѣсти", "отъцвисти", "процвисти", "испроврѣщи",
    ],
    Kleti4Labile => [
        "ѩти", "уѩти", "ожити", "заѩти", "изѧти", "наѩти", "отѧти", "обѧти",
        "поѩти", "учѧти", "надѫти", "иждити", "пожити", "приѩти", "прѣѩти",
        "вънѧти", "сънѧти", "запѧти", "съпѧти", "зачѧти", "въчѧти", "прижити",
        "подъѩти", "заклѧти", "припѧти", "пропѧти", "съпожити", "проклѧти",
        "прииждити", "въсприѩти",
    ],
    Kryti4UnstableArrestedInfStemExpanded => [
        "мыти", "рыти", "крыти", "омыти", "умыти", "окрыти", "укрыти", "измыти",
        "съшити", "закрыти", "покрыти", "съкрыти", "отъмыти", "издрыти",
        "отърыти", "прикрыти", "отъумыти", "подърыти",
    ],
    Metati3NoSoftening => [
        "оковати", "изискати", "обискати", "поискати", "поковати", "изметати",
        "пометати", "въметати", "основати", "истъкати", "сътъкати", "натъкати",
        "сънискати", "възискати", "отъметати", "приметати", "прѣметати",
        "разметати", "възметати",
    ],
    Mreti4UnstablePresentStemExpanded => [
        "умрѣти", "опрѣти", "въврѣти", "измрѣти", "проврѣти", "раскврѣти",
        "измрьмрѣти", "распрострѣти",
    ],
    Obuti4vAnomalousPastPassive => ["обути"],
    Peti4InfStemExpanded => ["испѣти", "съпѣти", "въспѣти"],
    Pisati3UnstableVocalism => [
        "имати", "заимати", "отьмати", "обьмати", "поимати", "изьмати",
        "излиꙗти", "налиꙗти", "облиꙗти", "полиꙗти", "вълиꙗти", "сълиꙗти",
        "усмиꙗти", "съзьдати", "приимати", "прѣимати", "въньмати", "съньмати",
        "пролиꙗти", "прѣлиꙗти", "разлиꙗти", "възлиꙗти", "напьсати",
        "съпьсати", "въпьсати", "посмиꙗти", "въсмиꙗти", "ичрьпати",
        "подъимати", "въсльпати", "просмиꙗти", "настьлати", "постьлати",
        "остръгати", "потрьѕати", "начрьпати", "почрьпати", "отътрьѕати",
        "протрьѕати", "прѣтрьѕати", "растрьѕати", "въстрьѕати", "въсприимати",
        "подъстьлати",
    ],
    Plivati3LabileArrested => [
        "бл҄ьвати", "пл҄ьвати", "обл҄ьвати", "опл҄ьвати", "избл҄ьвати",
        "запл҄ьвати",
    ],
    Pluti4InfStemExpanded => ["отъплути", "прѣплути", "въздрути"],
    Rekti4cAnomalousImperative => [
        "жещи", "пещи", "рещи", "тещи", "ожещи", "ужещи", "урещи", "отещи",
        "зажещи", "иждещи", "пожещи", "съжещи", "въжещи", "попещи", "зарещи",
        "нарещи", "порещи", "дотещи", "истещи", "потещи", "сътещи", "раждещи",
        "издрещи", "отърещи", "прорещи", "прѣрещи", "отътещи", "притещи",
        "прѣтещи", "растещи", "прѣдърещи", "прѣдътещи",
    ],
    Videti2AnomalousImperative => ["видѣти"],
    Vleshti4UnstablePresentStemExpanded => [
        "облѣщи", "извлѣщи", "повлѣщи", "съвлѣщи", "въвлѣщи", "отъвлѣщи",
        "привлѣщи", "съоблѣщи", "прѣоблѣщи",
    ],
}

/// One source-listed member of a listed irregular verb profile.
///
/// The identity is closed over the union of Table 434's compact anchors and
/// the marked OSD dictionary rows licensed by §421's family-equivalence rule.
/// Each member is routed through a bounded root-allomorph map; callers never
/// have to infer a prefix boundary or present stem from an arbitrary spelling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IrregularVerbFamilyMember {
    group: IrregularVerbGroup,
    lemma: &'static str,
}

impl IrregularVerbFamilyMember {
    pub const TABLE_434_ANCHOR_COUNT: usize = 70;
    pub const OSD_ADDITIONAL_MEMBER_COUNT: usize = 250;
    pub const COUNT: usize = Self::TABLE_434_ANCHOR_COUNT + Self::OSD_ADDITIONAL_MEMBER_COUNT;

    pub fn all() -> impl Iterator<Item = Self> {
        IrregularVerbGroup::ALL
            .into_iter()
            .flat_map(|group| {
                group
                    .family_anchors()
                    .iter()
                    .copied()
                    .map(move |lemma| Self { group, lemma })
            })
            .chain(OSD_ADDITIONAL_FAMILY_MEMBERS)
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
        if OSD_ADDITIONAL_FAMILY_MEMBERS.contains(&self) {
            return match self.lemma {
                "видѣти" | "рещи" => VerbAspect::Biaspectual,
                "жещи" | "пещи" | "тещи" | "имати" | "бл҄ьвати" | "пл҄ьвати" | "крыти" | "мыти"
                | "рыти" | "отъметати" => VerbAspect::Imperfective,
                "ѩти" | "обути" => VerbAspect::Perfective,
                lemma if lemma.ends_with("имати") || lemma.ends_with("ьмати") => {
                    // LOVe's imati record explicitly assigns the prefixed
                    // family the same imperfective aspect as the simple verb.
                    VerbAspect::Imperfective
                }
                lemma if lemma.ends_with("пьсати") => VerbAspect::Biaspectual,
                _ => VerbAspect::Perfective,
            };
        }
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
            IrregularVerbAnalysis::PolivanovaListedIrregularProfile => Some(self.lexeme()),
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
    pub const ALL: [Self; 16] = [
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
        Self::Rekti4cAnomalousImperative,
        Self::Videti2AnomalousImperative,
        Self::Obuti4vAnomalousPastPassive,
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
            Self::Rekti4cAnomalousImperative => "рещи",
            Self::Videti2AnomalousImperative => "видѣти",
            Self::Obuti4vAnomalousPastPassive => "обути",
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
            Self::Rekti4cAnomalousImperative | Self::Videti2AnomalousImperative => "§464",
            Self::Obuti4vAnomalousPastPassive => "§509",
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
            Self::Rekti4cAnomalousImperative
            | Self::Videti2AnomalousImperative
            | Self::Obuti4vAnomalousPastPassive => &[],
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
    if let Some(lexeme) = assemble_source_family_member(member) {
        return lexeme;
    }
    assemble_table_434_family_member(member)
}

fn assemble_table_434_family_member(member: IrregularVerbFamilyMember) -> VerbLexeme {
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
        IrregularVerbGroup::Rekti4cAnomalousImperative
        | IrregularVerbGroup::Videti2AnomalousImperative
        | IrregularVerbGroup::Obuti4vAnomalousPastPassive => {
            // The exhaustive source-family assembler above owns every member
            // of these bounded anomaly profiles. Preserve a typed
            // missing-metadata failure if the internal inventory ever drifts.
            new_family_lexeme(lemma, aspect)
        }
    }
}

fn assemble_source_family_member(member: IrregularVerbFamilyMember) -> Option<VerbLexeme> {
    let lemma = member.canonical_lemma();
    let aspect = member.aspect();
    match member.group() {
        IrregularVerbGroup::Metati3NoSoftening => class_three_family_member(
            lemma,
            aspect,
            &[
                ("искати", "иск", PresentShape::Hard),
                ("ковати", "ков", PresentShape::Hard),
                ("метати", "мет", PresentShape::Hard),
                ("ръвати", "ръв", PresentShape::Hard),
                ("сновати", "снов", PresentShape::Hard),
                ("съсати", "със", PresentShape::Hard),
                ("тъкати", "тък", PresentShape::Hard),
            ],
        ),
        IrregularVerbGroup::Pisati3UnstableVocalism => class_three_family_member(
            lemma,
            aspect,
            &[
                ("дъхати", "душ", PresentShape::MixedSoft),
                ("зиꙗти", "зѣ", PresentShape::Iotated),
                ("зьдати", "зижд", PresentShape::MixedSoft),
                ("имати", "ѥмл", PresentShape::Soft),
                ("ьмати", "емл", PresentShape::Soft),
                ("лиꙗти", "лѣ", PresentShape::Iotated),
                ("пьсати", "пиш", PresentShape::MixedSoft),
                ("пльзати", "плѣж", PresentShape::MixedSoft),
                ("сльпати", "слѣпл", PresentShape::Soft),
                ("смиꙗти", "смѣ", PresentShape::Iotated),
                ("стьлати", "стел", PresentShape::Soft),
                ("стръгати", "струж", PresentShape::MixedSoft),
                ("трьѕати", "трѣж", PresentShape::MixedSoft),
                ("чрьпати", "чрѣпл", PresentShape::Soft),
            ],
        ),
        IrregularVerbGroup::Birati3NoSofteningUnstableVocalism => class_three_family_member(
            lemma,
            aspect,
            &[
                ("бьрати", "бер", PresentShape::Hard),
                ("дьрати", "дер", PresentShape::Hard),
                ("жьдати", "жид", PresentShape::Hard),
                ("зъвати", "зов", PresentShape::Hard),
                ("пьрати", "пер", PresentShape::Hard),
            ],
        ),
        IrregularVerbGroup::Plivati3LabileArrested => class_three_family_member(
            lemma,
            aspect,
            &[
                ("бл҄ьвати", "бл҄ю", PresentShape::Iotated),
                ("пл҄ьвати", "пл҄ю", PresentShape::Iotated),
                ("бльвати", "блю", PresentShape::Iotated),
                ("пльвати", "плю", PresentShape::Iotated),
            ],
        ),
        IrregularVerbGroup::Kleti4Labile => {
            let infinitive_stem = lemma.strip_suffix("ти")?;
            let present_stem = prefixed_root(
                lemma,
                &[
                    ("дѫти", "дъм"),
                    ("иждити", "иждив"),
                    ("жити", "жив"),
                    ("съжѧти", "съжьм"),
                    ("клѧти", "кльн"),
                    ("пѧти", "пьн"),
                    ("чѧти", "чьн"),
                ],
            )
            .or_else(|| replace_suffix(lemma, "ѩти", "им"))
            .or_else(|| replace_suffix(lemma, "ѧти", "ьм"))?;
            Some(build_kleti_member(
                lemma,
                aspect,
                &present_stem,
                infinitive_stem,
            ))
        }
        IrregularVerbGroup::Brati4LabileSoftened => {
            let infinitive_stem = lemma.strip_suffix("ти")?;
            let present_stem = prefixed_root(
                lemma,
                &[
                    ("брати", "бор"),
                    ("жрьти", "жьр"),
                    ("жѧти", "жьн"),
                    ("клати", "кол"),
                    ("млѣти", "мел"),
                    ("трьти", "тьр"),
                ],
            )?;
            Some(build_brati_member(
                lemma,
                aspect,
                &present_stem,
                infinitive_stem,
            ))
        }
        IrregularVerbGroup::Pluti4InfStemExpanded => {
            let infinitive_stem = lemma.strip_suffix("ти")?;
            let present_stem = prefixed_root(
                lemma,
                &[
                    ("плути", "плов"),
                    ("рути", "ров"),
                    ("слути", "слов"),
                    ("трути", "тров"),
                ],
            )?;
            Some(build_pluti_member(
                lemma,
                aspect,
                &present_stem,
                infinitive_stem,
            ))
        }
        IrregularVerbGroup::Biti4UnstableInfStemExpanded => {
            if !["бити", "вити", "гнити", "лити", "пити"]
                .iter()
                .any(|ending| lemma.ends_with(ending))
            {
                return None;
            }
            let stem = lemma.strip_suffix("ти")?;
            Some(build_biti_member(lemma, aspect, stem))
        }
        IrregularVerbGroup::Kryti4UnstableArrestedInfStemExpanded => {
            let stem = lemma.strip_suffix("ти")?;
            let passive_stem = prefixed_root(
                lemma,
                &[
                    ("крꙑти", "кръв"),
                    ("крыти", "кръв"),
                    ("мꙑти", "мъв"),
                    ("мыти", "мъв"),
                    ("рꙑти", "ръв"),
                    ("рыти", "ръв"),
                    ("шити", "шьв"),
                ],
            )?;
            Some(build_kryti_member(lemma, aspect, stem, &passive_stem))
        }
        IrregularVerbGroup::Peti4InfStemExpanded => {
            let prefix = lemma.strip_suffix("пѣти")?;
            Some(build_peti_member_with_stems(
                lemma,
                aspect,
                &format!("{prefix}по"),
                &format!("{prefix}пѣ"),
            ))
        }
        IrregularVerbGroup::Mreti4UnstablePresentStemExpanded => {
            let infinitive_stem = lemma.strip_suffix("ти")?;
            let (present_stem, past_stem) = if lemma.ends_with("мрьмрѣти") {
                (
                    replace_suffix(lemma, "мрьмрѣти", "мрьмьр")?,
                    replace_suffix(lemma, "мрьмрѣти", "мрьмрь")?,
                )
            } else {
                let present = prefixed_root(
                    lemma,
                    &[
                        ("врѣти", "вьр"),
                        ("жрѣти", "жьр"),
                        ("мрѣти", "мьр"),
                        ("прѣти", "пьр"),
                        ("скврѣти", "сквьр"),
                        ("стрѣти", "стьр"),
                    ],
                )?;
                let past = prefixed_root(
                    lemma,
                    &[
                        ("врѣти", "врь"),
                        ("жрѣти", "жрь"),
                        ("мрѣти", "мрь"),
                        ("прѣти", "прь"),
                        ("скврѣти", "скврь"),
                        ("стрѣти", "стрь"),
                    ],
                )?;
                (present, past)
            };
            Some(build_mreti_member(
                lemma,
                aspect,
                &present_stem,
                infinitive_stem,
                &past_stem,
            ))
        }
        IrregularVerbGroup::Vleshti4UnstablePresentStemExpanded => {
            let stems = prefixed_stem_set(
                lemma,
                &[
                    ("облѣщи", ["облѣк", "облѣч", "облѣц", "обльк", "обльч"]),
                    ("влѣщи", ["влѣк", "влѣч", "влѣц", "вльк", "вльч"]),
                    ("брѣщи", ["брѣг", "брѣж", "брѣѕ", "брьг", "брьж"]),
                ],
            )?;
            Some(build_velar_shti_member(
                lemma, aspect, &stems[0], &stems[1], &stems[2], &stems[3], &stems[4],
            ))
        }
        IrregularVerbGroup::Chisti4UnstablePresentStemExpanded => {
            let stems = prefixed_stem_set(
                lemma,
                &[
                    ("врѣщи", ["врьг", "врьж", "врьѕ", "врьг", "врьж"]),
                    ("врѣсти", ["врьз", "врьз", "врьз", "врьз", "врьз"]),
                    ("нисти", ["ньз", "ньз", "ньз", "ньз", "ньз"]),
                    ("стрѣщи", ["стриг", "стриж", "стриѕ", "стриг", "стриж"]),
                    ("тлѣщи", ["тльк", "тльч", "тльц", "тльк", "тльч"]),
                    ("цвисти", ["цвьт", "цвьт", "цвьт", "цвьт", "цвьт"]),
                    ("щисти", ["щьт", "щьт", "щьт", "щьт", "щьт"]),
                    ("чисти", ["чьт", "чьт", "чьт", "чьт", "чьт"]),
                    ("чрѣти", ["чрьп", "чрьп", "чрьп", "чрьп", "чрьп"]),
                ],
            )?;
            Some(build_chisti_member(
                lemma, aspect, &stems[0], &stems[1], &stems[2], &stems[3], &stems[4],
            ))
        }
        IrregularVerbGroup::Rekti4cAnomalousImperative => {
            let first_present = match lemma {
                "иждещи" => "иждег".to_string(),
                "раждещи" => "раждег".to_string(),
                _ => prefixed_root(
                    lemma,
                    &[
                        ("жещи", "жег"),
                        ("пещи", "пек"),
                        ("рещи", "рек"),
                        ("тещи", "тек"),
                    ],
                )?,
            };
            let (other_present, imperative_stem) =
                if let Some(base) = first_present.strip_suffix("ек") {
                    (format!("{base}еч"), format!("{base}ьц"))
                } else {
                    let base = first_present.strip_suffix("ег")?;
                    (format!("{base}еж"), format!("{base}ьѕ"))
                };
            Some(build_rekti_member(
                lemma,
                aspect,
                &first_present,
                &other_present,
                &imperative_stem,
            ))
        }
        IrregularVerbGroup::Videti2AnomalousImperative if lemma == "видѣти" => {
            Some(build_videti_member())
        }
        IrregularVerbGroup::Obuti4vAnomalousPastPassive if lemma == "обути" => {
            Some(build_obuti_member())
        }
        IrregularVerbGroup::Videti2AnomalousImperative
        | IrregularVerbGroup::Obuti4vAnomalousPastPassive => None,
    }
}

fn class_three_family_member(
    lemma: &str,
    aspect: VerbAspect,
    roots: &[(&str, &str, PresentShape)],
) -> Option<VerbLexeme> {
    let infinitive_stem = lemma.strip_suffix("ти")?;
    for (citation_tail, present_root, shape) in roots {
        if let Some(present_stem) = replace_suffix(lemma, citation_tail, present_root) {
            return Some(build_class_three_member(
                lemma,
                aspect,
                infinitive_stem,
                &present_stem,
                *shape,
            ));
        }
    }
    None
}

fn prefixed_root(lemma: &str, roots: &[(&str, &str)]) -> Option<String> {
    roots
        .iter()
        .find_map(|(citation_tail, root)| replace_suffix(lemma, citation_tail, root))
}

fn replace_suffix(lemma: &str, citation_tail: &str, replacement: &str) -> Option<String> {
    lemma
        .strip_suffix(citation_tail)
        .map(|prefix| format!("{prefix}{replacement}"))
}

fn prefixed_stem_set(lemma: &str, roots: &[(&str, [&str; 5])]) -> Option<[String; 5]> {
    roots.iter().find_map(|(citation_tail, stems)| {
        let prefix = lemma.strip_suffix(citation_tail)?;
        Some(stems.map(|stem| format!("{prefix}{stem}")))
    })
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
    build_peti_member_with_stems(lemma, aspect, "по", "пѣ")
}

fn build_peti_member_with_stems(
    lemma: &str,
    aspect: VerbAspect,
    present_stem: &str,
    infinitive_stem: &str,
) -> VerbLexeme {
    let mut lexeme = new_family_lexeme(lemma, aspect);
    insert_iotated_present(&mut lexeme, present_stem);
    set_imperfect(&mut lexeme, present_stem, ImperfectFormation::PresentYatA);
    set_vowel_aorist(&mut lexeme, infinitive_stem);
    set_imperative(&mut lexeme, present_stem, ImperativeFormation::ISeries);
    insert_imperative_singular(&mut lexeme, &format!("{infinitive_stem}и"));
    set_l_participle(&mut lexeme, infinitive_stem);
    configure_iotated_present_participles(&mut lexeme, present_stem);
    set_past_active(
        &mut lexeme,
        infinitive_stem,
        PastActiveParticipleFormation::Vush,
    );
    set_past_passive(
        &mut lexeme,
        infinitive_stem,
        PastPassiveParticipleFormation::N,
    );
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

fn build_rekti_member(
    lemma: &str,
    aspect: VerbAspect,
    first_present: &str,
    other_present: &str,
    imperative_stem: &str,
) -> VerbLexeme {
    let mut lexeme = new_family_lexeme(lemma, aspect);
    insert_velar_present(&mut lexeme, first_present, other_present);
    insert_shti_infinitive_and_supine(&mut lexeme, lemma);
    set_imperfect(&mut lexeme, first_present, ImperfectFormation::PalatalizedA);
    set_new_aorist(&mut lexeme, first_present);
    set_imperative(&mut lexeme, imperative_stem, ImperativeFormation::YatSeries);
    set_l_participle(&mut lexeme, first_present);
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
    set_past_active(
        &mut lexeme,
        first_present,
        PastActiveParticipleFormation::Ush,
    );
    set_past_passive(
        &mut lexeme,
        other_present,
        PastPassiveParticipleFormation::En,
    );
    lexeme
}

fn build_videti_member() -> VerbLexeme {
    let mut lexeme = new_family_lexeme("видѣти", VerbAspect::Biaspectual);
    insert_present_forms(
        &mut lexeme,
        [
            "виждѫ",
            "видиши",
            "видитъ",
            "видивѣ",
            "видита",
            "видите",
            "видимъ",
            "видите",
            "видѧтъ",
        ]
        .map(str::to_string),
    );
    set_imperfect(&mut lexeme, "видѣ", ImperfectFormation::A);
    set_vowel_aorist(&mut lexeme, "видѣ");
    set_imperative(&mut lexeme, "вид", ImperativeFormation::ISeries);
    insert_imperative_singular(&mut lexeme, "виждь");
    set_l_participle(&mut lexeme, "видѣ");
    set_present_active(
        &mut lexeme,
        "вид",
        PresentActiveParticipleFormation::YeshtSoft,
    );
    set_present_passive(&mut lexeme, "вид", PresentPassiveParticipleFormation::Im);
    set_past_active(&mut lexeme, "видѣ", PastActiveParticipleFormation::Vush);
    set_past_passive(&mut lexeme, "видѣ", PastPassiveParticipleFormation::N);
    lexeme
}

fn build_obuti_member() -> VerbLexeme {
    let mut lexeme = new_family_lexeme("обути", VerbAspect::Perfective);
    insert_iotated_present(&mut lexeme, "обу");
    lexeme.stems.imperfect = Some("обу".to_string());
    lexeme.formations.imperfect = Some(ImperfectFormation::PresentA);
    lexeme.formations.imperfect_variant_policy = Some(ImperfectVariantPolicy::IotatedOnly);
    set_vowel_aorist(&mut lexeme, "обу");
    set_imperative(&mut lexeme, "обу", ImperativeFormation::ISeries);
    set_l_participle(&mut lexeme, "обу");
    configure_iotated_present_participles(&mut lexeme, "обу");
    set_past_active(&mut lexeme, "обу", PastActiveParticipleFormation::Vush);
    set_past_passive(&mut lexeme, "обув", PastPassiveParticipleFormation::En);
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

fn set_vowel_aorist(lexeme: &mut VerbLexeme, stem: &str) {
    set_sigmatic_vowel_aorist(lexeme, stem, stem);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::verb::{finite, imperative, infinitive, l_participle, participle, supine};
    use crate::{
        AdjectiveCell, AdjectiveForm, Animacy, Case, Gender, ImperativeCell, LParticipleCell,
        Number, ParticipleCell, ParticipleKind, Person,
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
        let expected_counts = [7, 13, 5, 2, 7, 6, 4, 5, 4, 1, 6, 2, 8, 0, 0, 0];
        let mut anchors = BTreeSet::new();
        for (group, count) in IrregularVerbGroup::ALL.into_iter().zip(expected_counts) {
            assert_eq!(group.family_anchors().len(), count, "{group:?}");
            if count > 0 {
                assert!(group.family_anchors().contains(&group.representative()));
            }
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
        assert_eq!(
            anchors.len(),
            IrregularVerbFamilyMember::TABLE_434_ANCHOR_COUNT
        );
    }

    #[test]
    fn representative_profiles_match_source_key_forms() {
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
            (
                IrregularVerbGroup::Rekti4cAnomalousImperative,
                "рекѫ",
                "речеши",
                "рьцѣте",
                "речаахъ",
                "рекохъ",
                "рече",
            ),
            (
                IrregularVerbGroup::Videti2AnomalousImperative,
                "виждѫ",
                "видиши",
                "видите",
                "видѣахъ",
                "видѣхъ",
                "видѣ",
            ),
            (
                IrregularVerbGroup::Obuti4vAnomalousPastPassive,
                "обуѭ",
                "обуѥши",
                "обуите",
                "обуꙗхъ",
                "обухъ",
                "обу",
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
    fn source_assembler_preserves_every_legacy_table_434_cell() {
        for member in
            IrregularVerbFamilyMember::all().take(IrregularVerbFamilyMember::TABLE_434_ANCHOR_COUNT)
        {
            let source = assemble_source_family_member(member)
                .unwrap_or_else(|| panic!("missing source assembler for {member:?}"));
            let legacy = assemble_table_434_family_member(member);

            for cell in FiniteVerbCell::all() {
                assert_eq!(
                    finite(&source, cell),
                    finite(&legacy, cell),
                    "{member:?} {cell:?}"
                );
            }
            for cell in ImperativeCell::SUPPORTED {
                assert_eq!(
                    imperative(&source, cell),
                    imperative(&legacy, cell),
                    "{member:?} {cell:?}"
                );
            }
            assert_eq!(infinitive(&source), infinitive(&legacy), "{member:?}");
            assert_eq!(supine(&source), supine(&legacy), "{member:?}");
            for cell in LParticipleCell::all() {
                assert_eq!(
                    l_participle(&source, cell),
                    l_participle(&legacy, cell),
                    "{member:?} {cell:?}"
                );
            }
            for kind in ParticipleKind::ALL {
                for cell in ParticipleCell::for_kind(kind) {
                    assert_eq!(
                        participle(&source, cell),
                        participle(&legacy, cell),
                        "{member:?} {cell:?}"
                    );
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
                    .contains(&member.canonical_lemma())
                    || OSD_ADDITIONAL_FAMILY_MEMBERS.contains(&member),
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
        assert_eq!(
            goldens.len(),
            IrregularVerbFamilyMember::TABLE_434_ANCHOR_COUNT
        );

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
    fn osd_family_derivatives_and_bounded_anomalies_preserve_their_seams() {
        let first_present = finite_cell(FiniteTense::Present, Person::First, Number::Singular);
        let first_imperfect = finite_cell(FiniteTense::Imperfect, Person::First, Number::Singular);
        let third_aorist = finite_cell(FiniteTense::Aorist, Person::Third, Number::Singular);
        let singular_imperative = ImperativeCell {
            person: Person::Second,
            number: Number::Singular,
        };
        let masculine_singular = LParticipleCell {
            gender: Gender::Masculine,
            number: Number::Singular,
        };
        let past_passive = short_nominative(ParticipleKind::PastPassive);

        for (lemma, present, imperfect, aorist, imperative_form, l_form) in [
            (
                "потещи",
                "потекѫ",
                "потечаахъ",
                "потече",
                "потьци",
                "потеклъ",
            ),
            (
                "издрещи",
                "издрекѫ",
                "издречаахъ",
                "издрече",
                "издрьци",
                "издреклъ",
            ),
            (
                "раждещи",
                "раждегѫ",
                "раждежаахъ",
                "раждеже",
                "раждьѕи",
                "раждеглъ",
            ),
        ] {
            let lexeme = IrregularVerbFamilyMember::classify_source_lemma(lemma)
                .unwrap_or_else(|| panic!("missing OSD member {lemma}"))
                .lexeme();
            assert_eq!(
                finite(&lexeme, first_present).expect("present").text,
                present
            );
            assert_eq!(
                finite(&lexeme, first_imperfect).expect("imperfect").text,
                imperfect
            );
            assert_eq!(finite(&lexeme, third_aorist).expect("aorist").text, aorist);
            assert_eq!(
                imperative(&lexeme, singular_imperative)
                    .expect("singular imperative")
                    .text,
                imperative_form
            );
            assert_eq!(
                l_participle(&lexeme, masculine_singular)
                    .expect("l-participle")
                    .text,
                l_form
            );
        }

        let videti = IrregularVerbFamilyMember::classify_source_lemma("видѣти")
            .expect("bounded videti anomaly")
            .lexeme();
        assert_eq!(
            imperative(&videti, singular_imperative)
                .expect("videti singular imperative")
                .text,
            "виждь"
        );

        let obuti = IrregularVerbFamilyMember::classify_source_lemma("обути")
            .expect("bounded obuti anomaly")
            .lexeme();
        assert_eq!(
            participle(&obuti, past_passive)
                .expect("obuti past passive")
                .text,
            "обувенъ"
        );

        for (lemma, present, aorist, passive) in [
            ("облѣщи", "облѣкѫ", "облѣче", "обльченъ"),
            ("заѩти", "заимѫ", "заѩ", "заѩтъ"),
        ] {
            let lexeme = IrregularVerbFamilyMember::classify_source_lemma(lemma)
                .unwrap_or_else(|| panic!("missing OSD member {lemma}"))
                .lexeme();
            assert_eq!(
                finite(&lexeme, first_present).expect("present").text,
                present
            );
            assert_eq!(finite(&lexeme, third_aorist).expect("aorist").text, aorist);
            assert_eq!(
                participle(&lexeme, past_passive)
                    .expect("past passive")
                    .text,
                passive
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
