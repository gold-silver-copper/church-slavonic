use synodal_church_slavonic_core::{OrthographyProfile, Result};

use crate::PartOfSpeech;

use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct LexemeSpec {
    inner: Box<LexemeSpecInner>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub(crate) enum LexemeSpecInner {
    Noun(NounSpec),
    Adjective(AdjectiveSpec),
    Determiner(DeterminerSpec),
    Numeral(NumeralSpec),
    Pronoun(PronounSpec),
    Verb(Box<VerbSpec>),
}

impl LexemeSpec {
    pub fn validate(&self) -> Result<()> {
        match self.inner.as_ref() {
            LexemeSpecInner::Noun(spec) => spec.validate(),
            LexemeSpecInner::Adjective(spec) => spec.validate(),
            LexemeSpecInner::Determiner(spec) => spec.validate(),
            LexemeSpecInner::Numeral(spec) => spec.validate(),
            LexemeSpecInner::Pronoun(spec) => spec.validate(),
            LexemeSpecInner::Verb(spec) => spec.validate(),
        }
    }

    #[must_use]
    pub fn lemma(&self) -> &str {
        match self.inner.as_ref() {
            LexemeSpecInner::Noun(spec) => spec.lemma(),
            LexemeSpecInner::Adjective(spec) => spec.lemma(),
            LexemeSpecInner::Determiner(spec) => spec.lemma(),
            LexemeSpecInner::Numeral(spec) => spec.lemma(),
            LexemeSpecInner::Pronoun(spec) => spec.lemma(),
            LexemeSpecInner::Verb(spec) => spec.lemma(),
        }
    }

    #[must_use]
    pub fn part_of_speech(&self) -> PartOfSpeech {
        match self.inner.as_ref() {
            LexemeSpecInner::Noun(_) => PartOfSpeech::Noun,
            LexemeSpecInner::Adjective(_) => PartOfSpeech::Adjective,
            LexemeSpecInner::Determiner(_) => PartOfSpeech::Determiner,
            LexemeSpecInner::Numeral(_) => PartOfSpeech::Numeral,
            LexemeSpecInner::Pronoun(_) => PartOfSpeech::Pronoun,
            LexemeSpecInner::Verb(_) => PartOfSpeech::Verb,
        }
    }

    #[must_use]
    pub fn orthography_ready(&self, profile: OrthographyProfile) -> bool {
        profile != OrthographyProfile::SynodalLiturgical
            || (self.context().accent.is_some() && self.context().positional.is_some())
    }

    pub(crate) fn context(&self) -> &SpecContext {
        match self.inner.as_ref() {
            LexemeSpecInner::Noun(spec) => &spec.context,
            LexemeSpecInner::Adjective(spec) => &spec.context,
            LexemeSpecInner::Determiner(spec) => &spec.context,
            LexemeSpecInner::Numeral(spec) => &spec.context,
            LexemeSpecInner::Pronoun(spec) => &spec.context,
            LexemeSpecInner::Verb(spec) => &spec.context,
        }
    }

    pub(crate) fn inner(&self) -> &LexemeSpecInner {
        &self.inner
    }
}

impl From<NounSpec> for LexemeSpec {
    fn from(spec: NounSpec) -> Self {
        Self {
            inner: Box::new(LexemeSpecInner::Noun(spec)),
        }
    }
}

impl From<AdjectiveSpec> for LexemeSpec {
    fn from(spec: AdjectiveSpec) -> Self {
        Self {
            inner: Box::new(LexemeSpecInner::Adjective(spec)),
        }
    }
}

impl From<DeterminerSpec> for LexemeSpec {
    fn from(spec: DeterminerSpec) -> Self {
        Self {
            inner: Box::new(LexemeSpecInner::Determiner(spec)),
        }
    }
}

impl From<NumeralSpec> for LexemeSpec {
    fn from(spec: NumeralSpec) -> Self {
        Self {
            inner: Box::new(LexemeSpecInner::Numeral(spec)),
        }
    }
}

impl From<PronounSpec> for LexemeSpec {
    fn from(spec: PronounSpec) -> Self {
        Self {
            inner: Box::new(LexemeSpecInner::Pronoun(spec)),
        }
    }
}

impl From<VerbSpec> for LexemeSpec {
    fn from(spec: VerbSpec) -> Self {
        Self {
            inner: Box::new(LexemeSpecInner::Verb(Box::new(spec))),
        }
    }
}
