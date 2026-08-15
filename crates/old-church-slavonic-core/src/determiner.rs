//! Source-bounded Old Church Slavonic determiner identities and productive profiles.

use crate::adjective::AdjectiveLexeme;
use crate::pronoun::{
    IrregularAgreeingIdentity, PronominalDeclension, PronominalLexeme, StandardPronominalIdentity,
};
use crate::{
    AdjectiveCell, AdjectiveClass, AdjectiveForm, DeterminerCell, InflectionError, PartOfSpeech,
    PredictedForm, RequestedCell,
};

/// The productive inflectional profile of an agreeing determiner.
///
/// The category is syntactic/lexical rather than a separate set of terminals:
/// regular determiners use either the pronominal (`2/p`) or adjectival (`2/a`)
/// declension. The exceptional `кꙑи` paradigm is represented only by
/// [`DeterminerIdentity`], not by an arbitrary-lexeme profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DeterminerDeclension {
    Pronominal(PronominalDeclension),
    Adjectival {
        class: AdjectiveClass,
        form: AdjectiveForm,
    },
}

impl DeterminerDeclension {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Pronominal(PronominalDeclension::Hard) => "2-p-hard",
            Self::Pronominal(PronominalDeclension::Soft) => "2-p-soft",
            Self::Pronominal(PronominalDeclension::J) => "2-p-j",
            Self::Adjectival {
                class: AdjectiveClass::Hard,
                form: AdjectiveForm::Short,
            } => "2-a-hard-short",
            Self::Adjectival {
                class: AdjectiveClass::Soft,
                form: AdjectiveForm::Short,
            } => "2-a-soft-short",
            Self::Adjectival {
                class: AdjectiveClass::Hard,
                form: AdjectiveForm::Long,
            } => "2-a-hard-long",
            Self::Adjectival {
                class: AdjectiveClass::Soft,
                form: AdjectiveForm::Long,
            } => "2-a-soft-long",
        }
    }
}

/// Complete caller-supplied metadata for a productive determiner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeterminerLexeme {
    pub lemma: String,
    pub declension: DeterminerDeclension,
}

/// The source-exhaustive lexical inventory whose primary API ownership is the
/// OCS determiner facade.
///
/// Polivanova §§314–316 supply the eight regular `2/p` identities, §§285 and
/// 303–305 classify long-only `которꙑи`, Paradigmatic Dictionary entry 343
/// classifies `ѥтеръ` as `2/a`, and §§375–376 give the exceptional `кꙑи`
/// paradigm. UT OCS Online §§13 and 23.2 provide an independent
/// terminal-system crosscheck.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DeterminerIdentity {
    RelativeMannerYak,
    RelativeQuantityYelik,
    InterrogativeMannerKak,
    InterrogativeQuantityKolik,
    DemonstrativeQuantitySelik,
    DemonstrativeMannerTak,
    DemonstrativeQuantityTolik,
    InterrogativePossessiveChii,
    InterrogativeKyi,
    InterrogativeKotoryi,
    IndefiniteYeter,
}

impl DeterminerIdentity {
    pub const ALL: [Self; 11] = [
        Self::RelativeMannerYak,
        Self::RelativeQuantityYelik,
        Self::InterrogativeMannerKak,
        Self::InterrogativeQuantityKolik,
        Self::DemonstrativeQuantitySelik,
        Self::DemonstrativeMannerTak,
        Self::DemonstrativeQuantityTolik,
        Self::InterrogativePossessiveChii,
        Self::InterrogativeKyi,
        Self::InterrogativeKotoryi,
        Self::IndefiniteYeter,
    ];

    pub const fn canonical_lemma(self) -> &'static str {
        match self {
            Self::RelativeMannerYak => "ꙗкъ",
            Self::RelativeQuantityYelik => "ѥликъ",
            Self::InterrogativeMannerKak => "какъ",
            Self::InterrogativeQuantityKolik => "коликъ",
            Self::DemonstrativeQuantitySelik => "селикъ",
            Self::DemonstrativeMannerTak => "такъ",
            Self::DemonstrativeQuantityTolik => "толикъ",
            Self::InterrogativePossessiveChii => "чии",
            Self::InterrogativeKyi => "кꙑи",
            Self::InterrogativeKotoryi => "которꙑи",
            Self::IndefiniteYeter => "ѥтеръ",
        }
    }

    pub const fn source_union_aliases(self) -> &'static [&'static str] {
        match self {
            Self::RelativeMannerYak => &["ꙗкъ"],
            Self::RelativeQuantityYelik => &["ѥликъ"],
            Self::InterrogativeMannerKak => &["какъ"],
            Self::InterrogativeQuantityKolik => &["коликъ"],
            Self::DemonstrativeQuantitySelik => &["селикъ"],
            Self::DemonstrativeMannerTak => &["такъ"],
            Self::DemonstrativeQuantityTolik => &["толикъ"],
            Self::InterrogativePossessiveChii => &["чии"],
            Self::InterrogativeKyi => &["кꙑи"],
            Self::InterrogativeKotoryi => &["которꙑи", "которыи"],
            Self::IndefiniteYeter => &["ѥтеръ", "етеръ"],
        }
    }

    pub fn classify_source_union_lemma(lemma: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|identity| identity.source_union_aliases().contains(&lemma))
    }

    pub const fn standard_pronominal(self) -> Option<StandardPronominalIdentity> {
        match self {
            Self::RelativeMannerYak => Some(StandardPronominalIdentity::RelativeMannerYak),
            Self::RelativeQuantityYelik => Some(StandardPronominalIdentity::RelativeQuantityYelik),
            Self::InterrogativeMannerKak => {
                Some(StandardPronominalIdentity::InterrogativeMannerKak)
            }
            Self::InterrogativeQuantityKolik => {
                Some(StandardPronominalIdentity::InterrogativeQuantityKolik)
            }
            Self::DemonstrativeQuantitySelik => {
                Some(StandardPronominalIdentity::DemonstrativeQuantitySelik)
            }
            Self::DemonstrativeMannerTak => {
                Some(StandardPronominalIdentity::DemonstrativeMannerTak)
            }
            Self::DemonstrativeQuantityTolik => {
                Some(StandardPronominalIdentity::DemonstrativeQuantityTolik)
            }
            Self::InterrogativePossessiveChii => {
                Some(StandardPronominalIdentity::InterrogativePossessiveChii)
            }
            Self::InterrogativeKyi | Self::InterrogativeKotoryi | Self::IndefiniteYeter => None,
        }
    }

    pub const fn irregular_agreeing(self) -> Option<IrregularAgreeingIdentity> {
        match self {
            Self::InterrogativeKyi => Some(IrregularAgreeingIdentity::InterrogativeKyi),
            _ => None,
        }
    }

    pub const fn productive_lexeme(self) -> Option<DeterminerLexemeRef> {
        if let Some(identity) = self.standard_pronominal() {
            return Some(DeterminerLexemeRef {
                lemma: self.canonical_lemma(),
                declension: DeterminerDeclension::Pronominal(identity.declension()),
            });
        }
        match self {
            Self::InterrogativeKotoryi => Some(DeterminerLexemeRef {
                lemma: self.canonical_lemma(),
                declension: DeterminerDeclension::Adjectival {
                    class: AdjectiveClass::Hard,
                    form: AdjectiveForm::Long,
                },
            }),
            Self::IndefiniteYeter => Some(DeterminerLexemeRef {
                lemma: self.canonical_lemma(),
                declension: DeterminerDeclension::Adjectival {
                    class: AdjectiveClass::Hard,
                    form: AdjectiveForm::Short,
                },
            }),
            Self::InterrogativeKyi => None,
            _ => None,
        }
    }
}

/// A borrowed static productive specification used by the closed identity
/// inventory without allocating.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeterminerLexemeRef {
    pub lemma: &'static str,
    pub declension: DeterminerDeclension,
}

impl DeterminerLexemeRef {
    pub fn to_owned(self) -> DeterminerLexeme {
        DeterminerLexeme {
            lemma: self.lemma.to_string(),
            declension: self.declension,
        }
    }
}

/// Decline an arbitrary determiner from complete caller-supplied metadata.
pub fn decline(
    lexeme: &DeterminerLexeme,
    cell: DeterminerCell,
) -> Result<PredictedForm, InflectionError> {
    let prediction = match lexeme.declension {
        DeterminerDeclension::Pronominal(declension) => {
            crate::pronoun::decline_pronominal_for_part_of_speech(
                &PronominalLexeme {
                    lemma: lexeme.lemma.clone(),
                    declension,
                },
                PartOfSpeech::Determiner,
                cell.case,
                cell.number,
                cell.gender,
            )
        }
        DeterminerDeclension::Adjectival { class, form } => {
            crate::adjective::decline_from_citation(
                &AdjectiveLexeme {
                    lemma: lexeme.lemma.clone(),
                    class,
                },
                form,
                AdjectiveCell {
                    case: cell.case,
                    number: cell.number,
                    gender: cell.gender,
                    animacy: cell.animacy,
                    form,
                },
            )
        }
    };
    prediction.map_err(|error| remap_cell_error(error, &lexeme.lemma, cell))
}

fn remap_cell_error(error: InflectionError, lemma: &str, cell: DeterminerCell) -> InflectionError {
    match error {
        InflectionError::HistoricallyInvalidCell { .. } => {
            InflectionError::historically_invalid(lemma, RequestedCell::Determiner(cell))
        }
        InflectionError::UnsupportedCell { .. } => {
            InflectionError::unsupported(lemma, RequestedCell::Determiner(cell))
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adjective::LongOnlyAdjectiveIdentity;
    use crate::{Animacy, Case, Gender, Number};
    use std::collections::BTreeSet;

    #[test]
    fn identity_inventory_is_exhaustive_nonoverlapping_and_morphologically_owned() {
        assert_eq!(DeterminerIdentity::ALL.len(), 11);
        let mut aliases = BTreeSet::new();
        let mut regular = 0;
        let mut irregular = 0;
        let mut adjectival = 0;
        for identity in DeterminerIdentity::ALL {
            assert_eq!(
                DeterminerIdentity::classify_source_union_lemma(identity.canonical_lemma()),
                Some(identity)
            );
            for alias in identity.source_union_aliases() {
                assert!(aliases.insert(*alias), "duplicate determiner alias {alias}");
            }
            if identity.standard_pronominal().is_some() {
                regular += 1;
            } else if identity.irregular_agreeing().is_some() {
                irregular += 1;
            } else if identity.productive_lexeme().is_some() {
                adjectival += 1;
            }
        }
        assert_eq!((regular, irregular, adjectival), (8, 1, 2));
        assert_eq!(aliases.len(), 13);
    }

    #[test]
    fn productive_profiles_cover_every_cell_and_preserve_real_defectivity() {
        for identity in DeterminerIdentity::ALL {
            let Some(lexeme) = identity.productive_lexeme() else {
                continue;
            };
            let lexeme = lexeme.to_owned();
            let mut valid = 0;
            let mut invalid = 0;
            for cell in DeterminerCell::all() {
                match decline(&lexeme, cell) {
                    Ok(form) => {
                        assert!(!form.text.is_empty());
                        valid += 1;
                    }
                    Err(InflectionError::HistoricallyInvalidCell {
                        cell: RequestedCell::Determiner(requested),
                        ..
                    }) => {
                        assert_eq!(requested, cell);
                        invalid += 1;
                    }
                    other => panic!("unexpected determiner outcome: {other:?}"),
                }
            }
            if identity.standard_pronominal().is_some() {
                assert_eq!((valid, invalid), (108, 18));
            } else {
                assert_eq!((valid, invalid), (126, 0));
            }
        }
    }

    #[test]
    fn adjective_backed_profiles_match_their_source_classes() {
        let forms = [
            (
                DeterminerIdentity::IndefiniteYeter,
                Case::Genitive,
                Number::Singular,
                Gender::Masculine,
                Animacy::Inanimate,
                "ѥтера",
            ),
            (
                DeterminerIdentity::IndefiniteYeter,
                Case::Nominative,
                Number::Plural,
                Gender::Masculine,
                Animacy::Inanimate,
                "ѥтери",
            ),
            (
                DeterminerIdentity::InterrogativeKotoryi,
                Case::Accusative,
                Number::Singular,
                Gender::Masculine,
                Animacy::Animate,
                "котораѥго",
            ),
        ];
        for (identity, case, number, gender, animacy, expected) in forms {
            let form = decline(
                &identity
                    .productive_lexeme()
                    .expect("productive identity")
                    .to_owned(),
                DeterminerCell {
                    case,
                    number,
                    gender,
                    animacy,
                },
            )
            .expect("licensed determiner cell");
            assert_eq!(form.text, expected);
        }
    }

    #[test]
    fn strict_metadata_rejects_a_contradictory_citation() {
        let lexeme = DeterminerLexeme {
            lemma: "чии".to_string(),
            declension: DeterminerDeclension::Pronominal(PronominalDeclension::Hard),
        };
        assert!(matches!(
            decline(
                &lexeme,
                DeterminerCell {
                    case: Case::Nominative,
                    number: Number::Singular,
                    gender: Gender::Masculine,
                    animacy: Animacy::Inanimate,
                }
            ),
            Err(InflectionError::InvalidLemma { .. })
        ));
    }

    #[test]
    fn strict_metadata_disambiguates_a_soft_long_citation() {
        let lexeme = DeterminerLexeme {
            lemma: "синии".to_string(),
            declension: DeterminerDeclension::Adjectival {
                class: AdjectiveClass::Soft,
                form: AdjectiveForm::Long,
            },
        };
        assert_eq!(
            decline(
                &lexeme,
                DeterminerCell {
                    case: Case::Nominative,
                    number: Number::Singular,
                    gender: Gender::Masculine,
                    animacy: Animacy::Inanimate,
                },
            )
            .expect("explicit soft long determiner")
            .text,
            "синии"
        );
    }

    #[test]
    fn long_only_identity_reuses_the_single_adjective_lexical_fact() {
        assert_eq!(
            LongOnlyAdjectiveIdentity::classify_source_union_lemma(
                DeterminerIdentity::InterrogativeKotoryi.canonical_lemma()
            ),
            Some(LongOnlyAdjectiveIdentity::InterrogativeKotoryi)
        );
    }
}
