use synodal_church_slavonic_core::{
    AccentParadigm, AdjectiveClass, AdjectiveForm, AdjectiveLexeme, Comparison,
    ComparisonFormation, DeterminerDeclension, DeterminerLexeme, DeterminerNumberInventory, Error,
    FormSet, Gender, GrammarCell, NounDeclension, NounLexeme, NumeralDeclension, NumeralLexeme,
    NumeralNumberInventory, PositionalParadigm, PronounDeclension, PronounEnvironment,
    PronounFormSelection, PronounLexeme, PronounNumberInventory, PronounPostpositive,
    PronounPrefix, Result, ShortMasculineStemFormation, SynodalWord, validate_adjective_lexeme,
    validate_determiner_lexeme, validate_noun_lexeme, validate_numeral_lexeme,
    validate_pronoun_lexeme,
};

use crate::{
    Inflector, Paradigm, PartOfSpeech,
    paradigm::{adjective_cells, noun_cells, numeral_cells, pronoun_cells},
};

use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct NounSpec {
    pub(crate) lexeme: NounLexeme,
    pub(crate) context: SpecContext,
}

impl NounSpec {
    pub fn new(
        lemma: impl Into<String>,
        stem: impl Into<String>,
        gender: Gender,
        declension: NounDeclension,
        source: SpecificationSource,
    ) -> Result<Self> {
        let spec = Self {
            lexeme: NounLexeme {
                lemma: SynodalWord::parse(lemma)?,
                stem: SynodalWord::parse(stem)?,
                gender,
                declension,
                number_inventory: synodal_church_slavonic_core::NounNumberInventory::All,
                animacy_inventory: synodal_church_slavonic_core::NounAnimacyInventory::All,
            },
            context: SpecContext::new(source),
        };
        spec.validate()?;
        Ok(spec)
    }

    #[must_use]
    pub fn lemma(&self) -> &str {
        self.lexeme.lemma.canonical()
    }

    pub fn form(&self, cell: synodal_church_slavonic_core::NounCell) -> Result<FormSet> {
        self.form_with(Inflector::default(), cell)
    }

    pub fn form_with(
        &self,
        inflector: Inflector,
        cell: synodal_church_slavonic_core::NounCell,
    ) -> Result<FormSet> {
        inflector.form_spec(&LexemeSpec::from(self.clone()), GrammarCell::Noun(cell))
    }

    pub fn paradigm(&self, animacy: synodal_church_slavonic_core::Animacy) -> Paradigm {
        self.paradigm_with(Inflector::default(), animacy)
    }

    pub fn paradigm_with(
        &self,
        inflector: Inflector,
        animacy: synodal_church_slavonic_core::Animacy,
    ) -> Paradigm {
        let spec = LexemeSpec::from(self.clone());
        Paradigm::build_explicit(
            self.lemma().into(),
            PartOfSpeech::Noun,
            noun_cells(animacy),
            |cell| inflector.form_spec(&spec, cell),
        )
    }

    pub fn with_accent_paradigm(mut self, accent: AccentParadigm) -> Result<Self> {
        self.context.accent = Some(accent);
        self.validate()?;
        Ok(self)
    }

    pub fn with_positional_paradigm(mut self, positional: PositionalParadigm) -> Result<Self> {
        self.context.positional = Some(positional);
        self.validate()?;
        Ok(self)
    }

    /// Restricts this noun to the historically licensed number inventory.
    /// Complete paradigms retain requests outside that inventory as typed
    /// `HistoricallyInvalidCell` outcomes.
    pub fn with_number_inventory(
        mut self,
        inventory: synodal_church_slavonic_core::NounNumberInventory,
    ) -> Result<Self> {
        self.lexeme.number_inventory = inventory;
        self.validate()?;
        Ok(self)
    }

    /// Restricts this noun to its independently reviewed lexical animacy.
    /// Incompatible requests remain visible as typed historical-cell errors.
    pub fn with_animacy_inventory(
        mut self,
        inventory: synodal_church_slavonic_core::NounAnimacyInventory,
    ) -> Result<Self> {
        self.lexeme.animacy_inventory = inventory;
        self.validate()?;
        Ok(self)
    }

    pub fn with_defective_cell(mut self, cell: DefectiveCell) -> Result<Self> {
        self.context.defective_cells.push(cell);
        self.validate()?;
        Ok(self)
    }

    pub fn validate(&self) -> Result<()> {
        self.context.validate()?;
        validate_context_cells(&self.context, |cell| matches!(cell, GrammarCell::Noun(_)))?;
        validate_noun_lexeme(&self.lexeme)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct AdjectiveSpec {
    pub(crate) lexeme: AdjectiveLexeme,
    pub(crate) context: SpecContext,
}

impl AdjectiveSpec {
    pub fn new(
        lemma: impl Into<String>,
        stem: impl Into<String>,
        class: AdjectiveClass,
        source: SpecificationSource,
    ) -> Result<Self> {
        Ok(Self {
            lexeme: AdjectiveLexeme {
                lemma: SynodalWord::parse(lemma)?,
                stem: SynodalWord::parse(stem)?,
                class,
                short_masculine_stem: None,
                short_masculine_formation: None,
                comparative_stem: None,
                comparison_formation: None,
            },
            context: SpecContext::new(source),
        })
    }

    pub fn comparison(
        mut self,
        stem: impl Into<String>,
        formation: ComparisonFormation,
    ) -> Result<Self> {
        self.lexeme.comparative_stem = Some(SynodalWord::parse(stem)?);
        self.lexeme.comparison_formation = Some(formation);
        self.validate()?;
        Ok(self)
    }

    /// Supplies the independently reviewed positive stem used before the
    /// short masculine citation ending and the source-defined relation to the
    /// ordinary stem.
    pub fn short_masculine_stem(
        mut self,
        stem: impl Into<String>,
        formation: ShortMasculineStemFormation,
    ) -> Result<Self> {
        self.lexeme.short_masculine_stem = Some(SynodalWord::parse(stem)?);
        self.lexeme.short_masculine_formation = Some(formation);
        self.validate()?;
        Ok(self)
    }

    #[must_use]
    pub fn lemma(&self) -> &str {
        self.lexeme.lemma.canonical()
    }

    pub fn form(&self, cell: synodal_church_slavonic_core::AdjectiveCell) -> Result<FormSet> {
        self.form_with(Inflector::default(), cell)
    }

    pub fn form_with(
        &self,
        inflector: Inflector,
        cell: synodal_church_slavonic_core::AdjectiveCell,
    ) -> Result<FormSet> {
        inflector.form_spec(
            &LexemeSpec::from(self.clone()),
            GrammarCell::Adjective(cell),
        )
    }

    pub fn paradigm(&self, form: AdjectiveForm, comparison: Comparison) -> Paradigm {
        self.paradigm_with(Inflector::default(), form, comparison)
    }

    pub fn paradigm_with(
        &self,
        inflector: Inflector,
        form: AdjectiveForm,
        comparison: Comparison,
    ) -> Paradigm {
        let spec = LexemeSpec::from(self.clone());
        let cells = adjective_cells(form).into_iter().map(move |mut cell| {
            cell.comparison = comparison;
            GrammarCell::Adjective(cell)
        });
        Paradigm::build_explicit(
            self.lemma().into(),
            PartOfSpeech::Adjective,
            cells,
            |cell| inflector.form_spec(&spec, cell),
        )
    }

    pub fn with_accent_paradigm(mut self, accent: AccentParadigm) -> Result<Self> {
        self.context.accent = Some(accent);
        self.validate()?;
        Ok(self)
    }

    pub fn with_positional_paradigm(mut self, positional: PositionalParadigm) -> Result<Self> {
        self.context.positional = Some(positional);
        self.validate()?;
        Ok(self)
    }

    pub fn with_defective_cell(mut self, cell: DefectiveCell) -> Result<Self> {
        self.context.defective_cells.push(cell);
        self.validate()?;
        Ok(self)
    }

    pub fn validate(&self) -> Result<()> {
        self.context.validate()?;
        validate_context_cells(&self.context, |cell| {
            matches!(cell, GrammarCell::Adjective(_))
        })?;
        validate_adjective_lexeme(&self.lexeme)?;
        if self.lexeme.comparative_stem.is_some() != self.lexeme.comparison_formation.is_some() {
            return Err(Error::ContradictoryMetadata {
                reason: "comparison stem and typed comparison formation must be supplied together"
                    .into(),
            });
        }
        if let (Some(stem), Some(formation)) = (
            &self.lexeme.comparative_stem,
            self.lexeme.comparison_formation,
        ) {
            let valid = match formation {
                ComparisonFormation::AncientHard => {
                    stem.canonical().ends_with('ш') && !stem.canonical().ends_with("ьш")
                }
                ComparisonFormation::AncientSoft => stem.canonical().ends_with("ьш"),
                ComparisonFormation::LaterYat => stem.canonical().ends_with("ѣйш"),
                ComparisonFormation::LaterAi => stem.canonical().ends_with("айш"),
            };
            if !valid {
                return Err(Error::ContradictoryMetadata {
                    reason: format!(
                        "comparison stem {:?} does not match formation {formation:?}",
                        stem.canonical()
                    ),
                });
            }
        }
        Ok(())
    }
}

/// Caller-supplied typed metadata for a Synodal determiner. Determiner form
/// and number restrictions remain lexical facts rather than adjective
/// defaults inferred from the requested agreement cell.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct DeterminerSpec {
    pub(crate) lexeme: DeterminerLexeme,
    pub(crate) context: SpecContext,
}

impl DeterminerSpec {
    pub fn new(
        lemma: impl Into<String>,
        stem: impl Into<String>,
        declension: DeterminerDeclension,
        source: SpecificationSource,
    ) -> Result<Self> {
        let spec = Self {
            lexeme: DeterminerLexeme::new(
                SynodalWord::parse(lemma)?,
                SynodalWord::parse(stem)?,
                declension,
            ),
            context: SpecContext::new(source),
        };
        spec.validate()?;
        Ok(spec)
    }

    #[must_use]
    pub fn lemma(&self) -> &str {
        self.lexeme.lemma.canonical()
    }

    pub fn with_number_inventory(mut self, inventory: DeterminerNumberInventory) -> Result<Self> {
        self.lexeme.number_inventory = inventory;
        self.validate()?;
        Ok(self)
    }

    pub fn with_accent_paradigm(mut self, accent: AccentParadigm) -> Result<Self> {
        self.context.accent = Some(accent);
        self.validate()?;
        Ok(self)
    }

    pub fn with_positional_paradigm(mut self, positional: PositionalParadigm) -> Result<Self> {
        self.context.positional = Some(positional);
        self.validate()?;
        Ok(self)
    }

    pub fn with_defective_cell(mut self, cell: DefectiveCell) -> Result<Self> {
        self.context.defective_cells.push(cell);
        self.validate()?;
        Ok(self)
    }

    pub fn form(&self, cell: synodal_church_slavonic_core::AdjectiveCell) -> Result<FormSet> {
        self.form_with(Inflector::default(), cell)
    }

    pub fn form_with(
        &self,
        inflector: Inflector,
        cell: synodal_church_slavonic_core::AdjectiveCell,
    ) -> Result<FormSet> {
        inflector.form_spec(
            &LexemeSpec::from(self.clone()),
            GrammarCell::Determiner(cell),
        )
    }

    #[must_use]
    pub fn paradigm(&self, form: AdjectiveForm) -> Paradigm {
        self.paradigm_with(Inflector::default(), form)
    }

    #[must_use]
    pub fn paradigm_with(&self, inflector: Inflector, form: AdjectiveForm) -> Paradigm {
        let spec = LexemeSpec::from(self.clone());
        let cells = adjective_cells(form)
            .into_iter()
            .map(GrammarCell::Determiner);
        Paradigm::build_explicit(
            self.lemma().into(),
            PartOfSpeech::Determiner,
            cells,
            |cell| inflector.form_spec(&spec, cell),
        )
    }

    pub fn validate(&self) -> Result<()> {
        self.context.validate()?;
        validate_context_cells(&self.context, |cell| {
            matches!(cell, GrammarCell::Determiner(_))
        })?;
        validate_determiner_lexeme(&self.lexeme)
    }
}

/// Caller-supplied typed metadata for a Synodal numeral word. Compound,
/// distributive, and periphrastic numeral constructions use the separate
/// structured composition API because their realizations may contain several
/// independently inflected words.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct NumeralSpec {
    pub(crate) lexeme: NumeralLexeme,
    pub(crate) context: SpecContext,
}

impl NumeralSpec {
    pub fn new(
        lemma: impl Into<String>,
        stem: impl Into<String>,
        declension: NumeralDeclension,
        source: SpecificationSource,
    ) -> Result<Self> {
        let spec = Self {
            lexeme: NumeralLexeme::new(
                SynodalWord::parse(lemma)?,
                SynodalWord::parse(stem)?,
                declension,
            ),
            context: SpecContext::new(source),
        };
        spec.validate()?;
        Ok(spec)
    }

    #[must_use]
    pub fn lemma(&self) -> &str {
        self.lexeme.lemma.canonical()
    }

    pub fn with_number_inventory(mut self, inventory: NumeralNumberInventory) -> Result<Self> {
        self.lexeme.number_inventory = inventory;
        self.validate()?;
        Ok(self)
    }

    pub fn with_accent_paradigm(mut self, accent: AccentParadigm) -> Result<Self> {
        self.context.accent = Some(accent);
        self.validate()?;
        Ok(self)
    }

    pub fn with_positional_paradigm(mut self, positional: PositionalParadigm) -> Result<Self> {
        self.context.positional = Some(positional);
        self.validate()?;
        Ok(self)
    }

    pub fn with_defective_cell(mut self, cell: DefectiveCell) -> Result<Self> {
        self.context.defective_cells.push(cell);
        self.validate()?;
        Ok(self)
    }

    pub fn form(&self, cell: synodal_church_slavonic_core::NumeralCell) -> Result<FormSet> {
        self.form_with(Inflector::default(), cell)
    }

    pub fn form_with(
        &self,
        inflector: Inflector,
        cell: synodal_church_slavonic_core::NumeralCell,
    ) -> Result<FormSet> {
        inflector.form_spec(&LexemeSpec::from(self.clone()), GrammarCell::Numeral(cell))
    }

    #[must_use]
    pub fn paradigm(&self) -> Paradigm {
        self.paradigm_with(Inflector::default())
    }

    #[must_use]
    pub fn paradigm_with(&self, inflector: Inflector) -> Paradigm {
        let spec = LexemeSpec::from(self.clone());
        Paradigm::build_explicit(
            self.lemma().into(),
            PartOfSpeech::Numeral,
            numeral_cells(self.lexeme.declension.kind()),
            |cell| inflector.form_spec(&spec, cell),
        )
    }

    pub fn validate(&self) -> Result<()> {
        self.context.validate()?;
        validate_context_cells(&self.context, |cell| {
            matches!(cell, GrammarCell::Numeral(_))
        })?;
        validate_numeral_lexeme(&self.lexeme)
    }
}

/// Caller-supplied typed metadata for a Synodal pronoun. The specification
/// keeps lexical person/gender profiles, clitic selection, and conditioned
/// third-person allomorphy separate from the requested case cell.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct PronounSpec {
    pub(crate) lexeme: PronounLexeme,
    pub(crate) context: SpecContext,
}

impl PronounSpec {
    pub fn closed(
        lemma: impl Into<String>,
        declension: PronounDeclension,
        source: SpecificationSource,
    ) -> Result<Self> {
        let spec = Self {
            lexeme: PronounLexeme::closed(SynodalWord::parse(lemma)?, declension),
            context: SpecContext::new(source),
        };
        spec.validate()?;
        Ok(spec)
    }

    pub fn regular(
        lemma: impl Into<String>,
        stem: impl Into<String>,
        declension: PronounDeclension,
        source: SpecificationSource,
    ) -> Result<Self> {
        let spec = Self {
            lexeme: PronounLexeme::regular(
                SynodalWord::parse(lemma)?,
                SynodalWord::parse(stem)?,
                declension,
            ),
            context: SpecContext::new(source),
        };
        spec.validate()?;
        Ok(spec)
    }

    #[must_use]
    pub fn lemma(&self) -> &str {
        self.lexeme.lemma.canonical()
    }

    pub fn with_selection(mut self, selection: PronounFormSelection) -> Result<Self> {
        self.lexeme.selection = selection;
        self.validate()?;
        Ok(self)
    }

    pub fn with_number_inventory(mut self, inventory: PronounNumberInventory) -> Result<Self> {
        self.lexeme.number_inventory = inventory;
        self.validate()?;
        Ok(self)
    }

    pub fn with_environment(mut self, environment: PronounEnvironment) -> Result<Self> {
        self.lexeme.environment = environment;
        self.validate()?;
        Ok(self)
    }

    pub fn with_prefix(mut self, prefix: PronounPrefix) -> Result<Self> {
        self.lexeme.prefix = Some(prefix);
        self.validate()?;
        Ok(self)
    }

    pub fn with_postpositive(mut self, postpositive: PronounPostpositive) -> Result<Self> {
        self.lexeme.postpositive = Some(postpositive);
        self.validate()?;
        Ok(self)
    }

    pub fn with_accent_paradigm(mut self, accent: AccentParadigm) -> Result<Self> {
        self.context.accent = Some(accent);
        self.validate()?;
        Ok(self)
    }

    pub fn with_positional_paradigm(mut self, positional: PositionalParadigm) -> Result<Self> {
        self.context.positional = Some(positional);
        self.validate()?;
        Ok(self)
    }

    pub fn with_defective_cell(mut self, cell: DefectiveCell) -> Result<Self> {
        self.context.defective_cells.push(cell);
        self.validate()?;
        Ok(self)
    }

    pub fn form(&self, cell: synodal_church_slavonic_core::PronounCell) -> Result<FormSet> {
        self.form_with(Inflector::default(), cell)
    }

    pub fn form_with(
        &self,
        inflector: Inflector,
        cell: synodal_church_slavonic_core::PronounCell,
    ) -> Result<FormSet> {
        inflector.form_spec(&LexemeSpec::from(self.clone()), GrammarCell::Pronoun(cell))
    }

    #[must_use]
    pub fn paradigm(&self) -> Paradigm {
        self.paradigm_with(Inflector::default())
    }

    #[must_use]
    pub fn paradigm_with(&self, inflector: Inflector) -> Paradigm {
        let spec = LexemeSpec::from(self.clone());
        let profiles = match self.lexeme.declension {
            PronounDeclension::PersonalFirst => vec![(None, Some(crate::Person::First))],
            PronounDeclension::PersonalSecond => vec![(None, Some(crate::Person::Second))],
            PronounDeclension::Reflexive
            | PronounDeclension::InterrogativeWho
            | PronounDeclension::InterrogativeWhat => vec![(None, None)],
            PronounDeclension::ThirdPerson => {
                let person = if self.lexeme.postpositive == Some(PronounPostpositive::Zhe) {
                    None
                } else {
                    Some(crate::Person::Third)
                };
                Gender::ALL
                    .into_iter()
                    .map(|gender| (Some(gender), person))
                    .collect()
            }
            PronounDeclension::ThirdPersonAndDemonstrative => Gender::ALL
                .into_iter()
                .flat_map(|gender| {
                    [None, Some(crate::Person::Third)]
                        .into_iter()
                        .map(move |person| (Some(gender), person))
                })
                .collect(),
            PronounDeclension::Soft
            | PronounDeclension::SoftIAlternating
            | PronounDeclension::Hard
            | PronounDeclension::MixedPossessive
            | PronounDeclension::ShortHard
            | PronounDeclension::ShortOvMixed
            | PronounDeclension::ShortVelar
            | PronounDeclension::QuantityVelar
            | PronounDeclension::FullHard
            | PronounDeclension::FullSoft
            | PronounDeclension::FullVelar
            | PronounDeclension::ProximalSei
            | PronounDeclension::InterrogativeKii => Gender::ALL
                .into_iter()
                .map(|gender| (Some(gender), None))
                .collect(),
        };
        Paradigm::build_explicit(
            self.lemma().into(),
            PartOfSpeech::Pronoun,
            pronoun_cells(&profiles),
            |cell| inflector.form_spec(&spec, cell),
        )
    }

    pub fn validate(&self) -> Result<()> {
        self.context.validate()?;
        validate_context_cells(&self.context, |cell| {
            matches!(cell, GrammarCell::Pronoun(_))
        })?;
        validate_pronoun_lexeme(&self.lexeme)
    }
}
