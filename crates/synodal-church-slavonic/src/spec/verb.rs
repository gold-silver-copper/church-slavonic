use synodal_church_slavonic_core::{
    AccentParadigm, AdjectiveForm, AoristFormation, Aspect, Error, FiniteTense, FormSet,
    GrammarCell, ImperativeFormation, ImperfectFormation, MetadataField, NounLexeme,
    ParticiplePrincipalPart, ParticipleTense, ParticipleVoice, PositionalParadigm,
    PresentPrincipalParts, Result, SynodalWord, VerbConjugation, VerbLexeme, VerbSystem,
};

use crate::{
    Inflector, Paradigm, PartOfSpeech,
    paradigm::{finite_cells, participle_cells, verb_cells},
};

use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct VerbSpec {
    pub(crate) lexeme: VerbLexeme,
    pub(crate) context: SpecContext,
}

impl VerbSpec {
    pub fn builder(
        lemma: impl Into<String>,
        aspect: Aspect,
        conjugation: VerbConjugation,
        source: SpecificationSource,
    ) -> Result<VerbSpecBuilder> {
        Ok(VerbSpecBuilder {
            spec: Self {
                lexeme: VerbLexeme {
                    lemma: SynodalWord::parse(lemma)?,
                    aspect,
                    conjugation,
                    present_stem: None,
                    present_first_singular: None,
                    present_third_plural: None,
                    future_stem: None,
                    future_first_singular: None,
                    future_third_plural: None,
                    imperfect_stem: None,
                    imperfect_formation: None,
                    aorist_stem: None,
                    aorist_formation: None,
                    imperative_stem: None,
                    imperative_formation: None,
                    l_participle_stem: None,
                    l_participle_masculine_singular_stem: None,
                    present_active_participle: None,
                    past_active_participle: None,
                    present_passive_participle: None,
                    past_passive_participle: None,
                    verbal_noun: None,
                },
                context: SpecContext::new(source),
            },
        })
    }

    #[must_use]
    pub fn lemma(&self) -> &str {
        self.lexeme.lemma.canonical()
    }

    pub fn form(&self, cell: GrammarCell) -> Result<FormSet> {
        self.form_with(Inflector::default(), cell)
    }

    pub fn form_with(&self, inflector: Inflector, cell: GrammarCell) -> Result<FormSet> {
        inflector.form_spec(&LexemeSpec::from(self.clone()), cell)
    }

    pub fn finite_paradigm(&self, tense: FiniteTense) -> Paradigm {
        self.finite_paradigm_with(Inflector::default(), tense)
    }

    pub fn finite_paradigm_with(&self, inflector: Inflector, tense: FiniteTense) -> Paradigm {
        let spec = LexemeSpec::from(self.clone());
        Paradigm::build_explicit(
            self.lemma().into(),
            PartOfSpeech::Verb,
            finite_cells(tense),
            |cell| inflector.form_spec(&spec, cell),
        )
    }

    #[must_use]
    pub fn system_paradigm(&self, system: VerbSystem) -> Paradigm {
        self.system_paradigm_with(Inflector::default(), system)
    }

    #[must_use]
    pub fn system_paradigm_with(&self, inflector: Inflector, system: VerbSystem) -> Paradigm {
        let spec = LexemeSpec::from(self.clone());
        Paradigm::build_explicit(
            self.lemma().into(),
            PartOfSpeech::Verb,
            verb_cells(system),
            |cell| inflector.form_spec(&spec, cell),
        )
    }

    #[must_use]
    pub fn all_system_paradigms(&self) -> Vec<(VerbSystem, Paradigm)> {
        VerbSystem::ALL
            .into_iter()
            .map(|system| (system, self.system_paradigm(system)))
            .collect()
    }

    /// Reports principal parts absent from this specification's productive
    /// background. Caller-specified exact overrides may still satisfy cells.
    #[must_use]
    pub fn missing_principal_parts(&self, system: VerbSystem) -> Vec<MetadataField> {
        self.lexeme.missing_principal_parts(system)
    }

    pub fn participle_paradigm(
        &self,
        tense: ParticipleTense,
        voice: ParticipleVoice,
        form: AdjectiveForm,
    ) -> Paradigm {
        self.participle_paradigm_with(Inflector::default(), tense, voice, form)
    }

    pub fn participle_paradigm_with(
        &self,
        inflector: Inflector,
        tense: ParticipleTense,
        voice: ParticipleVoice,
        form: AdjectiveForm,
    ) -> Paradigm {
        let spec = LexemeSpec::from(self.clone());
        Paradigm::build_explicit(
            self.lemma().into(),
            PartOfSpeech::Verb,
            participle_cells(tense, voice, form),
            |cell| inflector.form_spec(&spec, cell),
        )
    }

    pub fn validate(&self) -> Result<()> {
        self.context.validate()?;
        validate_context_cells(&self.context, |cell| {
            matches!(
                cell,
                GrammarCell::FiniteVerb(_)
                    | GrammarCell::Imperative(_)
                    | GrammarCell::Infinitive
                    | GrammarCell::LParticiple(_)
                    | GrammarCell::Participle(_)
                    | GrammarCell::Supine
                    | GrammarCell::VerbalNoun(_)
            )
        })?;
        validate_pair(
            self.lexeme.imperfect_stem.is_some(),
            self.lexeme.imperfect_formation.is_some(),
            "imperfect stem and formation",
        )?;
        validate_pair(
            self.lexeme.aorist_stem.is_some(),
            self.lexeme.aorist_formation.is_some(),
            "aorist stem and formation",
        )?;
        validate_pair(
            self.lexeme.imperative_stem.is_some(),
            self.lexeme.imperative_formation.is_some(),
            "imperative stem and formation",
        )?;
        let future_part_count = [
            self.lexeme.future_stem.is_some(),
            self.lexeme.future_first_singular.is_some(),
            self.lexeme.future_third_plural.is_some(),
        ]
        .into_iter()
        .filter(|present| *present)
        .count();
        if !matches!(future_part_count, 0 | 3) {
            return Err(Error::ContradictoryMetadata {
                reason: "future stem, first singular, and third plural must be supplied together"
                    .into(),
            });
        }
        if self.lexeme.l_participle_masculine_singular_stem.is_some()
            && self.lexeme.l_participle_stem.is_none()
        {
            return Err(Error::ContradictoryMetadata {
                reason:
                    "an l-participle masculine-singular stem requires the general l-participle stem"
                        .into(),
            });
        }
        if self.lexeme.aspect == Aspect::Perfective && self.lexeme.imperfect_formation.is_some() {
            return Err(Error::ContradictoryMetadata {
                reason: "a perfective specification cannot license a productive imperfect".into(),
            });
        }
        if self.lexeme.aspect == Aspect::Perfective
            && (self.lexeme.present_active_participle.is_some()
                || self.lexeme.present_passive_participle.is_some())
        {
            return Err(Error::ContradictoryMetadata {
                reason: "a perfective specification cannot license a productive present participle"
                    .into(),
            });
        }
        validate_participle(
            self.lexeme.present_active_participle.as_ref(),
            true,
            true,
            self.lexeme.conjugation,
        )?;
        validate_participle(
            self.lexeme.past_active_participle.as_ref(),
            true,
            false,
            self.lexeme.conjugation,
        )?;
        validate_participle(
            self.lexeme.present_passive_participle.as_ref(),
            false,
            true,
            self.lexeme.conjugation,
        )?;
        validate_participle(
            self.lexeme.past_passive_participle.as_ref(),
            false,
            false,
            self.lexeme.conjugation,
        )?;
        if let Some(principal_part) = &self.lexeme.verbal_noun {
            principal_part.validate()?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct VerbSpecBuilder {
    spec: VerbSpec,
}

macro_rules! word_setter {
    ($name:ident, $field:ident) => {
        pub fn $name(mut self, value: impl Into<String>) -> Result<Self> {
            self.spec.lexeme.$field = Some(SynodalWord::parse(value)?);
            Ok(self)
        }
    };
}

impl VerbSpecBuilder {
    word_setter!(present_stem, present_stem);
    word_setter!(present_first_singular, present_first_singular);
    word_setter!(present_third_plural, present_third_plural);
    word_setter!(future_stem, future_stem);
    word_setter!(future_first_singular, future_first_singular);
    word_setter!(future_third_plural, future_third_plural);
    word_setter!(l_participle_stem, l_participle_stem);
    word_setter!(
        l_participle_masculine_singular_stem,
        l_participle_masculine_singular_stem
    );

    /// Installs a complete, independently specified present series atomically.
    #[must_use]
    pub fn present_parts(mut self, parts: PresentPrincipalParts) -> Self {
        self.spec.lexeme.present_stem = Some(parts.stem);
        self.spec.lexeme.present_first_singular = Some(parts.first_singular);
        self.spec.lexeme.present_third_plural = Some(parts.third_plural);
        self
    }

    /// Parses and installs all three required present principal parts without
    /// deriving either lexical edge form from the medial stem.
    pub fn present_series(
        self,
        stem: impl Into<String>,
        first_singular: impl Into<String>,
        third_plural: impl Into<String>,
    ) -> Result<Self> {
        Ok(self.present_parts(PresentPrincipalParts::parse(
            stem,
            first_singular,
            third_plural,
        )?))
    }

    /// Installs a complete, independently suppletive simple-future series.
    /// Perfective verbs without this triple continue to reuse their present
    /// principal parts, as in the regular Alypy §84 pattern.
    #[must_use]
    pub fn future_parts(mut self, parts: PresentPrincipalParts) -> Self {
        self.spec.lexeme.future_stem = Some(parts.stem);
        self.spec.lexeme.future_first_singular = Some(parts.first_singular);
        self.spec.lexeme.future_third_plural = Some(parts.third_plural);
        self
    }

    pub fn future_series(
        self,
        stem: impl Into<String>,
        first_singular: impl Into<String>,
        third_plural: impl Into<String>,
    ) -> Result<Self> {
        Ok(self.future_parts(PresentPrincipalParts::parse(
            stem,
            first_singular,
            third_plural,
        )?))
    }

    pub fn imperfect(
        mut self,
        stem: impl Into<String>,
        formation: ImperfectFormation,
    ) -> Result<Self> {
        self.spec.lexeme.imperfect_stem = Some(SynodalWord::parse(stem)?);
        self.spec.lexeme.imperfect_formation = Some(formation);
        Ok(self)
    }

    pub fn aorist(mut self, stem: impl Into<String>, formation: AoristFormation) -> Result<Self> {
        self.spec.lexeme.aorist_stem = Some(SynodalWord::parse(stem)?);
        self.spec.lexeme.aorist_formation = Some(formation);
        Ok(self)
    }

    pub fn imperative(
        mut self,
        stem: impl Into<String>,
        formation: ImperativeFormation,
    ) -> Result<Self> {
        self.spec.lexeme.imperative_stem = Some(SynodalWord::parse(stem)?);
        self.spec.lexeme.imperative_formation = Some(formation);
        Ok(self)
    }

    pub fn present_active_participle(mut self, part: ParticiplePrincipalPart) -> Self {
        self.spec.lexeme.present_active_participle = Some(part);
        self
    }

    pub fn past_active_participle(mut self, part: ParticiplePrincipalPart) -> Self {
        self.spec.lexeme.past_active_participle = Some(part);
        self
    }

    pub fn present_passive_participle(mut self, part: ParticiplePrincipalPart) -> Self {
        self.spec.lexeme.present_passive_participle = Some(part);
        self
    }

    pub fn past_passive_participle(mut self, part: ParticiplePrincipalPart) -> Self {
        self.spec.lexeme.past_passive_participle = Some(part);
        self
    }

    /// Supplies an independently reviewed past-passive platform for the
    /// productive Alypy §27 `-їе` formation.
    pub fn verbal_noun_ie(mut self, platform: impl Into<String>) -> Result<Self> {
        self.spec.lexeme.verbal_noun =
            Some(synodal_church_slavonic_core::VerbalNounPrincipalPart::past_passive_ie(platform)?);
        Ok(self)
    }

    /// Supplies a complete lexical deverbal noun when §27 does not license
    /// selection of its suffix from the verb alone.
    pub fn lexical_verbal_noun(mut self, noun: NounLexeme) -> Result<Self> {
        self.spec.lexeme.verbal_noun =
            Some(synodal_church_slavonic_core::VerbalNounPrincipalPart::explicit_lexical(noun)?);
        Ok(self)
    }

    pub fn accent_paradigm(mut self, accent: AccentParadigm) -> Self {
        self.spec.context.accent = Some(accent);
        self
    }

    pub fn positional_paradigm(mut self, positional: PositionalParadigm) -> Self {
        self.spec.context.positional = Some(positional);
        self
    }

    pub fn defective_cell(mut self, cell: DefectiveCell) -> Self {
        self.spec.context.defective_cells.push(cell);
        self
    }

    pub fn build(self) -> Result<VerbSpec> {
        self.spec.validate()?;
        Ok(self.spec)
    }
}
