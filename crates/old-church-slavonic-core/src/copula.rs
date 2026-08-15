//! Independently reviewed suppletive copular series used by OCS periphrases.

use crate::{Number, Person, RuleId};

/// A finite copular series whose identity must not be inferred from one modern
/// dictionary lemma. OCS distributes the present `ѥс-`, future `бѫд-`,
/// past `бѣ-`, and conditional `би-` series differently across sources.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CopulaSeries {
    PresentEs,
    FutureBud,
    ImperfectBe,
    AoristBe,
    ConditionalBi,
    ConditionalAoristBy,
}

impl CopulaSeries {
    pub const ALL: [Self; 6] = [
        Self::PresentEs,
        Self::FutureBud,
        Self::ImperfectBe,
        Self::AoristBe,
        Self::ConditionalBi,
        Self::ConditionalAoristBy,
    ];

    pub const fn rule_id(self) -> RuleId {
        match self {
            Self::PresentEs => RuleId::VerbCopulaPresentEs,
            Self::FutureBud => RuleId::VerbCopulaFutureBud,
            Self::ImperfectBe => RuleId::VerbCopulaImperfectBe,
            Self::AoristBe => RuleId::VerbCopulaAoristBe,
            Self::ConditionalBi => RuleId::VerbCopulaConditionalBi,
            Self::ConditionalAoristBy => RuleId::VerbCopulaConditionalAoristBy,
        }
    }

    pub const fn lemma(self) -> &'static str {
        match self {
            Self::PresentEs => "ѥсмь",
            Self::FutureBud
            | Self::ImperfectBe
            | Self::AoristBe
            | Self::ConditionalBi
            | Self::ConditionalAoristBy => "бꙑти",
        }
    }

    pub const fn authority(self) -> &'static str {
        match self {
            Self::PresentEs => "UT OCS Online lesson 5 §24.1; Polivanova 2023 §§538–542",
            Self::FutureBud => {
                "UT OCS Online lessons 5 §§24.3–24.4 and 9 §45.2; Polivanova 2023 §§543–549"
            }
            Self::ImperfectBe | Self::AoristBe => {
                "UT OCS Online lesson 5 §24.2; Polivanova 2023 §§544–545"
            }
            Self::ConditionalBi | Self::ConditionalAoristBy => {
                "UT OCS Online lesson 6 §27; Polivanova 2023 §§546–549"
            }
        }
    }

    /// Return source-ordered forms for one complete person-number cell.
    pub fn forms(self, person: Person, number: Number) -> Vec<CopulaVariant> {
        use CopulaVariantStatus::{Reconstructed, SourceBacked};
        use Number::{Dual, Plural, Singular};
        use Person::{First, Second, Third};

        let forms: &[CopulaVariant] = match (self, person, number) {
            (Self::PresentEs, First, Singular) => &[CopulaVariant::new("ѥсмь", SourceBacked)],
            (Self::PresentEs, Second, Singular) => &[CopulaVariant::new("ѥси", SourceBacked)],
            (Self::PresentEs, Third, Singular) => &[CopulaVariant::new("ѥстъ", SourceBacked)],
            (Self::PresentEs, First, Dual) => &[CopulaVariant::new("ѥсвѣ", SourceBacked)],
            (Self::PresentEs, Second, Dual) => &[CopulaVariant::new("ѥста", SourceBacked)],
            (Self::PresentEs, Third, Dual) => &[CopulaVariant::new("ѥсте", SourceBacked)],
            (Self::PresentEs, First, Plural) => &[CopulaVariant::new("ѥсмъ", SourceBacked)],
            (Self::PresentEs, Second, Plural) => &[CopulaVariant::new("ѥсте", SourceBacked)],
            (Self::PresentEs, Third, Plural) => &[CopulaVariant::new("сѫтъ", SourceBacked)],

            (Self::FutureBud, First, Singular) => &[CopulaVariant::new("бѫдѫ", SourceBacked)],
            (Self::FutureBud, Second, Singular) => &[CopulaVariant::new("бѫдеши", SourceBacked)],
            (Self::FutureBud, Third, Singular) => &[CopulaVariant::new("бѫдетъ", SourceBacked)],
            (Self::FutureBud, First, Dual) => &[CopulaVariant::new("бѫдевѣ", SourceBacked)],
            (Self::FutureBud, Second, Dual) => &[CopulaVariant::new("бѫдета", SourceBacked)],
            (Self::FutureBud, Third, Dual) => &[CopulaVariant::new("бѫдете", SourceBacked)],
            (Self::FutureBud, First, Plural) => &[CopulaVariant::new("бѫдемъ", SourceBacked)],
            (Self::FutureBud, Second, Plural) => &[CopulaVariant::new("бѫдете", SourceBacked)],
            (Self::FutureBud, Third, Plural) => &[CopulaVariant::new("бѫдѫтъ", SourceBacked)],

            (Self::ImperfectBe, First, Singular) => &[CopulaVariant::new("бѣахъ", SourceBacked)],
            (Self::ImperfectBe, Second | Third, Singular) => {
                &[CopulaVariant::new("бѣаше", SourceBacked)]
            }
            (Self::ImperfectBe, First, Dual) => &[CopulaVariant::new("бѣаховѣ", SourceBacked)],
            (Self::ImperfectBe, Second, Dual) => &[CopulaVariant::new("бѣашета", SourceBacked)],
            (Self::ImperfectBe, Third, Dual) => &[CopulaVariant::new("бѣашете", SourceBacked)],
            (Self::ImperfectBe, First, Plural) => &[CopulaVariant::new("бѣахомъ", SourceBacked)],
            (Self::ImperfectBe, Second, Plural) => &[CopulaVariant::new("бѣашете", SourceBacked)],
            (Self::ImperfectBe, Third, Plural) => &[CopulaVariant::new("бѣахѫ", SourceBacked)],

            (Self::AoristBe, First, Singular) => &[CopulaVariant::new("бѣхъ", SourceBacked)],
            (Self::AoristBe, Second | Third, Singular) => &[CopulaVariant::new("бѣ", SourceBacked)],
            (Self::AoristBe, First, Dual) => &[CopulaVariant::new("бѣховѣ", SourceBacked)],
            (Self::AoristBe, Second, Dual) => &[CopulaVariant::new("бѣста", SourceBacked)],
            (Self::AoristBe, Third, Dual) => &[CopulaVariant::new("бѣсте", SourceBacked)],
            (Self::AoristBe, First, Plural) => &[CopulaVariant::new("бѣхомъ", SourceBacked)],
            (Self::AoristBe, Second, Plural) => &[CopulaVariant::new("бѣсте", SourceBacked)],
            (Self::AoristBe, Third, Plural) => &[CopulaVariant::new("бѣшѧ", SourceBacked)],

            (Self::ConditionalBi, First, Singular) => &[CopulaVariant::new("бимь", SourceBacked)],
            (Self::ConditionalBi, Second | Third, Singular) => {
                &[CopulaVariant::new("би", SourceBacked)]
            }
            (Self::ConditionalBi, First, Dual) => &[CopulaVariant::new("бивѣ", Reconstructed)],
            (Self::ConditionalBi, Second, Dual) => &[CopulaVariant::new("биста", Reconstructed)],
            (Self::ConditionalBi, Third, Dual) => &[CopulaVariant::new("бисте", Reconstructed)],
            (Self::ConditionalBi, First, Plural) => &[
                CopulaVariant::new("бимъ", SourceBacked),
                CopulaVariant::new("бихомъ", SourceBacked),
            ],
            (Self::ConditionalBi, Second, Plural) => &[
                CopulaVariant::new("бисте", SourceBacked),
                CopulaVariant::new("бите", Reconstructed),
            ],
            (Self::ConditionalBi, Third, Plural) => &[
                CopulaVariant::new("бѫ", SourceBacked),
                CopulaVariant::new("бишѧ", SourceBacked),
            ],

            (Self::ConditionalAoristBy, First, Singular) => {
                &[CopulaVariant::new("бꙑхъ", SourceBacked)]
            }
            (Self::ConditionalAoristBy, Second | Third, Singular) => {
                &[CopulaVariant::new("бꙑ", SourceBacked)]
            }
            (Self::ConditionalAoristBy, First, Dual) => {
                &[CopulaVariant::new("бꙑховѣ", SourceBacked)]
            }
            (Self::ConditionalAoristBy, Second, Dual) => {
                &[CopulaVariant::new("бꙑста", SourceBacked)]
            }
            (Self::ConditionalAoristBy, Third, Dual) => {
                &[CopulaVariant::new("бꙑсте", SourceBacked)]
            }
            (Self::ConditionalAoristBy, First, Plural) => {
                &[CopulaVariant::new("бꙑхомъ", SourceBacked)]
            }
            (Self::ConditionalAoristBy, Second, Plural) => {
                &[CopulaVariant::new("бꙑсте", SourceBacked)]
            }
            (Self::ConditionalAoristBy, Third, Plural) => {
                &[CopulaVariant::new("бꙑшѧ", SourceBacked)]
            }
        };
        forms.to_vec()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CopulaVariant {
    pub text: &'static str,
    pub status: CopulaVariantStatus,
}

impl CopulaVariant {
    pub const fn new(text: &'static str, status: CopulaVariantStatus) -> Self {
        Self { text, status }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopulaVariantStatus {
    SourceBacked,
    Reconstructed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_copular_series_has_nine_nonempty_cells() {
        for series in CopulaSeries::ALL {
            for number in Number::ALL {
                for person in Person::ALL {
                    let forms = series.forms(person, number);
                    assert!(!forms.is_empty(), "{series:?} {person:?} {number:?}");
                    assert!(forms.iter().all(|variant| !variant.text.is_empty()));
                }
            }
        }
    }

    #[test]
    fn every_copular_series_matches_the_reviewed_cell_golden() {
        let expected: [(CopulaSeries, [&str; 9]); 6] = [
            (
                CopulaSeries::PresentEs,
                [
                    "ѥсмь", "ѥси", "ѥстъ", "ѥсвѣ", "ѥста", "ѥсте", "ѥсмъ", "ѥсте", "сѫтъ",
                ],
            ),
            (
                CopulaSeries::FutureBud,
                [
                    "бѫдѫ",
                    "бѫдеши",
                    "бѫдетъ",
                    "бѫдевѣ",
                    "бѫдета",
                    "бѫдете",
                    "бѫдемъ",
                    "бѫдете",
                    "бѫдѫтъ",
                ],
            ),
            (
                CopulaSeries::ImperfectBe,
                [
                    "бѣахъ",
                    "бѣаше",
                    "бѣаше",
                    "бѣаховѣ",
                    "бѣашета",
                    "бѣашете",
                    "бѣахомъ",
                    "бѣашете",
                    "бѣахѫ",
                ],
            ),
            (
                CopulaSeries::AoristBe,
                [
                    "бѣхъ",
                    "бѣ",
                    "бѣ",
                    "бѣховѣ",
                    "бѣста",
                    "бѣсте",
                    "бѣхомъ",
                    "бѣсте",
                    "бѣшѧ",
                ],
            ),
            (
                CopulaSeries::ConditionalBi,
                [
                    "бимь",
                    "би",
                    "би",
                    "бивѣ",
                    "биста",
                    "бисте",
                    "бимъ || бихомъ",
                    "бисте || бите",
                    "бѫ || бишѧ",
                ],
            ),
            (
                CopulaSeries::ConditionalAoristBy,
                [
                    "бꙑхъ",
                    "бꙑ",
                    "бꙑ",
                    "бꙑховѣ",
                    "бꙑста",
                    "бꙑсте",
                    "бꙑхомъ",
                    "бꙑсте",
                    "бꙑшѧ",
                ],
            ),
        ];

        for (series, expected_cells) in expected {
            let actual_cells = Number::ALL
                .into_iter()
                .flat_map(|number| {
                    Person::ALL.into_iter().map(move |person| {
                        series
                            .forms(person, number)
                            .iter()
                            .map(|variant| variant.text)
                            .collect::<Vec<_>>()
                            .join(" || ")
                    })
                })
                .collect::<Vec<_>>();
            assert_eq!(actual_cells, expected_cells, "{series:?}");
        }
    }

    #[test]
    fn only_the_source_marked_conditional_cells_are_reconstructed() {
        for series in CopulaSeries::ALL {
            for number in Number::ALL {
                for person in Person::ALL {
                    let has_reconstruction = series
                        .forms(person, number)
                        .iter()
                        .any(|variant| variant.status == CopulaVariantStatus::Reconstructed);
                    assert_eq!(
                        has_reconstruction,
                        series == CopulaSeries::ConditionalBi
                            && (number == Number::Dual
                                || (person == Person::Second && number == Number::Plural))
                    );
                }
            }
        }
    }
}
