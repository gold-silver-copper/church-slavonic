//! Source-bounded productive verb profiles from Polivanova's OSD dictionary.
//!
//! The embedded inventory records identities, class assignments, and the
//! otherwise unrecoverable consonant stem of class 4c. Paradigms are assembled
//! from the productive rules in Polivanova 2023 §§409–462; no surface paradigm
//! is copied from the source.

use crate::verb::VerbLexeme;
use crate::{
    AoristFormation, ImperativeFormation, ImperfectFormation, ImperfectVariantPolicy,
    InflectionError, PastActiveParticipleFormation, PastPassiveParticipleFormation,
    PresentActiveParticipleFormation, PresentFormation, PresentPassiveParticipleFormation,
    VerbClass,
};
use std::collections::BTreeSet;
use std::sync::OnceLock;

const SOURCE_ROWS: &str = include_str!("../data/polivanova_regular_verbs.tsv");

/// One of Polivanova's seven productive verb classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PolivanovaRegularVerbClass {
    One,
    Two,
    Three,
    FourConsonant,
    FourVowel,
    Five,
    Six,
    Seven,
}

impl PolivanovaRegularVerbClass {
    pub const ALL: [Self; 8] = [
        Self::One,
        Self::Two,
        Self::Three,
        Self::FourConsonant,
        Self::FourVowel,
        Self::Five,
        Self::Six,
        Self::Seven,
    ];

    pub const fn code(self) -> &'static str {
        match self {
            Self::One => "1",
            Self::Two => "2",
            Self::Three => "3",
            Self::FourConsonant => "4c",
            Self::FourVowel => "4v",
            Self::Five => "5",
            Self::Six => "6",
            Self::Seven => "7",
        }
    }

    pub const fn source_section(self) -> &'static str {
        match self {
            Self::One => "§§486–488",
            Self::Two => "§§489–493",
            Self::Three => "§§494–498",
            Self::FourConsonant | Self::FourVowel => "§§499–502",
            Self::Five => "§§503–505",
            Self::Six => "§§506–511",
            Self::Seven => "§§512–515",
        }
    }

    fn from_code(code: &str) -> Self {
        match code {
            "2" => Self::Two,
            "3" => Self::Three,
            "4c" => Self::FourConsonant,
            "4v" => Self::FourVowel,
            "5" => Self::Five,
            "6" => Self::Six,
            "7" => Self::Seven,
            _ => Self::One,
        }
    }
}

/// One exact OSD row. Row identity is retained even when normalized lemmas are
/// homographic or when two homonyms have identical productive morphology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RegularVerbSourceMember {
    source_row: u16,
    lemma: &'static str,
    class: PolivanovaRegularVerbClass,
    class_four_basic_stem: Option<&'static str>,
}

impl RegularVerbSourceMember {
    pub const COUNT: usize = 2_297;

    pub fn all() -> impl Iterator<Item = Self> {
        source_members().iter().copied()
    }

    pub fn from_source_row(source_row: u16) -> Option<Self> {
        source_members()
            .binary_search_by_key(&source_row, |member| member.source_row)
            .ok()
            .map(|index| source_members()[index])
    }

    pub const fn source_row(self) -> u16 {
        self.source_row
    }

    pub const fn canonical_lemma(self) -> &'static str {
        self.lemma
    }

    pub const fn class(self) -> PolivanovaRegularVerbClass {
        self.class
    }

    pub const fn class_four_basic_stem(self) -> Option<&'static str> {
        self.class_four_basic_stem
    }

    pub fn specification(self) -> Result<PolivanovaRegularVerbSpecification, InflectionError> {
        PolivanovaRegularVerbSpecification::new(
            self.lemma,
            self.class,
            self.class_four_basic_stem.map(str::to_string),
        )
    }

    /// Assemble every productive analysis licensed for this source identity.
    /// `сѣти` and its regular prefixed family retain both n- and t-participles;
    /// §485's `увѧсти` takes the lexically selected t-participle.
    pub fn lexemes(self) -> Result<Vec<VerbLexeme>, InflectionError> {
        self.specification()?.lexemes()
    }
}

/// All regular source rows sharing one normalized citation spelling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RegularVerbFamily {
    lemma: &'static str,
}

impl RegularVerbFamily {
    pub fn all() -> impl Iterator<Item = Self> {
        regular_families().iter().copied()
    }

    pub fn classify_source_lemma(lemma: &str) -> Option<Self> {
        regular_families()
            .binary_search_by(|family| family.lemma.cmp(lemma))
            .ok()
            .map(|index| regular_families()[index])
    }

    pub const fn canonical_lemma(self) -> &'static str {
        self.lemma
    }

    pub fn members(self) -> impl Iterator<Item = RegularVerbSourceMember> {
        RegularVerbSourceMember::all().filter(move |member| member.lemma == self.lemma)
    }
}

fn source_members() -> &'static [RegularVerbSourceMember] {
    static MEMBERS: OnceLock<Vec<RegularVerbSourceMember>> = OnceLock::new();
    MEMBERS
        .get_or_init(|| SOURCE_ROWS.lines().skip(1).map(parse_source_row).collect())
        .as_slice()
}

fn regular_families() -> &'static [RegularVerbFamily] {
    static FAMILIES: OnceLock<Vec<RegularVerbFamily>> = OnceLock::new();
    FAMILIES
        .get_or_init(|| {
            RegularVerbSourceMember::all()
                .map(RegularVerbSourceMember::canonical_lemma)
                .collect::<BTreeSet<_>>()
                .into_iter()
                .map(|lemma| RegularVerbFamily { lemma })
                .collect()
        })
        .as_slice()
}

/// Validated productive principal-part specification for one regular verb.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolivanovaRegularVerbSpecification {
    lemma: String,
    class: PolivanovaRegularVerbClass,
    class_four_basic_stem: Option<String>,
}

impl PolivanovaRegularVerbSpecification {
    pub fn new(
        lemma: impl Into<String>,
        class: PolivanovaRegularVerbClass,
        class_four_basic_stem: Option<String>,
    ) -> Result<Self, InflectionError> {
        let lemma = crate::orthography::canonical_display(&lemma.into())?;
        if !lemma.ends_with('и') {
            return Err(InflectionError::InvalidInput {
                reason: "a regular OCS verb specification needs an infinitive ending in и"
                    .to_string(),
            });
        }
        let class_four_basic_stem = class_four_basic_stem
            .map(|stem| crate::orthography::canonical_display(&stem))
            .transpose()?;
        if (class == PolivanovaRegularVerbClass::FourConsonant) != class_four_basic_stem.is_some() {
            return Err(InflectionError::InvalidInput {
                reason: "only class 4c requires its morphophonological consonant stem".to_string(),
            });
        }
        Ok(Self {
            lemma,
            class,
            class_four_basic_stem,
        })
    }

    pub fn lexemes(&self) -> Result<Vec<VerbLexeme>, InflectionError> {
        let variants = if self.class == PolivanovaRegularVerbClass::FourVowel
            && expanded_stem(&self.lemma)?.ends_with("сѣ")
        {
            &[
                PastPassiveParticipleFormation::N,
                PastPassiveParticipleFormation::T,
            ][..]
        } else if self.class == PolivanovaRegularVerbClass::FourConsonant && self.lemma == "увѧсти"
        {
            &[PastPassiveParticipleFormation::T][..]
        } else {
            &[default_past_passive(self.class)][..]
        };
        variants
            .iter()
            .map(|formation| self.lexeme(*formation))
            .collect()
    }

    fn lexeme(
        &self,
        past_passive_formation: PastPassiveParticipleFormation,
    ) -> Result<VerbLexeme, InflectionError> {
        let expanded = if self.class == PolivanovaRegularVerbClass::FourConsonant {
            String::new()
        } else {
            expanded_stem(&self.lemma)?
        };
        let mut lexeme = VerbLexeme::new(
            self.lemma.clone(),
            if matches!(
                self.class,
                PolivanovaRegularVerbClass::One | PolivanovaRegularVerbClass::Two
            ) {
                VerbClass::II1
            } else {
                VerbClass::IA1
            },
        );

        match self.class {
            PolivanovaRegularVerbClass::One => {
                let basic = without_last_char(&expanded)?;
                let softened = substitutive_softening(&basic);
                let glide = ends_in_vowel(&basic);
                let marked = softened.ends_with('҄');
                let surface_softened = softened.trim_end_matches('҄').to_string();
                lexeme.stems.present = Some(basic.clone());
                lexeme.stems.present_first_singular = Some(surface_softened);
                if !glide && !marked {
                    lexeme.formations.present = Some(PresentFormation::HardI);
                }
                lexeme.stems.imperfect = Some(if glide || marked {
                    format!("{}ꙗ", softened.trim_end_matches('҄'))
                } else {
                    format!("{softened}а")
                });
                lexeme.formations.imperfect = Some(ImperfectFormation::A);
                lexeme.formations.imperfect_variant_policy =
                    Some(ImperfectVariantPolicy::UncontractedOnly);
                set_vowel_aorist(&mut lexeme, &expanded);
                lexeme.stems.imperative = Some(basic.clone());
                lexeme.formations.imperative = Some(ImperativeFormation::ISeries);
                lexeme.stems.l_participle = Some(expanded.clone());
                lexeme.stems.present_active_participle = Some(basic.clone());
                lexeme.formations.present_active_participle =
                    Some(PresentActiveParticipleFormation::YeshtSoft);
                lexeme.stems.present_passive_participle = Some(basic);
                lexeme.formations.present_passive_participle =
                    Some(PresentPassiveParticipleFormation::Im);
                lexeme.stems.past_active_participle = Some(if glide {
                    softened.trim_end_matches('҄').to_string()
                } else {
                    softened.clone()
                });
                lexeme.formations.past_active_participle = Some(if glide {
                    PastActiveParticipleFormation::IshAfterGlide
                } else {
                    PastActiveParticipleFormation::Ish
                });
                lexeme.stems.past_passive_participle = Some(if glide || marked {
                    format!("{}ѥ", softened.trim_end_matches('҄'))
                } else {
                    format!("{softened}е")
                });
                lexeme.formations.past_passive_participle = Some(PastPassiveParticipleFormation::N);
            }
            PolivanovaRegularVerbClass::Two => {
                let basic = without_last_char(&expanded)?;
                let softened = substitutive_softening(&basic);
                lexeme.stems.present = Some(basic.clone());
                lexeme.stems.present_first_singular =
                    Some(softened.trim_end_matches('҄').to_string());
                if !ends_in_vowel(&basic) && !softened.ends_with('҄') {
                    lexeme.formations.present = Some(PresentFormation::HardI);
                }
                set_expanded_past_systems(&mut lexeme, &expanded, ImperfectFormation::A);
                set_i_present_nominals(&mut lexeme, &basic);
            }
            PolivanovaRegularVerbClass::Three => {
                let basic = without_last_char(&expanded)?;
                let softened = substitutive_softening(&basic);
                let iotated = ends_in_vowel(&basic) || softened.ends_with('҄');
                let present = softened.trim_end_matches('҄').to_string();
                lexeme.stems.present = Some(present.clone());
                if iotated {
                    lexeme.formations.present = Some(PresentFormation::IotatedE);
                }
                set_expanded_past_systems(&mut lexeme, &expanded, ImperfectFormation::A);
                set_e_present_nominals(&mut lexeme, &present, true, iotated);
            }
            PolivanovaRegularVerbClass::FourConsonant => {
                let basic = self.class_four_basic_stem.clone().ok_or_else(|| {
                    InflectionError::InvalidInput {
                        reason: "class 4c lacks its consonant stem".to_string(),
                    }
                })?;
                let velar = ends_in_velar(&basic);
                let marked = basic.ends_with('҄');
                let surface_basic = basic.trim_end_matches('҄').to_string();
                lexeme.stems.present = Some(if velar {
                    first_palatalize(&basic)
                } else {
                    surface_basic.clone()
                });
                lexeme.stems.present_first_singular = Some(surface_basic.clone());
                lexeme.stems.present_third_plural = Some(surface_basic.clone());
                if marked {
                    lexeme.formations.present = Some(PresentFormation::IotatedE);
                }
                lexeme.stems.imperfect = Some(basic.clone());
                lexeme.formations.imperfect = Some(if velar {
                    ImperfectFormation::PalatalizedA
                } else {
                    ImperfectFormation::YatA
                });
                lexeme.formations.imperfect_variant_policy =
                    Some(ImperfectVariantPolicy::UncontractedOnly);
                lexeme.stems.aorist = Some(basic.clone());
                lexeme.formations.aorist = Some(AoristFormation::New);
                set_e_present_nominals(&mut lexeme, &surface_basic, false, marked);
                lexeme.stems.imperative = Some(if velar {
                    second_palatalize(&basic)
                } else {
                    surface_basic.clone()
                });
                lexeme.formations.imperative = Some(if is_soft_consonant(&basic) {
                    ImperativeFormation::ISeries
                } else {
                    ImperativeFormation::YatSeries
                });
                lexeme.stems.l_participle = Some(adjust_before_l(&basic));
                lexeme.stems.past_active_participle = Some(basic.clone());
                lexeme.formations.past_active_participle = Some(PastActiveParticipleFormation::Ush);
                let passive_stem = if past_passive_formation == PastPassiveParticipleFormation::T {
                    self.lemma
                        .strip_suffix("ти")
                        .ok_or_else(|| InflectionError::InvalidInput {
                            reason: "a class 4c t-participle needs a -ти surface seam".to_string(),
                        })?
                        .to_string()
                } else if velar {
                    first_palatalize(&basic)
                } else {
                    basic
                };
                lexeme.stems.past_passive_participle = Some(passive_stem);
                lexeme.formations.past_passive_participle = Some(past_passive_formation);
            }
            PolivanovaRegularVerbClass::FourVowel => {
                lexeme.stems.present = Some(expanded.clone());
                lexeme.formations.present = Some(PresentFormation::IotatedE);
                set_expanded_past_systems(&mut lexeme, &expanded, ImperfectFormation::A);
                set_e_present_nominals(&mut lexeme, &expanded, true, true);
                lexeme.stems.past_passive_participle = Some(expanded);
                lexeme.formations.past_passive_participle = Some(past_passive_formation);
            }
            PolivanovaRegularVerbClass::Five => {
                let basic =
                    expanded
                        .strip_suffix('ѫ')
                        .ok_or_else(|| InflectionError::InvalidInput {
                            reason: "class 5 requires the expanded suffix -нѫ".to_string(),
                        })?;
                lexeme.stems.present = Some(basic.to_string());
                lexeme.stems.imperfect = Some(basic.to_string());
                lexeme.formations.imperfect = Some(ImperfectFormation::YatA);
                lexeme.formations.imperfect_variant_policy =
                    Some(ImperfectVariantPolicy::UncontractedOnly);
                set_vowel_aorist(&mut lexeme, &expanded);
                lexeme.stems.imperative = Some(basic.to_string());
                lexeme.formations.imperative = Some(ImperativeFormation::YatSeries);
                lexeme.stems.l_participle = Some(expanded.clone());
                set_e_present_nominals(&mut lexeme, basic, false, false);
                lexeme.stems.past_active_participle = Some(expanded.clone());
                lexeme.formations.past_active_participle =
                    Some(PastActiveParticipleFormation::Vush);
                lexeme.stems.past_passive_participle = Some(format!("{basic}ов"));
                lexeme.formations.past_passive_participle =
                    Some(PastPassiveParticipleFormation::En);
            }
            PolivanovaRegularVerbClass::Six => {
                let present = class_six_present(&expanded)?;
                lexeme.stems.present = Some(present.clone());
                lexeme.formations.present = Some(PresentFormation::IotatedE);
                set_expanded_past_systems(&mut lexeme, &expanded, ImperfectFormation::A);
                set_e_present_nominals(&mut lexeme, &present, true, true);
            }
            PolivanovaRegularVerbClass::Seven => {
                lexeme.stems.present = Some(expanded.clone());
                lexeme.formations.present = Some(PresentFormation::IotatedE);
                set_expanded_past_systems(&mut lexeme, &expanded, ImperfectFormation::A);
                set_e_present_nominals(&mut lexeme, &expanded, true, true);
            }
        }
        Ok(lexeme)
    }
}

fn parse_source_row(line: &'static str) -> RegularVerbSourceMember {
    let mut fields = line.split('\t');
    let source_row = fields
        .next()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or_default();
    let lemma = fields.next().unwrap_or_default();
    let class = PolivanovaRegularVerbClass::from_code(fields.next().unwrap_or_default());
    let class_four_basic_stem = match fields.next().unwrap_or_default() {
        "-" | "" => None,
        stem => Some(stem),
    };
    RegularVerbSourceMember {
        source_row,
        lemma,
        class,
        class_four_basic_stem,
    }
}

fn expanded_stem(lemma: &str) -> Result<String, InflectionError> {
    lemma
        .strip_suffix("ти")
        .filter(|stem| !stem.is_empty())
        .map(str::to_string)
        .ok_or_else(|| InflectionError::InvalidInput {
            reason: "a regular non-4c infinitive must expose its -ти boundary".to_string(),
        })
}

fn without_last_char(value: &str) -> Result<String, InflectionError> {
    let mut chars = value.chars();
    let Some(last) = chars.next_back() else {
        return Err(InflectionError::InvalidInput {
            reason: "a regular expanded stem cannot be empty".to_string(),
        });
    };
    if !matches!(last, 'и' | 'ѣ' | 'а' | 'ꙗ') {
        return Err(InflectionError::InvalidInput {
            reason: format!("regular thematic stem ends in unexpected {last:?}"),
        });
    }
    Ok(chars.collect())
}

fn set_vowel_aorist(lexeme: &mut VerbLexeme, expanded: &str) {
    lexeme.stems.aorist = Some(expanded.to_string());
    lexeme.stems.aorist_second_third_singular = Some(expanded.to_string());
    lexeme.formations.aorist = Some(AoristFormation::SigmaticVowel);
}

fn set_expanded_past_systems(
    lexeme: &mut VerbLexeme,
    expanded: &str,
    imperfect: ImperfectFormation,
) {
    lexeme.stems.imperfect = Some(expanded.to_string());
    lexeme.formations.imperfect = Some(imperfect);
    lexeme.formations.imperfect_variant_policy = Some(ImperfectVariantPolicy::UncontractedOnly);
    set_vowel_aorist(lexeme, expanded);
    lexeme.stems.l_participle = Some(expanded.to_string());
    lexeme.stems.past_active_participle = Some(expanded.to_string());
    lexeme.formations.past_active_participle = Some(PastActiveParticipleFormation::Vush);
    lexeme.stems.past_passive_participle = Some(expanded.to_string());
    lexeme.formations.past_passive_participle = Some(PastPassiveParticipleFormation::N);
}

fn set_i_present_nominals(lexeme: &mut VerbLexeme, basic: &str) {
    lexeme.stems.imperative = Some(basic.to_string());
    lexeme.formations.imperative = Some(ImperativeFormation::ISeries);
    lexeme.stems.present_active_participle = Some(basic.to_string());
    lexeme.formations.present_active_participle = Some(PresentActiveParticipleFormation::YeshtSoft);
    lexeme.stems.present_passive_participle = Some(basic.to_string());
    lexeme.formations.present_passive_participle = Some(PresentPassiveParticipleFormation::Im);
}

fn set_e_present_nominals(
    lexeme: &mut VerbLexeme,
    basic: &str,
    soft_imperative: bool,
    iotated: bool,
) {
    lexeme.stems.imperative = Some(basic.to_string());
    lexeme.formations.imperative = Some(if soft_imperative {
        ImperativeFormation::ISeries
    } else {
        ImperativeFormation::YatSeries
    });
    lexeme.stems.present_active_participle = Some(basic.to_string());
    lexeme.formations.present_active_participle = Some(if iotated {
        PresentActiveParticipleFormation::IotatedYushtSoft
    } else if is_soft_consonant(basic) {
        PresentActiveParticipleFormation::MixedYushtSoft
    } else {
        PresentActiveParticipleFormation::YushtHard
    });
    lexeme.stems.present_passive_participle = Some(basic.to_string());
    lexeme.formations.present_passive_participle = Some(if iotated {
        PresentPassiveParticipleFormation::IotatedEm
    } else if is_soft_consonant(basic) {
        PresentPassiveParticipleFormation::Em
    } else {
        PresentPassiveParticipleFormation::Om
    });
}

fn default_past_passive(class: PolivanovaRegularVerbClass) -> PastPassiveParticipleFormation {
    match class {
        PolivanovaRegularVerbClass::FourConsonant => PastPassiveParticipleFormation::En,
        _ => PastPassiveParticipleFormation::N,
    }
}

fn class_six_present(expanded: &str) -> Result<String, InflectionError> {
    if let Some(base) = expanded.strip_suffix("ѥва") {
        return Ok(format!("{base}ю"));
    }
    if let Some(base) = expanded.strip_suffix("ева") {
        return Ok(format!("{base}у"));
    }
    if let Some(base) = expanded.strip_suffix("ова") {
        return Ok(format!("{base}у"));
    }
    Err(InflectionError::InvalidInput {
        reason: "class 6 requires expanded -ова-, -ева-, or -ѥва-".to_string(),
    })
}

fn substitutive_softening(stem: &str) -> String {
    for (from, to) in [("ст", "щ"), ("зд", "жд"), ("ск", "щ"), ("зг", "жд")] {
        if let Some(base) = stem.strip_suffix(from) {
            return format!("{base}{to}");
        }
    }
    let Some(last) = stem.chars().last() else {
        return String::new();
    };
    let replacement = match last {
        'п' => "пл҄",
        'б' => "бл҄",
        'в' => "вл҄",
        'м' => "мл҄",
        'т' => "щ",
        'д' => "жд",
        'с' => "ш",
        'з' => "ж",
        'л' => "л҄",
        'н' => "н҄",
        'р' => "р҄",
        'к' | 'ц' => "ч",
        'г' | 'ѕ' => "ж",
        'х' => "ш",
        _ => return stem.to_string(),
    };
    let prefix = &stem[..stem.len() - last.len_utf8()];
    format!("{prefix}{replacement}")
}

fn first_palatalize(stem: &str) -> String {
    replace_final(stem, [('к', "ч"), ('г', "ж"), ('х', "ш")])
}

fn second_palatalize(stem: &str) -> String {
    replace_final(stem, [('к', "ц"), ('г', "ѕ"), ('х', "с")])
}

fn replace_final<const N: usize>(stem: &str, replacements: [(char, &str); N]) -> String {
    let Some(last) = stem.chars().last() else {
        return String::new();
    };
    let Some((_, replacement)) = replacements.iter().find(|(from, _)| *from == last) else {
        return stem.to_string();
    };
    format!("{}{replacement}", &stem[..stem.len() - last.len_utf8()])
}

fn adjust_before_l(stem: &str) -> String {
    if stem.ends_with(['т', 'д']) {
        stem[..stem.len() - 'т'.len_utf8()].to_string()
    } else {
        stem.to_string()
    }
}

fn ends_in_velar(stem: &str) -> bool {
    stem.ends_with(['к', 'г', 'х'])
}

fn is_soft_consonant(stem: &str) -> bool {
    stem.ends_with(['ч', 'ж', 'ш', 'щ', '҄']) || stem.ends_with("жд")
}

fn ends_in_vowel(stem: &str) -> bool {
    stem.chars().last().is_some_and(|character| {
        matches!(
            character,
            'а' | 'е' | 'и' | 'о' | 'у' | 'ы' | 'ѣ' | 'ю' | 'ꙗ' | 'ѧ' | 'ѫ' | 'ѭ'
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AdjectiveCell, AdjectiveForm, Animacy, Case, FiniteTense, FiniteVerbCell, Gender,
        ImperativeCell, LParticipleCell, Number, ParticipleCell, ParticipleKind, Person,
    };

    #[test]
    fn embedded_inventory_is_exact_and_row_addressable() {
        let members = RegularVerbSourceMember::all().collect::<Vec<_>>();
        assert_eq!(members.len(), RegularVerbSourceMember::COUNT);
        assert!(members.iter().all(|member| member.source_row != 0));
        assert!(
            members
                .windows(2)
                .all(|pair| pair[0].source_row < pair[1].source_row)
        );
        assert_eq!(
            members
                .iter()
                .map(|member| member.source_row)
                .collect::<BTreeSet<_>>()
                .len(),
            RegularVerbSourceMember::COUNT,
            "each embedded row must retain a unique OSD identity"
        );
        assert!(members.iter().all(|member| {
            (member.class == PolivanovaRegularVerbClass::FourConsonant)
                == member.class_four_basic_stem.is_some()
                && member.specification().is_ok()
        }));
        assert_eq!(
            PolivanovaRegularVerbClass::ALL.map(|class| {
                members
                    .iter()
                    .filter(|member| member.class == class)
                    .count()
            }),
            [826, 137, 164, 122, 23, 132, 136, 757]
        );
        assert_eq!(
            RegularVerbSourceMember::from_source_row(381)
                .map(RegularVerbSourceMember::class_four_basic_stem),
            Some(Some("вед"))
        );
        assert_eq!(RegularVerbFamily::all().count(), 2_283);
        assert!(
            RegularVerbFamily::all()
                .collect::<Vec<_>>()
                .windows(2)
                .all(|pair| pair[0].lemma < pair[1].lemma)
        );
    }

    #[test]
    fn class_representatives_match_table_429() {
        let cases = [
            ("л҄юбити", "л҄юблѭ", "л҄юбиши", "л҄юблꙗахъ", "л҄юбихъ", "л҄юбите"),
            (
                "трьпѣти",
                "трьплѭ",
                "трьпиши",
                "трьпѣахъ",
                "трьпѣхъ",
                "трьпите",
            ),
            (
                "плакати",
                "плачѫ",
                "плачеши",
                "плакаахъ",
                "плакахъ",
                "плачите",
            ),
            ("нести", "несѫ", "несеши", "несѣахъ", "несохъ", "несѣте"),
            (
                "двигнѫти",
                "двигнѫ",
                "двигнеши",
                "двигнѣахъ",
                "двигнѫхъ",
                "двигнѣте",
            ),
            (
                "миловати",
                "милуѭ",
                "милуѥши",
                "миловаахъ",
                "миловахъ",
                "милуите",
            ),
            ("дѣлати", "дѣлаѭ", "дѣлаѥши", "дѣлаахъ", "дѣлахъ", "дѣлаите"),
        ];
        for (lemma, first, second, imperfect, aorist, imperative) in cases {
            let lexeme = RegularVerbFamily::classify_source_lemma(lemma)
                .and_then(|family| family.members().next())
                .expect("source representative")
                .lexemes()
                .expect("valid profile")
                .remove(0);
            let finite = |tense, person, number| {
                crate::verb::finite(
                    &lexeme,
                    FiniteVerbCell {
                        tense,
                        person,
                        number,
                    },
                )
                .expect("finite form")
                .text
            };
            assert_eq!(
                finite(FiniteTense::Present, Person::First, Number::Singular),
                first
            );
            assert_eq!(
                finite(FiniteTense::Present, Person::Second, Number::Singular),
                second
            );
            assert_eq!(
                finite(FiniteTense::Imperfect, Person::First, Number::Singular),
                imperfect
            );
            assert_eq!(
                finite(FiniteTense::Aorist, Person::First, Number::Singular),
                aorist
            );
            assert_eq!(
                crate::verb::imperative(
                    &lexeme,
                    ImperativeCell {
                        person: Person::Second,
                        number: Number::Plural,
                    },
                )
                .expect("imperative")
                .text,
                imperative
            );
        }
    }

    #[test]
    fn every_source_analysis_generates_every_licensed_cell() {
        for member in RegularVerbSourceMember::all() {
            for lexeme in member.lexemes().expect("valid generated source profile") {
                for tense in FiniteTense::ALL {
                    for number in Number::ALL {
                        for person in Person::ALL {
                            crate::verb::finite(
                                &lexeme,
                                FiniteVerbCell {
                                    tense,
                                    person,
                                    number,
                                },
                            )
                            .unwrap_or_else(|error| {
                                panic!("{} {:?}: {error:?}", member.lemma, tense)
                            });
                        }
                    }
                }
                for cell in ImperativeCell::SUPPORTED {
                    crate::verb::imperative(&lexeme, cell)
                        .unwrap_or_else(|error| panic!("{} imperative: {error:?}", member.lemma));
                }
                crate::verb::infinitive(&lexeme).expect("infinitive");
                crate::verb::supine(&lexeme).expect("supine");
                for cell in LParticipleCell::all() {
                    crate::verb::l_participle(&lexeme, cell).expect("l-participle");
                }
                for kind in ParticipleKind::ALL {
                    for number in Number::ALL {
                        for case in Case::ALL {
                            for gender in Gender::ALL {
                                crate::verb::participle(
                                    &lexeme,
                                    ParticipleCell {
                                        kind,
                                        adjective: AdjectiveCell {
                                            case,
                                            number,
                                            gender,
                                            animacy: Animacy::Inanimate,
                                            form: AdjectiveForm::Short,
                                        },
                                    },
                                )
                                .unwrap_or_else(|error| {
                                    panic!("{} {kind:?}: {error:?}", member.lemma)
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn homographs_and_lexical_t_participles_are_preserved() {
        let vesti = RegularVerbFamily::classify_source_lemma("вести").expect("вести family");
        assert_eq!(vesti.members().count(), 2);
        let presents = vesti
            .members()
            .map(|member| {
                let lexeme = member.lexemes().expect("profile").remove(0);
                crate::verb::finite(
                    &lexeme,
                    FiniteVerbCell {
                        tense: FiniteTense::Present,
                        person: Person::First,
                        number: Number::Singular,
                    },
                )
                .expect("present")
                .text
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(
            presents,
            BTreeSet::from(["ведѫ".to_string(), "везѫ".to_string()])
        );

        assert_eq!(
            RegularVerbFamily::classify_source_lemma("сѣти")
                .expect("сѣти")
                .members()
                .next()
                .expect("member")
                .lexemes()
                .expect("profiles")
                .len(),
            2
        );
        let uvęsti = RegularVerbFamily::classify_source_lemma("увѧсти")
            .expect("увѧсти")
            .members()
            .next()
            .expect("member")
            .lexemes()
            .expect("profile");
        assert_eq!(
            uvęsti[0].formations.past_passive_participle,
            Some(PastPassiveParticipleFormation::T)
        );
    }
}
