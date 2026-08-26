//! Source-reviewed Synodal Church Slavonic numeral-word morphology.
//!
//! Since the phase-4 numeral merge (docs/UNIFIED_LANGUAGE_PROMPT.md) the
//! shared closed cardinal tables (one through four, the five–nine plural
//! obliques, ten, hundred) and the agreeing collective plural terminals
//! live in the merged kernel `church_slavonic_core::numeral`, queried with
//! `Recension::SynodalRussian`; this module is the family adapter that
//! keeps the public API, validation, `FormSet` plumbing, and the
//! noun/adjective-backed and family-only classes (see
//! `church_slavonic_core::divergence::UNMERGED`).

use crate::{
    AdjectiveCell, AdjectiveClass, AdjectiveForm, AdjectiveLexeme, Animacy, Case, Comparison,
    Error, FormSet, Gender, NounCell, NounDeclension, NounLexeme, NounNumberInventory, Number,
    NumeralCell, NumeralKind, OrthographyProfile, Result, SynodalWord,
    morphology::{decline_adjective, decline_noun, normative_variants},
};
use church_slavonic_core::{Recension, numeral as kernel};

const SYN: Recension = Recension::SynodalRussian;

/// Productive and closed numeral-word paradigms licensed by Alypy §§61–70.
///
/// Compound cardinals, compound ordinals, distributives, and periphrastic
/// fractions are constructions rather than additional word declensions. They
/// are intentionally represented by the structured composition API.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum NumeralDeclension {
    CardinalOne,
    CardinalTwo,
    CardinalBoth,
    CardinalThree,
    CardinalFour,
    CardinalIStem,
    CardinalTen,
    CardinalHundred,
    CardinalSecondHard,
    CardinalSecondMixed,
    CardinalFirstHardMasculine,
    CardinalThirdFeminine,
    OrdinalHard,
    OrdinalIi,
    CollectiveAgreeing,
    CollectiveGoverningNeuter,
    CollectiveHardPlural,
    MultiplicativeHard,
    MultiplicativeSoft,
    FractionalHard,
    FractionalFirstHardUStem,
    FractionalSecondHard,
    FractionalThirdFeminine,
}

impl NumeralDeclension {
    pub const ALL: [Self; 23] = [
        Self::CardinalOne,
        Self::CardinalTwo,
        Self::CardinalBoth,
        Self::CardinalThree,
        Self::CardinalFour,
        Self::CardinalIStem,
        Self::CardinalTen,
        Self::CardinalHundred,
        Self::CardinalSecondHard,
        Self::CardinalSecondMixed,
        Self::CardinalFirstHardMasculine,
        Self::CardinalThirdFeminine,
        Self::OrdinalHard,
        Self::OrdinalIi,
        Self::CollectiveAgreeing,
        Self::CollectiveGoverningNeuter,
        Self::CollectiveHardPlural,
        Self::MultiplicativeHard,
        Self::MultiplicativeSoft,
        Self::FractionalHard,
        Self::FractionalFirstHardUStem,
        Self::FractionalSecondHard,
        Self::FractionalThirdFeminine,
    ];

    #[must_use]
    pub const fn kind(self) -> NumeralKind {
        match self {
            Self::CardinalOne
            | Self::CardinalTwo
            | Self::CardinalBoth
            | Self::CardinalThree
            | Self::CardinalFour
            | Self::CardinalIStem
            | Self::CardinalTen
            | Self::CardinalHundred
            | Self::CardinalSecondHard
            | Self::CardinalSecondMixed
            | Self::CardinalFirstHardMasculine
            | Self::CardinalThirdFeminine => NumeralKind::Cardinal,
            Self::OrdinalHard | Self::OrdinalIi => NumeralKind::Ordinal,
            Self::CollectiveAgreeing
            | Self::CollectiveGoverningNeuter
            | Self::CollectiveHardPlural => NumeralKind::Collective,
            Self::MultiplicativeHard | Self::MultiplicativeSoft => NumeralKind::Multiplicative,
            Self::FractionalHard
            | Self::FractionalFirstHardUStem
            | Self::FractionalSecondHard
            | Self::FractionalThirdFeminine => NumeralKind::Fractional,
        }
    }

    #[must_use]
    pub const fn default_number_inventory(self) -> NumeralNumberInventory {
        match self {
            Self::CardinalOne => NumeralNumberInventory::SingularOnly,
            Self::CardinalTwo | Self::CardinalBoth => NumeralNumberInventory::DualOnly,
            Self::CardinalThree
            | Self::CardinalFour
            | Self::CollectiveAgreeing
            | Self::CollectiveHardPlural => NumeralNumberInventory::PluralOnly,
            Self::CardinalIStem => NumeralNumberInventory::SingularAndPlural,
            Self::CollectiveGoverningNeuter => NumeralNumberInventory::SingularOnly,
            Self::CardinalTen
            | Self::CardinalHundred
            | Self::CardinalSecondHard
            | Self::CardinalSecondMixed
            | Self::CardinalFirstHardMasculine
            | Self::CardinalThirdFeminine
            | Self::OrdinalHard
            | Self::OrdinalIi
            | Self::MultiplicativeHard
            | Self::MultiplicativeSoft
            | Self::FractionalHard
            | Self::FractionalFirstHardUStem
            | Self::FractionalSecondHard
            | Self::FractionalThirdFeminine => NumeralNumberInventory::All,
        }
    }
}

/// Lexically licensed grammatical numbers for a numeral word.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum NumeralNumberInventory {
    #[default]
    All,
    SingularOnly,
    DualOnly,
    PluralOnly,
    SingularAndDual,
    SingularAndPlural,
    DualAndPlural,
}

impl NumeralNumberInventory {
    #[must_use]
    pub const fn contains(self, number: Number) -> bool {
        matches!(
            (self, number),
            (Self::All, _)
                | (Self::SingularOnly, Number::Singular)
                | (Self::DualOnly, Number::Dual)
                | (Self::PluralOnly, Number::Plural)
                | (Self::SingularAndDual, Number::Singular | Number::Dual)
                | (Self::SingularAndPlural, Number::Singular | Number::Plural)
                | (Self::DualAndPlural, Number::Dual | Number::Plural)
        )
    }
}

/// Complete typed metadata for one productive Synodal numeral word.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct NumeralLexeme {
    pub lemma: SynodalWord,
    pub stem: SynodalWord,
    pub declension: NumeralDeclension,
    pub number_inventory: NumeralNumberInventory,
}

impl NumeralLexeme {
    #[must_use]
    pub const fn new(lemma: SynodalWord, stem: SynodalWord, declension: NumeralDeclension) -> Self {
        Self {
            lemma,
            stem,
            declension,
            number_inventory: declension.default_number_inventory(),
        }
    }

    #[must_use]
    pub const fn with_number_inventory(mut self, inventory: NumeralNumberInventory) -> Self {
        self.number_inventory = inventory;
        self
    }
}

/// Validates numeral-class metadata without generating a form.
pub fn validate_numeral_lexeme(lexeme: &NumeralLexeme) -> Result<()> {
    let lemma = lexeme.lemma.canonical();
    let stem = lexeme.stem.canonical();
    if lemma.is_empty() || stem.is_empty() {
        return contradictory("a productive numeral requires nonempty lemma and stem metadata");
    }
    for number in Number::ALL {
        if lexeme.number_inventory.contains(number)
            && !lexeme
                .declension
                .default_number_inventory()
                .contains(number)
        {
            return contradictory(
                "the supplied numeral number inventory exceeds the source-licensed class",
            );
        }
    }
    match lexeme.declension {
        NumeralDeclension::CardinalOne if lemma != "єдинъ" || stem != "єдин" => {
            contradictory("the cardinal-one class requires єдинъ with stem єдин")
        }
        NumeralDeclension::CardinalTwo if lemma != "два" || stem != "дв" => {
            contradictory("the cardinal-two class requires два with stem дв")
        }
        NumeralDeclension::CardinalBoth if lemma != "оба" || stem != "об" => {
            contradictory("the cardinal-both class requires оба with stem об")
        }
        NumeralDeclension::CardinalThree if lemma != "три" || stem != "тр" => {
            contradictory("the cardinal-three class requires три with stem тр")
        }
        NumeralDeclension::CardinalFour if lemma != "четыре" || stem != "четыр" => {
            contradictory("the cardinal-four class requires четыре with stem четыр")
        }
        NumeralDeclension::CardinalIStem
            if !lemma.ends_with('ь') || lemma.strip_suffix('ь') != Some(stem) =>
        {
            contradictory("an i-stem cardinal requires a citation in -ь and its bare stem")
        }
        NumeralDeclension::CardinalTen if lemma != "десѧть" || stem != "десѧт" => {
            contradictory("the cardinal-ten class requires десѧть with stem десѧт")
        }
        NumeralDeclension::CardinalHundred if lemma != "сто" || stem != "ст" => {
            contradictory("the cardinal-hundred class requires сто with stem ст")
        }
        NumeralDeclension::CollectiveAgreeing | NumeralDeclension::CollectiveGoverningNeuter
            if !stem.ends_with('о') =>
        {
            contradictory("the двои/двое collective classes require a stem ending in -о")
        }
        NumeralDeclension::CollectiveHardPlural if !stem.ends_with("ер") => {
            contradictory("the higher collective class requires a reviewed -ер- stem")
        }
        NumeralDeclension::CardinalSecondHard => {
            validate_noun_shape(lexeme, Gender::Feminine, NounDeclension::SecondHard)
        }
        NumeralDeclension::CardinalSecondMixed => {
            validate_noun_shape(lexeme, Gender::Feminine, NounDeclension::SecondMixed)
        }
        NumeralDeclension::CardinalFirstHardMasculine => validate_noun_shape(
            lexeme,
            Gender::Masculine,
            NounDeclension::FirstHardMasculine,
        ),
        NumeralDeclension::CardinalThirdFeminine => {
            validate_noun_shape(lexeme, Gender::Feminine, NounDeclension::ThirdFeminine)
        }
        NumeralDeclension::FractionalFirstHardUStem => validate_noun_shape(
            lexeme,
            Gender::Masculine,
            NounDeclension::FirstHardMasculineUStem,
        ),
        NumeralDeclension::FractionalSecondHard => {
            validate_noun_shape(lexeme, Gender::Feminine, NounDeclension::SecondHard)
        }
        NumeralDeclension::FractionalThirdFeminine => {
            validate_noun_shape(lexeme, Gender::Feminine, NounDeclension::ThirdFeminine)
        }
        NumeralDeclension::CardinalOne
        | NumeralDeclension::CardinalTwo
        | NumeralDeclension::CardinalBoth
        | NumeralDeclension::CardinalThree
        | NumeralDeclension::CardinalFour
        | NumeralDeclension::CardinalIStem
        | NumeralDeclension::CardinalTen
        | NumeralDeclension::CardinalHundred
        | NumeralDeclension::OrdinalHard
        | NumeralDeclension::OrdinalIi
        | NumeralDeclension::CollectiveAgreeing
        | NumeralDeclension::CollectiveGoverningNeuter
        | NumeralDeclension::CollectiveHardPlural
        | NumeralDeclension::MultiplicativeHard
        | NumeralDeclension::MultiplicativeSoft
        | NumeralDeclension::FractionalHard => Ok(()),
    }
}

/// Generates one source-licensed Synodal numeral-word cell.
pub fn decline_numeral(
    lexeme: &NumeralLexeme,
    cell: NumeralCell,
    profile: OrthographyProfile,
) -> Result<FormSet> {
    validate_numeral_lexeme(lexeme)?;
    if cell.kind != lexeme.declension.kind() {
        return historically_invalid("the requested numeral kind does not match the lexical class");
    }
    if !lexeme.number_inventory.contains(cell.number) {
        return historically_invalid(
            "the requested grammatical number is absent from this numeral paradigm",
        );
    }
    let (forms, rule) = match lexeme.declension {
        NumeralDeclension::CardinalOne => (
            cardinal_one_forms(cell)?,
            "SYN-NUMERAL-CARDINAL-ONE-ALYPY-62",
        ),
        NumeralDeclension::CardinalTwo => (
            cardinal_two_forms(cell, false)?,
            "SYN-NUMERAL-CARDINAL-TWO-BOTH-ALYPY-62",
        ),
        NumeralDeclension::CardinalBoth => (
            cardinal_two_forms(cell, true)?,
            "SYN-NUMERAL-CARDINAL-TWO-BOTH-ALYPY-62",
        ),
        NumeralDeclension::CardinalThree => (
            cardinal_three_forms(cell)?,
            "SYN-NUMERAL-CARDINAL-THREE-ALYPY-62",
        ),
        NumeralDeclension::CardinalFour => (
            cardinal_four_forms(cell)?,
            "SYN-NUMERAL-CARDINAL-FOUR-ALYPY-62",
        ),
        NumeralDeclension::CardinalIStem => (
            cardinal_i_stem_forms(lexeme, cell)?,
            "SYN-NUMERAL-CARDINAL-I-STEM-ALYPY-62",
        ),
        NumeralDeclension::CardinalTen => (
            cardinal_ten_forms(cell)?,
            "SYN-NUMERAL-CARDINAL-TEN-ALYPY-62",
        ),
        NumeralDeclension::CardinalHundred => (
            cardinal_hundred_forms(cell)?,
            "SYN-NUMERAL-CARDINAL-HUNDRED-ALYPY-62",
        ),
        NumeralDeclension::CardinalSecondHard => (
            noun_like_forms(lexeme, cell, Gender::Feminine, NounDeclension::SecondHard)?,
            "SYN-NUMERAL-CARDINAL-MAGNITUDE-NOUN-ALYPY-61-62",
        ),
        NumeralDeclension::CardinalSecondMixed => (
            noun_like_forms(lexeme, cell, Gender::Feminine, NounDeclension::SecondMixed)?,
            "SYN-NUMERAL-CARDINAL-MAGNITUDE-NOUN-ALYPY-61-62",
        ),
        NumeralDeclension::CardinalFirstHardMasculine => (
            noun_like_forms(
                lexeme,
                cell,
                Gender::Masculine,
                NounDeclension::FirstHardMasculine,
            )?,
            "SYN-NUMERAL-CARDINAL-MAGNITUDE-NOUN-ALYPY-61-62",
        ),
        NumeralDeclension::CardinalThirdFeminine => (
            noun_like_forms(
                lexeme,
                cell,
                Gender::Feminine,
                NounDeclension::ThirdFeminine,
            )?,
            "SYN-NUMERAL-CARDINAL-MAGNITUDE-NOUN-ALYPY-61-62",
        ),
        NumeralDeclension::OrdinalHard => (
            adjective_like_forms(lexeme, cell, AdjectiveClass::Hard)?,
            "SYN-NUMERAL-ORDINAL-ADJECTIVAL-ALYPY-68",
        ),
        NumeralDeclension::OrdinalIi => (
            adjective_like_forms(lexeme, cell, AdjectiveClass::PossessiveIi)?,
            "SYN-NUMERAL-ORDINAL-ADJECTIVAL-ALYPY-68",
        ),
        NumeralDeclension::CollectiveAgreeing => (
            collective_agreeing_forms(lexeme.stem.canonical(), cell)?,
            "SYN-NUMERAL-COLLECTIVE-AGREEING-ALYPY-69",
        ),
        NumeralDeclension::CollectiveGoverningNeuter => (
            collective_governing_forms(lexeme.stem.canonical(), cell)?,
            "SYN-NUMERAL-COLLECTIVE-GOVERNING-ALYPY-69",
        ),
        NumeralDeclension::CollectiveHardPlural => (
            collective_hard_plural_forms(lexeme.stem.canonical(), cell)?,
            "SYN-NUMERAL-COLLECTIVE-HARD-PLURAL-ALYPY-69",
        ),
        NumeralDeclension::MultiplicativeHard => (
            adjective_like_forms(lexeme, cell, AdjectiveClass::Hard)?,
            "SYN-NUMERAL-MULTIPLICATIVE-ADJECTIVAL-ALYPY-61-70",
        ),
        NumeralDeclension::MultiplicativeSoft => (
            adjective_like_forms(lexeme, cell, AdjectiveClass::Soft)?,
            "SYN-NUMERAL-MULTIPLICATIVE-ADJECTIVAL-ALYPY-61-70",
        ),
        NumeralDeclension::FractionalHard => (
            adjective_like_forms(lexeme, cell, AdjectiveClass::Hard)?,
            "SYN-NUMERAL-FRACTIONAL-ADJECTIVAL-ALYPY-51-TARGET",
        ),
        NumeralDeclension::FractionalFirstHardUStem => (
            noun_like_forms(
                lexeme,
                cell,
                Gender::Masculine,
                NounDeclension::FirstHardMasculineUStem,
            )?,
            "SYN-NUMERAL-FRACTIONAL-NOUN-ALYPY-61-70",
        ),
        NumeralDeclension::FractionalSecondHard => (
            noun_like_forms(lexeme, cell, Gender::Feminine, NounDeclension::SecondHard)?,
            "SYN-NUMERAL-FRACTIONAL-NOUN-ALYPY-61-70",
        ),
        NumeralDeclension::FractionalThirdFeminine => (
            noun_like_forms(
                lexeme,
                cell,
                Gender::Feminine,
                NounDeclension::ThirdFeminine,
            )?,
            "SYN-NUMERAL-FRACTIONAL-NOUN-ALYPY-61-70",
        ),
    };
    normative_variants(
        forms,
        rule,
        profile,
        "numeral-declension",
        lexeme.lemma.canonical(),
    )
}

fn validate_noun_shape(
    lexeme: &NumeralLexeme,
    gender: Gender,
    declension: NounDeclension,
) -> Result<()> {
    crate::validate_noun_lexeme(&NounLexeme::new(
        lexeme.lemma.clone(),
        lexeme.stem.clone(),
        gender,
        declension,
    ))
}

fn contradictory<T>(reason: &str) -> Result<T> {
    Err(Error::ContradictoryMetadata {
        reason: reason.into(),
    })
}

fn historically_invalid<T>(reason: &str) -> Result<T> {
    Err(Error::HistoricallyInvalidCell {
        reason: reason.into(),
    })
}

fn require_gender(cell: NumeralCell) -> Result<Gender> {
    cell.gender.ok_or(Error::MissingMetadata {
        field: crate::MetadataField::Gender,
    })
}

fn reject_gender(cell: NumeralCell) -> Result<()> {
    if cell.gender.is_some() {
        historically_invalid("this substantival numeral has no agreement-gender dimension")
    } else {
        Ok(())
    }
}

fn reject_vocative(cell: NumeralCell) -> Result<()> {
    if cell.case == Case::Vocative {
        historically_invalid("Alypy's numeral paradigm does not license a vocative cell")
    } else {
        Ok(())
    }
}

fn join(stem: &str, ending: &str) -> String {
    let mut result = String::with_capacity(stem.len() + ending.len());
    result.push_str(stem);
    result.push_str(ending);
    result
}

fn cardinal_one_forms(cell: NumeralCell) -> Result<Vec<String>> {
    reject_vocative(cell)?;
    let gender = require_gender(cell)?;
    // Merged kernel: the cardinal one (singular-only in this recension).
    Ok(
        kernel::cardinal_one_cell(cell.case, cell.number, gender, cell.animacy, SYN)
            .iter()
            .map(|form| (*form).into())
            .collect(),
    )
}

fn cardinal_two_forms(cell: NumeralCell, both: bool) -> Result<Vec<String>> {
    reject_vocative(cell)?;
    let gender = require_gender(cell)?;
    let paradigm = if both {
        kernel::PairedCardinal::Both
    } else {
        kernel::PairedCardinal::Two
    };
    // Merged kernel: the dual-only paired cardinals.
    Ok(
        kernel::paired_cardinal_cell(paradigm, cell.case, gender, SYN)
            .iter()
            .map(|form| (*form).into())
            .collect(),
    )
}

fn cardinal_three_forms(cell: NumeralCell) -> Result<Vec<String>> {
    reject_vocative(cell)?;
    let gender = require_gender(cell)?;
    // Merged kernel: the plural-only cardinal three.
    Ok(
        kernel::cardinal_three_cell(cell.case, gender, cell.animacy, SYN)
            .iter()
            .map(|form| (*form).into())
            .collect(),
    )
}

fn cardinal_four_forms(cell: NumeralCell) -> Result<Vec<String>> {
    reject_vocative(cell)?;
    require_gender(cell)?;
    // Merged kernel: the plural-only cardinal four.
    Ok(
        kernel::cardinal_four_cell(cell.case, cell.gender.expect("validated gender"), SYN)
            .iter()
            .map(|form| (*form).into())
            .collect(),
    )
}

fn cardinal_i_stem_forms(lexeme: &NumeralLexeme, cell: NumeralCell) -> Result<Vec<String>> {
    reject_gender(cell)?;
    reject_vocative(cell)?;
    match (cell.number, cell.case) {
        (Number::Singular, _) => noun_like_forms(
            lexeme,
            cell,
            Gender::Feminine,
            NounDeclension::ThirdFeminine,
        ),
        // Merged kernel: the five-through-nine plural adjectival obliques
        // (divergence num:five-nine-plural-obliques).
        (Number::Plural, Case::Genitive | Case::Locative | Case::Dative) => Ok(
            kernel::i_stem_cardinal_plural_oblique_ending(cell.case, SYN)
                .iter()
                .map(|ending| join(lexeme.stem.canonical(), ending))
                .collect(),
        ),
        _ => historically_invalid(
            "five through nine license only singular noun cells and the listed plural adjectival obliques",
        ),
    }
}

fn cardinal_ten_forms(cell: NumeralCell) -> Result<Vec<String>> {
    reject_gender(cell)?;
    reject_vocative(cell)?;
    // Merged kernel: the mixed cardinal-ten tables.
    Ok(kernel::cardinal_ten_cell(cell.case, cell.number, SYN)
        .iter()
        .map(|form| (*form).into())
        .collect())
}

fn cardinal_hundred_forms(cell: NumeralCell) -> Result<Vec<String>> {
    reject_gender(cell)?;
    reject_vocative(cell)?;
    // Merged kernel: the inherited neuter o-stem hundred.
    Ok(kernel::cardinal_hundred_cell(cell.case, cell.number, SYN)
        .iter()
        .map(|form| (*form).into())
        .collect())
}

fn noun_like_forms(
    lexeme: &NumeralLexeme,
    cell: NumeralCell,
    gender: Gender,
    declension: NounDeclension,
) -> Result<Vec<String>> {
    reject_gender(cell)?;
    reject_vocative(cell)?;
    let noun = NounLexeme::new(
        lexeme.lemma.clone(),
        lexeme.stem.clone(),
        gender,
        declension,
    )
    .with_number_inventory(match lexeme.number_inventory {
        NumeralNumberInventory::All => NounNumberInventory::All,
        NumeralNumberInventory::SingularOnly => NounNumberInventory::SingularOnly,
        NumeralNumberInventory::DualOnly => NounNumberInventory::DualOnly,
        NumeralNumberInventory::PluralOnly => NounNumberInventory::PluralOnly,
        NumeralNumberInventory::SingularAndDual => NounNumberInventory::SingularAndDual,
        NumeralNumberInventory::SingularAndPlural => NounNumberInventory::SingularAndPlural,
        NumeralNumberInventory::DualAndPlural => NounNumberInventory::DualAndPlural,
    });
    let forms = decline_noun(
        &noun,
        NounCell {
            case: cell.case,
            number: cell.number,
            animacy: Animacy::Inanimate,
        },
        OrthographyProfile::Expanded,
    )?;
    Ok(forms
        .variants()
        .iter()
        .map(|variant| variant.expanded.clone())
        .collect())
}

fn adjective_like_forms(
    lexeme: &NumeralLexeme,
    cell: NumeralCell,
    class: AdjectiveClass,
) -> Result<Vec<String>> {
    let gender = require_gender(cell)?;
    let adjective = AdjectiveLexeme {
        lemma: lexeme.lemma.clone(),
        stem: lexeme.stem.clone(),
        class,
        short_masculine_stem: None,
        short_masculine_formation: None,
        comparative_stem: None,
        comparison_formation: None,
    };
    let forms = decline_adjective(
        &adjective,
        AdjectiveCell {
            case: cell.case,
            number: cell.number,
            gender,
            animacy: cell.animacy,
            form: AdjectiveForm::Long,
            comparison: Comparison::Positive,
        },
        OrthographyProfile::Expanded,
    )?;
    Ok(forms
        .variants()
        .iter()
        .map(|variant| variant.expanded.clone())
        .collect())
}

fn collective_agreeing_forms(stem: &str, cell: NumeralCell) -> Result<Vec<String>> {
    let gender = require_gender(cell)?;
    // Merged kernel: the agreeing collective plural terminals.
    Ok(
        kernel::collective_agreeing_plural_ending(cell.case, gender, cell.animacy, SYN)
            .iter()
            .map(|ending| join(stem, ending))
            .collect(),
    )
}

fn collective_governing_forms(stem: &str, cell: NumeralCell) -> Result<Vec<String>> {
    use Case::{Accusative as Acc, Dative as Dat, Genitive as Gen, Instrumental as Ins};
    use Case::{Locative as Loc, Nominative as Nom};
    reject_vocative(cell)?;
    if require_gender(cell)? != Gender::Neuter {
        return historically_invalid("the governing двое/трое/обое profile is neuter singular");
    }
    let endings: &[&str] = match cell.case {
        Nom | Acc => &["е"],
        Gen => &["егѡ", "его"],
        Dat => &["емꙋ"],
        Ins => &["имъ"],
        Loc => &["емъ"],
        Case::Vocative => unreachable!(),
    };
    Ok(endings.iter().map(|ending| join(stem, ending)).collect())
}

fn collective_hard_plural_forms(stem: &str, cell: NumeralCell) -> Result<Vec<String>> {
    use Case::{Accusative as Acc, Dative as Dat, Genitive as Gen, Instrumental as Ins};
    use Case::{Locative as Loc, Nominative as Nom, Vocative as Voc};
    let _ = require_gender(cell)?;
    let ending = match (cell.case, cell.animacy) {
        (Nom | Voc, _) => "ы",
        (Gen | Loc, _) => "ыхъ",
        (Dat, _) => "ымъ",
        (Acc, Animacy::Animate) => "ыхъ",
        (Acc, Animacy::Inanimate) => "ы",
        (Ins, _) => "ыми",
    };
    Ok(vec![join(stem, ending)])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn word(value: &str) -> SynodalWord {
        SynodalWord::parse(value).expect("test word")
    }

    fn lexeme(lemma: &str, stem: &str, declension: NumeralDeclension) -> NumeralLexeme {
        NumeralLexeme::new(word(lemma), word(stem), declension)
    }

    fn cell(
        kind: NumeralKind,
        case: Case,
        number: Number,
        gender: Option<Gender>,
        animacy: Animacy,
    ) -> NumeralCell {
        NumeralCell {
            kind,
            case,
            number,
            gender,
            animacy,
        }
    }

    #[test]
    fn source_cardinal_tables_preserve_ordered_variants() {
        let one = lexeme("єдинъ", "єдин", NumeralDeclension::CardinalOne);
        let two = lexeme("два", "дв", NumeralDeclension::CardinalTwo);
        let three = lexeme("три", "тр", NumeralDeclension::CardinalThree);
        let ten = lexeme("десѧть", "десѧт", NumeralDeclension::CardinalTen);
        let examples = [
            (
                &one,
                cell(
                    NumeralKind::Cardinal,
                    Case::Genitive,
                    Number::Singular,
                    Some(Gender::Masculine),
                    Animacy::Inanimate,
                ),
                vec!["єдинагѡ", "єдинаго"],
            ),
            (
                &two,
                cell(
                    NumeralKind::Cardinal,
                    Case::Genitive,
                    Number::Dual,
                    Some(Gender::Feminine),
                    Animacy::Inanimate,
                ),
                vec!["двою", "двꙋ"],
            ),
            (
                &three,
                cell(
                    NumeralKind::Cardinal,
                    Case::Accusative,
                    Number::Plural,
                    Some(Gender::Masculine),
                    Animacy::Animate,
                ),
                vec!["трїехъ", "трехъ", "три"],
            ),
            (
                &ten,
                cell(
                    NumeralKind::Cardinal,
                    Case::Genitive,
                    Number::Plural,
                    None,
                    Animacy::Inanimate,
                ),
                vec!["десѧтъ", "десѧтихъ"],
            ),
        ];
        for (lexeme, cell, expected) in examples {
            assert_eq!(
                decline_numeral(lexeme, cell, OrthographyProfile::Expanded)
                    .expect("source cell")
                    .texts()
                    .collect::<Vec<_>>(),
                expected
            );
        }
    }

    #[test]
    fn third_ordinal_uses_the_full_ii_agreement_table() {
        let third = lexeme("третїй", "трет", NumeralDeclension::OrdinalIi);
        for (case, number, gender, animacy, expected) in [
            (
                Case::Nominative,
                Number::Singular,
                Gender::Masculine,
                Animacy::Inanimate,
                "третїй",
            ),
            (
                Case::Genitive,
                Number::Singular,
                Gender::Masculine,
                Animacy::Inanimate,
                "третїѧгѡ",
            ),
            (
                Case::Accusative,
                Number::Singular,
                Gender::Feminine,
                Animacy::Inanimate,
                "третїю",
            ),
            (
                Case::Nominative,
                Number::Singular,
                Gender::Neuter,
                Animacy::Inanimate,
                "третїе",
            ),
            (
                Case::Genitive,
                Number::Plural,
                Gender::Feminine,
                Animacy::Inanimate,
                "третїихъ",
            ),
        ] {
            let form = decline_numeral(
                &third,
                cell(NumeralKind::Ordinal, case, number, Some(gender), animacy),
                OrthographyProfile::Expanded,
            )
            .expect("licensed third-ordinal cell");
            assert_eq!(
                form.primary_text(),
                expected,
                "{case:?} {number:?} {gender:?}"
            );
        }
    }

    #[test]
    fn noun_adjective_and_collective_profiles_are_distinct() {
        let hundred = lexeme("сто", "ст", NumeralDeclension::CardinalHundred);
        let fifth = lexeme("пѧтый", "пѧт", NumeralDeclension::OrdinalHard);
        let dvoi = lexeme("двои", "дво", NumeralDeclension::CollectiveAgreeing);
        let dvoe = lexeme("двое", "дво", NumeralDeclension::CollectiveGoverningNeuter);
        let pyatery = lexeme("пѧтеры", "пѧтер", NumeralDeclension::CollectiveHardPlural);
        let half_tenth = lexeme("полдесѧтый", "полдесѧт", NumeralDeclension::FractionalHard);
        assert_eq!(
            decline_numeral(
                &hundred,
                cell(
                    NumeralKind::Cardinal,
                    Case::Instrumental,
                    Number::Dual,
                    None,
                    Animacy::Inanimate,
                ),
                OrthographyProfile::Expanded,
            )
            .expect("hundred")
            .primary_text(),
            "стома"
        );
        assert_eq!(
            decline_numeral(
                &fifth,
                cell(
                    NumeralKind::Ordinal,
                    Case::Dative,
                    Number::Dual,
                    Some(Gender::Feminine),
                    Animacy::Inanimate,
                ),
                OrthographyProfile::Expanded,
            )
            .expect("ordinal")
            .primary_text(),
            "пѧтыма"
        );
        assert_eq!(
            decline_numeral(
                &dvoi,
                cell(
                    NumeralKind::Collective,
                    Case::Nominative,
                    Number::Plural,
                    Some(Gender::Neuter),
                    Animacy::Inanimate,
                ),
                OrthographyProfile::Expanded,
            )
            .expect("agreeing collective")
            .primary_text(),
            "двоѧ"
        );
        assert_eq!(
            decline_numeral(
                &dvoe,
                cell(
                    NumeralKind::Collective,
                    Case::Genitive,
                    Number::Singular,
                    Some(Gender::Neuter),
                    Animacy::Inanimate,
                ),
                OrthographyProfile::Expanded,
            )
            .expect("governing collective")
            .texts()
            .collect::<Vec<_>>(),
            ["двоегѡ", "двоего"]
        );
        assert_eq!(
            decline_numeral(
                &pyatery,
                cell(
                    NumeralKind::Collective,
                    Case::Genitive,
                    Number::Plural,
                    Some(Gender::Feminine),
                    Animacy::Inanimate,
                ),
                OrthographyProfile::Expanded,
            )
            .expect("higher collective")
            .primary_text(),
            "пѧтерыхъ"
        );
        assert_eq!(
            decline_numeral(
                &half_tenth,
                cell(
                    NumeralKind::Fractional,
                    Case::Genitive,
                    Number::Singular,
                    Some(Gender::Feminine),
                    Animacy::Inanimate,
                ),
                OrthographyProfile::Expanded,
            )
            .expect("fractional adjective")
            .primary_text(),
            "полдесѧтыѧ"
        );
    }

    #[test]
    fn invalid_kind_number_gender_and_vocative_fail_typed() {
        let five = lexeme("пѧть", "пѧт", NumeralDeclension::CardinalIStem);
        for invalid in [
            cell(
                NumeralKind::Ordinal,
                Case::Nominative,
                Number::Singular,
                None,
                Animacy::Inanimate,
            ),
            cell(
                NumeralKind::Cardinal,
                Case::Nominative,
                Number::Dual,
                None,
                Animacy::Inanimate,
            ),
            cell(
                NumeralKind::Cardinal,
                Case::Nominative,
                Number::Singular,
                Some(Gender::Feminine),
                Animacy::Inanimate,
            ),
            cell(
                NumeralKind::Cardinal,
                Case::Vocative,
                Number::Singular,
                None,
                Animacy::Inanimate,
            ),
        ] {
            assert!(matches!(
                decline_numeral(&five, invalid, OrthographyProfile::Expanded),
                Err(Error::HistoricallyInvalidCell { .. })
            ));
        }
    }
}
