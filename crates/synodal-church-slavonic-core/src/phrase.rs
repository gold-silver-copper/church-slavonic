use crate::{Error, FormSet, Recension, Result};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum AnalyticConstruction {
    CompoundFuture,
    Perfect,
    Pluperfect,
    Conditional,
    AnalyticPassive,
    PeriphrasticTense,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum PhraseRole {
    Auxiliary,
    Infinitive,
    LParticiple,
    ActiveParticiple,
    PassiveParticiple,
    Particle,
    Complement,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct PhraseToken {
    pub role: PhraseRole,
    pub forms: FormSet,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct RealizedPhrase {
    construction: AnalyticConstruction,
    tokens: Vec<PhraseToken>,
    warnings: Vec<String>,
}

impl RealizedPhrase {
    pub fn new(construction: AnalyticConstruction, tokens: Vec<PhraseToken>) -> Result<Self> {
        if tokens.is_empty() {
            return Err(Error::EmptyFormSet);
        }
        if tokens
            .iter()
            .any(|token| token.forms.target_recension() != Recension::SynodalRussian)
        {
            return Err(Error::ContradictoryMetadata {
                reason: "every analytic phrase token must target Synodal Russian".into(),
            });
        }
        Ok(Self {
            construction,
            tokens,
            warnings: Vec::new(),
        })
    }

    #[must_use]
    pub const fn construction(&self) -> AnalyticConstruction {
        self.construction
    }

    #[must_use]
    pub fn tokens(&self) -> &[PhraseToken] {
        &self.tokens
    }

    #[must_use]
    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }

    #[must_use]
    pub fn primary_text(&self) -> String {
        self.tokens
            .iter()
            .map(|token| token.forms.primary_text())
            .collect::<Vec<_>>()
            .join(" ")
    }
}
