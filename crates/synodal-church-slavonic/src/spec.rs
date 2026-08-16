use std::collections::BTreeSet;

use synodal_church_slavonic_core::{
    AccentMark, AccentParadigm, AccentScope, ActiveParticipleShortFormation, AdjectiveClass,
    AdjectiveForm, AdjectiveLexeme, AoristFormation, Aspect, AuthorityRole, Comparison,
    ComparisonFormation, DeterminerDeclension, DeterminerLexeme, DeterminerNumberInventory,
    EpistemicRole, Error, Evidence, EvidenceId, EvidenceKind, FiniteTense, FormSet, Gender,
    GrammarCell, ImperativeFormation, ImperfectFormation, MetadataField, NounDeclension,
    NounLexeme, NumeralDeclension, NumeralLexeme, NumeralNumberInventory, OrthographyProfile,
    ParticiplePrincipalPart, ParticipleTense, ParticipleVoice, PresentPrincipalParts,
    PronounDeclension, PronounEnvironment, PronounFormSelection, PronounLexeme,
    PronounNumberInventory, PronounPostpositive, PronounPrefix, Recension, RenderedText, Result,
    ShortMasculineStemFormation, SourceId, SynodalWord, VerbConjugation, VerbLexeme, VerbSystem,
    validate_adjective_lexeme, validate_determiner_lexeme, validate_noun_lexeme,
    validate_numeral_lexeme, validate_pronoun_lexeme,
};

use crate::{
    Inflector, Paradigm, PartOfSpeech,
    paradigm::{
        adjective_cells, finite_cells, noun_cells, numeral_cells, participle_cells, pronoun_cells,
        verb_cells,
    },
};

/// Provenance attached to caller-supplied lexical metadata. It identifies a
/// prediction input; it never turns the resulting form into an attestation.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct SpecificationSource {
    evidence_id: EvidenceId,
    source_id: SourceId,
    citation: String,
}

impl SpecificationSource {
    pub fn new(
        evidence_id: impl Into<EvidenceId>,
        source_id: impl Into<SourceId>,
        citation: impl Into<String>,
    ) -> Result<Self> {
        let evidence_id = evidence_id.into();
        let source_id = source_id.into();
        let citation = citation.into();
        if evidence_id.as_str().trim().is_empty()
            || source_id.as_str().trim().is_empty()
            || citation.trim().is_empty()
        {
            return Err(Error::ContradictoryMetadata {
                reason:
                    "caller-supplied metadata requires nonempty evidence, source, and citation IDs"
                        .into(),
            });
        }
        Ok(Self {
            evidence_id,
            source_id,
            citation,
        })
    }

    #[must_use]
    pub fn evidence_id(&self) -> &EvidenceId {
        &self.evidence_id
    }

    #[must_use]
    pub fn source_id(&self) -> &SourceId {
        &self.source_id
    }

    #[must_use]
    pub fn citation(&self) -> &str {
        &self.citation
    }

    pub(crate) fn evidence(&self, kind: EvidenceKind) -> Evidence {
        let authority_roles = if kind == EvidenceKind::AccentParadigm {
            vec![AuthorityRole::Accentual, AuthorityRole::Orthographic]
        } else {
            vec![AuthorityRole::Lexical, AuthorityRole::Morphological]
        };
        Evidence {
            id: self.evidence_id.clone(),
            source: self.source_id.clone(),
            source_recension: Recension::SynodalRussian,
            kind,
            authority_roles,
            epistemic_role: EpistemicRole::CallerSuppliedMetadata,
            citation: self.citation.clone(),
            note: Some("caller-supplied Synodal lexical metadata".into()),
        }
    }

    /// Builds an explicitly sourced fixed-stem accent paradigm suitable for a
    /// caller-supplied specification.
    #[must_use]
    pub fn fixed_stem_accent(
        &self,
        paradigm_id: impl Into<String>,
        scope: AccentScope,
        vowel_from_start: u8,
        mark: AccentMark,
    ) -> AccentParadigm {
        AccentParadigm::fixed_stem(
            paradigm_id,
            scope,
            vowel_from_start,
            mark,
            self.evidence(EvidenceKind::AccentParadigm),
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum DefectKind {
    HistoricallyAbsent,
    EvidenceIncomplete,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct DefectiveCell {
    pub cell: GrammarCell,
    pub kind: DefectKind,
    pub field: synodal_church_slavonic_core::MetadataField,
    pub reason: String,
}

impl DefectiveCell {
    #[must_use]
    pub fn historically_absent(cell: GrammarCell, reason: impl Into<String>) -> Self {
        Self {
            cell,
            kind: DefectKind::HistoricallyAbsent,
            field: synodal_church_slavonic_core::MetadataField::IrregularOverride,
            reason: reason.into(),
        }
    }

    #[must_use]
    pub fn evidence_incomplete(
        cell: GrammarCell,
        field: synodal_church_slavonic_core::MetadataField,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            cell,
            kind: DefectKind::EvidenceIncomplete,
            field,
            reason: reason.into(),
        }
    }
}

/// A caller-specified irregular form. `liturgical` is an explicit lexical
/// accent/printing override and therefore precedes a reusable accent paradigm.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct SpecifiedForm {
    pub cell: GrammarCell,
    pub expanded: SynodalWord,
    pub liturgical: Option<RenderedText>,
    pub source: SpecificationSource,
}

impl SpecifiedForm {
    pub fn new(
        cell: GrammarCell,
        expanded: impl Into<String>,
        liturgical: Option<impl Into<String>>,
        source: SpecificationSource,
    ) -> Result<Self> {
        Ok(Self {
            cell,
            expanded: SynodalWord::parse(expanded)?,
            liturgical: liturgical
                .map(|value| RenderedText::parse(value.into()))
                .transpose()?,
            source,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub(crate) struct SpecContext {
    pub source: SpecificationSource,
    pub accent: Option<AccentParadigm>,
    pub irregular_forms: Vec<SpecifiedForm>,
    pub defective_cells: Vec<DefectiveCell>,
}

impl SpecContext {
    fn new(source: SpecificationSource) -> Self {
        Self {
            source,
            accent: None,
            irregular_forms: vec![],
            defective_cells: vec![],
        }
    }

    fn validate(&self) -> Result<()> {
        if let Some(accent) = &self.accent {
            accent.validate()?;
            if accent.evidence.source_recension != Recension::SynodalRussian {
                return Err(Error::ContradictoryMetadata {
                    reason:
                        "an explicit Synodal specification cannot use a non-Synodal accent paradigm"
                            .into(),
                });
            }
        }
        let mut irregular = BTreeSet::new();
        let mut irregular_cells = BTreeSet::new();
        for form in &self.irregular_forms {
            irregular_cells.insert(form.cell);
            if !irregular.insert((form.cell, &form.expanded, &form.liturgical)) {
                return Err(Error::ContradictoryMetadata {
                    reason: "an explicit specification contains a duplicate irregular form".into(),
                });
            }
        }
        let mut defective = BTreeSet::new();
        for cell in &self.defective_cells {
            if cell.reason.trim().is_empty() {
                return Err(Error::ContradictoryMetadata {
                    reason: "a defective cell must include a nonempty diagnostic reason".into(),
                });
            }
            if !defective.insert(cell.cell) || irregular_cells.contains(&cell.cell) {
                return Err(Error::ContradictoryMetadata {
                    reason: "a cell cannot be both irregular and defective, or listed twice".into(),
                });
            }
        }
        Ok(())
    }
}

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

    pub fn with_irregular_form(mut self, form: SpecifiedForm) -> Result<Self> {
        self.context.irregular_forms.push(form);
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

    pub fn with_irregular_form(mut self, form: SpecifiedForm) -> Result<Self> {
        self.context.irregular_forms.push(form);
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

    pub fn with_irregular_form(mut self, form: SpecifiedForm) -> Result<Self> {
        self.context.irregular_forms.push(form);
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

    pub fn with_irregular_form(mut self, form: SpecifiedForm) -> Result<Self> {
        self.context.irregular_forms.push(form);
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

    pub fn with_irregular_form(mut self, form: SpecifiedForm) -> Result<Self> {
        self.context.irregular_forms.push(form);
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

    pub fn irregular_form(mut self, form: SpecifiedForm) -> Self {
        self.spec.context.irregular_forms.push(form);
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
        profile != OrthographyProfile::SynodalLiturgical || self.context().accent.is_some()
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

fn validate_pair(left: bool, right: bool, label: &str) -> Result<()> {
    if left != right {
        return Err(Error::ContradictoryMetadata {
            reason: format!("{label} must be supplied together"),
        });
    }
    Ok(())
}

fn validate_context_cells(
    context: &SpecContext,
    accepts: impl Fn(GrammarCell) -> bool,
) -> Result<()> {
    if context
        .irregular_forms
        .iter()
        .any(|form| !accepts(form.cell))
        || context
            .defective_cells
            .iter()
            .any(|cell| !accepts(cell.cell))
    {
        return Err(Error::ContradictoryMetadata {
            reason: "an irregular or defective cell belongs to a different part of speech".into(),
        });
    }
    Ok(())
}

fn validate_participle(
    part: Option<&ParticiplePrincipalPart>,
    active: bool,
    present: bool,
    conjugation: VerbConjugation,
) -> Result<()> {
    let Some(part) = part else {
        return Ok(());
    };
    if !active && part.short_formation.is_some() {
        return Err(Error::ContradictoryMetadata {
            reason: "passive short participles must not use an active citation-edge formation"
                .into(),
        });
    }
    if active && part.short_stem.is_some() != part.short_formation.is_some() {
        return Err(Error::ContradictoryMetadata {
            reason: "an active short-participle stem requires its typed citation-edge formation"
                .into(),
        });
    }
    if let Some(formation) = part.short_formation {
        let formation_is_present = matches!(
            formation,
            ActiveParticipleShortFormation::PresentFirstUnpalatalized
                | ActiveParticipleShortFormation::PresentFirstPalatalized
                | ActiveParticipleShortFormation::PresentSecond
                | ActiveParticipleShortFormation::PresentAfterSibilant
        );
        if formation_is_present != present {
            return Err(Error::ContradictoryMetadata {
                reason: "short-participle formation does not match participle tense".into(),
            });
        }
        let conjugation_matches = match formation {
            ActiveParticipleShortFormation::PresentFirstUnpalatalized => {
                conjugation == VerbConjugation::FirstUnpalatalized
            }
            ActiveParticipleShortFormation::PresentFirstPalatalized => {
                conjugation == VerbConjugation::FirstPalatalized
            }
            ActiveParticipleShortFormation::PresentSecond
            | ActiveParticipleShortFormation::PresentAfterSibilant => {
                conjugation == VerbConjugation::Second
            }
            ActiveParticipleShortFormation::PastConsonant
            | ActiveParticipleShortFormation::PastVowel
            | ActiveParticipleShortFormation::PastIotated => true,
        };
        if !conjugation_matches {
            return Err(Error::ContradictoryMetadata {
                reason: "present participle formation contradicts the supplied conjugation class"
                    .into(),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ParadigmStatus;
    use synodal_church_slavonic_core::{
        AccentMark, AccentPlacement, AccentRule, AccentScope, AdjectiveCell, Animacy, Case,
        FormSource, MetadataField, NounCell, Number, NumeralCell, NumeralKind, ParticipleCell,
        Person,
    };

    fn source() -> SpecificationSource {
        SpecificationSource::new(
            "caller-lexicon-entry",
            "caller-reviewed-lexicon",
            "caller lexicon, entry 1",
        )
        .expect("source")
    }

    fn mudr_accent() -> AccentParadigm {
        AccentParadigm {
            id: "synodal-accent:mudr-fixed-stem".into(),
            accent_rules: vec![AccentRule {
                scope: AccentScope::Adjective {
                    form: AdjectiveForm::Long,
                    comparison: Comparison::Positive,
                    numbers: vec![Number::Singular],
                },
                placement: AccentPlacement::StemVowelFromStart(0),
                mark: AccentMark::Acute,
            }],
            breathing_rules: vec![],
            evidence: Evidence {
                id: EvidenceId::from("alypy-57-mudryi"),
                source: SourceId::from("alypy-gamanovich-grammar-web-2023"),
                source_recension: Recension::SynodalRussian,
                kind: EvidenceKind::AccentParadigm,
                authority_roles: vec![AuthorityRole::Accentual, AuthorityRole::Orthographic],
                epistemic_role: EpistemicRole::SynodalNormativeAuthority,
                citation: "Alypy (Gamanovich), §57, мꙋ́дръ adjective paradigm".into(),
                note: None,
            },
        }
    }

    #[test]
    fn unregistered_noun_uses_typed_metadata_without_lookup() {
        let spec = NounSpec::new(
            "псалтирникъ",
            "псалтирник",
            Gender::Masculine,
            NounDeclension::FirstHardMasculine,
            source(),
        )
        .expect("valid spec");
        assert!(Inflector::default().resolve("псалтирникъ").is_err());
        let forms = spec
            .form(NounCell {
                case: Case::Dative,
                number: Number::Plural,
                animacy: Animacy::Animate,
            })
            .expect("productive form");
        assert_eq!(forms.primary_text(), "псалтирникомъ");
        assert!(matches!(
            forms.primary().source,
            FormSource::CallerSpecifiedPrediction { .. }
        ));
        assert_eq!(spec.paradigm(Animacy::Animate).iter().count(), 21);
    }

    #[test]
    fn unregistered_numeral_uses_the_same_typed_kernel_as_registry_numerals() {
        let spec = NumeralSpec::new(
            "девѧть",
            "девѧт",
            NumeralDeclension::CardinalIStem,
            source(),
        )
        .expect("valid numeral spec");
        let genitive = spec
            .form(NumeralCell {
                kind: NumeralKind::Cardinal,
                case: Case::Genitive,
                number: Number::Singular,
                gender: None,
                animacy: Animacy::Inanimate,
            })
            .expect("productive numeral cell");
        assert_eq!(genitive.primary_text(), "девѧти");
        assert!(matches!(
            genitive.primary().source,
            FormSource::CallerSpecifiedPrediction { .. }
        ));
        assert!(matches!(
            spec.form(NumeralCell {
                kind: NumeralKind::Cardinal,
                case: Case::Accusative,
                number: Number::Plural,
                gender: None,
                animacy: Animacy::Inanimate,
            }),
            Err(Error::HistoricallyInvalidCell { .. })
        ));
    }

    #[test]
    fn explicit_short_comparison_and_active_participle_close_productive_gaps() {
        let adjective = AdjectiveSpec::new("мꙋдръ", "мꙋдр", AdjectiveClass::Hard, source())
            .expect("adjective")
            .comparison("мꙋдрѣйш", ComparisonFormation::LaterYat)
            .expect("comparison metadata");
        let comparison = adjective
            .form(AdjectiveCell {
                case: Case::Nominative,
                number: Number::Singular,
                gender: Gender::Masculine,
                animacy: Animacy::Inanimate,
                form: AdjectiveForm::Short,
                comparison: Comparison::Comparative,
            })
            .expect("short comparison");
        assert_eq!(comparison.primary_text(), "мꙋдрѣй");

        let short_superlative = adjective
            .form(AdjectiveCell {
                case: Case::Nominative,
                number: Number::Singular,
                gender: Gender::Masculine,
                animacy: Animacy::Inanimate,
                form: AdjectiveForm::Short,
                comparison: Comparison::Superlative,
            })
            .expect("predicate short superlative");
        assert_eq!(
            short_superlative.texts().collect::<Vec<_>>(),
            ["мꙋдрѣйшъ", "мꙋдрѣй"]
        );
        let short_superlative_paradigm =
            adjective.paradigm(AdjectiveForm::Short, Comparison::Superlative);
        assert_eq!(short_superlative_paradigm.successes().count(), 9);
        assert_eq!(
            short_superlative_paradigm
                .with_status(ParadigmStatus::HistoricallyInvalid)
                .count(),
            63
        );

        let present_part = ParticiplePrincipalPart {
            short_stem: Some(SynodalWord::parse("несꙋщ").expect("stem")),
            short_formation: Some(ActiveParticipleShortFormation::PresentFirstUnpalatalized),
            long_stem: Some(SynodalWord::parse("несꙋщ").expect("stem")),
            class: AdjectiveClass::Hard,
        };
        let verb = VerbSpec::builder(
            "нести",
            Aspect::Imperfective,
            VerbConjugation::FirstUnpalatalized,
            source(),
        )
        .expect("builder")
        .present_stem("нес")
        .expect("stem")
        .present_first_singular("несꙋ")
        .expect("edge")
        .present_third_plural("несꙋтъ")
        .expect("edge")
        .present_active_participle(present_part)
        .build()
        .expect("verb");
        let finite = verb
            .form(GrammarCell::FiniteVerb(
                synodal_church_slavonic_core::FiniteVerbCell {
                    tense: FiniteTense::Present,
                    person: Person::First,
                    number: Number::Singular,
                },
            ))
            .expect("present");
        assert_eq!(finite.primary_text(), "несꙋ");
        let participle = verb
            .form(GrammarCell::Participle(ParticipleCell {
                tense: ParticipleTense::Present,
                voice: ParticipleVoice::Active,
                agreement: AdjectiveCell {
                    case: Case::Nominative,
                    number: Number::Singular,
                    gender: Gender::Masculine,
                    animacy: Animacy::Inanimate,
                    form: AdjectiveForm::Short,
                    comparison: Comparison::Positive,
                },
            }))
            .expect("short active participle");
        assert_eq!(participle.texts().collect::<Vec<_>>(), ["несый", "несꙋщь"]);
    }

    #[test]
    fn explicit_pronoun_specs_preserve_profiles_clitics_and_context() {
        let possessive = PronounSpec::regular("твой", "тво", PronounDeclension::Soft, source())
            .expect("regular soft pronoun");
        let form = possessive
            .form(synodal_church_slavonic_core::PronounCell {
                case: Case::Genitive,
                number: Number::Singular,
                gender: Some(Gender::Feminine),
                person: None,
                animacy: Animacy::Inanimate,
            })
            .expect("soft-pronoun genitive");
        assert_eq!(form.primary_text(), "твоеѧ");
        assert!(matches!(
            &form.primary().source,
            FormSource::CallerSpecifiedPrediction { .. }
        ));
        let paradigm = possessive.paradigm();
        assert_eq!(paradigm.successes().count(), 108);
        assert_eq!(
            paradigm
                .with_status(ParadigmStatus::HistoricallyInvalid)
                .count(),
            18
        );

        let clitic = PronounSpec::closed("азъ", PronounDeclension::PersonalFirst, source())
            .expect("first-person specification")
            .with_selection(PronounFormSelection::Enclitic)
            .expect("clitic selection");
        assert_eq!(
            clitic
                .form(synodal_church_slavonic_core::PronounCell {
                    case: Case::Dative,
                    number: Number::Singular,
                    gender: None,
                    person: Some(Person::First),
                    animacy: Animacy::Inanimate,
                })
                .expect("first-person enclitic")
                .primary_text(),
            "ми"
        );

        let relative = PronounSpec::closed("иже", PronounDeclension::ThirdPerson, source())
            .expect("third-person base")
            .with_postpositive(PronounPostpositive::Zhe)
            .expect("relative composition");
        assert_eq!(
            relative
                .form(synodal_church_slavonic_core::PronounCell {
                    case: Case::Nominative,
                    number: Number::Plural,
                    gender: Some(Gender::Feminine),
                    person: None,
                    animacy: Animacy::Inanimate,
                })
                .expect("relative nominative")
                .primary_text(),
            "ꙗже"
        );
    }

    #[test]
    fn explicit_determiner_specs_preserve_class_and_number_restrictions() {
        let vsyak =
            DeterminerSpec::new("всѧкъ", "всѧк", DeterminerDeclension::VsyakMixed, source())
                .expect("mixed determiner specification");
        let generated = vsyak
            .form(AdjectiveCell {
                case: Case::Dative,
                number: Number::Singular,
                gender: Gender::Feminine,
                animacy: Animacy::Inanimate,
                form: AdjectiveForm::Short,
                comparison: Comparison::Positive,
            })
            .expect("licensed mixed cell");
        assert_eq!(generated.texts().collect::<Vec<_>>(), ["всѧцѣй", "всѧкой"]);
        assert!(matches!(
            generated.primary().source,
            FormSource::CallerSpecifiedPrediction { .. }
        ));
        let paradigm = vsyak.paradigm(AdjectiveForm::Short);
        assert_eq!(paradigm.successes().count(), 48);
        assert_eq!(
            paradigm
                .with_status(ParadigmStatus::HistoricallyInvalid)
                .count(),
            24
        );

        assert!(matches!(
            DeterminerSpec::new("всѧкъ", "всѧк", DeterminerDeclension::VsyakMixed, source(),)
                .expect("default no-dual specification")
                .with_number_inventory(DeterminerNumberInventory::All),
            Err(Error::ContradictoryMetadata { .. })
        ));
    }

    #[test]
    fn productive_pronominal_tables_match_every_reviewed_exact_cell() {
        for (lemma, stem, declension) in [
            ("мой", "мо", PronounDeclension::Soft),
            ("твой", "тво", PronounDeclension::Soft),
            ("свой", "сво", PronounDeclension::Soft),
            ("нашъ", "наш", PronounDeclension::MixedPossessive),
            ("вашъ", "ваш", PronounDeclension::MixedPossessive),
            ("той", "т", PronounDeclension::Hard),
        ] {
            let explicit = PronounSpec::regular(lemma, stem, declension, source())
                .expect("regular pronoun spec");
            let reviewed = crate::Pronoun::resolve(lemma).expect("reviewed pronoun identity");
            for number in Number::ALL {
                for case in Case::ALL {
                    if case == Case::Vocative {
                        continue;
                    }
                    for gender in Gender::ALL {
                        for animacy in if case == Case::Accusative {
                            Animacy::ALL.as_slice()
                        } else {
                            &[Animacy::Inanimate]
                        } {
                            let cell = synodal_church_slavonic_core::PronounCell {
                                case,
                                number,
                                gender: Some(gender),
                                person: None,
                                animacy: *animacy,
                            };
                            let mut predicted = explicit
                                .form(cell)
                                .expect("productive pronoun table cell")
                                .texts()
                                .map(str::to_owned)
                                .collect::<Vec<_>>();
                            let mut exact = reviewed
                                .form(cell)
                                .expect("reviewed exact pronoun table cell")
                                .texts()
                                .map(str::to_owned)
                                .collect::<Vec<_>>();
                            predicted.sort();
                            exact.sort();
                            assert_eq!(predicted, exact, "{lemma} {cell:?}");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn explicit_accent_paradigm_realizes_multiple_cells() {
        let spec = AdjectiveSpec::new("мꙋдръ", "мꙋдр", AdjectiveClass::Hard, source())
            .expect("adjective")
            .with_accent_paradigm(mudr_accent())
            .expect("accent");
        let inflector = Inflector::builder()
            .orthography(OrthographyProfile::SynodalLiturgical)
            .build();
        for case in [Case::Genitive, Case::Dative, Case::Instrumental] {
            let forms = spec
                .form_with(
                    inflector,
                    AdjectiveCell {
                        case,
                        number: Number::Singular,
                        gender: Gender::Masculine,
                        animacy: Animacy::Inanimate,
                        form: AdjectiveForm::Long,
                        comparison: Comparison::Positive,
                    },
                )
                .expect("accented form");
            assert!(forms.primary_text().starts_with("мꙋ́др"));
            assert!(
                forms
                    .primary()
                    .evidence
                    .iter()
                    .any(|evidence| { evidence.kind == EvidenceKind::AccentParadigm })
            );
        }
    }

    #[test]
    fn partial_irregular_falls_back_and_defect_is_structured() {
        let dative = GrammarCell::Noun(NounCell {
            case: Case::Dative,
            number: Number::Singular,
            animacy: Animacy::Animate,
        });
        let defective = GrammarCell::Noun(NounCell {
            case: Case::Locative,
            number: Number::Dual,
            animacy: Animacy::Animate,
        });
        let spec = NounSpec::new(
            "сынъ",
            "сын",
            Gender::Masculine,
            NounDeclension::FirstHardMasculine,
            source(),
        )
        .expect("noun")
        .with_irregular_form(
            SpecifiedForm::new(
                dative,
                "сынови",
                None::<String>,
                SpecificationSource::new(
                    "alypy-37-synovi",
                    "alypy-gamanovich-grammar-web-2023",
                    "Alypy §37",
                )
                .expect("source"),
            )
            .expect("override"),
        )
        .expect("irregular")
        .with_defective_cell(DefectiveCell::evidence_incomplete(
            defective,
            MetadataField::IrregularOverride,
            "no reviewed dual locative override is supplied by this specification",
        ))
        .expect("defect");
        assert_eq!(
            spec.form(match dative {
                GrammarCell::Noun(cell) => cell,
                _ => unreachable!(),
            })
            .expect("override")
            .primary_text(),
            "сынови"
        );
        let fallback = spec
            .form(NounCell {
                case: Case::Genitive,
                number: Number::Dual,
                animacy: Animacy::Animate,
            })
            .expect("licensed regular background");
        assert_eq!(fallback.primary_text(), "сынꙋ");
        let paradigm = spec.paradigm(Animacy::Animate);
        let row = paradigm
            .iter()
            .find(|row| row.cell() == defective)
            .expect("defective row retained");
        assert_eq!(row.status(), crate::ParadigmStatus::EvidenceIncomplete);
    }

    #[test]
    fn caller_irregular_variants_preserve_declared_order_per_cell() {
        let cell = GrammarCell::Noun(NounCell {
            case: Case::Genitive,
            number: Number::Singular,
            animacy: Animacy::Inanimate,
        });
        let first =
            SpecifiedForm::new(cell, "любве", Some("любве́"), source()).expect("first variant");
        let second =
            SpecifiedForm::new(cell, "любви", Some("любвѝ"), source()).expect("second variant");
        let spec = NounSpec::new(
            "любовь",
            "любв",
            Gender::Feminine,
            NounDeclension::FourthFeminineOvSyncopating,
            source(),
        )
        .expect("noun")
        .with_irregular_form(first.clone())
        .expect("first override")
        .with_irregular_form(second)
        .expect("ordered override");

        assert_eq!(
            spec.form(match cell {
                GrammarCell::Noun(cell) => cell,
                _ => unreachable!(),
            })
            .expect("ordered caller variants")
            .texts()
            .collect::<Vec<_>>(),
            ["любве", "любви"]
        );
        assert!(matches!(
            spec.clone().with_irregular_form(first),
            Err(Error::ContradictoryMetadata { .. })
        ));

        let liturgical = spec
            .form_with(
                Inflector::builder()
                    .orthography(OrthographyProfile::SynodalLiturgical)
                    .build(),
                match cell {
                    GrammarCell::Noun(cell) => cell,
                    _ => unreachable!(),
                },
            )
            .expect("ordered accented caller variants");
        assert_eq!(liturgical.texts().collect::<Vec<_>>(), ["любве́", "любвѝ"]);
    }

    #[test]
    fn paradigm_distinguishes_missing_metadata_from_invalid_cells() {
        let verb = VerbSpec::builder(
            "нести",
            Aspect::Perfective,
            VerbConjugation::FirstUnpalatalized,
            source(),
        )
        .expect("builder")
        .build()
        .expect("verb");
        let finite = verb.finite_paradigm(FiniteTense::Aorist);
        assert!(finite.iter().all(|row| {
            matches!(
                row.status(),
                crate::ParadigmStatus::MissingMetadata | crate::ParadigmStatus::HistoricallyInvalid
            )
        }));
    }

    #[test]
    fn specialized_paradigm_inventories_are_deterministic_and_duplicate_free() {
        fn assert_inventory(paradigm: &Paradigm, expected: usize) {
            let cells = paradigm.iter().map(|row| row.cell()).collect::<Vec<_>>();
            assert_eq!(cells.len(), expected);
            assert_eq!(
                cells.iter().copied().collect::<BTreeSet<_>>().len(),
                expected
            );
            let repeated = paradigm.iter().map(|row| row.cell()).collect::<Vec<_>>();
            assert_eq!(cells, repeated);
        }

        let noun = NounSpec::new(
            "псалтирникъ",
            "псалтирник",
            Gender::Masculine,
            NounDeclension::FirstHardMasculine,
            source(),
        )
        .expect("noun");
        assert_inventory(&noun.paradigm(Animacy::Animate), 21);

        let adjective =
            AdjectiveSpec::new("мꙋдръ", "мꙋдр", AdjectiveClass::Hard, source()).expect("adjective");
        assert_inventory(
            &adjective.paradigm(AdjectiveForm::Long, Comparison::Positive),
            72,
        );

        let verb = VerbSpec::builder(
            "нести",
            Aspect::Imperfective,
            VerbConjugation::FirstUnpalatalized,
            source(),
        )
        .expect("verb")
        .build()
        .expect("verb spec");
        assert_inventory(&verb.finite_paradigm(FiniteTense::Present), 9);
        assert_inventory(
            &verb.participle_paradigm(
                ParticipleTense::Present,
                ParticipleVoice::Active,
                AdjectiveForm::Long,
            ),
            72,
        );
    }

    #[test]
    fn typed_present_parts_and_unified_verb_system_paradigms_are_complete() {
        let incomplete = VerbSpec::builder(
            "нести",
            Aspect::Imperfective,
            VerbConjugation::FirstUnpalatalized,
            source(),
        )
        .expect("builder")
        .build()
        .expect("incomplete verb remains inspectable");
        assert_eq!(
            incomplete.missing_principal_parts(VerbSystem::Finite(FiniteTense::Present)),
            vec![
                MetadataField::PresentStem,
                MetadataField::PresentFirstSingular,
                MetadataField::PresentThirdPlural,
            ]
        );
        let missing_present = incomplete.system_paradigm(VerbSystem::Finite(FiniteTense::Present));
        assert_eq!(missing_present.iter().count(), 9);
        assert_eq!(
            missing_present
                .iter()
                .next()
                .expect("present row")
                .error_code(),
            Some(synodal_church_slavonic_core::ErrorCode::MissingPrincipalPart)
        );

        let verb = VerbSpec::builder(
            "нести",
            Aspect::Imperfective,
            VerbConjugation::FirstUnpalatalized,
            source(),
        )
        .expect("builder")
        .present_series("нес", "несꙋ", "несꙋтъ")
        .expect("present parts")
        .imperative("нес", ImperativeFormation::ISeries)
        .expect("imperative parts")
        .l_participle_stem("нес")
        .expect("l-participle part")
        .build()
        .expect("verb");
        assert!(
            verb.missing_principal_parts(VerbSystem::Finite(FiniteTense::Present))
                .is_empty()
        );
        assert_eq!(
            verb.system_paradigm(VerbSystem::Infinitive)
                .successes()
                .count(),
            1
        );
        assert_eq!(
            verb.system_paradigm(VerbSystem::LParticiple)
                .successes()
                .count(),
            9
        );
        let imperative = verb.system_paradigm(VerbSystem::Imperative);
        assert_eq!(imperative.iter().count(), 9);
        assert_eq!(
            imperative
                .with_status(crate::ParadigmStatus::HistoricallyInvalid)
                .count(),
            3
        );
        assert_eq!(verb.all_system_paradigms().len(), VerbSystem::ALL.len());

        let absent = GrammarCell::Imperative(synodal_church_slavonic_core::ImperativeCell {
            person: Person::Second,
            number: Number::Singular,
        });
        let defective = VerbSpec::builder(
            "нести",
            Aspect::Imperfective,
            VerbConjugation::FirstUnpalatalized,
            source(),
        )
        .expect("builder")
        .defective_cell(DefectiveCell::historically_absent(
            absent,
            "this caller-reviewed lexeme lacks an imperative",
        ))
        .build()
        .expect("defective verb");
        let paradigm = defective.system_paradigm(VerbSystem::Imperative);
        assert_eq!(
            paradigm
                .iter()
                .find(|row| row.cell() == absent)
                .expect("defective cell retained")
                .status(),
            crate::ParadigmStatus::HistoricallyInvalid
        );
    }

    #[test]
    fn perfective_spec_exposes_the_complete_productive_simple_future() {
        let verb = VerbSpec::builder(
            "понести",
            Aspect::Perfective,
            VerbConjugation::FirstUnpalatalized,
            source(),
        )
        .expect("builder")
        .present_series("понес", "понесꙋ", "понесꙋтъ")
        .expect("complete present-shaped principal parts")
        .build()
        .expect("perfective verb");

        assert!(
            verb.missing_principal_parts(VerbSystem::Finite(FiniteTense::Future))
                .is_empty()
        );
        let future = verb.finite_paradigm(FiniteTense::Future);
        assert_eq!(future.iter().count(), 9);
        assert_eq!(future.successes().count(), 9);
        assert_eq!(future.failures().count(), 0);
        assert_eq!(
            verb.form(GrammarCell::FiniteVerb(
                synodal_church_slavonic_core::FiniteVerbCell {
                    tense: FiniteTense::Future,
                    person: Person::Third,
                    number: Number::Singular,
                }
            ))
            .expect("productive simple future")
            .primary_text(),
            "понесетъ"
        );

        let suppletive = VerbSpec::builder(
            "възѧти",
            Aspect::Perfective,
            VerbConjugation::FirstPalatalized,
            source(),
        )
        .expect("builder")
        .present_series("вземл", "вземлю", "вземлютъ")
        .expect("present series")
        .future_series("возм", "возмꙋ", "возмꙋтъ")
        .expect("future series")
        .build()
        .expect("suppletive future verb");
        assert_eq!(
            suppletive
                .form(GrammarCell::FiniteVerb(
                    synodal_church_slavonic_core::FiniteVerbCell {
                        tense: FiniteTense::Present,
                        person: Person::Second,
                        number: Number::Singular,
                    }
                ))
                .expect("present form")
                .primary_text(),
            "вземлеши"
        );
        assert_eq!(
            suppletive
                .form(GrammarCell::FiniteVerb(
                    synodal_church_slavonic_core::FiniteVerbCell {
                        tense: FiniteTense::Future,
                        person: Person::Second,
                        number: Number::Singular,
                    }
                ))
                .expect("future form")
                .primary_text(),
            "возмеши"
        );

        let partial = VerbSpec::builder(
            "възѧти",
            Aspect::Perfective,
            VerbConjugation::FirstPalatalized,
            source(),
        )
        .expect("builder")
        .future_stem("возм")
        .expect("future stem")
        .build();
        assert!(matches!(partial, Err(Error::ContradictoryMetadata { .. })));
    }

    #[test]
    fn absent_supine_and_productive_verbal_noun_are_distinguished() {
        let verb = VerbSpec::builder(
            "нести",
            Aspect::Imperfective,
            VerbConjugation::FirstUnpalatalized,
            source(),
        )
        .expect("verb")
        .verbal_noun_ie("молен")
        .expect("reviewed Alypy §27 platform")
        .build()
        .expect("verb spec");
        assert!(matches!(
            verb.form(GrammarCell::Supine),
            Err(Error::HistoricallyInvalidCell { .. })
        ));
        assert_eq!(
            verb.form(GrammarCell::VerbalNoun(NounCell {
                case: Case::Nominative,
                number: Number::Singular,
                animacy: Animacy::Inanimate,
            }))
            .expect("productive verbal noun")
            .primary_text(),
            "моленїе"
        );

        let compatibility = VerbSpec::builder(
            "нести",
            Aspect::Imperfective,
            VerbConjugation::FirstUnpalatalized,
            source(),
        )
        .expect("verb")
        .irregular_form(
            SpecifiedForm::new(GrammarCell::Supine, "нестъ", None::<String>, source())
                .expect("explicit compatibility cell"),
        )
        .build()
        .expect("verb with explicit compatibility cell");
        assert_eq!(
            compatibility
                .form(GrammarCell::Supine)
                .expect("caller exact compatibility form")
                .primary_text(),
            "нестъ"
        );
    }

    #[test]
    fn explicit_and_registry_routes_share_new_productive_rules() {
        let adjective = AdjectiveSpec::new("мꙋдръ", "мꙋдр", AdjectiveClass::Hard, source())
            .expect("adjective")
            .comparison("мꙋдрѣйш", ComparisonFormation::LaterYat)
            .expect("comparison");
        let adjective_cell = AdjectiveCell {
            case: Case::Dative,
            number: Number::Dual,
            gender: Gender::Feminine,
            animacy: Animacy::Inanimate,
            form: AdjectiveForm::Short,
            comparison: Comparison::Comparative,
        };
        let explicit = adjective.form(adjective_cell).expect("explicit");
        let paradigm = adjective.paradigm(AdjectiveForm::Short, Comparison::Comparative);
        assert_eq!(
            paradigm
                .form(GrammarCell::Adjective(adjective_cell))
                .expect("paradigm cell")
                .primary_text(),
            explicit.primary_text()
        );
        let registered = crate::Adjective::resolve("мꙋдръ")
            .expect("registered")
            .form(adjective_cell)
            .expect("registered form");
        assert_eq!(explicit.primary_text(), registered.primary_text());
        assert_eq!(
            explicit.primary().rule_trace.steps()[0].rule,
            registered.primary().rule_trace.steps()[0].rule
        );

        let part = ParticiplePrincipalPart {
            short_stem: Some(SynodalWord::parse("несꙋщ").expect("stem")),
            short_formation: Some(ActiveParticipleShortFormation::PresentFirstUnpalatalized),
            long_stem: Some(SynodalWord::parse("несꙋщ").expect("stem")),
            class: AdjectiveClass::Hard,
        };
        let verb = VerbSpec::builder(
            "нести",
            Aspect::Imperfective,
            VerbConjugation::FirstUnpalatalized,
            source(),
        )
        .expect("verb")
        .present_active_participle(part)
        .build()
        .expect("verb spec");
        let participle_cell = ParticipleCell {
            tense: ParticipleTense::Present,
            voice: ParticipleVoice::Active,
            agreement: AdjectiveCell {
                case: Case::Instrumental,
                number: Number::Plural,
                gender: Gender::Masculine,
                animacy: Animacy::Inanimate,
                form: AdjectiveForm::Short,
                comparison: Comparison::Positive,
            },
        };
        let explicit = verb
            .form(GrammarCell::Participle(participle_cell))
            .expect("explicit participle");
        let registered = crate::Participle::resolve("нести")
            .expect("registered verb")
            .form(participle_cell)
            .expect("registered participle");
        assert_eq!(explicit.primary_text(), registered.primary_text());
        assert_eq!(
            explicit.primary().rule_trace.steps()[0].rule,
            registered.primary().rule_trace.steps()[0].rule
        );
    }

    #[test]
    fn explicit_specs_reject_hostile_unicode_without_panicking() {
        for lemma in ["слово\u{e000}", "slovo", "сло\u{0301}\u{0486}во"] {
            assert!(
                NounSpec::new(
                    lemma,
                    "слов",
                    Gender::Neuter,
                    NounDeclension::FirstHardNeuter,
                    source(),
                )
                .is_err()
            );
        }
    }

    #[test]
    fn typed_specs_reject_contradictory_formation_metadata() {
        assert!(matches!(
            NounSpec::new(
                "жена",
                "жен",
                Gender::Neuter,
                NounDeclension::SecondHard,
                source(),
            ),
            Err(Error::ContradictoryMetadata { .. })
        ));
        assert!(matches!(
            AdjectiveSpec::new("мꙋдръ", "мꙋдр", AdjectiveClass::Hard, source())
                .expect("base adjective")
                .comparison("мꙋдрѣйш", ComparisonFormation::LaterAi),
            Err(Error::ContradictoryMetadata { .. })
        ));
        let part = ParticiplePrincipalPart {
            short_stem: Some(SynodalWord::parse("несꙋщ").expect("stem")),
            short_formation: Some(ActiveParticipleShortFormation::PresentFirstUnpalatalized),
            long_stem: None,
            class: AdjectiveClass::Hard,
        };
        assert!(matches!(
            VerbSpec::builder(
                "любити",
                Aspect::Imperfective,
                VerbConjugation::Second,
                source(),
            )
            .expect("builder")
            .present_active_participle(part)
            .build(),
            Err(Error::ContradictoryMetadata { .. })
        ));
        assert!(matches!(
            VerbSpec::builder(
                "сотворити",
                Aspect::Perfective,
                VerbConjugation::Second,
                source(),
            )
            .expect("builder")
            .imperfect("сотвор", ImperfectFormation::Ah)
            .expect("metadata")
            .build(),
            Err(Error::ContradictoryMetadata { .. })
        ));
        assert!(matches!(
            VerbSpec::builder(
                "изити",
                Aspect::Perfective,
                VerbConjugation::FirstUnpalatalized,
                source(),
            )
            .expect("builder")
            .l_participle_masculine_singular_stem("изше")
            .expect("mobile-vowel edge")
            .build(),
            Err(Error::ContradictoryMetadata { reason })
                if reason.contains("general l-participle stem")
        ));
        let empty_reason = DefectiveCell::evidence_incomplete(
            GrammarCell::Noun(NounCell {
                case: Case::Dative,
                number: Number::Singular,
                animacy: Animacy::Inanimate,
            }),
            MetadataField::IrregularOverride,
            "   ",
        );
        assert!(matches!(
            NounSpec::new(
                "слово",
                "слов",
                Gender::Neuter,
                NounDeclension::FirstHardNeuter,
                source(),
            )
            .expect("base noun")
            .with_defective_cell(empty_reason),
            Err(Error::ContradictoryMetadata { .. })
        ));
    }
}
