use super::*;

pub const MIN_CARDINAL_VALUE: u32 = 1;
pub const MAX_CARDINAL_VALUE: u32 = 1_000_000;
pub const MIN_COMPOUND_ORDINAL_VALUE: u16 = 11;
pub const MAX_COMPOUND_ORDINAL_VALUE: u16 = 1_000;

/// Case, optional agreement gender, and animacy for a cardinal expression.
/// Gender is present exactly when the final component is one through four
/// (including the agreement analysis of eleven through fourteen).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct CompoundNumeralCell {
    pub case: Case,
    pub gender: Option<Gender>,
    pub animacy: Animacy,
}

/// Position of the counted noun relative to a compound numeral (Alypy §66).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum NumeralNounPosition {
    Following,
    Preceding,
}

/// Source-licensed relation between a numeral and the counted noun.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum NumeralGovernment {
    Agreement {
        number: Number,
    },
    GenitivePlural,
    /// Marked subject, predicate, and appositional syntax from Alypy §67.
    ContextualNominativePlural,
}

/// Structural strategy used by one correlated numeral realization.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum NumeralComposition {
    Simple,
    TeenFirstComponentDeclined,
    TeenSecondComponentDeclined,
    TeenBothComponentsDeclined,
    TeenBothComponentsDual,
    TensAgreement,
    TensGovernment,
    TensBothComponentsSingular,
    TensBothComponentsPlural,
    HundredsAgreement,
    HundredsGovernment,
    MagnitudeAgreement,
    MagnitudeGovernment,
    Magnitude,
    AdditiveFinalConjunction,
    AdditiveAllConjunctions,
    AdditiveAsyndetic,
    CompoundOrdinalFused,
    CompoundOrdinalAnalyticTeen,
    CompoundOrdinalAsyndetic,
    CompoundOrdinalConjunction,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct CardinalPhraseAnalysis {
    pub construction: NumeralComposition,
    pub tokens: Vec<PhraseToken>,
}

impl CardinalPhraseAnalysis {
    #[must_use]
    pub fn primary_text(&self) -> String {
        render_tokens(&self.tokens)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct RealizedCardinal {
    value: u32,
    cell: CompoundNumeralCell,
    following_government: Vec<NumeralGovernment>,
    preceding_government: Vec<NumeralGovernment>,
    government_evidence: Vec<Evidence>,
    pub(super) analyses: Vec<CardinalPhraseAnalysis>,
}

impl RealizedCardinal {
    pub(super) fn new(
        value: u32,
        cell: CompoundNumeralCell,
        following_government: Vec<NumeralGovernment>,
        preceding_government: Vec<NumeralGovernment>,
        government_evidence: Vec<Evidence>,
        analyses: Vec<CardinalPhraseAnalysis>,
    ) -> Result<Self> {
        if !(MIN_CARDINAL_VALUE..=MAX_CARDINAL_VALUE).contains(&value) {
            return Err(Error::OutOfRange {
                value,
                maximum: MAX_CARDINAL_VALUE,
            });
        }
        if analyses.is_empty() || analyses.iter().any(|analysis| analysis.tokens.is_empty()) {
            return Err(Error::EmptyFormSet);
        }
        Ok(Self {
            value,
            cell,
            following_government,
            preceding_government,
            government_evidence,
            analyses,
        })
    }

    #[must_use]
    pub const fn value(&self) -> u32 {
        self.value
    }

    #[must_use]
    pub const fn cell(&self) -> CompoundNumeralCell {
        self.cell
    }

    #[must_use]
    pub fn government(&self, position: NumeralNounPosition) -> &[NumeralGovernment] {
        match position {
            NumeralNounPosition::Following => &self.following_government,
            NumeralNounPosition::Preceding => &self.preceding_government,
        }
    }

    /// Normative evidence for the agreement/government alternatives exposed
    /// by [`Self::government`].
    #[must_use]
    pub fn government_evidence(&self) -> &[Evidence] {
        &self.government_evidence
    }

    #[must_use]
    pub fn analyses(&self) -> &[CardinalPhraseAnalysis] {
        &self.analyses
    }

    #[must_use]
    pub fn primary_text(&self) -> String {
        self.analyses[0].primary_text()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct OrdinalPhraseAnalysis {
    pub construction: NumeralComposition,
    pub tokens: Vec<PhraseToken>,
}

impl OrdinalPhraseAnalysis {
    #[must_use]
    pub fn primary_text(&self) -> String {
        render_tokens(&self.tokens)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct RealizedOrdinal {
    value: u16,
    cell: NumeralCell,
    analyses: Vec<OrdinalPhraseAnalysis>,
}

impl RealizedOrdinal {
    pub(super) fn new(
        value: u16,
        cell: NumeralCell,
        analyses: Vec<OrdinalPhraseAnalysis>,
    ) -> Result<Self> {
        if value == 0 || value > MAX_COMPOUND_ORDINAL_VALUE || analyses.is_empty() {
            return Err(Error::OutOfRange {
                value: u32::from(value),
                maximum: u32::from(MAX_COMPOUND_ORDINAL_VALUE),
            });
        }
        Ok(Self {
            value,
            cell,
            analyses,
        })
    }

    #[must_use]
    pub const fn value(&self) -> u16 {
        self.value
    }

    #[must_use]
    pub const fn cell(&self) -> NumeralCell {
        self.cell
    }

    #[must_use]
    pub fn analyses(&self) -> &[OrdinalPhraseAnalysis] {
        &self.analyses
    }

    #[must_use]
    pub fn primary_text(&self) -> String {
        self.analyses[0].primary_text()
    }
}
