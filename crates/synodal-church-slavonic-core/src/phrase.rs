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
    EncliticPronoun,
    ThirdPersonPrepositionalContraction,
    NegativePronounPrepositional,
    CompoundCardinal,
    CompoundOrdinal,
    RepeatedDistributive,
    MultiplicativeKrat,
    FractionalPart,
}

/// Accentual behavior of a short personal/reflexive pronoun after its host
/// (Alypy §47).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum PronounCliticProsody {
    /// The host's final vowel receives an acute and the enclitic is unaccented.
    AfterFinalVowelStress,
    /// Logical emphasis retains the short pronoun's lexical accent.
    LogicallyStressed,
}

/// Interrogative base retained after a negative `ни-` prefix is separated by
/// a governing preposition (Alypy §48).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum NegativePronounBase {
    Who,
    What,
    Kii,
    Kotoryi,
}

impl NegativePronounBase {
    pub const ALL: [Self; 4] = [Self::Who, Self::What, Self::Kii, Self::Kotoryi];
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum PhraseRole {
    Host,
    FusedPrepositionPronoun,
    Auxiliary,
    Infinitive,
    LParticiple,
    ActiveParticiple,
    PassiveParticiple,
    Particle,
    Preposition,
    Pronoun,
    Numeral,
    Conjunction,
    MultiplicativeUnit,
    FractionNoun,
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
