//! Explicit-principal-part Old Church Slavonic verb morphology.

use crate::{
    AdjectiveClass, AdjectiveForm, AoristFormation, Case, FiniteTense, FiniteVerbCell, Gender,
    ImperativeCell, ImperativeFormation, ImperfectFormation, ImperfectVariantPolicy,
    InflectionError, LParticipleCell, MetadataField, Number, ParticipleCell, ParticipleKind,
    PastActiveParticipleFormation, PastPassiveParticipleFormation, Person, PredictedForm,
    PresentActiveParticipleFormation, PresentPassiveParticipleFormation, RequestedCell, RuleId,
    RuleStep, VerbAspect, VerbClass,
};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VerbStems {
    pub present: Option<String>,
    pub present_first_singular: Option<String>,
    pub imperfect: Option<String>,
    pub aorist: Option<String>,
    /// Independently supplied complete 2sg/3sg aorist principal part.
    ///
    /// Sigmatic aorists use a separate singular subbundle, including lexical
    /// zero, `-тъ`, and `-стъ` realizations, so this value is never derived from
    /// the main sigmatic stem.
    pub aorist_second_third_singular: Option<String>,
    pub imperative: Option<String>,
    pub present_active_participle: Option<String>,
    pub present_passive_participle: Option<String>,
    pub past_active_participle: Option<String>,
    pub past_passive_participle: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VerbFormations {
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
}

impl VerbLexeme {
    pub fn new(lemma: impl Into<String>, class: VerbClass) -> Self {
        Self {
            lemma: lemma.into(),
            class,
            aspect: None,
            stems: VerbStems::default(),
            formations: VerbFormations::default(),
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

pub fn finite(lexeme: &VerbLexeme, cell: FiniteVerbCell) -> Result<PredictedForm, InflectionError> {
    crate::orthography::canonical_display(&lexeme.lemma)?;
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
    let ending = match (formation, cell.person, cell.number) {
        (_, Person::Second | Person::Third, Number::Singular) => "и",
        (ImperativeFormation::ISeries, Person::First, Number::Dual) => "ивѣ",
        (ImperativeFormation::ISeries, Person::Second, Number::Dual) => "ита",
        (ImperativeFormation::ISeries, Person::First, Number::Plural) => "имъ",
        (ImperativeFormation::ISeries, Person::Second, Number::Plural) => "ите",
        (ImperativeFormation::YatSeries, Person::First, Number::Dual) => "ѣвѣ",
        (ImperativeFormation::YatSeries, Person::Second, Number::Dual) => "ѣта",
        (ImperativeFormation::YatSeries, Person::First, Number::Plural) => "ѣмъ",
        (ImperativeFormation::YatSeries, Person::Second, Number::Plural) => "ѣте",
        _ => {
            return Err(InflectionError::historically_invalid(
                &lexeme.lemma,
                RequestedCell::Imperative(cell),
            ));
        }
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
    if !lemma.ends_with("ти") || lemma.len() <= "ти".len() {
        return Err(InflectionError::InvalidInput {
            reason: "an OCS infinitive citation must end in ти".to_string(),
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
    let stem = lemma
        .strip_suffix("ти")
        .filter(|stem| !stem.is_empty())
        .ok_or_else(|| InflectionError::InvalidInput {
            reason: "a regularly derived supine needs an infinitive ending in ти".to_string(),
        })?;
    let text = format!("{stem}тъ");
    Ok(single_step(
        &lemma,
        &text,
        RuleId::VerbSupine,
        "replace the regular infinitive ending ти with the supine ending тъ",
    ))
}

pub fn l_participle(
    lexeme: &VerbLexeme,
    cell: LParticipleCell,
) -> Result<PredictedForm, InflectionError> {
    crate::orthography::canonical_display(&lexeme.lemma)?;
    let stem = required_stem(lexeme.stems.aorist.as_deref(), MetadataField::AoristStem)?;
    let ending = match (cell.gender, cell.number) {
        (Gender::Masculine, Number::Singular) => "лъ",
        (Gender::Feminine, Number::Singular) => "ла",
        (Gender::Neuter, Number::Singular) => "ло",
        (Gender::Masculine, Number::Dual) => "ла",
        (Gender::Feminine | Gender::Neuter, Number::Dual) => "лѣ",
        (Gender::Masculine, Number::Plural) => "ли",
        (Gender::Feminine, Number::Plural) => "лꙑ",
        (Gender::Neuter, Number::Plural) => "ла",
    };
    Ok(join(
        &stem,
        ending,
        RuleId::VerbLParticiple,
        "attach the l-participle agreement ending to the explicit aorist stem",
    ))
}

pub fn participle(
    lexeme: &VerbLexeme,
    cell: ParticipleCell,
) -> Result<PredictedForm, InflectionError> {
    crate::orthography::canonical_display(&lexeme.lemma)?;
    match cell.kind {
        ParticipleKind::PresentActive => present_active_participle(lexeme, cell),
        ParticipleKind::PresentPassive => present_passive_participle(lexeme, cell),
        ParticipleKind::PastActive => past_active_participle(lexeme, cell),
        ParticipleKind::PastPassive => past_passive_participle(lexeme, cell),
    }
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
    } else {
        default_stem
    };
    let (ending, rule_id) = present_ending(&lexeme.lemma, lexeme.class, cell)?;
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
    let ImperfectVariantPolicy::UncontractedOnly = variant_policy;
    let stem = required_stem(
        lexeme.stems.imperfect.as_deref(),
        MetadataField::ImperfectStem,
    )?;
    let (stem, marker, rule_id) = match formation {
        ImperfectFormation::A => (stem, "а", RuleId::VerbImperfectA),
        ImperfectFormation::YatA => (stem, "ѣа", RuleId::VerbImperfectYatA),
        ImperfectFormation::PalatalizedA => {
            let changed = first_palatalize(&stem);
            if changed == stem {
                return Err(InflectionError::InvalidInput {
                    reason: "the palatalized imperfect formation requires a final velar stem"
                        .to_string(),
                });
            }
            (changed, "аа", RuleId::VerbImperfectPalatalizedA)
        }
    };
    let personal = imperfect_personal_ending(cell);
    let ending = format!("{marker}{personal}");
    Ok(join(
        &stem,
        &ending,
        rule_id,
        "attach the explicitly selected imperfect marker and personal ending",
    ))
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
    let (oblique, nominative) = match formation {
        PresentActiveParticipleFormation::YushtHard => (format!("{stem}ѫшт"), format!("{stem}ꙑ")),
        PresentActiveParticipleFormation::YushtSoft => (format!("{stem}ѫшт"), format!("{stem}ѩ")),
        PresentActiveParticipleFormation::YeshtSoft => (format!("{stem}ѧшт"), format!("{stem}ѧ")),
    };
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
    let (base, suffix, nominative_suffix, reason) = match formation {
        PastActiveParticipleFormation::Ush => (
            stem,
            "ъш",
            "ъ",
            "attach the hard -ъш- past-active suffix to the explicit consonant stem",
        ),
        PastActiveParticipleFormation::Ish => (
            stem,
            "ьш",
            "ь",
            "attach the fronted -ьш- suffix to the explicitly transformed i-stem",
        ),
        PastActiveParticipleFormation::VushAfterJDeletion => (
            stem,
            "въш",
            "въ",
            "apply the declared final-j deletion and attach the -въш- suffix",
        ),
        PastActiveParticipleFormation::VushAfterOvToU => {
            let Some(base) = stem.strip_suffix("ов") else {
                return Err(InflectionError::InvalidInput {
                    reason: "the ov-to-u past-active formation requires a stem ending in -ов"
                        .to_string(),
                });
            };
            (
                format!("{base}оу"),
                "въш",
                "въ",
                "change final -ов to -оу and attach the -въш- suffix",
            )
        }
        PastActiveParticipleFormation::Vush => (
            stem,
            "въш",
            "въ",
            "attach the -въш- suffix to the explicit vowel stem",
        ),
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
    let adjectival_stem = match formation {
        PresentPassiveParticipleFormation::Im => format!("{stem}им"),
        PresentPassiveParticipleFormation::Em => format!("{stem}ем"),
        PresentPassiveParticipleFormation::Om => format!("{stem}ом"),
    };
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
    let adjectival_stem = match formation {
        PastPassiveParticipleFormation::T => format!("{stem}т"),
        PastPassiveParticipleFormation::N => format!("{stem}н"),
        PastPassiveParticipleFormation::En => format!("{stem}ен"),
    };
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
    match (cell.person, cell.number) {
        (Person::First, Number::Singular) => "хъ",
        (Person::Second | Person::Third, Number::Singular) => "ше",
        (Person::First, Number::Dual) => "ховѣ",
        (Person::Second, Number::Dual) => "шета",
        (Person::Third, Number::Dual) => "шете",
        (Person::First, Number::Plural) => "хомъ",
        (Person::Second, Number::Plural) => "шете",
        (Person::Third, Number::Plural) => "хѫ",
    }
}

fn asigmatic_aorist_cell(stem: &str, cell: FiniteVerbCell) -> (String, &'static str) {
    match (cell.person, cell.number) {
        (Person::First, Number::Singular) => (stem.to_string(), "ъ"),
        (Person::Second | Person::Third, Number::Singular) => (first_palatalize(stem), "е"),
        (Person::First, Number::Dual) => (stem.to_string(), "овѣ"),
        (Person::Second, Number::Dual) => (first_palatalize(stem), "ета"),
        (Person::Third, Number::Dual) => (first_palatalize(stem), "ете"),
        (Person::First, Number::Plural) => (stem.to_string(), "омъ"),
        (Person::Second, Number::Plural) => (first_palatalize(stem), "ете"),
        (Person::Third, Number::Plural) => (stem.to_string(), "ѫ"),
    }
}

fn new_aorist_cell(stem: &str, cell: FiniteVerbCell) -> (String, &'static str) {
    match (cell.person, cell.number) {
        (Person::First, Number::Singular) => (stem.to_string(), "охъ"),
        (Person::Second | Person::Third, Number::Singular) => (first_palatalize(stem), "е"),
        (Person::First, Number::Dual) => (stem.to_string(), "оховѣ"),
        (Person::Second, Number::Dual) => (stem.to_string(), "оста"),
        (Person::Third, Number::Dual) => (stem.to_string(), "осте"),
        (Person::First, Number::Plural) => (stem.to_string(), "охомъ"),
        (Person::Second, Number::Plural) => (stem.to_string(), "осте"),
        (Person::Third, Number::Plural) => (stem.to_string(), "ошѧ"),
    }
}

fn sigmatic_aorist_ending(kind: SigmaticKind, cell: FiniteVerbCell) -> &'static str {
    match (kind, cell.person, cell.number) {
        (SigmaticKind::Primary, Person::First, Number::Singular) => "съ",
        (SigmaticKind::Secondary | SigmaticKind::VowelStem, Person::First, Number::Singular) => {
            "хъ"
        }
        (SigmaticKind::Primary, Person::First, Number::Dual) => "совѣ",
        (SigmaticKind::Secondary | SigmaticKind::VowelStem, Person::First, Number::Dual) => "ховѣ",
        (
            SigmaticKind::Primary | SigmaticKind::Secondary | SigmaticKind::VowelStem,
            Person::Second,
            Number::Dual,
        ) => "ста",
        (
            SigmaticKind::Primary | SigmaticKind::Secondary | SigmaticKind::VowelStem,
            Person::Third,
            Number::Dual,
        ) => "сте",
        (SigmaticKind::Primary, Person::First, Number::Plural) => "сомъ",
        (SigmaticKind::Secondary | SigmaticKind::VowelStem, Person::First, Number::Plural) => {
            "хомъ"
        }
        (
            SigmaticKind::Primary | SigmaticKind::Secondary | SigmaticKind::VowelStem,
            Person::Second,
            Number::Plural,
        ) => "сте",
        (SigmaticKind::Primary, Person::Third, Number::Plural) => "сѧ",
        (SigmaticKind::Secondary | SigmaticKind::VowelStem, Person::Third, Number::Plural) => "шѧ",
        (
            SigmaticKind::Primary | SigmaticKind::Secondary | SigmaticKind::VowelStem,
            Person::Second | Person::Third,
            Number::Singular,
        ) => "",
    }
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
    let ending = match (first, second, cell.person, cell.number) {
        (true, _, Person::First, Number::Singular) => "ѫ",
        (true, _, Person::Second, Number::Singular) => "еши",
        (true, _, Person::Third, Number::Singular) => "етъ",
        (true, _, Person::First, Number::Dual) => "евѣ",
        (true, _, Person::Second, Number::Dual) => "ета",
        (true, _, Person::Third, Number::Dual) => "ете",
        (true, _, Person::First, Number::Plural) => "емъ",
        (true, _, Person::Second, Number::Plural) => "ете",
        (true, _, Person::Third, Number::Plural) => "ѫтъ",
        (_, true, Person::First, Number::Singular) => "ѭ",
        (_, true, Person::Second, Number::Singular) => "иши",
        (_, true, Person::Third, Number::Singular) => "итъ",
        (_, true, Person::First, Number::Dual) => "ивѣ",
        (_, true, Person::Second, Number::Dual) => "ита",
        (_, true, Person::Third, Number::Dual) => "ите",
        (_, true, Person::First, Number::Plural) => "имъ",
        (_, true, Person::Second, Number::Plural) => "ите",
        (_, true, Person::Third, Number::Plural) => "ѧтъ",
        _ => {
            return Err(InflectionError::unsupported(
                lemma,
                RequestedCell::FiniteVerb(cell),
            ));
        }
    };
    Ok((ending, rule))
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
    }
}
