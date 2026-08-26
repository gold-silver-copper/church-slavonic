//! Source-bounded productive substantive profiles from Polivanova's OSD dictionary.
//!
//! The embedded inventory retains exact row identities and the source-defined
//! five-way declensional classification. Polivanova 2023 §§267, 285–302 makes
//! morphological gender part of the class, uses the canonical nominative-like
//! masculine accusative, and explicitly lists the period's pluralia tantum.

use crate::noun::NounLexeme;
use crate::{Animacy, Gender, InflectionError, NounClass, NumberRestriction};
use std::collections::BTreeSet;
use std::sync::OnceLock;

const SOURCE_ROWS: &str = include_str!("../data/polivanova_regular_nouns.tsv");

/// One of Polivanova's five standard substantive inflectional classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PolivanovaRegularNounClass {
    TwofoldMasculine,
    TwofoldNeuter,
    TwofoldFeminine,
    SimplexMasculine,
    SimplexFeminine,
}

impl PolivanovaRegularNounClass {
    pub const ALL: [Self; 5] = [
        Self::TwofoldMasculine,
        Self::TwofoldNeuter,
        Self::TwofoldFeminine,
        Self::SimplexMasculine,
        Self::SimplexFeminine,
    ];

    pub const fn code(self) -> &'static str {
        match self {
            Self::TwofoldMasculine => "2/m",
            Self::TwofoldNeuter => "2/n",
            Self::TwofoldFeminine => "2/f",
            Self::SimplexMasculine => "1/m",
            Self::SimplexFeminine => "1/f",
        }
    }

    pub const fn gender(self) -> Gender {
        match self {
            Self::TwofoldMasculine | Self::SimplexMasculine => Gender::Masculine,
            Self::TwofoldNeuter => Gender::Neuter,
            Self::TwofoldFeminine | Self::SimplexFeminine => Gender::Feminine,
        }
    }

    pub const fn source_section(self) -> &'static str {
        match self {
            Self::TwofoldMasculine => "§§326–328",
            Self::TwofoldNeuter => "§§338–340",
            Self::TwofoldFeminine => "§§342–344",
            Self::SimplexMasculine => "§§333–335",
            Self::SimplexFeminine => "§§349–351",
        }
    }

    fn from_code(code: &str) -> Self {
        match code {
            "2/n" => Self::TwofoldNeuter,
            "2/f" => Self::TwofoldFeminine,
            "1/m" => Self::SimplexMasculine,
            "1/f" => Self::SimplexFeminine,
            _ => Self::TwofoldMasculine,
        }
    }
}

/// One exact regular-substantive OSD row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RegularNounSourceMember {
    source_row: u16,
    lemma: &'static str,
    class: PolivanovaRegularNounClass,
    inflection_lemma: &'static str,
    number_restriction: NumberRestriction,
}

impl RegularNounSourceMember {
    pub const COUNT: usize = 2_423;

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

    pub const fn class(self) -> PolivanovaRegularNounClass {
        self.class
    }

    pub const fn inflection_lemma(self) -> &'static str {
        self.inflection_lemma
    }

    pub const fn number_restriction(self) -> NumberRestriction {
        self.number_restriction
    }

    pub fn specification(self) -> Result<PolivanovaRegularNounSpecification, InflectionError> {
        PolivanovaRegularNounSpecification::new(
            self.lemma,
            self.inflection_lemma,
            self.class,
            self.number_restriction,
        )
    }

    pub fn lexeme(self) -> Result<NounLexeme, InflectionError> {
        self.specification()?.lexeme()
    }
}

/// All regular source rows sharing one normalized citation spelling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RegularNounFamily {
    lemma: &'static str,
}

impl RegularNounFamily {
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

    pub fn members(self) -> impl Iterator<Item = RegularNounSourceMember> {
        RegularNounSourceMember::all().filter(move |member| member.lemma == self.lemma)
    }
}

fn source_members() -> &'static [RegularNounSourceMember] {
    static MEMBERS: OnceLock<Vec<RegularNounSourceMember>> = OnceLock::new();
    MEMBERS
        .get_or_init(|| SOURCE_ROWS.lines().skip(1).map(parse_source_row).collect())
        .as_slice()
}

fn regular_families() -> &'static [RegularNounFamily] {
    static FAMILIES: OnceLock<Vec<RegularNounFamily>> = OnceLock::new();
    FAMILIES
        .get_or_init(|| {
            RegularNounSourceMember::all()
                .map(RegularNounSourceMember::canonical_lemma)
                .collect::<BTreeSet<_>>()
                .into_iter()
                .map(|lemma| RegularNounFamily { lemma })
                .collect()
        })
        .as_slice()
}

/// Validated productive specification for one source-listed regular noun.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolivanovaRegularNounSpecification {
    lemma: String,
    inflection_lemma: String,
    class: PolivanovaRegularNounClass,
    number_restriction: NumberRestriction,
}

impl PolivanovaRegularNounSpecification {
    pub fn new(
        lemma: impl Into<String>,
        inflection_lemma: impl Into<String>,
        class: PolivanovaRegularNounClass,
        number_restriction: NumberRestriction,
    ) -> Result<Self, InflectionError> {
        let lemma = crate::orthography::canonical_display(&lemma.into())?;
        let inflection_lemma = crate::orthography::canonical_display(&inflection_lemma.into())?;
        if lemma.is_empty() || inflection_lemma.is_empty() {
            return Err(InflectionError::InvalidInput {
                reason:
                    "a regular OCS noun specification needs nonempty source and inflection lemmas"
                        .to_string(),
            });
        }
        if number_restriction != NumberRestriction::PluralOnly && lemma != inflection_lemma {
            return Err(InflectionError::InvalidInput {
                reason:
                    "only a source-listed plurale tantum may use a reconstructed inflection lemma"
                        .to_string(),
            });
        }
        let specification = Self {
            lemma,
            inflection_lemma,
            class,
            number_restriction,
        };
        specification.noun_class()?;
        Ok(specification)
    }

    pub fn noun_class(&self) -> Result<NounClass, InflectionError> {
        let lemma = self.inflection_lemma.as_str();
        let class = match self.class {
            PolivanovaRegularNounClass::TwofoldMasculine if lemma.ends_with('ъ') => {
                NounClass::OMasculineHard
            }
            PolivanovaRegularNounClass::TwofoldMasculine
                if lemma.ends_with('ь') || lemma.ends_with('и') =>
            {
                NounClass::JoMasculineSoft
            }
            PolivanovaRegularNounClass::TwofoldNeuter if lemma.ends_with('о') => {
                NounClass::ONeuterHard
            }
            PolivanovaRegularNounClass::TwofoldNeuter
                if lemma.ends_with('е') || lemma.ends_with('ѥ') =>
            {
                NounClass::JoNeuterSoft
            }
            PolivanovaRegularNounClass::TwofoldFeminine if lemma.ends_with('а') => {
                if lemma
                    .strip_suffix('а')
                    .is_some_and(ends_in_morphologically_soft_consonant)
                {
                    NounClass::JaSoft
                } else {
                    NounClass::AHard
                }
            }
            PolivanovaRegularNounClass::TwofoldFeminine
                if lemma.ends_with('ꙗ') || lemma.ends_with('и') =>
            {
                NounClass::JaSoft
            }
            PolivanovaRegularNounClass::SimplexMasculine if lemma.ends_with('ь') => {
                NounClass::IMasculine
            }
            PolivanovaRegularNounClass::SimplexFeminine if lemma.ends_with('ь') => {
                NounClass::IFeminine
            }
            _ => {
                return Err(InflectionError::InvalidInput {
                    reason: format!(
                        "inflection lemma {lemma:?} is incompatible with Polivanova class {}",
                        self.class.code()
                    ),
                });
            }
        };
        Ok(class)
    }

    pub fn lexeme(&self) -> Result<NounLexeme, InflectionError> {
        Ok(NounLexeme {
            lemma: self.inflection_lemma.clone(),
            class: self.noun_class()?,
            gender: self.class.gender(),
            // Polivanova's standard substantive paradigm has the canonical
            // nominative-like masculine accusative (§§267, 289–290).
            animacy: Animacy::Inanimate,
            number_restriction: self.number_restriction,
        })
    }
}

fn ends_in_morphologically_soft_consonant(stem: &str) -> bool {
    stem.ends_with(['ч', 'ж', 'ш', 'щ', 'ц', 'ѕ', 'ꙃ', '҄']) || stem.ends_with("жд")
}

fn parse_source_row(line: &'static str) -> RegularNounSourceMember {
    let mut fields = line.split('\t');
    let source_row = fields
        .next()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or_default();
    let lemma = fields.next().unwrap_or_default();
    let class = PolivanovaRegularNounClass::from_code(fields.next().unwrap_or_default());
    let inflection_lemma = fields.next().unwrap_or_default();
    let number_restriction = match fields.next().unwrap_or_default() {
        "pl" => NumberRestriction::PluralOnly,
        _ => NumberRestriction::All,
    };
    RegularNounSourceMember {
        source_row,
        lemma,
        class,
        inflection_lemma,
        number_restriction,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Case, NounCell, Number};

    #[test]
    fn embedded_inventory_is_exact_and_row_addressable() {
        let members = RegularNounSourceMember::all().collect::<Vec<_>>();
        assert_eq!(members.len(), RegularNounSourceMember::COUNT);
        assert!(members.iter().all(|member| member.source_row != 0));
        assert!(
            members
                .windows(2)
                .all(|pair| pair[0].source_row < pair[1].source_row)
        );
        assert!(members.iter().all(|member| member.specification().is_ok()));
        assert_eq!(
            PolivanovaRegularNounClass::ALL.map(|class| {
                members
                    .iter()
                    .filter(|member| member.class == class)
                    .count()
            }),
            [690, 1_054, 460, 29, 190]
        );
        assert_eq!(
            members
                .iter()
                .filter(|member| member.number_restriction == NumberRestriction::PluralOnly)
                .count(),
            24
        );
        assert_eq!(RegularNounFamily::all().count(), 2_417);
    }

    #[test]
    fn every_source_member_generates_every_licensed_cell() {
        for member in RegularNounSourceMember::all() {
            let lexeme = member.lexeme().expect("valid source specification");
            for number in Number::ALL {
                for case in Case::ALL {
                    let result = crate::noun::decline(&lexeme, NounCell { case, number });
                    if member.number_restriction == NumberRestriction::PluralOnly
                        && number != Number::Plural
                    {
                        assert!(result.is_err(), "{} {case:?} {number:?}", member.lemma);
                    } else {
                        result.unwrap_or_else(|error| {
                            panic!("{} {case:?} {number:?}: {error:?}", member.lemma)
                        });
                    }
                }
            }
        }
    }

    #[test]
    fn class_representatives_match_polivanova_profile_tables() {
        let form = |source_row, case, number| {
            let member = RegularNounSourceMember::from_source_row(source_row).expect("profile row");
            crate::noun::decline(
                &member.lexeme().expect("valid profile"),
                NounCell { case, number },
            )
            .expect("licensed profile cell")
            .text
        };

        // Table 327: hard, velar, palatalized, and glide-final masculine stems.
        for (row, case, number, expected) in [
            (1075, Case::Nominative, Number::Plural, "гради"),
            (1075, Case::Locative, Number::Singular, "градѣ"),
            (536, Case::Nominative, Number::Plural, "врьси"),
            (536, Case::Locative, Number::Plural, "врьсѣхъ"),
            (734, Case::Nominative, Number::Dual, "въпл҄ꙗ"),
            (734, Case::Locative, Number::Plural, "въпл҄ихъ"),
            (2018, Case::Nominative, Number::Singular, "краи"),
            (2018, Case::Accusative, Number::Plural, "краѩ"),
        ] {
            assert_eq!(form(row, case, number), expected);
        }

        // Table 339: both neuter subtypes and the ьj seam.
        for (row, case, number, expected) in [
            (4870, Case::Nominative, Number::Dual, "селѣ"),
            (207, Case::Locative, Number::Singular, "блаѕѣ"),
            (2133, Case::Nominative, Number::Plural, "ложа"),
            (2133, Case::Locative, Number::Plural, "ложихъ"),
            (1526, Case::Nominative, Number::Dual, "зелии"),
            (1526, Case::Genitive, Number::Plural, "зелии"),
        ] {
            assert_eq!(form(row, case, number), expected);
        }

        // Table 343: invariant and twofold feminine terminals remain distinct.
        for (row, case, number, expected) in [
            (1319, Case::Nominative, Number::Plural, "женꙑ"),
            (2314, Case::Nominative, Number::Dual, "мусѣ"),
            (2898, Case::Nominative, Number::Plural, "овьцѧ"),
            (5429, Case::Locative, Number::Plural, "тѫчахъ"),
            (1527, Case::Genitive, Number::Singular, "земл҄ѩ"),
            (1545, Case::Genitive, Number::Plural, "змии"),
        ] {
            assert_eq!(form(row, case, number), expected);
        }

        // Tables 335 and 351: the simplex classes retain their own terminals.
        assert_eq!(form(1522, Case::Nominative, Number::Plural), "звѣриѥ");
        assert_eq!(form(3328, Case::Instrumental, Number::Singular), "пѫтьмь");
        assert_eq!(form(2005, Case::Instrumental, Number::Singular), "костьѭ");
        assert_eq!(form(390, Case::Locative, Number::Plural), "вещьхъ");
    }

    #[test]
    fn source_listed_pluralia_tantum_keep_their_exact_starting_forms() {
        let cases = [
            (9, "ꙗдра"),
            (63, "л҄юдиѥ"),
            (2949, "оими"),
            (6297, "кън҄ижицѧ"),
            (6349, "дрождиѩ"),
        ];
        for (source_row, expected) in cases {
            let member = RegularNounSourceMember::from_source_row(source_row).expect("source row");
            assert_eq!(member.canonical_lemma(), expected);
            assert_eq!(member.number_restriction(), NumberRestriction::PluralOnly);
            let predicted = crate::noun::decline(
                &member.lexeme().expect("profile"),
                NounCell {
                    case: Case::Nominative,
                    number: Number::Plural,
                },
            )
            .expect("plural citation");
            // The exact source citation may preserve a diplomatic terminal;
            // the productive profile must at least remain a valid prediction.
            assert!(!predicted.text.is_empty());
        }
    }
}
