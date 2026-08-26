//! Explicit-principal-part Old Church Slavonic verb morphology.

use crate::{
    AdjectiveClass, AdjectiveForm, AoristFormation, Case, FiniteTense, FiniteVerbCell, Gender,
    ImperativeCell, ImperativeFormation, ImperfectFormation, ImperfectVariantPolicy,
    InflectionError, LParticipleCell, MetadataField, Number, ParticipleCell, ParticipleKind,
    PastActiveParticipleFormation, PastPassiveParticipleFormation, Person, PredictedForm,
    PresentActiveParticipleFormation, PresentFormation, PresentPassiveParticipleFormation,
    RequestedCell, RuleId, RuleStep, VerbAspect, VerbClass, VerbDefectKind, VerbMorphologyCell,
    VerbMorphologySystem,
};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VerbStems {
    pub present: Option<String>,
    pub present_first_singular: Option<String>,
    /// Independent 3pl edge allomorph, needed when a final velar remains
    /// unpalatalized before `-ѫтъ` but palatalizes before e-initial terminals.
    pub present_third_plural: Option<String>,
    pub imperfect: Option<String>,
    pub aorist: Option<String>,
    /// Independently supplied complete 2sg/3sg aorist principal part.
    ///
    /// Sigmatic aorists use a separate singular subbundle, including lexical
    /// zero, `-тъ`, and `-стъ` realizations, so this value is never derived from
    /// the main sigmatic stem.
    pub aorist_second_third_singular: Option<String>,
    pub imperative: Option<String>,
    pub l_participle: Option<String>,
    pub present_active_participle: Option<String>,
    pub present_passive_participle: Option<String>,
    pub past_active_participle: Option<String>,
    pub past_passive_participle: Option<String>,
    /// Complete nominal derivational platform before `-иѥ`.
    ///
    /// This is independent because OCS verbal nouns can exist without an
    /// attested past passive participle.
    pub verbal_noun: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VerbFormations {
    pub present: Option<PresentFormation>,
    pub imperfect: Option<ImperfectFormation>,
    pub imperfect_variant_policy: Option<ImperfectVariantPolicy>,
    pub aorist: Option<AoristFormation>,
    pub imperative: Option<ImperativeFormation>,
    pub present_active_participle: Option<PresentActiveParticipleFormation>,
    pub present_passive_participle: Option<PresentPassiveParticipleFormation>,
    pub past_active_participle: Option<PastActiveParticipleFormation>,
    pub past_passive_participle: Option<PastPassiveParticipleFormation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerbLexeme {
    pub lemma: String,
    pub class: VerbClass,
    pub aspect: Option<VerbAspect>,
    pub stems: VerbStems,
    pub formations: VerbFormations,
    /// Source-reviewed surface cells. Exact cells win before a declared defect
    /// or productive fallback, allowing sparse paradigms to retain attested
    /// forms without inventing an entire exceptional conjugation.
    pub exact_forms: BTreeMap<VerbMorphologyCell, String>,
    /// Defects applying to a whole subsystem unless an exact cell is present.
    pub defective_systems: BTreeMap<VerbMorphologySystem, VerbDefectKind>,
    /// Cell-specific defects, checked before a system-level defect.
    pub defective_cells: BTreeMap<VerbMorphologyCell, VerbDefectKind>,
}

impl VerbLexeme {
    pub fn new(lemma: impl Into<String>, class: VerbClass) -> Self {
        Self {
            lemma: lemma.into(),
            class,
            aspect: None,
            stems: VerbStems::default(),
            formations: VerbFormations::default(),
            exact_forms: BTreeMap::new(),
            defective_systems: BTreeMap::new(),
            defective_cells: BTreeMap::new(),
        }
    }

    pub fn builder(
        lemma: impl Into<String>,
        class: VerbClass,
    ) -> Result<VerbLexemeBuilder, InflectionError> {
        let lemma = crate::orthography::canonical_display(&lemma.into())?;
        Ok(VerbLexemeBuilder {
            lexeme: Self::new(lemma, class),
        })
    }
}

pub(crate) fn insert_imperative_singular(lexeme: &mut VerbLexeme, form: &str) {
    for person in [Person::Second, Person::Third] {
        lexeme.exact_forms.insert(
            VerbMorphologyCell::Imperative(ImperativeCell {
                person,
                number: Number::Singular,
            }),
            form.to_string(),
        );
    }
}

pub(crate) fn set_imperfect(lexeme: &mut VerbLexeme, stem: &str, formation: ImperfectFormation) {
    lexeme.stems.imperfect = Some(stem.to_string());
    lexeme.formations.imperfect = Some(formation);
    lexeme.formations.imperfect_variant_policy = Some(ImperfectVariantPolicy::UncontractedOnly);
}

pub(crate) fn set_sigmatic_vowel_aorist(lexeme: &mut VerbLexeme, stem: &str, singular: &str) {
    lexeme.stems.aorist = Some(stem.to_string());
    lexeme.stems.aorist_second_third_singular = Some(singular.to_string());
    lexeme.formations.aorist = Some(AoristFormation::SigmaticVowel);
}

pub(crate) fn set_new_aorist(lexeme: &mut VerbLexeme, stem: &str) {
    lexeme.stems.aorist = Some(stem.to_string());
    lexeme.formations.aorist = Some(AoristFormation::New);
}

pub(crate) fn set_imperative(lexeme: &mut VerbLexeme, stem: &str, formation: ImperativeFormation) {
    lexeme.stems.imperative = Some(stem.to_string());
    lexeme.formations.imperative = Some(formation);
}

pub(crate) fn set_l_participle(lexeme: &mut VerbLexeme, stem: &str) {
    lexeme.stems.l_participle = Some(stem.to_string());
}

pub(crate) fn set_present_active(
    lexeme: &mut VerbLexeme,
    stem: &str,
    formation: PresentActiveParticipleFormation,
) {
    lexeme.stems.present_active_participle = Some(stem.to_string());
    lexeme.formations.present_active_participle = Some(formation);
}

pub(crate) fn set_present_passive(
    lexeme: &mut VerbLexeme,
    stem: &str,
    formation: PresentPassiveParticipleFormation,
) {
    lexeme.stems.present_passive_participle = Some(stem.to_string());
    lexeme.formations.present_passive_participle = Some(formation);
}

pub(crate) fn set_past_active(
    lexeme: &mut VerbLexeme,
    stem: &str,
    formation: PastActiveParticipleFormation,
) {
    lexeme.stems.past_active_participle = Some(stem.to_string());
    lexeme.formations.past_active_participle = Some(formation);
}

pub(crate) fn set_past_passive(
    lexeme: &mut VerbLexeme,
    stem: &str,
    formation: PastPassiveParticipleFormation,
) {
    lexeme.stems.past_passive_participle = Some(stem.to_string());
    lexeme.formations.past_passive_participle = Some(formation);
}

/// Validated construction path for callers that prefer impossible stem/formation
/// pairs to fail at metadata assembly time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerbLexemeBuilder {
    lexeme: VerbLexeme,
}

impl VerbLexemeBuilder {
    pub fn aspect(mut self, aspect: VerbAspect) -> Self {
        self.lexeme.aspect = Some(aspect);
        self
    }

    pub fn present(
        mut self,
        stem: impl Into<String>,
        first_singular: Option<String>,
    ) -> Result<Self, InflectionError> {
        self.lexeme.stems.present = Some(validated_stem(stem)?);
        self.lexeme.stems.present_first_singular =
            first_singular.map(validated_stem).transpose()?;
        Ok(self)
    }

    /// Select the iotated e-conjugation surface series and, where needed,
    /// supply an independent 3pl edge allomorph.
    pub fn iotated_present(
        mut self,
        stem: impl Into<String>,
        first_singular: Option<String>,
        third_plural: Option<String>,
    ) -> Result<Self, InflectionError> {
        self.lexeme.stems.present = Some(validated_stem(stem)?);
        self.lexeme.stems.present_first_singular =
            first_singular.map(validated_stem).transpose()?;
        self.lexeme.stems.present_third_plural = third_plural.map(validated_stem).transpose()?;
        self.lexeme.formations.present = Some(PresentFormation::IotatedE);
        Ok(self)
    }

    pub fn imperfect(
        mut self,
        stem: impl Into<String>,
        formation: ImperfectFormation,
        variant_policy: ImperfectVariantPolicy,
    ) -> Result<Self, InflectionError> {
        self.lexeme.stems.imperfect = Some(validated_stem(stem)?);
        self.lexeme.formations.imperfect = Some(formation);
        self.lexeme.formations.imperfect_variant_policy = Some(variant_policy);
        Ok(self)
    }

    pub fn aorist(
        mut self,
        stem: impl Into<String>,
        formation: AoristFormation,
    ) -> Result<Self, InflectionError> {
        if matches!(
            formation,
            AoristFormation::SigmaticPrimary
                | AoristFormation::SigmaticSecondary
                | AoristFormation::SigmaticVowel
        ) {
            return Err(InflectionError::InvalidInput {
                reason: "a sigmatic aorist requires the independent 2sg/3sg principal part; use sigmatic_aorist"
                    .to_string(),
            });
        }
        self.lexeme.stems.aorist = Some(validated_stem(stem)?);
        self.lexeme.formations.aorist = Some(formation);
        Ok(self)
    }

    /// Add a source-audited sigmatic main stem and complete 2sg/3sg form.
    ///
    /// `main_stem` is the surface stem after lexical vowel gradation and before
    /// the formation's `с/х` endings: `нѣ-` for `нѣсъ`, `рѣ-` for `рѣхъ`.
    /// `second_third_singular` is the complete syncretic form, such as `рече`,
    /// `ѧ`, `ѧтъ`, or `быстъ`.
    pub fn sigmatic_aorist(
        mut self,
        main_stem: impl Into<String>,
        second_third_singular: impl Into<String>,
        formation: AoristFormation,
    ) -> Result<Self, InflectionError> {
        if !matches!(
            formation,
            AoristFormation::SigmaticPrimary
                | AoristFormation::SigmaticSecondary
                | AoristFormation::SigmaticVowel
        ) {
            return Err(InflectionError::InvalidInput {
                reason: "sigmatic_aorist accepts only a sigmatic aorist formation".to_string(),
            });
        }
        let main_stem = validated_stem(main_stem)?;
        if formation == AoristFormation::SigmaticVowel && !ends_in_ocs_vowel(&main_stem) {
            return Err(InflectionError::InvalidInput {
                reason: "the vowel-stem sigmatic formation requires a vowel-final main stem"
                    .to_string(),
            });
        }
        self.lexeme.stems.aorist = Some(main_stem);
        self.lexeme.stems.aorist_second_third_singular =
            Some(validated_stem(second_third_singular)?);
        self.lexeme.formations.aorist = Some(formation);
        Ok(self)
    }

    pub fn imperative(
        mut self,
        stem: impl Into<String>,
        formation: ImperativeFormation,
    ) -> Result<Self, InflectionError> {
        self.lexeme.stems.imperative = Some(validated_stem(stem)?);
        self.lexeme.formations.imperative = Some(formation);
        Ok(self)
    }

    pub fn l_participle(mut self, stem: impl Into<String>) -> Result<Self, InflectionError> {
        self.lexeme.stems.l_participle = Some(validated_stem(stem)?);
        Ok(self)
    }

    /// Supply one source-reviewed exceptional surface cell.
    pub fn exact_form(
        mut self,
        cell: VerbMorphologyCell,
        form: impl Into<String>,
    ) -> Result<Self, InflectionError> {
        if let VerbMorphologyCell::Imperative(cell) = cell
            && !cell.is_supported()
        {
            return Err(InflectionError::InvalidInput {
                reason: "an exact imperative must use a historically licensed person-number cell"
                    .to_string(),
            });
        }
        let form = validated_form(form)?;
        if let Some(previous) = self.lexeme.exact_forms.insert(cell, form.clone())
            && previous != form
        {
            return Err(InflectionError::InvalidInput {
                reason: format!("conflicting exact forms {previous:?} and {form:?} for {cell:?}"),
            });
        }
        Ok(self)
    }

    /// Declare a whole subsystem lexically defective or unreconstructable.
    pub fn defective_system(mut self, system: VerbMorphologySystem, kind: VerbDefectKind) -> Self {
        self.lexeme.defective_systems.insert(system, kind);
        self
    }

    /// Declare one grammatical cell lexically defective or unreconstructable.
    pub fn defective_cell(mut self, cell: VerbMorphologyCell, kind: VerbDefectKind) -> Self {
        self.lexeme.defective_cells.insert(cell, kind);
        self
    }

    pub fn present_active_participle(
        mut self,
        stem: impl Into<String>,
        formation: PresentActiveParticipleFormation,
    ) -> Result<Self, InflectionError> {
        self.lexeme.stems.present_active_participle = Some(validated_stem(stem)?);
        self.lexeme.formations.present_active_participle = Some(formation);
        Ok(self)
    }

    pub fn present_passive_participle(
        mut self,
        stem: impl Into<String>,
        formation: PresentPassiveParticipleFormation,
    ) -> Result<Self, InflectionError> {
        self.lexeme.stems.present_passive_participle = Some(validated_stem(stem)?);
        self.lexeme.formations.present_passive_participle = Some(formation);
        Ok(self)
    }

    pub fn past_active_participle(
        mut self,
        stem: impl Into<String>,
        formation: PastActiveParticipleFormation,
    ) -> Result<Self, InflectionError> {
        let stem = validated_stem(stem)?;
        if formation == PastActiveParticipleFormation::VushAfterOvToU && !stem.ends_with("ов") {
            return Err(InflectionError::InvalidInput {
                reason: "the ov-to-u past-active formation requires a stem ending in -ов"
                    .to_string(),
            });
        }
        self.lexeme.stems.past_active_participle = Some(stem);
        self.lexeme.formations.past_active_participle = Some(formation);
        Ok(self)
    }

    pub fn past_passive_participle(
        mut self,
        stem: impl Into<String>,
        formation: PastPassiveParticipleFormation,
    ) -> Result<Self, InflectionError> {
        self.lexeme.stems.past_passive_participle = Some(validated_stem(stem)?);
        self.lexeme.formations.past_passive_participle = Some(formation);
        Ok(self)
    }

    /// Supply the complete source-reviewed verbal-noun platform before `-иѥ`.
    ///
    /// For example, `знан` produces citation `знаниѥ`. This independent input
    /// also represents intransitive formations for which no passive participle
    /// is attested.
    pub fn verbal_noun(mut self, stem: impl Into<String>) -> Result<Self, InflectionError> {
        self.lexeme.stems.verbal_noun = Some(validated_stem(stem)?);
        Ok(self)
    }

    pub fn build(self) -> VerbLexeme {
        self.lexeme
    }
}

fn validated_stem(stem: impl Into<String>) -> Result<String, InflectionError> {
    let stem = stem.into();
    if stem.is_empty() {
        return Err(InflectionError::InvalidInput {
            reason: "a verb principal-part stem must not be empty".to_string(),
        });
    }
    let stem = crate::orthography::canonical_display(&stem)?;
    if crate::orthography::detect_script(&stem) != crate::orthography::Script::Cyrillic {
        return Err(InflectionError::InvalidInput {
            reason: "a productive verb principal-part stem must be Cyrillic".to_string(),
        });
    }
    Ok(stem)
}

fn validated_form(form: impl Into<String>) -> Result<String, InflectionError> {
    let form = form.into();
    if form.is_empty() {
        return Err(InflectionError::InvalidInput {
            reason: "an exact irregular verb form must not be empty".to_string(),
        });
    }
    let form = crate::orthography::canonical_display(&form)?;
    if crate::orthography::detect_script(&form) != crate::orthography::Script::Cyrillic {
        return Err(InflectionError::InvalidInput {
            reason: "an exact irregular verb form must be Cyrillic".to_string(),
        });
    }
    Ok(form)
}

pub fn finite(lexeme: &VerbLexeme, cell: FiniteVerbCell) -> Result<PredictedForm, InflectionError> {
    crate::orthography::canonical_display(&lexeme.lemma)?;
    if let Some(result) = irregular_resolution(lexeme, VerbMorphologyCell::Finite(cell)) {
        return result;
    }
    match cell.tense {
        FiniteTense::Present => present(lexeme, cell),
        FiniteTense::Imperfect => imperfect(lexeme, cell),
        FiniteTense::Aorist => aorist(lexeme, cell),
    }
}

pub fn imperative(
    lexeme: &VerbLexeme,
    cell: ImperativeCell,
) -> Result<PredictedForm, InflectionError> {
    crate::orthography::canonical_display(&lexeme.lemma)?;
    if !cell.is_supported() {
        return Err(InflectionError::historically_invalid(
            &lexeme.lemma,
            RequestedCell::Imperative(cell),
        ));
    }
    if let Some(result) = irregular_resolution(lexeme, VerbMorphologyCell::Imperative(cell)) {
        return result;
    }
    let formation =
        lexeme
            .formations
            .imperative
            .ok_or_else(|| InflectionError::MissingLexicalMetadata {
                needed: vec![MetadataField::ImperativeFormation],
            })?;
    let stem = required_stem(
        lexeme.stems.imperative.as_deref(),
        MetadataField::ImperativeStem,
    )?;
    // Shim over the merged imperative kernel (OCS column): the family's
    // YatSeries is the kernel's EGrade axis (divergence
    // `verb:imperative-vowel-grade`), ISeries the shared i-grade.
    let series = match formation {
        ImperativeFormation::ISeries => church_slavonic_core::verb::ImperativeSeries::I,
        ImperativeFormation::YatSeries => church_slavonic_core::verb::ImperativeSeries::EGrade,
    };
    let column = church_slavonic_core::verb::imperative_ending(
        series,
        cell.person,
        cell.number,
        church_slavonic_core::Recension::OldChurchSlavonic,
    );
    let Some(&ending) = column.first() else {
        return Err(InflectionError::historically_invalid(
            &lexeme.lemma,
            RequestedCell::Imperative(cell),
        ));
    };
    Ok(join(
        &stem,
        ending,
        RuleId::VerbImperative,
        "attach the explicitly selected i-series or yat-series imperative ending",
    ))
}

pub fn infinitive(lexeme: &VerbLexeme) -> Result<PredictedForm, InflectionError> {
    let lemma = crate::orthography::canonical_display(&lexeme.lemma)?;
    if let Some(result) = irregular_resolution(lexeme, VerbMorphologyCell::Infinitive) {
        return result;
    }
    if !has_ocs_infinitive_ending(&lemma) {
        return Err(InflectionError::InvalidInput {
            reason: "an OCS infinitive citation must end in -ти or -щи".to_string(),
        });
    }
    Ok(single_step(
        &lemma,
        &lemma,
        RuleId::VerbInfinitive,
        "return the supplied infinitive citation form",
    ))
}

pub fn supine(lexeme: &VerbLexeme) -> Result<PredictedForm, InflectionError> {
    let lemma = crate::orthography::canonical_display(&lexeme.lemma)?;
    if let Some(result) = irregular_resolution(lexeme, VerbMorphologyCell::Supine) {
        return result;
    }
    if !has_ocs_infinitive_ending(&lemma) {
        return Err(InflectionError::InvalidInput {
            reason: "a regularly derived supine needs an infinitive ending in -ти or -щи"
                .to_string(),
        });
    }
    let stem = &lemma[..lemma.len() - 'и'.len_utf8()];
    let terminal = if ends_in_morphologically_soft_consonant(stem) {
        "ь"
    } else {
        "ъ"
    };
    let text = format!("{stem}{terminal}");
    Ok(single_step(
        &lemma,
        &text,
        RuleId::VerbSupine,
        "replace infinitival final и with the supine terminal, fronted after a soft consonant",
    ))
}

/// Form and decline the productive OCS verbal noun in `-иѥ`.
///
/// UT OCS Online lesson 8 §36 defines the noun as a past-passive platform plus
/// `-ьj-` and assigns it to the soft neuter `jo` declension. Polivanova 2023
/// §§483 and 865 independently classify the same pattern as nominal
/// derivation and show why an independent platform is necessary when the
/// corresponding participle is unattested.
pub fn verbal_noun(
    lexeme: &VerbLexeme,
    cell: crate::NounCell,
) -> Result<PredictedForm, InflectionError> {
    crate::orthography::canonical_display(&lexeme.lemma)?;
    if let Some(result) = irregular_resolution(lexeme, VerbMorphologyCell::VerbalNoun(cell)) {
        return result;
    }

    let (platform, formation_reason) = if let Some(stem) = lexeme.stems.verbal_noun.as_deref() {
        (
            required_stem(Some(stem), MetadataField::VerbalNounStem)?,
            "form the deverbal soft-neuter citation from the independent nominal platform",
        )
    } else {
        let stem = required_stem(
            lexeme.stems.past_passive_participle.as_deref(),
            MetadataField::VerbalNounStem,
        )?;
        let formation = lexeme.formations.past_passive_participle.ok_or_else(|| {
            InflectionError::MissingLexicalMetadata {
                needed: vec![MetadataField::VerbalNounStem],
            }
        })?;
        let platform = match formation {
            PastPassiveParticipleFormation::T => format!("{stem}т"),
            PastPassiveParticipleFormation::N => format!("{stem}н"),
            PastPassiveParticipleFormation::En => format!("{stem}ен"),
        };
        (
            platform,
            "form the deverbal soft-neuter citation from the typed past-passive platform",
        )
    };
    let citation = format!("{platform}иѥ");
    let noun = crate::noun::NounLexeme {
        lemma: citation.clone(),
        class: crate::NounClass::JoNeuterSoft,
        gender: Gender::Neuter,
        animacy: crate::Animacy::Inanimate,
        number_restriction: crate::NumberRestriction::All,
    };
    let declined = crate::noun::decline(&noun, cell)?;
    Ok(PredictedForm {
        text: declined.text.clone(),
        rule_id: RuleId::VerbVerbalNoun,
        trace: std::iter::once(RuleStep {
            rule_id: RuleId::VerbVerbalNoun,
            before: lexeme.lemma.clone(),
            after: citation,
            reason: formation_reason,
        })
        .chain(declined.trace)
        .collect(),
    })
}

pub fn l_participle(
    lexeme: &VerbLexeme,
    cell: LParticipleCell,
) -> Result<PredictedForm, InflectionError> {
    crate::orthography::canonical_display(&lexeme.lemma)?;
    if let Some(result) = irregular_resolution(lexeme, VerbMorphologyCell::LParticiple(cell)) {
        return result;
    }
    let stem = required_stem(
        lexeme.stems.l_participle.as_deref(),
        MetadataField::LParticipleStem,
    )?;
    // Shim over the merged l-participle kernel (OCS column; the Synodal
    // side levels the dual/plural — divergence `verb:l-participle-leveling`).
    let ending = church_slavonic_core::verb::l_participle_ending(
        cell.gender,
        cell.number,
        church_slavonic_core::Recension::OldChurchSlavonic,
    )[0];
    Ok(join(
        &stem,
        ending,
        RuleId::VerbLParticiple,
        "attach the l-participle agreement ending to the explicit l-participle stem",
    ))
}

pub fn participle(
    lexeme: &VerbLexeme,
    cell: ParticipleCell,
) -> Result<PredictedForm, InflectionError> {
    crate::orthography::canonical_display(&lexeme.lemma)?;
    if let Some(result) = irregular_resolution(lexeme, VerbMorphologyCell::Participle(cell)) {
        return result;
    }
    match cell.kind {
        ParticipleKind::PresentActive => present_active_participle(lexeme, cell),
        ParticipleKind::PresentPassive => present_passive_participle(lexeme, cell),
        ParticipleKind::PastActive => past_active_participle(lexeme, cell),
        ParticipleKind::PastPassive => past_passive_participle(lexeme, cell),
    }
}

fn irregular_resolution(
    lexeme: &VerbLexeme,
    cell: VerbMorphologyCell,
) -> Option<Result<PredictedForm, InflectionError>> {
    if let Some(form) = lexeme.exact_forms.get(&cell) {
        return Some(validated_form(form.clone()).map(|form| {
            single_step(
                &lexeme.lemma,
                &form,
                RuleId::VerbIrregularExact,
                "select the source-reviewed exact cell before irregular or productive fallback",
            )
        }));
    }
    let defect = lexeme
        .defective_cells
        .get(&cell)
        .or_else(|| lexeme.defective_systems.get(&cell.system()))?;
    let error = match defect {
        VerbDefectKind::HistoricallyInvalid => {
            InflectionError::historically_invalid(&lexeme.lemma, cell.requested())
        }
        VerbDefectKind::UnattestedUnreconstructable => {
            InflectionError::unattested_unreconstructable(&lexeme.lemma, cell.requested())
        }
    };
    Some(Err(error))
}

fn present(lexeme: &VerbLexeme, cell: FiniteVerbCell) -> Result<PredictedForm, InflectionError> {
    if !matches!(
        lexeme.class,
        VerbClass::IA1 | VerbClass::IA2 | VerbClass::II1 | VerbClass::II2 | VerbClass::II3
    ) {
        return Err(InflectionError::UnsupportedFormation {
            system: MetadataField::VerbClass,
            formation: format!("{:?}", lexeme.class),
        });
    }
    let default_stem = required_stem(lexeme.stems.present.as_deref(), MetadataField::PresentStem)?;
    let second_conjugation = matches!(
        lexeme.class,
        VerbClass::II1 | VerbClass::II2 | VerbClass::II3
    );
    let stem = if cell.person == Person::First && cell.number == Number::Singular {
        match lexeme.stems.present_first_singular.as_deref() {
            Some(stem) => required_stem(Some(stem), MetadataField::PresentFirstSingularStem)?,
            None if second_conjugation => {
                return Err(InflectionError::MissingLexicalMetadata {
                    needed: vec![MetadataField::PresentFirstSingularStem],
                });
            }
            None => default_stem.clone(),
        }
    } else if cell.person == Person::Third && cell.number == Number::Plural {
        match lexeme.stems.present_third_plural.as_deref() {
            Some(stem) => required_stem(Some(stem), MetadataField::PresentThirdPluralStem)?,
            None => default_stem.clone(),
        }
    } else {
        default_stem
    };
    let (ending, rule_id) = match lexeme.formations.present {
        Some(PresentFormation::IotatedE) => {
            if !matches!(lexeme.class, VerbClass::IA1 | VerbClass::IA2) {
                return Err(InflectionError::UnsupportedFormation {
                    system: MetadataField::PresentFormation,
                    formation: format!("{:?} with {:?}", PresentFormation::IotatedE, lexeme.class),
                });
            }
            (iotated_e_present_ending(cell), RuleId::VerbIA1)
        }
        Some(PresentFormation::HardI) => {
            if !matches!(
                lexeme.class,
                VerbClass::II1 | VerbClass::II2 | VerbClass::II3
            ) {
                return Err(InflectionError::UnsupportedFormation {
                    system: MetadataField::PresentFormation,
                    formation: format!("{:?} with {:?}", PresentFormation::HardI, lexeme.class),
                });
            }
            (hard_i_present_ending(cell), RuleId::VerbII1)
        }
        None => present_ending(&lexeme.lemma, lexeme.class, cell)?,
    };
    Ok(join(
        &stem,
        ending,
        rule_id,
        "attach the class-specific present ending to the supplied present allomorph",
    ))
}

fn imperfect(lexeme: &VerbLexeme, cell: FiniteVerbCell) -> Result<PredictedForm, InflectionError> {
    let formation =
        lexeme
            .formations
            .imperfect
            .ok_or_else(|| InflectionError::MissingLexicalMetadata {
                needed: vec![MetadataField::ImperfectFormation],
            })?;
    let variant_policy = lexeme.formations.imperfect_variant_policy.ok_or_else(|| {
        InflectionError::MissingLexicalMetadata {
            needed: vec![MetadataField::ImperfectVariantPolicy],
        }
    })?;
    let stem = required_stem(
        lexeme.stems.imperfect.as_deref(),
        MetadataField::ImperfectStem,
    )?;
    let stem = if formation == ImperfectFormation::PalatalizedA {
        let changed = first_palatalize(&stem);
        if changed == stem {
            return Err(InflectionError::InvalidInput {
                reason: "the palatalized imperfect formation requires a final velar stem"
                    .to_string(),
            });
        }
        changed
    } else {
        stem
    };
    // Shim over the merged imperfect-marker kernel (OCS columns list the
    // ordered uncontracted/contracted grades; divergence
    // `verb:imperfect-contraction` separates the Synodal grades).
    let kernel_marker = |marker, contracted: bool| {
        let column = church_slavonic_core::verb_past::imperfect_marker(
            marker,
            church_slavonic_core::Recension::OldChurchSlavonic,
        );
        if contracted { column[1] } else { column[0] }
    };
    use church_slavonic_core::verb_past::ImperfectMarker as KernelMarker;
    let (marker, rule_id, reason) = match (formation, variant_policy) {
        (ImperfectFormation::A, ImperfectVariantPolicy::UncontractedOnly) => (
            kernel_marker(KernelMarker::A, false),
            RuleId::VerbImperfectA,
            "attach the explicitly selected uncontracted imperfect marker and personal ending",
        ),
        (ImperfectFormation::A, ImperfectVariantPolicy::ContractedOnly) => (
            kernel_marker(KernelMarker::A, true),
            RuleId::VerbImperfectContractedA,
            "attach the source-selected contracted imperfect terminal to the explicit platform",
        ),
        (ImperfectFormation::YatA, ImperfectVariantPolicy::UncontractedOnly) => (
            kernel_marker(KernelMarker::YatA, false),
            RuleId::VerbImperfectYatA,
            "attach the explicitly selected uncontracted imperfect marker and personal ending",
        ),
        (ImperfectFormation::YatA, ImperfectVariantPolicy::ContractedOnly) => (
            kernel_marker(KernelMarker::YatA, true),
            RuleId::VerbImperfectContractedYatA,
            "attach the source-selected contracted imperfect terminal to the explicit platform",
        ),
        (ImperfectFormation::PalatalizedA, ImperfectVariantPolicy::UncontractedOnly) => (
            kernel_marker(KernelMarker::PalatalizedA, false),
            RuleId::VerbImperfectPalatalizedA,
            "attach the explicitly selected uncontracted imperfect marker and personal ending",
        ),
        (ImperfectFormation::PalatalizedA, ImperfectVariantPolicy::ContractedOnly) => (
            kernel_marker(KernelMarker::PalatalizedA, true),
            RuleId::VerbImperfectContractedPalatalizedA,
            "attach the source-selected contracted imperfect terminal to the explicit platform",
        ),
        (ImperfectFormation::PresentA, ImperfectVariantPolicy::UncontractedOnly) => (
            kernel_marker(KernelMarker::A, false),
            RuleId::VerbImperfectPresent,
            "attach the short uncontracted imperfect terminal to the explicit present-system stem",
        ),
        (ImperfectFormation::PresentA, ImperfectVariantPolicy::ContractedOnly) => (
            kernel_marker(KernelMarker::A, true),
            RuleId::VerbImperfectPresentContracted,
            "attach the short contracted imperfect terminal to the explicit present-system stem",
        ),
        (ImperfectFormation::PresentYatA, ImperfectVariantPolicy::UncontractedOnly) => (
            kernel_marker(KernelMarker::YatA, false),
            RuleId::VerbImperfectPresent,
            "attach the uncontracted imperfect terminal to the explicit present-system stem",
        ),
        (ImperfectFormation::PresentYatA, ImperfectVariantPolicy::ContractedOnly) => (
            kernel_marker(KernelMarker::YatA, true),
            RuleId::VerbImperfectPresentContracted,
            "attach the contracted imperfect terminal to the explicit present-system stem",
        ),
        (formation, ImperfectVariantPolicy::IotatedOnly) => (
            kernel_marker(
                match formation {
                    ImperfectFormation::A | ImperfectFormation::PresentA => KernelMarker::IotatedA,
                    ImperfectFormation::YatA | ImperfectFormation::PresentYatA => {
                        KernelMarker::IotatedYatA
                    }
                    ImperfectFormation::PalatalizedA => KernelMarker::IotatedPalatalizedA,
                },
                false,
            ),
            RuleId::VerbImperfectIotated,
            "attach the source-selected iotated imperfect terminal to the explicit workstem",
        ),
    };
    let personal = imperfect_personal_ending(cell);
    let ending = format!("{marker}{personal}");
    Ok(join(&stem, &ending, rule_id, reason))
}

fn aorist(lexeme: &VerbLexeme, cell: FiniteVerbCell) -> Result<PredictedForm, InflectionError> {
    let formation =
        lexeme
            .formations
            .aorist
            .ok_or_else(|| InflectionError::MissingLexicalMetadata {
                needed: vec![MetadataField::AoristFormation],
            })?;
    match formation {
        AoristFormation::Asigmatic => {
            let stem = required_stem(lexeme.stems.aorist.as_deref(), MetadataField::AoristStem)?;
            let (changed, ending) = asigmatic_aorist_cell(&stem, cell);
            Ok(join(
                &changed,
                ending,
                RuleId::VerbAoristAsigmatic,
                "attach the asigmatic aorist personal ending to the explicit aorist stem",
            ))
        }
        AoristFormation::New => {
            let stem = required_stem(lexeme.stems.aorist.as_deref(), MetadataField::AoristStem)?;
            let (changed, ending) = new_aorist_cell(&stem, cell);
            Ok(join(
                &changed,
                ending,
                RuleId::VerbAoristNew,
                "attach the new ox-aorist personal ending to the explicit aorist stem",
            ))
        }
        AoristFormation::SigmaticPrimary => sigmatic_aorist(lexeme, cell, SigmaticKind::Primary),
        AoristFormation::SigmaticSecondary => {
            sigmatic_aorist(lexeme, cell, SigmaticKind::Secondary)
        }
        AoristFormation::SigmaticVowel => sigmatic_aorist(lexeme, cell, SigmaticKind::VowelStem),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SigmaticKind {
    Primary,
    Secondary,
    VowelStem,
}

impl SigmaticKind {
    const fn rule_id(self) -> RuleId {
        match self {
            Self::Primary => RuleId::VerbAoristSigmaticPrimary,
            Self::Secondary => RuleId::VerbAoristSigmaticSecondary,
            Self::VowelStem => RuleId::VerbAoristSigmaticVowel,
        }
    }
}

fn sigmatic_aorist(
    lexeme: &VerbLexeme,
    cell: FiniteVerbCell,
    kind: SigmaticKind,
) -> Result<PredictedForm, InflectionError> {
    let stem = required_stem(lexeme.stems.aorist.as_deref(), MetadataField::AoristStem)?;
    if kind == SigmaticKind::VowelStem && !ends_in_ocs_vowel(&stem) {
        return Err(InflectionError::InvalidInput {
            reason: "the vowel-stem sigmatic formation requires a vowel-final main stem"
                .to_string(),
        });
    }
    if matches!(
        (cell.person, cell.number),
        (Person::Second | Person::Third, Number::Singular)
    ) {
        let form = required_stem(
            lexeme.stems.aorist_second_third_singular.as_deref(),
            MetadataField::AoristSecondThirdSingular,
        )?;
        let rule_id = kind.rule_id();
        return Ok(single_step(
            &stem,
            &form,
            rule_id,
            "select the independently supplied syncretic 2sg/3sg sigmatic-aorist principal part",
        ));
    }
    let ending = sigmatic_aorist_ending(kind, cell);
    Ok(join(
        &stem,
        ending,
        kind.rule_id(),
        "attach the selected old-sigmatic main-subbundle ending to the explicitly graded stem",
    ))
}

fn present_active_participle(
    lexeme: &VerbLexeme,
    cell: ParticipleCell,
) -> Result<PredictedForm, InflectionError> {
    let formation = lexeme.formations.present_active_participle.ok_or_else(|| {
        InflectionError::MissingLexicalMetadata {
            needed: vec![MetadataField::PresentActiveParticipleFormation],
        }
    })?;
    let stem = required_stem(
        lexeme.stems.present_active_participle.as_deref(),
        MetadataField::PresentActiveParticipleStem,
    )?;
    // Shim over the merged participle stem kernel (OCS columns; divergence
    // `verb:present-active-nominative-contraction` separates the Synodal
    // edges).
    let kernel_formation = match formation {
        PresentActiveParticipleFormation::YushtHard => {
            church_slavonic_core::verb_participle::PresentActiveFormation::HardUsht
        }
        PresentActiveParticipleFormation::YushtSoft => {
            church_slavonic_core::verb_participle::PresentActiveFormation::SoftUsht
        }
        PresentActiveParticipleFormation::YeshtSoft => {
            church_slavonic_core::verb_participle::PresentActiveFormation::SoftAsht
        }
        PresentActiveParticipleFormation::MixedYushtSoft => {
            church_slavonic_core::verb_participle::PresentActiveFormation::MixedUsht
        }
        PresentActiveParticipleFormation::IotatedYushtSoft => {
            church_slavonic_core::verb_participle::PresentActiveFormation::IotatedUsht
        }
    };
    const OCS: church_slavonic_core::Recension = church_slavonic_core::Recension::OldChurchSlavonic;
    let oblique_suffix =
        church_slavonic_core::verb_participle::present_active_oblique_suffix(kernel_formation, OCS)
            [0];
    let nominative_suffix = church_slavonic_core::verb_participle::present_active_nominative_edge(
        kernel_formation,
        OCS,
    )[0];
    let (oblique, nominative) = (
        format!("{stem}{oblique_suffix}"),
        format!("{stem}{nominative_suffix}"),
    );
    decline_active_participle(
        lexeme,
        cell,
        &oblique,
        &nominative,
        RuleId::VerbParticiplePresentActive,
        "form the present active participle from the explicit participial stem",
    )
}

fn past_active_participle(
    lexeme: &VerbLexeme,
    cell: ParticipleCell,
) -> Result<PredictedForm, InflectionError> {
    let formation = lexeme.formations.past_active_participle.ok_or_else(|| {
        InflectionError::MissingLexicalMetadata {
            needed: vec![MetadataField::PastActiveParticipleFormation],
        }
    })?;
    let stem = required_stem(
        lexeme.stems.past_active_participle.as_deref(),
        MetadataField::PastActiveParticipleStem,
    )?;
    // Shim over the merged participle stem kernel (OCS columns): the
    // family owns the ов→оу base transformation; the suffixes and
    // citation edges come from the kernel.
    let kernel_parts = |formation: church_slavonic_core::verb_participle::PastActiveFormation| {
        const OCS: church_slavonic_core::Recension =
            church_slavonic_core::Recension::OldChurchSlavonic;
        (
            church_slavonic_core::verb_participle::past_active_oblique_suffix(formation, OCS)[0],
            church_slavonic_core::verb_participle::past_active_nominative_edge(formation, OCS)[0],
        )
    };
    let (base, suffix, nominative_suffix, reason) = match formation {
        PastActiveParticipleFormation::Ush => {
            let (suffix, edge) = kernel_parts(
                church_slavonic_core::verb_participle::PastActiveFormation::ConsonantHard,
            );
            (
                stem,
                suffix,
                edge,
                "attach the hard -ъш- past-active suffix to the explicit consonant stem",
            )
        }
        PastActiveParticipleFormation::Ish => {
            let (suffix, edge) =
                kernel_parts(church_slavonic_core::verb_participle::PastActiveFormation::SoftI);
            (
                stem,
                suffix,
                edge,
                "attach the fronted -ьш- suffix to the explicitly transformed i-stem",
            )
        }
        PastActiveParticipleFormation::IshAfterGlide => {
            let (suffix, edge) =
                kernel_parts(church_slavonic_core::verb_participle::PastActiveFormation::GlideI);
            (
                stem,
                suffix,
                edge,
                "realize final j or i-glide before the fronted -ьш- past-active suffix",
            )
        }
        PastActiveParticipleFormation::VushAfterJDeletion => {
            let (suffix, edge) =
                kernel_parts(church_slavonic_core::verb_participle::PastActiveFormation::Vowel);
            (
                stem,
                suffix,
                edge,
                "apply the declared final-j deletion and attach the -въш- suffix",
            )
        }
        PastActiveParticipleFormation::VushAfterOvToU => {
            let Some(base) = stem.strip_suffix("ов") else {
                return Err(InflectionError::InvalidInput {
                    reason: "the ov-to-u past-active formation requires a stem ending in -ов"
                        .to_string(),
                });
            };
            let (suffix, edge) =
                kernel_parts(church_slavonic_core::verb_participle::PastActiveFormation::Vowel);
            (
                format!("{base}оу"),
                suffix,
                edge,
                "change final -ов to -оу and attach the -въш- suffix",
            )
        }
        PastActiveParticipleFormation::Vush => {
            let (suffix, edge) =
                kernel_parts(church_slavonic_core::verb_participle::PastActiveFormation::Vowel);
            (
                stem,
                suffix,
                edge,
                "attach the -въш- suffix to the explicit vowel stem",
            )
        }
    };
    let oblique = format!("{base}{suffix}");
    let nominative = format!("{base}{nominative_suffix}");
    decline_active_participle(
        lexeme,
        cell,
        &oblique,
        &nominative,
        RuleId::VerbParticiplePastActive,
        reason,
    )
}

fn present_passive_participle(
    lexeme: &VerbLexeme,
    cell: ParticipleCell,
) -> Result<PredictedForm, InflectionError> {
    let formation = lexeme
        .formations
        .present_passive_participle
        .ok_or_else(|| InflectionError::MissingLexicalMetadata {
            needed: vec![MetadataField::PresentPassiveParticipleFormation],
        })?;
    let stem = required_stem(
        lexeme.stems.present_passive_participle.as_deref(),
        MetadataField::PresentPassiveParticipleStem,
    )?;
    // Shim over the merged present-passive suffix kernel (OCS column).
    let kernel_formation = match formation {
        PresentPassiveParticipleFormation::Im => {
            church_slavonic_core::verb_participle::PresentPassiveFormation::Im
        }
        PresentPassiveParticipleFormation::Em => {
            church_slavonic_core::verb_participle::PresentPassiveFormation::Em
        }
        PresentPassiveParticipleFormation::IotatedEm => {
            church_slavonic_core::verb_participle::PresentPassiveFormation::IotatedEm
        }
        PresentPassiveParticipleFormation::Om => {
            church_slavonic_core::verb_participle::PresentPassiveFormation::Om
        }
    };
    let suffix = church_slavonic_core::verb_participle::present_passive_suffix(
        kernel_formation,
        church_slavonic_core::Recension::OldChurchSlavonic,
    )[0];
    let adjectival_stem = format!("{stem}{suffix}");
    decline_passive_participle(
        lexeme,
        cell,
        &adjectival_stem,
        RuleId::VerbParticiplePresentPassive,
        "form the present passive participle from the explicit present-system stem",
    )
}

fn past_passive_participle(
    lexeme: &VerbLexeme,
    cell: ParticipleCell,
) -> Result<PredictedForm, InflectionError> {
    let formation = lexeme.formations.past_passive_participle.ok_or_else(|| {
        InflectionError::MissingLexicalMetadata {
            needed: vec![MetadataField::PastPassiveParticipleFormation],
        }
    })?;
    let stem = required_stem(
        lexeme.stems.past_passive_participle.as_deref(),
        MetadataField::PastPassiveParticipleStem,
    )?;
    // Shim over the merged past-passive suffix kernel (shared inventory).
    let kernel_formation = match formation {
        PastPassiveParticipleFormation::T => {
            church_slavonic_core::verb_participle::PastPassiveFormation::T
        }
        PastPassiveParticipleFormation::N => {
            church_slavonic_core::verb_participle::PastPassiveFormation::N
        }
        PastPassiveParticipleFormation::En => {
            church_slavonic_core::verb_participle::PastPassiveFormation::En
        }
    };
    let suffix = church_slavonic_core::verb_participle::past_passive_suffix(
        kernel_formation,
        church_slavonic_core::Recension::OldChurchSlavonic,
    )[0];
    let adjectival_stem = format!("{stem}{suffix}");
    decline_passive_participle(
        lexeme,
        cell,
        &adjectival_stem,
        RuleId::VerbParticiplePastPassive,
        "form the past passive participle from the explicit infinitive-system stem",
    )
}

fn decline_passive_participle(
    lexeme: &VerbLexeme,
    cell: ParticipleCell,
    adjectival_stem: &str,
    rule_id: RuleId,
    formation_reason: &'static str,
) -> Result<PredictedForm, InflectionError> {
    let agreed =
        crate::adjective::decline_stem(adjectival_stem, AdjectiveClass::Hard, cell.adjective)?;
    Ok(participle_result(
        lexeme,
        adjectival_stem,
        agreed.text,
        agreed.trace,
        rule_id,
        formation_reason,
    ))
}

fn decline_active_participle(
    lexeme: &VerbLexeme,
    cell: ParticipleCell,
    oblique_stem: &str,
    nominative_mn_singular: &str,
    rule_id: RuleId,
    formation_reason: &'static str,
) -> Result<PredictedForm, InflectionError> {
    use AdjectiveForm::Short;
    use Gender::{Feminine, Masculine, Neuter};
    use Number::{Plural, Singular};

    let agreement = cell.adjective;
    let special = match (
        agreement.form,
        agreement.case,
        agreement.number,
        agreement.gender,
    ) {
        (Short, Case::Nominative | Case::Vocative, Singular, Masculine | Neuter) => {
            Some(nominative_mn_singular.to_string())
        }
        (Short, Case::Nominative | Case::Vocative, Singular, Feminine) => {
            Some(format!("{oblique_stem}и"))
        }
        (Short, Case::Nominative | Case::Vocative, Plural, Masculine) => {
            Some(format!("{oblique_stem}е"))
        }
        _ => None,
    };
    let (text, agreement_trace) = match special {
        Some(text) => {
            let trace = vec![RuleStep {
                rule_id,
                before: oblique_stem.to_string(),
                after: text.clone(),
                reason: "apply the source-described active-participle nominative seam",
            }];
            (text, trace)
        }
        None => {
            let agreed =
                crate::adjective::decline_stem(oblique_stem, AdjectiveClass::Soft, agreement)?;
            (agreed.text, agreed.trace)
        }
    };
    Ok(participle_result(
        lexeme,
        oblique_stem,
        text,
        agreement_trace,
        rule_id,
        formation_reason,
    ))
}

fn participle_result(
    lexeme: &VerbLexeme,
    adjectival_stem: &str,
    text: String,
    agreement_trace: Vec<RuleStep>,
    rule_id: RuleId,
    formation_reason: &'static str,
) -> PredictedForm {
    PredictedForm {
        text: text.clone(),
        rule_id,
        trace: std::iter::once(RuleStep {
            rule_id,
            before: lexeme.lemma.clone(),
            after: adjectival_stem.to_string(),
            reason: formation_reason,
        })
        .chain(agreement_trace)
        .collect(),
    }
}

fn imperfect_personal_ending(cell: FiniteVerbCell) -> &'static str {
    // Shim over the merged imperfect personal-ending kernel (OCS column;
    // divergences `verb:imperfect-hardening` and `verb:dual-first-person-va`
    // separate the Synodal column).
    church_slavonic_core::verb_past::imperfect_personal_ending(
        cell.person,
        cell.number,
        church_slavonic_core::Recension::OldChurchSlavonic,
    )[0]
}

fn ocs_aorist_ending(
    series: church_slavonic_core::verb_past::AoristSeries,
    cell: FiniteVerbCell,
) -> &'static str {
    // Shim over the merged aorist kernel (OCS column; divergence
    // `verb:aorist-inventory` names the per-recension series asymmetry).
    church_slavonic_core::verb_past::aorist_ending(
        series,
        cell.person,
        cell.number,
        church_slavonic_core::Recension::OldChurchSlavonic,
    )[0]
}

fn asigmatic_aorist_cell(stem: &str, cell: FiniteVerbCell) -> (String, &'static str) {
    // The family owns the second/third palatalization seam; the endings
    // come from the kernel's asigmatic column.
    let palatalized = matches!(
        (cell.person, cell.number),
        (Person::Second | Person::Third, Number::Singular)
            | (Person::Second | Person::Third, Number::Dual)
            | (Person::Second, Number::Plural)
    );
    let stem = if palatalized {
        first_palatalize(stem)
    } else {
        stem.to_string()
    };
    (
        stem,
        ocs_aorist_ending(
            church_slavonic_core::verb_past::AoristSeries::Asigmatic,
            cell,
        ),
    )
}

fn new_aorist_cell(stem: &str, cell: FiniteVerbCell) -> (String, &'static str) {
    let palatalized = matches!(
        (cell.person, cell.number),
        (Person::Second | Person::Third, Number::Singular)
    );
    let stem = if palatalized {
        first_palatalize(stem)
    } else {
        stem.to_string()
    };
    (
        stem,
        ocs_aorist_ending(church_slavonic_core::verb_past::AoristSeries::New, cell),
    )
}

fn sigmatic_aorist_ending(kind: SigmaticKind, cell: FiniteVerbCell) -> &'static str {
    // Shim over the merged aorist kernel (OCS columns of the sigmatic
    // series; the second/third singular is the genuine zero ending, filled
    // family-side by the supplied syncretic principal part).
    let series = match kind {
        SigmaticKind::Primary => church_slavonic_core::verb_past::AoristSeries::SigmaticPrimary,
        SigmaticKind::Secondary => church_slavonic_core::verb_past::AoristSeries::SigmaticSecondary,
        SigmaticKind::VowelStem => church_slavonic_core::verb_past::AoristSeries::SigmaticVowel,
    };
    ocs_aorist_ending(series, cell)
}

fn required_stem(value: Option<&str>, field: MetadataField) -> Result<String, InflectionError> {
    let value = value.ok_or_else(|| InflectionError::MissingLexicalMetadata {
        needed: vec![field],
    })?;
    crate::orthography::canonical_display(value)
}

fn ends_in_ocs_vowel(stem: &str) -> bool {
    stem.chars().last().is_some_and(|last| {
        matches!(
            last,
            'а' | 'е'
                | 'є'
                | 'и'
                | 'і'
                | 'ї'
                | 'о'
                | 'ѡ'
                | 'ꙋ'
                | 'у'
                | 'ы'
                | 'ꙑ'
                | 'ь'
                | 'ъ'
                | 'ѣ'
                | 'ю'
                | 'ꙗ'
                | 'ѧ'
                | 'ѫ'
                | 'ѩ'
                | 'ѭ'
        )
    })
}

fn ends_in_morphologically_soft_consonant(stem: &str) -> bool {
    stem.ends_with(['ч', 'ж', 'ш', 'щ', '҄']) || stem.ends_with("жд")
}

fn has_ocs_infinitive_ending(lemma: &str) -> bool {
    lemma.ends_with("ти") || lemma.ends_with("щи")
}

fn join(stem: &str, ending: &str, rule_id: RuleId, reason: &'static str) -> PredictedForm {
    let text = format!("{stem}{ending}");
    single_step(stem, &text, rule_id, reason)
}

fn single_step(before: &str, after: &str, rule_id: RuleId, reason: &'static str) -> PredictedForm {
    PredictedForm {
        text: after.to_string(),
        rule_id,
        trace: vec![RuleStep {
            rule_id,
            before: before.to_string(),
            after: after.to_string(),
            reason,
        }],
    }
}

fn first_palatalize(stem: &str) -> String {
    replace_final(stem, [('к', "ч"), ('г', "ж"), ('х', "ш")])
}

fn replace_final<const N: usize>(stem: &str, replacements: [(char, &str); N]) -> String {
    let Some(last) = stem.chars().last() else {
        return String::new();
    };
    let Some((_, replacement)) = replacements.iter().find(|(from, _)| *from == last) else {
        return stem.to_string();
    };
    let prefix_len = stem.len() - last.len_utf8();
    format!("{}{replacement}", &stem[..prefix_len])
}

fn present_ending(
    lemma: &str,
    class: VerbClass,
    cell: FiniteVerbCell,
) -> Result<(&'static str, RuleId), InflectionError> {
    // Shim over the merged present kernel (OCS columns): IA classes read
    // the hard first-conjugation series, II classes the soft
    // second-conjugation series.
    let first = matches!(class, VerbClass::IA1 | VerbClass::IA2);
    let second = matches!(class, VerbClass::II1 | VerbClass::II2 | VerbClass::II3);
    let rule = match class {
        VerbClass::IA1 => RuleId::VerbIA1,
        VerbClass::IA2 => RuleId::VerbIA2,
        VerbClass::II1 => RuleId::VerbII1,
        VerbClass::II2 => RuleId::VerbII2,
        VerbClass::II3 => RuleId::VerbII3,
        _ => {
            return Err(InflectionError::unsupported(
                lemma,
                RequestedCell::FiniteVerb(cell),
            ));
        }
    };
    let series = if first {
        church_slavonic_core::verb::PresentSeries::FirstHard
    } else if second {
        church_slavonic_core::verb::PresentSeries::SecondSoft
    } else {
        return Err(InflectionError::unsupported(
            lemma,
            RequestedCell::FiniteVerb(cell),
        ));
    };
    let column = church_slavonic_core::verb::present_ending(
        series,
        cell.person,
        cell.number,
        church_slavonic_core::Recension::OldChurchSlavonic,
    );
    match column.first() {
        Some(&ending) => Ok((ending, rule)),
        None => Err(InflectionError::unsupported(
            lemma,
            RequestedCell::FiniteVerb(cell),
        )),
    }
}

fn iotated_e_present_ending(cell: FiniteVerbCell) -> &'static str {
    // Shim over the kernel's iotated first-conjugation OCS column.
    church_slavonic_core::verb::present_ending(
        church_slavonic_core::verb::PresentSeries::FirstIotated,
        cell.person,
        cell.number,
        church_slavonic_core::Recension::OldChurchSlavonic,
    )[0]
}

fn hard_i_present_ending(cell: FiniteVerbCell) -> &'static str {
    // Shim over the kernel's hard-i second-conjugation OCS column.
    church_slavonic_core::verb::present_ending(
        church_slavonic_core::verb::PresentSeries::SecondHardI,
        cell.person,
        cell.number,
        church_slavonic_core::Recension::OldChurchSlavonic,
    )[0]
}

/// The reviewed lexeme profile of one verb lemma: every source-reviewed
/// analysis — the unique-verb identity kernels, the Alypy §104 irregular
/// analyses, and the Polivanova regular source rows — assembled as concrete
/// [`VerbLexeme`] profiles in the reviewed-authority order (unique first,
/// then the irregular analyses, then the regular family rows). This is the
/// pure composition half of the fat facade resolver's `ReviewedVerbProfile`
/// (which the resolver keeps for id/warning plumbing); it is the kernel
/// consulted by the pilot facade's declined-participle path.
///
/// Returns `None` when no reviewed family covers the lemma. A reviewed
/// analysis whose lexeme cannot be assembled (a source-invariant violation)
/// is an `Err`, mirroring the resolver's error propagation: the caller must
/// reject the request rather than fall back to another channel.
pub fn reviewed_verb_lexemes(lemma: &str) -> Option<Result<Vec<VerbLexeme>, InflectionError>> {
    use crate::irregular_verb::IrregularVerbFamilyMember;
    use crate::regular_verb::RegularVerbFamily;
    use crate::unique_verb::{UniqueVerbFamilyMember, UniqueVerbIdentity};

    let unique = UniqueVerbFamilyMember::classify_source_union_lemma(lemma).or_else(|| {
        UniqueVerbIdentity::classify_source_union_lemma(lemma).and_then(|identity| {
            UniqueVerbFamilyMember::classify_source_union_lemma(identity.canonical_lemma())
        })
    });
    let irregular = IrregularVerbFamilyMember::classify_source_lemma(lemma);
    let regular = RegularVerbFamily::classify_source_lemma(lemma);
    if unique.is_none() && irregular.is_none() && regular.is_none() {
        return None;
    }
    let mut lexemes: Vec<VerbLexeme> = Vec::new();
    if let Some(member) = unique {
        lexemes.push(member.lexeme());
    }
    if let Some(member) = irregular {
        for analysis in member.analyses() {
            match member.lexeme_for_analysis(*analysis) {
                Some(lexeme) => lexemes.push(lexeme),
                None => {
                    return Some(Err(InflectionError::InvalidInput {
                        reason: format!(
                            "reviewed analysis {} does not belong to {}",
                            analysis.code(),
                            member.canonical_lemma()
                        ),
                    }));
                }
            }
        }
    }
    if let Some(family) = regular {
        for member in family.members() {
            match member.lexemes() {
                Ok(member_lexemes) => lexemes.extend(member_lexemes),
                Err(error) => return Some(Err(error)),
            }
        }
    }
    Some(Ok(lexemes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AdjectiveCell, Animacy};

    fn finite_cell(tense: FiniteTense, person: Person, number: Number) -> FiniteVerbCell {
        FiniteVerbCell {
            tense,
            person,
            number,
        }
    }

    fn adjective_cell(
        kind: ParticipleKind,
        form: AdjectiveForm,
        case: Case,
        number: Number,
        gender: Gender,
    ) -> ParticipleCell {
        ParticipleCell {
            kind,
            adjective: AdjectiveCell {
                case,
                number,
                gender,
                animacy: Animacy::Inanimate,
                form,
            },
        }
    }

    fn finite_paradigm(lexeme: &VerbLexeme, tense: FiniteTense) -> Vec<String> {
        Number::ALL
            .into_iter()
            .flat_map(|number| {
                Person::ALL.into_iter().map(move |person| {
                    finite(lexeme, finite_cell(tense, person, number))
                        .expect("complete finite paradigm")
                        .text
                })
            })
            .collect()
    }

    #[test]
    fn explicit_present_allomorph_replaces_automatic_guessing() {
        let mut verb = VerbLexeme::new("правити", VerbClass::II1);
        verb.stems.present = Some("прав".to_string());
        let cell = finite_cell(FiniteTense::Present, Person::First, Number::Singular);
        assert_eq!(
            finite(&verb, cell),
            Err(InflectionError::MissingLexicalMetadata {
                needed: vec![MetadataField::PresentFirstSingularStem]
            })
        );
        verb.stems.present_first_singular = Some("правл".to_string());
        assert_eq!(
            finite(&verb, cell).expect("explicit allomorph").text,
            "правлѭ"
        );
    }

    #[test]
    fn present_surface_formations_and_third_plural_edge_are_complete_and_typed() {
        let mut iotated = VerbLexeme::new("дѣлати", VerbClass::IA1);
        iotated.stems.present = Some("дѣла".to_string());
        iotated.formations.present = Some(PresentFormation::IotatedE);
        assert_eq!(
            finite_paradigm(&iotated, FiniteTense::Present),
            [
                "дѣлаѭ",
                "дѣлаѥши",
                "дѣлаѥтъ",
                "дѣлаѥвѣ",
                "дѣлаѥта",
                "дѣлаѥте",
                "дѣлаѥмъ",
                "дѣлаѥте",
                "дѣлаѭтъ",
            ]
        );

        let mut hard_i = VerbLexeme::new("блажити", VerbClass::II1);
        hard_i.stems.present = Some("блаж".to_string());
        hard_i.stems.present_first_singular = Some("блаж".to_string());
        hard_i.formations.present = Some(PresentFormation::HardI);
        assert_eq!(
            finite_paradigm(&hard_i, FiniteTense::Present),
            [
                "блажѫ",
                "блажиши",
                "блажитъ",
                "блаживѣ",
                "блажита",
                "блажите",
                "блажимъ",
                "блажите",
                "блажѧтъ",
            ]
        );

        let mut edge = VerbLexeme::new("пещи", VerbClass::IA1);
        edge.stems.present = Some("печ".to_string());
        edge.stems.present_third_plural = Some("пек".to_string());
        assert_eq!(
            finite(
                &edge,
                finite_cell(FiniteTense::Present, Person::Third, Number::Plural)
            )
            .expect("independent 3pl allomorph")
            .text,
            "пекѫтъ"
        );

        hard_i.formations.present = Some(PresentFormation::IotatedE);
        assert!(matches!(
            finite(
                &hard_i,
                finite_cell(FiniteTense::Present, Person::Second, Number::Singular)
            ),
            Err(InflectionError::UnsupportedFormation {
                system: MetadataField::PresentFormation,
                ..
            })
        ));
    }

    #[test]
    fn verbal_noun_has_the_complete_soft_neuter_paradigm() {
        // UT OCS Online lesson 8 §36: verbal nouns use the soft neuter jo
        // declension. Potentially unattested cells remain rule-licensed
        // predictions rather than being omitted from the inventory.
        let verb = VerbLexeme::builder("знати", VerbClass::IA1)
            .expect("builder")
            .verbal_noun("знан")
            .expect("independent verbal-noun platform")
            .build();
        let expected = [
            "знаниѥ",
            "знаниꙗ",
            "знанию",
            "знаниѥ",
            "знаниѥмь",
            "знании",
            "знаниѥ",
            "знании",
            "знанию",
            "знаниѥма",
            "знании",
            "знаниѥма",
            "знанию",
            "знании",
            "знаниꙗ",
            "знании",
            "знаниѥмъ",
            "знаниꙗ",
            "знании",
            "знаниихъ",
            "знаниꙗ",
        ];
        let actual = crate::NounCell::all()
            .map(|cell| verbal_noun(&verb, cell).expect("licensed cell").text)
            .collect::<Vec<_>>();
        assert_eq!(actual, expected);
    }

    #[test]
    fn verbal_noun_accepts_independent_or_past_passive_platforms() {
        // Polivanova 2023 §276 n.4 and §865: a derived noun need not imply an
        // attested participle, hence the independent platform.
        let intransitive = VerbLexeme::builder("слути", VerbClass::Root)
            .expect("builder")
            .verbal_noun("слут")
            .expect("independent platform")
            .build();
        let citation = crate::NounCell {
            case: Case::Nominative,
            number: Number::Singular,
        };
        assert_eq!(
            verbal_noun(&intransitive, citation)
                .expect("independent derivation")
                .text,
            "слутиѥ"
        );

        let from_participle = VerbLexeme::builder("знати", VerbClass::IA1)
            .expect("builder")
            .past_passive_participle("зна", PastPassiveParticipleFormation::N)
            .expect("past-passive platform")
            .build();
        assert_eq!(
            verbal_noun(&from_participle, citation)
                .expect("shared platform")
                .text,
            "знаниѥ"
        );

        let missing = VerbLexeme::new("слути", VerbClass::Root);
        assert_eq!(
            verbal_noun(&missing, citation),
            Err(InflectionError::MissingLexicalMetadata {
                needed: vec![MetadataField::VerbalNounStem],
            })
        );
    }

    #[test]
    fn imperfect_has_all_person_number_cells_and_velar_seam() {
        let mut verb = VerbLexeme::new("нести", VerbClass::IA1);
        verb.stems.imperfect = Some("нес".to_string());
        verb.formations.imperfect = Some(ImperfectFormation::YatA);
        verb.formations.imperfect_variant_policy = Some(ImperfectVariantPolicy::UncontractedOnly);
        let expected = [
            "несѣахъ",
            "несѣаше",
            "несѣаше",
            "несѣаховѣ",
            "несѣашета",
            "несѣашете",
            "несѣахомъ",
            "несѣашете",
            "несѣахѫ",
        ];
        assert_eq!(finite_paradigm(&verb, FiniteTense::Imperfect), expected);

        // UT OCS Online, lesson 1, §4.2: the -ах- series.
        verb.stems.imperfect = Some("зна".to_string());
        verb.formations.imperfect = Some(ImperfectFormation::A);
        verb.formations.imperfect_variant_policy = Some(ImperfectVariantPolicy::UncontractedOnly);
        assert_eq!(
            finite_paradigm(&verb, FiniteTense::Imperfect),
            [
                "знаахъ",
                "знааше",
                "знааше",
                "знааховѣ",
                "знаашета",
                "знаашете",
                "знаахомъ",
                "знаашете",
                "знаахѫ",
            ]
        );

        verb.stems.imperfect = Some("мог".to_string());
        verb.formations.imperfect = Some(ImperfectFormation::PalatalizedA);
        verb.formations.imperfect_variant_policy = Some(ImperfectVariantPolicy::UncontractedOnly);
        assert_eq!(
            finite(
                &verb,
                finite_cell(FiniteTense::Imperfect, Person::First, Number::Singular)
            )
            .expect("palatalized imperfect")
            .text,
            "можаахъ"
        );
    }

    #[test]
    fn contracted_imperfect_has_every_person_number_cell_for_each_platform() {
        // Polivanova 2023 §§455, 467–468: contracted terminals omit the loose
        // -а- between the same imperfect platform and the х-/ш-initial ending.
        let mut verb = VerbLexeme::new("нести", VerbClass::IA1);
        verb.stems.imperfect = Some("нес".to_string());
        verb.formations.imperfect = Some(ImperfectFormation::YatA);
        verb.formations.imperfect_variant_policy = Some(ImperfectVariantPolicy::ContractedOnly);
        assert_eq!(
            finite_paradigm(&verb, FiniteTense::Imperfect),
            [
                "несѣхъ",
                "несѣше",
                "несѣше",
                "несѣховѣ",
                "несѣшета",
                "несѣшете",
                "несѣхомъ",
                "несѣшете",
                "несѣхѫ",
            ]
        );
        assert_eq!(
            finite(
                &verb,
                finite_cell(FiniteTense::Imperfect, Person::First, Number::Singular)
            )
            .expect("contracted yat platform")
            .rule_id,
            RuleId::VerbImperfectContractedYatA
        );

        verb.lemma = "вѣровати".to_string();
        verb.stems.imperfect = Some("вѣрова".to_string());
        verb.formations.imperfect = Some(ImperfectFormation::A);
        assert_eq!(
            finite_paradigm(&verb, FiniteTense::Imperfect),
            [
                "вѣровахъ",
                "вѣроваше",
                "вѣроваше",
                "вѣроваховѣ",
                "вѣровашета",
                "вѣровашете",
                "вѣровахомъ",
                "вѣровашете",
                "вѣровахѫ",
            ]
        );
        assert_eq!(
            finite(
                &verb,
                finite_cell(FiniteTense::Imperfect, Person::First, Number::Singular)
            )
            .expect("contracted a platform")
            .rule_id,
            RuleId::VerbImperfectContractedA
        );

        verb.lemma = "мощи".to_string();
        verb.stems.imperfect = Some("мог".to_string());
        verb.formations.imperfect = Some(ImperfectFormation::PalatalizedA);
        assert_eq!(
            finite_paradigm(&verb, FiniteTense::Imperfect),
            [
                "можахъ",
                "можаше",
                "можаше",
                "можаховѣ",
                "можашета",
                "можашете",
                "можахомъ",
                "можашете",
                "можахѫ",
            ]
        );
        assert_eq!(
            finite(
                &verb,
                finite_cell(FiniteTense::Imperfect, Person::Third, Number::Plural)
            )
            .expect("contracted palatalized platform")
            .rule_id,
            RuleId::VerbImperfectContractedPalatalizedA
        );
    }

    #[test]
    fn imperfect_variant_matrix_covers_every_typed_platform_and_cell() {
        // Polivanova 2023 Table 455.2 and §§467–472: every terminal set has
        // the same nine person-number cells. Platform type selects only the
        // explicit seam before those terminals.
        let endings = [
            "хъ", "ше", "ше", "ховѣ", "шета", "шете", "хомъ", "шете", "хѫ",
        ];
        let cases = [
            (
                "вѣровати",
                "вѣрова",
                ImperfectFormation::A,
                ImperfectVariantPolicy::UncontractedOnly,
                "вѣроваа",
            ),
            (
                "вѣровати",
                "вѣрова",
                ImperfectFormation::A,
                ImperfectVariantPolicy::ContractedOnly,
                "вѣрова",
            ),
            (
                "вѣровати",
                "вѣрова",
                ImperfectFormation::A,
                ImperfectVariantPolicy::IotatedOnly,
                "вѣроваꙗ",
            ),
            (
                "нести",
                "нес",
                ImperfectFormation::YatA,
                ImperfectVariantPolicy::UncontractedOnly,
                "несѣа",
            ),
            (
                "нести",
                "нес",
                ImperfectFormation::YatA,
                ImperfectVariantPolicy::ContractedOnly,
                "несѣ",
            ),
            (
                "нести",
                "нес",
                ImperfectFormation::YatA,
                ImperfectVariantPolicy::IotatedOnly,
                "несѣꙗ",
            ),
            (
                "мощи",
                "мог",
                ImperfectFormation::PalatalizedA,
                ImperfectVariantPolicy::UncontractedOnly,
                "можаа",
            ),
            (
                "мощи",
                "мог",
                ImperfectFormation::PalatalizedA,
                ImperfectVariantPolicy::ContractedOnly,
                "можа",
            ),
            (
                "мощи",
                "мог",
                ImperfectFormation::PalatalizedA,
                ImperfectVariantPolicy::IotatedOnly,
                "можаꙗ",
            ),
            (
                "радовати",
                "раду",
                ImperfectFormation::PresentA,
                ImperfectVariantPolicy::UncontractedOnly,
                "радуа",
            ),
            (
                "радовати",
                "раду",
                ImperfectFormation::PresentA,
                ImperfectVariantPolicy::ContractedOnly,
                "раду",
            ),
            (
                "радовати",
                "раду",
                ImperfectFormation::PresentA,
                ImperfectVariantPolicy::IotatedOnly,
                "радуꙗ",
            ),
            (
                "зъвати",
                "зов",
                ImperfectFormation::PresentYatA,
                ImperfectVariantPolicy::UncontractedOnly,
                "зовѣа",
            ),
            (
                "зъвати",
                "зов",
                ImperfectFormation::PresentYatA,
                ImperfectVariantPolicy::ContractedOnly,
                "зовѣ",
            ),
            (
                "зъвати",
                "зов",
                ImperfectFormation::PresentYatA,
                ImperfectVariantPolicy::IotatedOnly,
                "зовѣꙗ",
            ),
        ];

        for (lemma, stem, formation, policy, expected_base) in cases {
            let mut verb = VerbLexeme::new(lemma, VerbClass::IA1);
            verb.stems.imperfect = Some(stem.to_string());
            verb.formations.imperfect = Some(formation);
            verb.formations.imperfect_variant_policy = Some(policy);
            let expected = endings
                .iter()
                .map(|ending| format!("{expected_base}{ending}"))
                .collect::<Vec<_>>();
            assert_eq!(
                finite_paradigm(&verb, FiniteTense::Imperfect),
                expected,
                "{formation:?} with {policy:?}"
            );
        }
    }

    #[test]
    fn present_stem_and_iotated_imperfects_remain_distinct_typed_analyses() {
        // Polivanova 2023 §§469–472: the present imperfect uses the explicit
        // present workstem; the rare iotated series has its own -(ѣ)ꙗ- set.
        let mut verb = VerbLexeme::new("зъвати", VerbClass::IA1);
        verb.stems.imperfect = Some("зов".to_string());
        verb.formations.imperfect = Some(ImperfectFormation::PresentYatA);
        verb.formations.imperfect_variant_policy = Some(ImperfectVariantPolicy::UncontractedOnly);
        assert_eq!(
            finite_paradigm(&verb, FiniteTense::Imperfect),
            [
                "зовѣахъ",
                "зовѣаше",
                "зовѣаше",
                "зовѣаховѣ",
                "зовѣашета",
                "зовѣашете",
                "зовѣахомъ",
                "зовѣашете",
                "зовѣахѫ",
            ]
        );
        assert_eq!(
            finite(
                &verb,
                finite_cell(FiniteTense::Imperfect, Person::Third, Number::Singular)
            )
            .expect("present imperfect")
            .rule_id,
            RuleId::VerbImperfectPresent
        );

        verb.formations.imperfect_variant_policy = Some(ImperfectVariantPolicy::ContractedOnly);
        assert_eq!(
            finite(
                &verb,
                finite_cell(FiniteTense::Imperfect, Person::Third, Number::Singular)
            )
            .expect("contracted present imperfect")
            .text,
            "зовѣше"
        );
        assert_eq!(
            finite(
                &verb,
                finite_cell(FiniteTense::Imperfect, Person::Third, Number::Singular)
            )
            .expect("contracted present imperfect")
            .rule_id,
            RuleId::VerbImperfectPresentContracted
        );

        verb.lemma = "исъхнѫти".to_string();
        verb.stems.imperfect = Some("исъхн".to_string());
        verb.formations.imperfect = Some(ImperfectFormation::YatA);
        verb.formations.imperfect_variant_policy = Some(ImperfectVariantPolicy::IotatedOnly);
        assert_eq!(
            finite_paradigm(&verb, FiniteTense::Imperfect),
            [
                "исъхнѣꙗхъ",
                "исъхнѣꙗше",
                "исъхнѣꙗше",
                "исъхнѣꙗховѣ",
                "исъхнѣꙗшета",
                "исъхнѣꙗшете",
                "исъхнѣꙗхомъ",
                "исъхнѣꙗшете",
                "исъхнѣꙗхѫ",
            ]
        );
        assert_eq!(
            finite(
                &verb,
                finite_cell(FiniteTense::Imperfect, Person::Third, Number::Singular)
            )
            .expect("iotated imperfect")
            .rule_id,
            RuleId::VerbImperfectIotated
        );

        verb.lemma = "трьпѣти".to_string();
        verb.stems.imperfect = Some("трьпѣ".to_string());
        verb.formations.imperfect = Some(ImperfectFormation::A);
        assert_eq!(
            finite(
                &verb,
                finite_cell(FiniteTense::Imperfect, Person::First, Number::Singular)
            )
            .expect("iotated expanded platform")
            .text,
            "трьпѣꙗхъ"
        );

        verb.lemma = "радовати".to_string();
        verb.stems.imperfect = Some("раду".to_string());
        verb.formations.imperfect = Some(ImperfectFormation::PresentA);
        assert_eq!(
            finite(
                &verb,
                finite_cell(FiniteTense::Imperfect, Person::Third, Number::Singular)
            )
            .expect("present iotated vowel platform")
            .text,
            "радуꙗше"
        );
    }

    #[test]
    fn new_and_asigmatic_aorists_keep_formation_separate_from_aspect() {
        let mut verb = VerbLexeme::new("рещи", VerbClass::IA1);
        verb.aspect = Some(VerbAspect::Perfective);
        verb.stems.aorist = Some("рек".to_string());
        verb.formations.aorist = Some(AoristFormation::New);
        assert_eq!(
            finite(
                &verb,
                finite_cell(FiniteTense::Aorist, Person::First, Number::Singular)
            )
            .expect("new aorist")
            .text,
            "рекохъ"
        );
        assert_eq!(
            finite(
                &verb,
                finite_cell(FiniteTense::Aorist, Person::Third, Number::Singular)
            )
            .expect("new aorist palatal seam")
            .text,
            "рече"
        );
        verb.formations.aorist = Some(AoristFormation::Asigmatic);
        assert_eq!(
            finite(
                &verb,
                finite_cell(FiniteTense::Aorist, Person::First, Number::Plural)
            )
            .expect("asigmatic aorist")
            .text,
            "рекомъ"
        );

        // UT OCS Online, lesson 3, §14.3: complete new ox-aorist.
        verb.formations.aorist = Some(AoristFormation::New);
        assert_eq!(
            finite_paradigm(&verb, FiniteTense::Aorist),
            [
                "рекохъ",
                "рече",
                "рече",
                "рекоховѣ",
                "рекоста",
                "рекосте",
                "рекохомъ",
                "рекосте",
                "рекошѧ",
            ]
        );
        verb.stems.aorist = Some("мог".to_string());
        assert_eq!(
            finite_paradigm(&verb, FiniteTense::Aorist),
            [
                "могохъ",
                "може",
                "може",
                "могоховѣ",
                "могоста",
                "могосте",
                "могохомъ",
                "могосте",
                "могошѧ",
            ]
        );

        // The asigmatic formation is selected independently and has its own seam.
        verb.stems.aorist = Some("пек".to_string());
        verb.formations.aorist = Some(AoristFormation::Asigmatic);
        assert_eq!(
            finite_paradigm(&verb, FiniteTense::Aorist),
            [
                "пекъ",
                "пече",
                "пече",
                "пековѣ",
                "печета",
                "печете",
                "пекомъ",
                "печете",
                "пекѫ",
            ]
        );
    }

    #[test]
    fn sigmatic_aorists_use_independent_main_and_singular_principal_parts() {
        // UT OCS Online, lesson 3, §14.2; Polivanova 2023 §§476–480.
        let rese = VerbLexeme::builder("рєшти", VerbClass::IA1)
            .expect("valid lemma")
            .sigmatic_aorist("рѣ", "рєчє", AoristFormation::SigmaticSecondary)
            .expect("source-audited old sigmatic 2 metadata")
            .build();
        assert_eq!(
            finite_paradigm(&rese, FiniteTense::Aorist),
            [
                "рѣхъ",
                "рєчє",
                "рєчє",
                "рѣховѣ",
                "рѣста",
                "рѣсте",
                "рѣхомъ",
                "рѣсте",
                "рѣшѧ",
            ]
        );
        assert_eq!(
            finite(
                &rese,
                finite_cell(FiniteTense::Aorist, Person::First, Number::Singular)
            )
            .expect("old sigmatic 2")
            .rule_id,
            RuleId::VerbAoristSigmaticSecondary
        );

        let vesti = VerbLexeme::builder("вєсти", VerbClass::IA1)
            .expect("valid lemma")
            .sigmatic_aorist("вѣ", "вєдє", AoristFormation::SigmaticPrimary)
            .expect("source-audited old sigmatic 1 metadata")
            .build();
        assert_eq!(
            finite_paradigm(&vesti, FiniteTense::Aorist),
            [
                "вѣсъ",
                "вєдє",
                "вєдє",
                "вѣсовѣ",
                "вѣста",
                "вѣсте",
                "вѣсомъ",
                "вѣсте",
                "вѣсѧ",
            ]
        );
        assert_eq!(
            finite(
                &vesti,
                finite_cell(FiniteTense::Aorist, Person::Third, Number::Plural)
            )
            .expect("old sigmatic 1")
            .rule_id,
            RuleId::VerbAoristSigmaticPrimary
        );

        // The 2sg/3sg subbundle is a complete principal part. This permits the
        // independently attested zero and -тъ variants without combining them.
        for singular in ["ѧ", "ѧтъ"] {
            let yati = VerbLexeme::builder("ѧти", VerbClass::Irregular)
                .expect("valid lemma")
                .sigmatic_aorist("ѧ", singular, AoristFormation::SigmaticPrimary)
                .expect("source-audited singular variant")
                .build();
            assert_eq!(
                finite(
                    &yati,
                    finite_cell(FiniteTense::Aorist, Person::Second, Number::Singular)
                )
                .expect("independent 2sg principal part")
                .text,
                singular
            );
            assert_eq!(
                finite(
                    &yati,
                    finite_cell(FiniteTense::Aorist, Person::Third, Number::Singular)
                )
                .expect("independent 3sg principal part")
                .text,
                singular
            );
        }

        // Polivanova 2023 §§93, 455, 460: the standard aorist terminals
        // select their zero-o allomorph after a vowel-final workstem.
        let znati = VerbLexeme::builder("знати", VerbClass::IA1)
            .expect("valid lemma")
            .sigmatic_aorist("зна", "зна", AoristFormation::SigmaticVowel)
            .expect("source-audited vowel-stem sigmatic metadata")
            .build();
        assert_eq!(
            finite_paradigm(&znati, FiniteTense::Aorist),
            [
                "знахъ",
                "зна",
                "зна",
                "знаховѣ",
                "знаста",
                "знасте",
                "знахомъ",
                "знасте",
                "знашѧ",
            ]
        );
        assert_eq!(
            finite(
                &znati,
                finite_cell(FiniteTense::Aorist, Person::First, Number::Singular)
            )
            .expect("vowel-stem sigmatic")
            .rule_id,
            RuleId::VerbAoristSigmaticVowel
        );
    }

    #[test]
    fn sigmatic_aorists_fail_closed_when_principal_parts_conflict_or_are_missing() {
        assert!(matches!(
            VerbLexeme::builder("рещи", VerbClass::IA1)
                .expect("valid lemma")
                .aorist("рѣ", AoristFormation::SigmaticSecondary),
            Err(InflectionError::InvalidInput { .. })
        ));
        assert!(matches!(
            VerbLexeme::builder("нести", VerbClass::IA1)
                .expect("valid lemma")
                .sigmatic_aorist("нес", "несе", AoristFormation::SigmaticVowel),
            Err(InflectionError::InvalidInput { .. })
        ));
        assert!(matches!(
            VerbLexeme::builder("рещи", VerbClass::IA1)
                .expect("valid lemma")
                .sigmatic_aorist("рѣ", "рече", AoristFormation::New),
            Err(InflectionError::InvalidInput { .. })
        ));

        let mut incomplete = VerbLexeme::new("рещи", VerbClass::IA1);
        incomplete.stems.aorist = Some("рѣ".to_string());
        incomplete.formations.aorist = Some(AoristFormation::SigmaticSecondary);
        assert!(matches!(
            finite(
                &incomplete,
                finite_cell(FiniteTense::Aorist, Person::Third, Number::Singular)
            ),
            Err(InflectionError::MissingLexicalMetadata { needed })
                if needed == vec![MetadataField::AoristSecondThirdSingular]
        ));
        assert_eq!(
            finite(
                &incomplete,
                finite_cell(FiniteTense::Aorist, Person::First, Number::Plural)
            )
            .expect("main subbundle does not need the singular principal part")
            .text,
            "рѣхомъ"
        );
    }

    #[test]
    fn imperative_exposes_only_the_historical_cell_inventory() {
        let mut verb = VerbLexeme::new("нести", VerbClass::IA1);
        verb.stems.imperative = Some("нес".to_string());
        verb.formations.imperative = Some(ImperativeFormation::YatSeries);
        let forms = ImperativeCell::SUPPORTED
            .map(|cell| imperative(&verb, cell).expect("supported imperative").text);
        assert_eq!(
            forms,
            ["неси", "неси", "несѣвѣ", "несѣта", "несѣмъ", "несѣте"]
        );
        verb.stems.imperative = Some("мол".to_string());
        verb.formations.imperative = Some(ImperativeFormation::ISeries);
        let forms = ImperativeCell::SUPPORTED
            .map(|cell| imperative(&verb, cell).expect("supported i-series").text);
        assert_eq!(
            forms,
            ["моли", "моли", "моливѣ", "молита", "молимъ", "молите"]
        );
        assert!(matches!(
            imperative(
                &verb,
                ImperativeCell {
                    person: Person::Third,
                    number: Number::Plural
                }
            ),
            Err(InflectionError::HistoricallyInvalidCell { .. })
        ));
    }

    #[test]
    fn all_four_participles_form_stems_and_share_adjective_agreement() {
        let mut verb = VerbLexeme::new("нести", VerbClass::IA1);
        verb.stems.present_active_participle = Some("нес".to_string());
        verb.formations.present_active_participle =
            Some(PresentActiveParticipleFormation::YushtHard);
        verb.stems.present_passive_participle = Some("нес".to_string());
        verb.formations.present_passive_participle = Some(PresentPassiveParticipleFormation::Om);
        verb.stems.past_active_participle = Some("нес".to_string());
        verb.formations.past_active_participle = Some(PastActiveParticipleFormation::Ush);
        verb.stems.past_passive_participle = Some("нес".to_string());
        verb.formations.past_passive_participle = Some(PastPassiveParticipleFormation::En);

        assert_eq!(
            participle(
                &verb,
                adjective_cell(
                    ParticipleKind::PresentActive,
                    AdjectiveForm::Short,
                    Case::Nominative,
                    Number::Singular,
                    Gender::Masculine
                )
            )
            .expect("present active")
            .text,
            "несꙑ"
        );
        assert_eq!(
            participle(
                &verb,
                adjective_cell(
                    ParticipleKind::PresentPassive,
                    AdjectiveForm::Long,
                    Case::Dative,
                    Number::Dual,
                    Gender::Feminine
                )
            )
            .expect("present passive")
            .text,
            "несомꙑима"
        );
        assert_eq!(
            participle(
                &verb,
                adjective_cell(
                    ParticipleKind::PastActive,
                    AdjectiveForm::Short,
                    Case::Genitive,
                    Number::Singular,
                    Gender::Masculine
                )
            )
            .expect("past active")
            .text,
            "несъша"
        );
        verb.stems.past_active_participle = Some("правл".to_string());
        verb.formations.past_active_participle = Some(PastActiveParticipleFormation::Ish);
        assert_eq!(
            participle(
                &verb,
                adjective_cell(
                    ParticipleKind::PastActive,
                    AdjectiveForm::Short,
                    Case::Genitive,
                    Number::Singular,
                    Gender::Masculine
                )
            )
            .expect("transformed i-stem past active")
            .text,
            "правльша"
        );
        verb.stems.past_active_participle = Some("дѣла".to_string());
        verb.formations.past_active_participle =
            Some(PastActiveParticipleFormation::VushAfterJDeletion);
        assert_eq!(
            participle(
                &verb,
                adjective_cell(
                    ParticipleKind::PastActive,
                    AdjectiveForm::Short,
                    Case::Genitive,
                    Number::Singular,
                    Gender::Masculine
                )
            )
            .expect("j-deleting past active")
            .text,
            "дѣлавъша"
        );
        verb.stems.past_active_participle = Some("плов".to_string());
        verb.formations.past_active_participle =
            Some(PastActiveParticipleFormation::VushAfterOvToU);
        assert_eq!(
            participle(
                &verb,
                adjective_cell(
                    ParticipleKind::PastActive,
                    AdjectiveForm::Short,
                    Case::Genitive,
                    Number::Singular,
                    Gender::Masculine
                )
            )
            .expect("ov-to-u past active")
            .text,
            "плоувъша"
        );
        verb.stems.past_active_participle = Some("дѣла".to_string());
        assert!(matches!(
            participle(
                &verb,
                adjective_cell(
                    ParticipleKind::PastActive,
                    AdjectiveForm::Short,
                    Case::Genitive,
                    Number::Singular,
                    Gender::Masculine
                )
            ),
            Err(InflectionError::InvalidInput { .. })
        ));
        verb.stems.past_active_participle = Some("нес".to_string());
        verb.formations.past_active_participle = Some(PastActiveParticipleFormation::Ush);
        assert_eq!(
            participle(
                &verb,
                adjective_cell(
                    ParticipleKind::PastPassive,
                    AdjectiveForm::Short,
                    Case::Nominative,
                    Number::Singular,
                    Gender::Masculine
                )
            )
            .expect("past passive")
            .text,
            "несенъ"
        );

        // Every agreement cell is delegated to the shared adjective owner after
        // the source-cited participial stem is formed (UT lessons 6 and 7).
        for kind in [
            ParticipleKind::PresentActive,
            ParticipleKind::PresentPassive,
            ParticipleKind::PastActive,
            ParticipleKind::PastPassive,
        ] {
            for form in AdjectiveForm::ALL {
                for number in Number::ALL {
                    for case in Case::ALL {
                        for gender in Gender::ALL {
                            for animacy in [Animacy::Inanimate, Animacy::Animate] {
                                let adjective = AdjectiveCell {
                                    case,
                                    number,
                                    gender,
                                    animacy,
                                    form,
                                };
                                let result = participle(&verb, ParticipleCell { kind, adjective })
                                    .expect("all adjective agreement cells are supported");
                                assert_eq!(result.trace.len(), 2);
                            }
                        }
                    }
                }
            }
        }

        let long_oblique = AdjectiveCell {
            case: Case::Genitive,
            number: Number::Plural,
            gender: Gender::Masculine,
            animacy: Animacy::Inanimate,
            form: AdjectiveForm::Long,
        };
        let active = participle(
            &verb,
            ParticipleCell {
                kind: ParticipleKind::PresentActive,
                adjective: long_oblique,
            },
        )
        .expect("long active participle");
        let adjective =
            crate::adjective::decline_stem("несѫшт", AdjectiveClass::Soft, long_oblique)
                .expect("shared soft adjective agreement");
        assert_eq!(active.text, adjective.text);
        assert_eq!(active.trace[1].rule_id, RuleId::AdjectiveSoftLong);
    }

    #[test]
    fn irregular_exact_cells_precede_typed_defects_and_productive_fallback() {
        let exact = finite_cell(FiniteTense::Present, Person::First, Number::Singular);
        let unsupported = finite_cell(FiniteTense::Present, Person::Second, Number::Singular);
        let verb = VerbLexeme::builder("бꙑти", VerbClass::Irregular)
            .expect("valid unique verb")
            .exact_form(VerbMorphologyCell::Finite(exact), "ѥсмь")
            .expect("source-reviewed exact present")
            .defective_system(
                VerbMorphologySystem::Finite(FiniteTense::Present),
                VerbDefectKind::UnattestedUnreconstructable,
            )
            .build();

        let realized = finite(&verb, exact).expect("exact cell wins over system defect");
        assert_eq!(realized.text, "ѥсмь");
        assert_eq!(realized.rule_id, RuleId::VerbIrregularExact);
        assert!(matches!(
            finite(&verb, unsupported),
            Err(InflectionError::UnattestedUnreconstructableCell {
                cell: RequestedCell::FiniteVerb(cell),
                ..
            }) if cell == unsupported
        ));

        let invalid = ImperativeCell {
            person: Person::First,
            number: Number::Singular,
        };
        assert!(
            VerbLexeme::builder("бꙑти", VerbClass::Irregular)
                .expect("valid unique verb")
                .exact_form(VerbMorphologyCell::Imperative(invalid), "бꙑмь")
                .is_err()
        );

        let mut direct = VerbLexeme::new("бꙑти", VerbClass::Irregular);
        direct
            .exact_forms
            .insert(VerbMorphologyCell::Imperative(invalid), "бꙑмь".to_string());
        assert!(matches!(
            imperative(&direct, invalid),
            Err(InflectionError::HistoricallyInvalidCell { .. })
        ));
        direct
            .exact_forms
            .insert(VerbMorphologyCell::Finite(exact), "\0".to_string());
        assert!(matches!(
            finite(&direct, exact),
            Err(InflectionError::InvalidInput { .. })
        ));
    }

    #[test]
    fn irregular_defect_kinds_and_l_participle_stem_are_independent() {
        let l_cell = LParticipleCell {
            gender: Gender::Masculine,
            number: Number::Singular,
        };
        let mut iti = VerbLexeme::new("ити", VerbClass::Irregular);
        iti.stems.aorist = Some("ид".to_string());
        iti.stems.l_participle = Some("шь".to_string());
        assert_eq!(
            l_participle(&iti, l_cell)
                .expect("independent suppletive l-participle stem")
                .text,
            "шьлъ"
        );

        let mut defective = VerbLexeme::new("довьлѣти", VerbClass::Irregular);
        defective.defective_systems.insert(
            VerbMorphologySystem::Imperative,
            VerbDefectKind::HistoricallyInvalid,
        );
        let cell = ImperativeCell {
            person: Person::Second,
            number: Number::Singular,
        };
        assert!(matches!(
            imperative(&defective, cell),
            Err(InflectionError::HistoricallyInvalidCell {
                cell: RequestedCell::Imperative(requested),
                ..
            }) if requested == cell
        ));

        let missing = VerbLexeme::new("нести", VerbClass::IA1);
        assert_eq!(
            l_participle(&missing, l_cell),
            Err(InflectionError::MissingLexicalMetadata {
                needed: vec![MetadataField::LParticipleStem]
            })
        );
    }

    #[test]
    fn missing_and_hostile_metadata_fail_without_panicking() {
        let verb = VerbLexeme::new("нести", VerbClass::IA1);
        assert_eq!(
            finite(
                &verb,
                finite_cell(FiniteTense::Imperfect, Person::First, Number::Singular)
            ),
            Err(InflectionError::MissingLexicalMetadata {
                needed: vec![MetadataField::ImperfectFormation]
            })
        );
        let mut missing_policy = VerbLexeme::new("нести", VerbClass::IA1);
        missing_policy.stems.imperfect = Some("нес".to_string());
        missing_policy.formations.imperfect = Some(ImperfectFormation::YatA);
        assert_eq!(
            finite(
                &missing_policy,
                finite_cell(FiniteTense::Imperfect, Person::First, Number::Singular)
            ),
            Err(InflectionError::MissingLexicalMetadata {
                needed: vec![MetadataField::ImperfectVariantPolicy]
            })
        );
        let root = VerbLexeme::new("бꙋти", VerbClass::Root);
        assert!(matches!(
            finite(
                &root,
                finite_cell(FiniteTense::Present, Person::First, Number::Singular)
            ),
            Err(InflectionError::UnsupportedFormation {
                system: MetadataField::VerbClass,
                formation,
            }) if formation == "Root"
        ));
        let mut hostile = VerbLexeme::new("нести", VerbClass::IA1);
        hostile.stems.imperfect = Some("\0".to_string());
        hostile.formations.imperfect = Some(ImperfectFormation::A);
        hostile.formations.imperfect_variant_policy =
            Some(ImperfectVariantPolicy::UncontractedOnly);
        assert!(matches!(
            finite(
                &hostile,
                finite_cell(FiniteTense::Imperfect, Person::First, Number::Singular)
            ),
            Err(InflectionError::InvalidInput { .. })
        ));
        assert!(
            VerbLexeme::builder("правити", VerbClass::II1)
                .expect("valid Cyrillic lemma")
                .present("prav", None)
                .is_err()
        );
        assert!(
            VerbLexeme::builder("пловати", VerbClass::IA1)
                .expect("valid Cyrillic lemma")
                .past_active_participle("пла", PastActiveParticipleFormation::VushAfterOvToU,)
                .is_err()
        );
        let regular = VerbLexeme::new("нести", VerbClass::IA1);
        assert_eq!(
            infinitive(&regular).expect("regular infinitive").text,
            "нести"
        );
        assert_eq!(supine(&regular).expect("hard supine").text, "нестъ");
        let velar = VerbLexeme::new("рещи", VerbClass::IA1);
        assert_eq!(
            infinitive(&velar).expect("class-4c infinitive").text,
            "рещи"
        );
        assert_eq!(
            supine(&velar).expect("fronted class-4c supine").text,
            "рещь"
        );
        let non_infinitive = VerbLexeme::new("такси", VerbClass::IA1);
        assert!(infinitive(&non_infinitive).is_err());
        assert!(supine(&non_infinitive).is_err());
    }
}
