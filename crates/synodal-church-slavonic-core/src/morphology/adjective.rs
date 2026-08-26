use crate::{
    AdjectiveCell, AdjectiveForm, Animacy, Case, Comparison, Error, FormSet, Gender, MetadataField,
    Number, OrthographyProfile, Result, SynodalWord,
};

use super::*;
use church_slavonic_core::{Recension, adjective as kernel};

const SYN: Recension = Recension::SynodalRussian;

/// Since the phase-4 adjective merge the shared hard/soft short and long
/// ending tables live in the merged kernel `church_slavonic_core::adjective`,
/// queried with `Recension::SynodalRussian`; the velar, sibilant, possessive,
/// and comparison series remain Synodal-only family paradigms. The kernel's
/// totality test guarantees every Synodal cell is populated with exactly one
/// ending.
fn kernel_ending(
    class: kernel::AdjectiveClass,
    form: AdjectiveForm,
    cell: AdjectiveCell,
) -> &'static str {
    let endings = match form {
        AdjectiveForm::Short => kernel::short_ending(
            class,
            cell.case,
            cell.number,
            cell.gender,
            cell.animacy,
            SYN,
        ),
        AdjectiveForm::Long => kernel::long_ending(
            class,
            cell.case,
            cell.number,
            cell.gender,
            cell.animacy,
            SYN,
        ),
    };
    endings.first().copied().unwrap_or_default()
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum AdjectiveClass {
    Hard,
    Soft,
    /// A hard stem ending in a sibilant (`ш`, `щ`, `ж`, `ч`), which the
    /// Synodal orthography declines with the mixed series Alypy §58 prints
    /// for `-ш-` comparatives and §§95–98 for participles: hard `-агѡ`/`-ꙋю`
    /// beside soft `-емꙋ`/`-ихъ`/`-ими`, with the plural dative `-ымъ` and
    /// the plural feminine `-ыѧ` keeping `ы` to stay distinct from the
    /// singular instrumental `-имъ` and genitive `-їѧ`. Derived from the
    /// stem at declension time; never stored as reviewed metadata.
    HardSibilant,
    /// Hard adjective with a final velar stem. Its complete positive
    /// paradigm has the ending spellings and palatalizations printed
    /// separately in Alypy §57 (`благїй : блазїи`).
    Velar,
    /// Short possessives in `-ов-` / `-ев-` (Alypy §§50 and 53). Any
    /// independently attested compound-form cell remains a lexical exact
    /// override rather than licensing a long paradigm for every member.
    PossessiveHard,
    /// Short possessives in palatal `-ь` / `-ень` (Alypy §§50 and 53).
    /// A mobile vowel is supplied through the ordinary typed short-masculine
    /// principal part rather than inferred from spelling.
    PossessiveSoft,
    /// Possessives formed with the historical `-jь` suffix. These retain the
    /// soft-sign citation edge but otherwise use a mixed short paradigm:
    /// hard vowel endings alongside soft `-имъ`, `-ихъ`, and `-ими`
    /// (`человѣчь : человѣча : человѣчимъ`).
    PossessiveJ,
    /// Possessive and relational adjectives in `-ин-`. Alypy §51 explicitly
    /// licenses both the short and the long paradigm (`голубинъ : голубиный`).
    PossessiveIn,
    /// Possessive and relational adjectives in `-ск-`. Alypy §§11, 51, and
    /// 57 license both forms and require the cell-scoped `-ск- : -ст-`
    /// alternation before the soft endings (`человѣческїй : человѣчестѣмъ`).
    PossessiveSk,
    /// Possessives in `-їй`, whose `-ї-` belongs to the derivational suffix
    /// and whose complete predominantly short declension is printed
    /// separately in Alypy §56. That section also licenses occasional
    /// compound forms.
    PossessiveIi,
}

impl AdjectiveClass {
    pub const ALL: [Self; 9] = [
        Self::Hard,
        Self::Soft,
        Self::Velar,
        Self::PossessiveHard,
        Self::PossessiveSoft,
        Self::PossessiveJ,
        Self::PossessiveIn,
        Self::PossessiveSk,
        Self::PossessiveIi,
    ];
}

/// Source-defined relation between the ordinary positive stem and the stem
/// used before the short masculine citation ending.
///
/// The alternant remains an independently supplied principal part. The enum
/// validates why it differs, preventing a caller from smuggling an arbitrary
/// irregular stem through either productive rule.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum ShortMasculineStemFormation {
    /// Alypy §52: a full/general `-нн-` stem loses one `н` at the short
    /// masculine citation edge (`блаженн- : блаженъ`).
    DoubleNReduction,
    /// A mobile `е` appears before the final stem consonant at the short
    /// masculine citation edge (`преподобн- : преподобенъ`).
    MobileEInsertion,
    /// A mobile `о` appears before the final stem consonant at the short
    /// masculine citation edge (`ѕл- : ѕолъ`).
    MobileOInsertion,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct AdjectiveLexeme {
    pub lemma: SynodalWord,
    pub stem: SynodalWord,
    pub class: AdjectiveClass,
    /// Independently supplied positive stem used before the short masculine
    /// citation ending. It must be paired with `short_masculine_formation`.
    pub short_masculine_stem: Option<SynodalWord>,
    pub short_masculine_formation: Option<ShortMasculineStemFormation>,
    /// Fully specified stem before comparison endings (for example `мꙋдрѣйш`).
    pub comparative_stem: Option<SynodalWord>,
    /// Formation of the independently supplied comparison stem. This is
    /// required only for short comparison, whose masculine and neuter
    /// nominative edges delete part of the comparison suffix (Alypy §58).
    pub comparison_formation: Option<ComparisonFormation>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum ComparisonFormation {
    AncientHard,
    AncientSoft,
    LaterYat,
    LaterAi,
}

pub(crate) fn positive_adjective_surface(
    lexeme: &AdjectiveLexeme,
    cell: AdjectiveCell,
) -> Result<String> {
    match lexeme.class {
        AdjectiveClass::Velar => return velar_positive_adjective_surface(lexeme, cell),
        AdjectiveClass::PossessiveSk => return sk_positive_adjective_surface(lexeme, cell),
        AdjectiveClass::PossessiveJ => {
            return possessive_j_positive_adjective_surface(lexeme, cell);
        }
        AdjectiveClass::Hard
        | AdjectiveClass::HardSibilant
        | AdjectiveClass::Soft
        | AdjectiveClass::PossessiveHard
        | AdjectiveClass::PossessiveSoft
        | AdjectiveClass::PossessiveIn
        | AdjectiveClass::PossessiveIi => {}
    }
    let ending = match cell.form {
        AdjectiveForm::Short => short_adjective_ending(lexeme.class, cell)?,
        AdjectiveForm::Long => long_adjective_ending(lexeme.class, cell)?,
    };
    let stem = if cell.form == AdjectiveForm::Short
        && cell.number == Number::Singular
        && cell.gender == Gender::Masculine
        && matches!(ending, "ъ" | "ь")
    {
        lexeme.short_masculine_stem.as_ref().unwrap_or(&lexeme.stem)
    } else {
        &lexeme.stem
    };
    Ok(join(stem.canonical(), ending))
}

pub(crate) fn possessive_j_positive_adjective_surface(
    lexeme: &AdjectiveLexeme,
    cell: AdjectiveCell,
) -> Result<String> {
    if cell.form == AdjectiveForm::Long {
        return Err(Error::HistoricallyInvalidCell {
            reason: "the possessive -jь suffix licenses only the short paradigm; exceptional compound forms require exact lexical evidence".into(),
        });
    }
    let hard = short_adjective_ending(AdjectiveClass::Hard, cell)?;
    let ending = match hard {
        "ъ" if cell.number == Number::Singular && cell.gender == Gender::Masculine => "ь",
        "ымъ" => "имъ",
        "ыма" => "има",
        "ыхъ" => "ихъ",
        "ыми" => "ими",
        ending => ending,
    };
    Ok(join(lexeme.stem.canonical(), ending))
}

pub(crate) fn sk_positive_adjective_surface(
    lexeme: &AdjectiveLexeme,
    cell: AdjectiveCell,
) -> Result<String> {
    let mut stem = lexeme.stem.canonical().to_owned();
    let ending = match cell.form {
        AdjectiveForm::Short => {
            let hard = short_adjective_ending(AdjectiveClass::Hard, cell)?;
            if hard == "ѣ"
                || hard == "е"
                || matches!(
                    (cell.number, cell.gender, cell.case),
                    (
                        Number::Plural,
                        Gender::Masculine,
                        Case::Nominative | Case::Vocative
                    )
                )
            {
                stem = replace_final_sk_with_st(&stem)?;
            }
            match hard {
                "ы" => "и",
                "ымъ" => "имъ",
                "ыма" => "има",
                "ыхъ" => "ихъ",
                "ыми" => "ими",
                other => other,
            }
        }
        AdjectiveForm::Long => {
            if matches!(
                (cell.number, cell.gender, cell.case),
                (
                    Number::Singular,
                    Gender::Feminine,
                    Case::Dative | Case::Locative
                ) | (
                    Number::Singular,
                    Gender::Masculine | Gender::Neuter,
                    Case::Locative
                ) | (
                    Number::Dual,
                    Gender::Feminine | Gender::Neuter,
                    Case::Nominative | Case::Accusative | Case::Vocative
                ) | (
                    Number::Plural,
                    Gender::Masculine,
                    Case::Nominative | Case::Vocative
                )
            ) {
                stem = replace_final_sk_with_st(&stem)?;
            }
            velar_long_adjective_ending(cell)?
        }
    };
    Ok(join(&stem, ending))
}

pub(crate) fn replace_final_sk_with_st(stem: &str) -> Result<String> {
    let Some(base) = stem.strip_suffix("ск") else {
        return Err(Error::ContradictoryMetadata {
            reason: "a possessive -ск- adjective requires a stem ending in -ск".into(),
        });
    };
    Ok(join(base, "ст"))
}

pub(crate) fn velar_positive_adjective_surface(
    lexeme: &AdjectiveLexeme,
    cell: AdjectiveCell,
) -> Result<String> {
    let mut stem = lexeme.stem.canonical().to_owned();
    let ending = match cell.form {
        AdjectiveForm::Short => {
            let hard = short_adjective_ending(AdjectiveClass::Hard, cell)?;
            if hard == "ѣ"
                || matches!(
                    (cell.number, cell.gender, cell.case),
                    (
                        Number::Plural,
                        Gender::Masculine,
                        Case::Nominative | Case::Vocative
                    )
                )
            {
                stem = second_palatalize_final_velar(&stem);
            } else if hard == "е" {
                stem = palatalize_final_velar(&stem);
            }
            match hard {
                "ы" => "и",
                "ымъ" => "имъ",
                "ыма" => "има",
                "ыхъ" => "ихъ",
                "ыми" => "ими",
                other => other,
            }
        }
        AdjectiveForm::Long => {
            if matches!(
                (cell.number, cell.gender, cell.case),
                (
                    Number::Singular,
                    Gender::Feminine,
                    Case::Dative | Case::Locative
                ) | (
                    Number::Singular,
                    Gender::Masculine | Gender::Neuter,
                    Case::Locative
                ) | (
                    Number::Dual,
                    Gender::Feminine | Gender::Neuter,
                    Case::Nominative | Case::Accusative | Case::Vocative
                ) | (
                    Number::Plural,
                    Gender::Masculine,
                    Case::Nominative | Case::Vocative
                )
            ) {
                stem = second_palatalize_final_velar(&stem);
            }
            velar_long_adjective_ending(cell)?
        }
    };
    Ok(join(&stem, ending))
}

pub fn decline_adjective(
    lexeme: &AdjectiveLexeme,
    cell: AdjectiveCell,
    profile: OrthographyProfile,
) -> Result<FormSet> {
    validate_adjective_lexeme(lexeme)?;
    if matches!(
        lexeme.class,
        AdjectiveClass::PossessiveHard
            | AdjectiveClass::PossessiveSoft
            | AdjectiveClass::PossessiveJ
            | AdjectiveClass::PossessiveIn
            | AdjectiveClass::PossessiveSk
            | AdjectiveClass::PossessiveIi
    ) && cell.comparison != Comparison::Positive
    {
        return Err(Error::HistoricallyInvalidCell {
            reason: "possessive adjectives do not license comparison".into(),
        });
    }
    if matches!(
        lexeme.class,
        AdjectiveClass::PossessiveHard
            | AdjectiveClass::PossessiveSoft
            | AdjectiveClass::PossessiveJ
    ) && cell.form == AdjectiveForm::Long
    {
        return Err(Error::HistoricallyInvalidCell {
            reason: "this possessive suffix licenses only the short paradigm; exceptional compound forms require exact lexical evidence".into(),
        });
    }
    if cell.form == AdjectiveForm::Short && cell.comparison == Comparison::Comparative {
        let stem = lexeme
            .comparative_stem
            .as_ref()
            .ok_or(Error::MissingPrincipalPart {
                field: MetadataField::ComparisonStem,
            })?;
        let formation = lexeme.comparison_formation.ok_or(Error::MissingMetadata {
            field: MetadataField::ComparisonFormation,
        })?;
        return decline_short_comparison(lexeme, stem, formation, cell, profile);
    }
    if cell.form == AdjectiveForm::Short && cell.comparison == Comparison::Superlative {
        let stem = lexeme
            .comparative_stem
            .as_ref()
            .ok_or(Error::MissingPrincipalPart {
                field: MetadataField::ComparisonStem,
            })?;
        let formation = lexeme.comparison_formation.ok_or(Error::MissingMetadata {
            field: MetadataField::ComparisonFormation,
        })?;
        return decline_short_superlative_predicate(lexeme, stem, formation, cell, profile);
    }
    let (mut expanded, rule) = match cell.comparison {
        Comparison::Positive => (
            vec![positive_adjective_surface(lexeme, cell)?],
            match (lexeme.class, cell.form) {
                (AdjectiveClass::Hard | AdjectiveClass::HardSibilant, AdjectiveForm::Short) => {
                    "SYN-ADJ-SHORT-HARD-ALYPY-53"
                }
                (AdjectiveClass::HardSibilant, AdjectiveForm::Long) => {
                    "SYN-ADJ-LONG-SIBILANT-ALYPY-57-58"
                }
                (AdjectiveClass::Soft, AdjectiveForm::Short) => "SYN-ADJ-SHORT-SOFT-ALYPY-53",
                (AdjectiveClass::Velar, AdjectiveForm::Short) => "SYN-ADJ-SHORT-VELAR-ALYPY-53-57",
                (AdjectiveClass::Hard, AdjectiveForm::Long) => "SYN-ADJ-LONG-HARD-ALYPY-57",
                (AdjectiveClass::Soft, AdjectiveForm::Long) => "SYN-ADJ-LONG-SOFT-ALYPY-57",
                (AdjectiveClass::Velar, AdjectiveForm::Long) => "SYN-ADJ-LONG-VELAR-ALYPY-57",
                (AdjectiveClass::PossessiveHard, AdjectiveForm::Short) => {
                    "SYN-ADJ-POSSESSIVE-OV-EV-SHORT-ALYPY-50-53"
                }
                (AdjectiveClass::PossessiveSoft, AdjectiveForm::Short) => {
                    "SYN-ADJ-POSSESSIVE-SOFT-SHORT-ALYPY-50-53"
                }
                (AdjectiveClass::PossessiveJ, AdjectiveForm::Short) => {
                    "SYN-ADJ-POSSESSIVE-J-SHORT-ALYPY-50-53"
                }
                (AdjectiveClass::PossessiveIn, AdjectiveForm::Short) => {
                    "SYN-ADJ-POSSESSIVE-IN-SHORT-ALYPY-50-53"
                }
                (AdjectiveClass::PossessiveIn, AdjectiveForm::Long) => {
                    "SYN-ADJ-POSSESSIVE-IN-LONG-ALYPY-50-57"
                }
                (AdjectiveClass::PossessiveSk, AdjectiveForm::Short) => {
                    "SYN-ADJ-POSSESSIVE-SK-SHORT-ALYPY-11-50-53"
                }
                (AdjectiveClass::PossessiveSk, AdjectiveForm::Long) => {
                    "SYN-ADJ-POSSESSIVE-SK-LONG-ALYPY-11-50-57"
                }
                (AdjectiveClass::PossessiveIi, AdjectiveForm::Short) => {
                    "SYN-ADJ-POSSESSIVE-II-SHORT-ALYPY-56"
                }
                (AdjectiveClass::PossessiveIi, AdjectiveForm::Long) => {
                    "SYN-ADJ-POSSESSIVE-II-LONG-ALYPY-56"
                }
                (
                    AdjectiveClass::PossessiveHard
                    | AdjectiveClass::PossessiveSoft
                    | AdjectiveClass::PossessiveJ,
                    AdjectiveForm::Long,
                ) => unreachable!("long possessive cells are rejected above"),
            },
        ),
        Comparison::Comparative | Comparison::Superlative => {
            let stem = lexeme
                .comparative_stem
                .as_ref()
                .ok_or(Error::MissingPrincipalPart {
                    field: MetadataField::ComparisonStem,
                })?;
            (
                vec![join(
                    stem.canonical(),
                    comparison_long_adjective_ending(cell)?,
                )],
                match cell.comparison {
                    Comparison::Comparative => "SYN-ADJ-COMPARATIVE-LONG-ALYPY-58",
                    Comparison::Superlative => "SYN-ADJ-SUPERLATIVE-LONG-ALYPY-59",
                    Comparison::Positive => unreachable!(),
                },
            )
        }
    };
    if cell.case == Case::Accusative && cell.animacy == Animacy::Animate {
        let nominative_cell = AdjectiveCell {
            animacy: Animacy::Inanimate,
            ..cell
        };
        let nominative_like = match cell.comparison {
            Comparison::Positive => positive_adjective_surface(lexeme, nominative_cell)?,
            Comparison::Comparative | Comparison::Superlative => join(
                lexeme
                    .comparative_stem
                    .as_ref()
                    .ok_or(Error::MissingPrincipalPart {
                        field: MetadataField::ComparisonStem,
                    })?
                    .canonical(),
                comparison_long_adjective_ending(nominative_cell)?,
            ),
        };
        if cell.number == Number::Plural {
            expanded.insert(0, nominative_like);
        } else if !expanded.contains(&nominative_like) {
            expanded.push(nominative_like);
        }
        expanded.dedup();
    }
    normative_variants(
        expanded,
        rule,
        profile,
        match cell.form {
            AdjectiveForm::Short => "short-adjective-declension",
            AdjectiveForm::Long => "long-adjective-declension",
        },
        lexeme.lemma.canonical(),
    )
}

/// Validates source-defined positive-stem allomorphy independently of any
/// lexical registry. This is intentionally available to explicit callers as
/// well as the bundled facade.
pub fn validate_adjective_lexeme(lexeme: &AdjectiveLexeme) -> Result<()> {
    if lexeme.class == AdjectiveClass::Velar
        && !lexeme
            .stem
            .canonical()
            .chars()
            .last()
            .is_some_and(|character| matches!(character, 'г' | 'к' | 'х'))
    {
        return Err(Error::ContradictoryMetadata {
            reason: "a velar adjective requires a stem ending in г, к, or х".into(),
        });
    }
    if lexeme.class == AdjectiveClass::PossessiveIn && !lexeme.stem.canonical().ends_with("ин") {
        return Err(Error::ContradictoryMetadata {
            reason: "a possessive -ин- adjective requires a stem ending in -ин".into(),
        });
    }
    if lexeme.class == AdjectiveClass::PossessiveJ
        && (!lexeme.lemma.canonical().ends_with('ь')
            || lexeme.lemma.canonical().strip_suffix('ь') != Some(lexeme.stem.canonical()))
    {
        return Err(Error::ContradictoryMetadata {
            reason: "a possessive -jь adjective requires a soft-sign lemma built directly on the supplied stem".into(),
        });
    }
    if lexeme.class == AdjectiveClass::PossessiveSk && !lexeme.stem.canonical().ends_with("ск") {
        return Err(Error::ContradictoryMetadata {
            reason: "a possessive -ск- adjective requires a stem ending in -ск".into(),
        });
    }
    if lexeme.short_masculine_stem.is_some() != lexeme.short_masculine_formation.is_some() {
        return Err(Error::ContradictoryMetadata {
            reason: "short masculine stem and typed formation must be supplied together".into(),
        });
    }
    if let (Some(short), Some(formation)) = (
        &lexeme.short_masculine_stem,
        lexeme.short_masculine_formation,
    ) {
        let stem = lexeme.stem.canonical();
        let valid = match formation {
            ShortMasculineStemFormation::DoubleNReduction => stem
                .strip_suffix('н')
                .is_some_and(|single_n| stem.ends_with("нн") && single_n == short.canonical()),
            ShortMasculineStemFormation::MobileEInsertion => stem
                .char_indices()
                .last()
                .is_some_and(|(offset, final_character)| {
                    short.canonical() == format!("{}е{final_character}", &stem[..offset])
                }),
            ShortMasculineStemFormation::MobileOInsertion => stem
                .char_indices()
                .last()
                .is_some_and(|(offset, final_character)| {
                    short.canonical() == format!("{}о{final_character}", &stem[..offset])
                }),
        };
        if !valid {
            return Err(Error::ContradictoryMetadata {
                reason: format!(
                    "short masculine stem {:?} does not match {:?} from {:?}",
                    short.canonical(),
                    formation,
                    stem
                ),
            });
        }
    }
    Ok(())
}

pub(crate) fn short_adjective_ending(
    class: AdjectiveClass,
    cell: AdjectiveCell,
) -> Result<&'static str> {
    match class {
        AdjectiveClass::Soft | AdjectiveClass::PossessiveSoft => {
            return soft_short_adjective_ending(cell);
        }
        AdjectiveClass::PossessiveJ => {
            return Err(Error::ContradictoryMetadata {
                reason: "the possessive -jь class requires its mixed surface builder".into(),
            });
        }
        AdjectiveClass::PossessiveIi => return possessive_ii_short_ending(cell),
        AdjectiveClass::Hard
        | AdjectiveClass::HardSibilant
        | AdjectiveClass::Velar
        | AdjectiveClass::PossessiveHard
        | AdjectiveClass::PossessiveIn
        | AdjectiveClass::PossessiveSk => {}
    }
    // Merged kernel: the shared hard short (nominal) declension.
    Ok(kernel_ending(
        kernel::AdjectiveClass::Hard,
        AdjectiveForm::Short,
        cell,
    ))
}

pub(crate) fn possessive_ii_short_ending(cell: AdjectiveCell) -> Result<&'static str> {
    use Case::{
        Accusative as Acc, Dative as Dat, Genitive as Gen, Instrumental as Ins, Locative as Loc,
        Nominative as Nom, Vocative as Voc,
    };
    use Gender::{Feminine as F, Masculine as M, Neuter as N};
    use Number::{Dual as Du, Plural as Pl, Singular as Sg};
    let animate = |nominative, genitive| {
        if cell.animacy == Animacy::Animate {
            genitive
        } else {
            nominative
        }
    };
    Ok(match (cell.number, cell.gender, cell.case) {
        (Sg, M, Nom | Voc) => "їй",
        (Sg, M, Gen) => "їѧ",
        (Sg, M, Dat) => "їю",
        (Sg, M, Acc) => animate("їй", "їѧ"),
        (Sg, M, Ins) => "їимъ",
        (Sg, M, Loc) => "їи",
        (Sg, F, Nom | Gen | Voc) => "їѧ",
        (Sg, F, Dat | Loc) => "їи",
        (Sg, F, Acc) => "їю",
        (Sg, F, Ins) => "їею",
        (Sg, N, Nom | Acc | Voc) => "їе",
        (Sg, N, Gen) => "їѧ",
        (Sg, N, Dat) => "їю",
        (Sg, N, Ins) => "їимъ",
        (Sg, N, Loc) => "їи",
        (Du, M, Nom | Acc | Voc) => "їѧ",
        (Du, F | N, Nom | Acc | Voc) => "їи",
        (Du, _, Gen | Loc) => "їю",
        (Du, _, Dat | Ins) => "їима",
        (Pl, M, Nom | Voc) => "їи",
        (Pl, F | N, Nom | Voc) => "їѧ",
        (Pl, _, Gen | Loc) => "їихъ",
        (Pl, _, Dat) => "їимъ",
        (Pl, M | F, Acc) => animate(if cell.gender == M { "їи" } else { "їѧ" }, "їихъ"),
        (Pl, N, Acc) => "їѧ",
        (Pl, M | N, Ins) => "їи",
        (Pl, F, Ins) => "їими",
    })
}

pub(crate) fn soft_short_adjective_ending(cell: AdjectiveCell) -> Result<&'static str> {
    // Merged kernel: the shared soft short (nominal) declension.
    Ok(kernel_ending(
        kernel::AdjectiveClass::Soft,
        AdjectiveForm::Short,
        cell,
    ))
}

pub(crate) fn long_adjective_ending(
    class: AdjectiveClass,
    cell: AdjectiveCell,
) -> Result<&'static str> {
    match class {
        AdjectiveClass::Soft => return soft_long_adjective_ending(cell),
        AdjectiveClass::HardSibilant => return sibilant_long_adjective_ending(cell),
        AdjectiveClass::Velar => return velar_long_adjective_ending(cell),
        AdjectiveClass::PossessiveIi => return possessive_ii_long_ending(cell),
        AdjectiveClass::PossessiveHard
        | AdjectiveClass::PossessiveSoft
        | AdjectiveClass::PossessiveJ => {
            return Err(Error::HistoricallyInvalidCell {
                reason: "this possessive suffix has no productive long paradigm".into(),
            });
        }
        AdjectiveClass::Hard | AdjectiveClass::PossessiveIn | AdjectiveClass::PossessiveSk => {}
    }
    // Merged kernel: the shared hard long (compound) declension.
    Ok(kernel_ending(
        kernel::AdjectiveClass::Hard,
        AdjectiveForm::Long,
        cell,
    ))
}

/// Occasional compound forms of the `-їй` class (Alypy §56). Direct-case,
/// dual, and plural cells are syncretic with the short table; singular
/// obliques take the source-licensed pronominal extensions, including the
/// explicitly cited `божїѧгѡ`.
pub(crate) fn possessive_ii_long_ending(cell: AdjectiveCell) -> Result<&'static str> {
    use Case::{
        Accusative as Acc, Dative as Dat, Genitive as Gen, Instrumental as Ins, Locative as Loc,
        Nominative as Nom, Vocative as Voc,
    };
    use Gender::{Feminine as F, Masculine as M, Neuter as N};
    use Number::Singular as Sg;
    if cell.number != Sg {
        return possessive_ii_short_ending(cell);
    }
    let animate = |nominative, genitive| {
        if cell.animacy == Animacy::Animate {
            genitive
        } else {
            nominative
        }
    };
    Ok(match (cell.gender, cell.case) {
        (M, Nom | Voc) => "їй",
        (M, Gen) => "їѧгѡ",
        (M, Dat) => "їемꙋ",
        (M, Acc) => animate("їй", "їѧго"),
        (M, Ins) => "їимъ",
        (M, Loc) => "їемъ",
        (F, Nom | Voc) => "їѧ",
        (F, Gen | Dat | Loc) => "їей",
        (F, Acc) => "їю",
        (F, Ins) => "їею",
        (N, Nom | Acc | Voc) => "їе",
        (N, Gen) => "їѧгѡ",
        (N, Dat) => "їемꙋ",
        (N, Ins) => "їимъ",
        (N, Loc) => "їемъ",
    })
}

pub(crate) fn velar_long_adjective_ending(cell: AdjectiveCell) -> Result<&'static str> {
    use Case::{
        Accusative as Acc, Dative as Dat, Genitive as Gen, Instrumental as Ins, Locative as Loc,
        Nominative as Nom, Vocative as Voc,
    };
    use Gender::{Feminine as F, Masculine as M, Neuter as N};
    use Number::{Dual as Du, Plural as Pl, Singular as Sg};
    let animate = |nominative, genitive| {
        if cell.animacy == Animacy::Animate {
            genitive
        } else {
            nominative
        }
    };
    Ok(match (cell.number, cell.gender, cell.case) {
        (Sg, M, Nom | Voc) => "їй",
        (Sg, M, Gen) => "агѡ",
        (Sg, M, Dat) => "омꙋ",
        (Sg, M, Acc) => animate("їй", "аго"),
        (Sg, M, Ins) => "имъ",
        (Sg, M, Loc) => "ѣмъ",
        (Sg, F, Nom | Voc) => "аѧ",
        (Sg, F, Gen) => "їѧ",
        (Sg, F, Dat | Loc) => "ѣй",
        (Sg, F, Acc) => "ꙋю",
        (Sg, F, Ins) => "ою",
        (Sg, N, Nom | Acc | Voc) => "ое",
        (Sg, N, Gen) => "агѡ",
        (Sg, N, Dat) => "омꙋ",
        (Sg, N, Ins) => "имъ",
        (Sg, N, Loc) => "ѣмъ",
        (Du, M, Nom | Acc | Voc) => "аѧ",
        (Du, F | N, Nom | Acc | Voc) => "ѣи",
        (Du, _, Gen | Loc) => "ꙋю",
        (Du, _, Dat | Ins) => "има",
        (Pl, M, Nom | Voc) => "їи",
        (Pl, F, Nom | Voc) => "їѧ",
        (Pl, N, Nom | Acc | Voc) => "аѧ",
        (Pl, _, Gen | Loc) => "ихъ",
        (Pl, _, Dat) => "имъ",
        (Pl, M | F, Acc) => animate("їѧ", "ихъ"),
        (Pl, _, Ins) => "ими",
    })
}

pub(crate) fn soft_long_adjective_ending(cell: AdjectiveCell) -> Result<&'static str> {
    // Merged kernel: the shared soft long (compound) declension.
    Ok(kernel_ending(
        kernel::AdjectiveClass::Soft,
        AdjectiveForm::Long,
        cell,
    ))
}

/// Long comparison endings after the independently supplied `-(ь)ш-`,
/// `-ѣйш-`, or `-айш-` stem. Alypy §58 gives a mixed series: for example,
/// masculine `-шїй`, feminine `-шаѧ`, and neuter `-шее`, while the oblique
/// cells combine hard `-шагѡ`/`-шꙋю` with soft `-шемꙋ`/`-шихъ` endings.
/// Long endings after a hard sibilant stem. This is the §58 mixed series with
/// the two plural cells the Synodal corpus keeps in `ы` after a sibilant: the
/// dative `-ымъ` (`сꙋ́щымъ`, distinct from the singular instrumental
/// `сꙋ́щимъ`) and the feminine nominative/accusative `-ыѧ` (`сꙋ́щыѧ`,
/// distinct from the singular genitive `сꙋ́щїѧ`). The genitive/locative
/// `-ихъ` and instrumental `-ими` are never spelled with `ы` after a sibilant.
pub(crate) fn sibilant_long_adjective_ending(cell: AdjectiveCell) -> Result<&'static str> {
    use Case::{Accusative as Acc, Dative as Dat, Nominative as Nom, Vocative as Voc};
    use Gender::{Feminine as F, Masculine as M};
    use Number::Plural as Pl;
    Ok(match (cell.number, cell.gender, cell.case) {
        (Pl, _, Dat) => "ымъ",
        (Pl, F, Nom | Voc) => "ыѧ",
        (Pl, M | F, Acc) if cell.animacy != Animacy::Animate => "ыѧ",
        _ => comparison_long_adjective_ending(cell)?,
    })
}

pub(crate) fn comparison_long_adjective_ending(cell: AdjectiveCell) -> Result<&'static str> {
    use Case::{
        Accusative as Acc, Dative as Dat, Genitive as Gen, Instrumental as Ins, Locative as Loc,
        Nominative as Nom, Vocative as Voc,
    };
    use Gender::{Feminine as F, Masculine as M, Neuter as N};
    use Number::{Dual as Du, Plural as Pl, Singular as Sg};
    let animate = |nominative, genitive| {
        if cell.animacy == Animacy::Animate {
            genitive
        } else {
            nominative
        }
    };
    Ok(match (cell.number, cell.gender, cell.case) {
        (Sg, M, Nom | Voc) => "їй",
        (Sg, M, Gen) => "агѡ",
        (Sg, M, Dat) => "емꙋ",
        (Sg, M, Acc) => animate("їй", "аго"),
        (Sg, M, Ins) => "имъ",
        (Sg, M, Loc) => "емъ",
        (Sg, F, Nom | Voc) => "аѧ",
        (Sg, F, Gen) => "їѧ",
        (Sg, F, Dat | Loc) => "ей",
        (Sg, F, Acc) => "ꙋю",
        (Sg, F, Ins) => "ею",
        (Sg, N, Nom | Acc | Voc) => "ее",
        (Sg, N, Gen) => "агѡ",
        (Sg, N, Dat) => "емꙋ",
        (Sg, N, Ins) => "имъ",
        (Sg, N, Loc) => "емъ",
        (Du, M, Nom | Acc | Voc) => "аѧ",
        (Du, F | N, Nom | Acc | Voc) => "їи",
        (Du, _, Gen | Loc) => "ꙋю",
        (Du, _, Dat | Ins) => "има",
        (Pl, M, Nom | Voc) => "їи",
        (Pl, F, Nom | Voc) => "їѧ",
        (Pl, N, Nom | Acc | Voc) => "аѧ",
        (Pl, _, Gen | Loc) => "ихъ",
        (Pl, _, Dat) => "имъ",
        (Pl, M | F, Acc) => animate("їѧ", "ихъ"),
        (Pl, _, Ins) => "ими",
    })
}
