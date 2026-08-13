use synodal_church_slavonic_core::{
    AccentParadigm, AdjectiveForm, Assumption, AuthorityRole, Confidence, EpistemicRole, Error,
    Evidence, EvidenceId, EvidenceKind, FiniteTense, FormSet, FormSource, FormVariant,
    GenerationPolicy, GrammarCell, LexemeId, NumeralKind, OrthographyProfile, Recension, Result,
    RuleId, RuleTrace, SourceId, TraceStep, normalize_lookup_accentless,
};

use crate::{
    DefectKind, Inflector, LexemeSpec, SpecificationSource, SpecifiedForm,
    kernel::{ProductiveLexeme, generate_productive},
    registry,
    spec::LexemeSpecInner,
};

fn specified_form(
    inflector: Inflector,
    form: &SpecifiedForm,
    accent: Option<&AccentParadigm>,
) -> Result<FormSet> {
    specified_form_with_rule(
        inflector,
        form,
        accent,
        "SYN-CALLER-IRREGULAR-OVERRIDE",
        "caller-irregular-override",
    )
}

fn specified_form_with_rule(
    inflector: Inflector,
    form: &SpecifiedForm,
    accent: Option<&AccentParadigm>,
    rule: &'static str,
    stage: &'static str,
) -> Result<FormSet> {
    let expanded = form.expanded.canonical().to_owned();
    let rule = RuleId::from(rule);
    let evidence = form.source.evidence(EvidenceKind::LexicalMetadata);
    let evidence_id = evidence.id.clone();
    let (printed, accented, warnings) = match inflector.orthography() {
        OrthographyProfile::Expanded => (expanded.clone(), None, vec![]),
        OrthographyProfile::ExpandedAccentless => (
            normalize_lookup_accentless(&expanded),
            None,
            vec!["accent and breathing marks removed by requested profile".into()],
        ),
        OrthographyProfile::SynodalLiturgical => {
            if let Some(liturgical) = &form.liturgical {
                (
                    liturgical.as_str().to_owned(),
                    Some(liturgical.as_str().to_owned()),
                    vec![],
                )
            } else {
                (expanded.clone(), None, vec![])
            }
        }
    };
    let forms = FormSet::new(FormVariant {
        expanded: expanded.clone(),
        accented,
        printed: printed.clone(),
        romanization: None,
        source_recension: Some(Recension::SynodalRussian),
        target_recension: Recension::SynodalRussian,
        recension_mapping: None,
        confidence: Confidence::from_basis_points(9_500).unwrap_or(Confidence::CERTAIN),
        source: FormSource::CallerSpecifiedPrediction {
            rule: rule.clone(),
            evidence: evidence_id.clone(),
        },
        assumptions: vec![],
        evidence: vec![evidence],
        contradictions: vec![],
        warnings,
        rule_trace: RuleTrace::new(vec![TraceStep {
            rule,
            stage: stage.into(),
            input: expanded,
            output: printed,
            source_recension: Some(Recension::SynodalRussian),
            target_recension: Recension::SynodalRussian,
            mapping: None,
            evidence: vec![evidence_id],
        }]),
    })?;
    if inflector.orthography() == OrthographyProfile::SynodalLiturgical && form.liturgical.is_none()
    {
        let accent = accent.ok_or(Error::OrthographicMetadataRequired {
            field: synodal_church_slavonic_core::MetadataField::AccentParadigm,
        })?;
        apply_accent_paradigm(forms, form.cell, accent)
    } else {
        Ok(forms)
    }
}

pub(crate) fn provided_exact_forms(
    inflector: Inflector,
    forms: &[&SpecifiedForm],
    accent: Option<&AccentParadigm>,
) -> Result<FormSet> {
    let mut variants = Vec::new();
    for form in forms {
        variants.extend(
            specified_form_with_rule(
                inflector,
                form,
                accent,
                "SYN-PROVIDER-EXACT-OVERRIDE",
                "provider-exact-override",
            )?
            .variants()
            .iter()
            .cloned(),
        );
    }
    FormSet::try_from_variants(variants)
}

fn mark_caller_specified(forms: FormSet, source: &SpecificationSource) -> Result<FormSet> {
    let metadata = source.evidence(EvidenceKind::LexicalMetadata);
    let evidence_id = metadata.id.clone();
    let mut variants = Vec::with_capacity(forms.variants().len());
    for source_variant in forms.variants() {
        let mut variant = source_variant.clone();
        let rule = match &variant.source {
            FormSource::SynodalNormativeGeneration { rule } => rule.clone(),
            _ => {
                return Err(Error::ContradictoryMetadata {
                    reason: "caller metadata must enter the productive kernel before provenance classification"
                        .into(),
                });
            }
        };
        variant.evidence.push(metadata.clone());
        variant.source = FormSource::CallerSpecifiedPrediction {
            rule: rule.clone(),
            evidence: evidence_id.clone(),
        };
        variant.rule_trace.push(TraceStep {
            rule,
            stage: "caller-specified-lexical-metadata".into(),
            input: source.citation().into(),
            output: variant.printed.clone(),
            source_recension: Some(Recension::SynodalRussian),
            target_recension: Recension::SynodalRussian,
            mapping: None,
            evidence: vec![evidence_id.clone()],
        });
        variants.push(variant);
    }
    FormSet::try_from_variants(variants)
}

pub(crate) fn apply_accent_paradigm(
    forms: FormSet,
    cell: GrammarCell,
    paradigm: &AccentParadigm,
) -> Result<FormSet> {
    paradigm.validate()?;
    let evidence_id = paradigm.evidence.id.clone();
    let rule = RuleId::from(format!("SYN-ACCENT-PARADIGM:{}", paradigm.id));
    let mut variants = Vec::with_capacity(forms.variants().len());
    for source_variant in forms.variants() {
        let mut variant = source_variant.clone();
        let accented = paradigm.apply(cell, &variant.expanded)?;
        variant.accented = Some(accented.clone());
        variant.printed = accented.clone();
        variant.evidence.push(paradigm.evidence.clone());
        variant.rule_trace.push(TraceStep {
            rule: rule.clone(),
            stage: "accent-paradigm-realization".into(),
            input: variant.expanded.clone(),
            output: accented,
            source_recension: Some(Recension::SynodalRussian),
            target_recension: Recension::SynodalRussian,
            mapping: variant.recension_mapping.clone(),
            evidence: vec![evidence_id.clone()],
        });
        variants.push(variant);
    }
    FormSet::try_from_variants(variants)
}

pub(crate) fn resolve_spec(
    inflector: Inflector,
    spec: &LexemeSpec,
    cell: GrammarCell,
) -> Result<FormSet> {
    spec.validate()?;
    let context = spec.context();
    if let Some(form) = context
        .irregular_forms
        .iter()
        .find(|form| form.cell == cell)
    {
        return specified_form(inflector, form, context.accent.as_ref());
    }
    if let Some(defect) = context
        .defective_cells
        .iter()
        .find(|defect| defect.cell == cell)
    {
        return match defect.kind {
            DefectKind::HistoricallyAbsent => Err(Error::HistoricallyInvalidCell {
                reason: defect.reason.clone(),
            }),
            DefectKind::EvidenceIncomplete => Err(Error::EvidenceIncompleteCell {
                field: defect.field,
                reason: defect.reason.clone(),
            }),
        };
    }

    let rule_profile = if inflector.orthography() == OrthographyProfile::SynodalLiturgical {
        OrthographyProfile::Expanded
    } else {
        inflector.orthography()
    };
    let forms = match spec.inner() {
        LexemeSpecInner::Noun(spec) => {
            generate_productive(ProductiveLexeme::Noun(&spec.lexeme), cell, rule_profile)
        }
        LexemeSpecInner::Adjective(spec) => generate_productive(
            ProductiveLexeme::Adjective(&spec.lexeme),
            cell,
            rule_profile,
        ),
        LexemeSpecInner::Verb(spec) => {
            generate_productive(ProductiveLexeme::Verb(&spec.lexeme), cell, rule_profile)
        }
    }?;
    let forms = mark_caller_specified(forms, &context.source)?;
    if inflector.orthography() == OrthographyProfile::SynodalLiturgical {
        let accent = context
            .accent
            .as_ref()
            .ok_or(Error::OrthographicMetadataRequired {
                field: synodal_church_slavonic_core::MetadataField::AccentParadigm,
            })?;
        apply_accent_paradigm(forms, cell, accent)
    } else {
        Ok(forms)
    }
}

pub(crate) fn resolve_cell(
    inflector: Inflector,
    id: &LexemeId,
    cell: GrammarCell,
) -> Result<FormSet> {
    let key = cell_key(cell);
    let mut exact = Vec::new();
    let mut exact_key = key.clone();
    for candidate_key in exact_lookup_keys(cell) {
        let mut candidate = registry::exact_forms(id, &candidate_key);
        if !candidate.is_empty() && exact.is_empty() {
            exact_key = candidate_key;
        }
        exact.append(&mut candidate);
    }
    if !exact.is_empty() {
        return exact_forms(inflector, id, &exact_key, &exact);
    }
    if registry::is_exact_only(id) {
        return Err(Error::UnsupportedCell {
            reason: "this exact-only lexeme has no reviewed class or principal parts for the requested cell"
                .into(),
        });
    }

    let rule_profile = if inflector.orthography() == OrthographyProfile::SynodalLiturgical {
        OrthographyProfile::Expanded
    } else {
        inflector.orthography()
    };
    let forms = match cell {
        GrammarCell::Noun(_) => {
            let lexeme = registry::noun_lexeme(id)?;
            let forms = generate_productive(ProductiveLexeme::Noun(&lexeme), cell, rule_profile)?;
            if registry::noun_uses_inherited_class(id) {
                if inflector.generation_policy() == GenerationPolicy::Strict {
                    return Err(Error::UnsupportedCell {
                        reason: "only an inherited OCS class analysis is available under Strict"
                            .into(),
                    });
                }
                let alignments = registry::inherited_alignments(
                    id,
                    inflector.generation_policy(),
                    inflector.productive_mapping_threshold_basis_points(),
                )?;
                mark_inherited(forms, alignments)
            } else {
                Ok(forms)
            }
        }
        GrammarCell::Adjective(_) => {
            let lexeme = registry::adjective_lexeme(id)?;
            generate_productive(ProductiveLexeme::Adjective(&lexeme), cell, rule_profile)
        }
        GrammarCell::Determiner(_) => {
            let lexeme = registry::determiner_lexeme(id).map_err(|_| Error::UnsupportedCell {
                reason: "this determiner has no reviewed productive class for the requested cell"
                    .into(),
            })?;
            generate_productive(ProductiveLexeme::Adjective(&lexeme), cell, rule_profile)
        }
        GrammarCell::Numeral(numeral) if numeral.kind == NumeralKind::Ordinal => {
            let lexeme = registry::ordinal_lexeme(id)?;
            generate_productive(ProductiveLexeme::Adjective(&lexeme), cell, rule_profile)
        }
        cell @ (GrammarCell::FiniteVerb(_)
        | GrammarCell::Imperative(_)
        | GrammarCell::Infinitive
        | GrammarCell::LParticiple(_)
        | GrammarCell::Supine
        | GrammarCell::Participle(_)
        | GrammarCell::VerbalNoun(_)) => {
            let lexeme = registry::verb_lexeme(id)?;
            generate_productive(ProductiveLexeme::Verb(&lexeme), cell, rule_profile)
        }
        GrammarCell::Pronoun(_) | GrammarCell::Numeral(_) => Err(Error::UnsupportedCell {
            reason: "this pronoun or numeral cell is absent from the exact normative registry"
                .into(),
        }),
        GrammarCell::LexicalForm | GrammarCell::Indeclinable => Err(Error::UnsupportedCell {
            reason: "the requested lexical cell has no exact reviewed form".into(),
        }),
    }?;
    apply_generated_presentation(inflector, id, cell, &key, forms)
}

fn apply_generated_presentation(
    inflector: Inflector,
    id: &LexemeId,
    cell: GrammarCell,
    key: &str,
    forms: FormSet,
) -> Result<FormSet> {
    if inflector.orthography() != OrthographyProfile::SynodalLiturgical {
        return Ok(forms);
    }
    // Parse a reusable paradigm only when at least one generated variant lacks
    // an exact reviewed accent. This keeps the documented lexical-override
    // precedence semantic, including when reusable metadata is malformed.
    let needs_paradigm = forms
        .variants()
        .iter()
        .any(|variant| registry::accent_for(id, key, &variant.expanded).is_none());
    let paradigm = if needs_paradigm {
        registry::accent_paradigm_for(id, cell)?
    } else {
        None
    };
    let mut variants = Vec::with_capacity(forms.variants().len());
    for source_variant in forms.variants() {
        let mut variant = source_variant.clone();
        if let Some(accent) = registry::accent_for(id, key, &source_variant.expanded) {
            variant.accented = Some(accent.accented.into());
            variant.printed = accent.accented.into();
            let evidence_id = EvidenceId::from(accent.evidence_id);
            variant.evidence.push(Evidence {
                id: evidence_id.clone(),
                source: SourceId::from(accent.source_id),
                source_recension: match accent.source_recension {
                    "synodal-russian" => Recension::SynodalRussian,
                    value => {
                        return Err(Error::ContradictoryMetadata {
                            reason: format!("accent evidence has non-Synodal recension {value:?}"),
                        });
                    }
                },
                kind: EvidenceKind::AccentMetadata,
                authority_roles: vec![AuthorityRole::Accentual, AuthorityRole::Orthographic],
                epistemic_role: EpistemicRole::SynodalNormativeAuthority,
                citation: accent.evidence_id.into(),
                note: Some("exact target-recension accent realization".into()),
            });
            variant.rule_trace.push(TraceStep {
                rule: RuleId::from("SYN-ACCENT-REGISTRY"),
                stage: "exact-accent-realization".into(),
                input: variant.expanded.clone(),
                output: accent.accented.into(),
                source_recension: Some(Recension::SynodalRussian),
                target_recension: Recension::SynodalRussian,
                mapping: variant.recension_mapping.clone(),
                evidence: vec![evidence_id],
            });
        } else if let Some(paradigm) = &paradigm {
            let accented = paradigm.apply(cell, &variant.expanded)?;
            variant.accented = Some(accented.clone());
            variant.printed = accented.clone();
            variant.evidence.push(paradigm.evidence.clone());
            variant.rule_trace.push(TraceStep {
                rule: RuleId::from(format!("SYN-ACCENT-PARADIGM:{}", paradigm.id)),
                stage: "accent-paradigm-realization".into(),
                input: variant.expanded.clone(),
                output: accented,
                source_recension: Some(Recension::SynodalRussian),
                target_recension: Recension::SynodalRussian,
                mapping: variant.recension_mapping.clone(),
                evidence: vec![paradigm.evidence.id.clone()],
            });
        } else {
            return Err(Error::OrthographicMetadataRequired {
                field: synodal_church_slavonic_core::MetadataField::AccentParadigm,
            });
        }
        variants.push(variant);
    }
    FormSet::try_from_variants(variants)
}

fn mark_inherited(
    forms: FormSet,
    alignments: Vec<registry::InheritedAlignment>,
) -> Result<FormSet> {
    let mut inherited = Vec::with_capacity(forms.variants().len() * alignments.len());
    for alignment in alignments {
        for source_variant in forms.variants() {
            let mut variant = source_variant.clone();
            let rule = match &variant.source {
                FormSource::SynodalNormativeGeneration { rule } => rule.clone(),
                _ => {
                    return Err(Error::ContradictoryMetadata {
                        reason: "inherited realization must originate in a Synodal normative rule"
                            .into(),
                    });
                }
            };
            let evidence_ids: Vec<EvidenceId> = alignment
                .evidence_ids
                .iter()
                .map(|value| EvidenceId::from(value.as_str()))
                .collect();
            variant
                .evidence
                .extend(evidence_ids.iter().map(|evidence_id| Evidence {
                    id: evidence_id.clone(),
                    source: SourceId::from("english-wiktionary-ocs-kaikki-2026-08-07"),
                    source_recension: Recension::OldChurchSlavonic,
                    kind: EvidenceKind::RecensionTransformation,
                    authority_roles: vec![AuthorityRole::Lexical, AuthorityRole::Morphological],
                    epistemic_role: EpistemicRole::InheritedOcsEvidence,
                    citation: evidence_id.to_string(),
                    note: Some("reviewed OCS-to-Synodal alignment evidence".into()),
                }));
            variant.assumptions.push(Assumption {
                code: "inherited-ocs-class".into(),
                detail: format!(
                    "source lexeme {}; transformations: {}",
                    alignment.source_lexeme_id,
                    alignment.transformations.join(", ")
                ),
            });
            variant.source_recension = Some(Recension::OldChurchSlavonic);
            variant.recension_mapping = Some(alignment.mapping_id.clone());
            variant.confidence = variant.confidence.min(alignment.confidence);
            variant.source = FormSource::InheritedPrediction {
                source_recension: Recension::OldChurchSlavonic,
                mapping: alignment.mapping_id.clone(),
                rule,
            };
            let mut steps = Vec::with_capacity(variant.rule_trace.steps().len() + 1);
            steps.push(TraceStep {
                rule: RuleId::from("SYN-OCS-RECENSION-MAPPING"),
                stage: "recension-mapping".into(),
                input: alignment.source_lexeme_id.to_string(),
                output: variant.expanded.clone(),
                source_recension: Some(Recension::OldChurchSlavonic),
                target_recension: Recension::SynodalRussian,
                mapping: Some(alignment.mapping_id.clone()),
                evidence: evidence_ids,
            });
            steps.extend(variant.rule_trace.steps().iter().cloned());
            variant.rule_trace = RuleTrace::new(steps);
            inherited.push(variant);
        }
    }
    FormSet::try_from_variants(inherited)
}

pub(crate) fn cell_key(cell: GrammarCell) -> String {
    match cell {
        GrammarCell::LexicalForm => "lexical-form".into(),
        GrammarCell::Indeclinable => "indeclinable".into(),
        GrammarCell::Noun(cell) => format!(
            "noun:{}:{}:{}",
            case_name(cell.case),
            number_name(cell.number),
            animacy_name(cell.animacy)
        ),
        GrammarCell::Adjective(cell) => format!(
            "adjective:{}:{}:{}:{}:{}:{}",
            case_name(cell.case),
            number_name(cell.number),
            gender_name(cell.gender),
            animacy_name(cell.animacy),
            adjective_form_name(cell.form),
            comparison_name(cell.comparison)
        ),
        GrammarCell::FiniteVerb(cell) => format!(
            "{}:{}:{}",
            tense_name(cell.tense),
            person_name(cell.person),
            number_name(cell.number)
        ),
        GrammarCell::Imperative(cell) => format!(
            "imperative:{}:{}",
            person_name(cell.person),
            number_name(cell.number)
        ),
        GrammarCell::Infinitive => "infinitive".into(),
        GrammarCell::Supine => "supine".into(),
        GrammarCell::LParticiple(cell) => format!(
            "l-participle:{}:{}",
            gender_name(cell.gender),
            number_name(cell.number)
        ),
        GrammarCell::Participle(cell) => format!(
            "participle:{:?}:{:?}:{}:{}:{}:{}:{}:{}",
            cell.tense,
            cell.voice,
            case_name(cell.agreement.case),
            number_name(cell.agreement.number),
            gender_name(cell.agreement.gender),
            animacy_name(cell.agreement.animacy),
            adjective_form_name(cell.agreement.form),
            comparison_name(cell.agreement.comparison)
        )
        .to_lowercase(),
        GrammarCell::VerbalNoun(cell) => format!(
            "verbal-noun:{}:{}:{}",
            case_name(cell.case),
            number_name(cell.number),
            animacy_name(cell.animacy)
        ),
        GrammarCell::Pronoun(cell) => pronoun_key(cell, Some(cell.animacy)),
        GrammarCell::Determiner(cell) => format!(
            "determiner:{}:{}:{}:{}:{}:{}",
            case_name(cell.case),
            number_name(cell.number),
            gender_name(cell.gender),
            animacy_name(cell.animacy),
            adjective_form_name(cell.form),
            comparison_name(cell.comparison)
        ),
        GrammarCell::Numeral(cell) => numeral_key(cell, cell.gender, Some(cell.animacy)),
    }
}

fn exact_lookup_keys(cell: GrammarCell) -> Vec<String> {
    let mut keys = vec![cell_key(cell)];
    match cell {
        GrammarCell::Adjective(_) | GrammarCell::Determiner(_) | GrammarCell::Participle(_) => {
            let neutral = keys[0]
                .replace(":inanimate:", ":any:")
                .replace(":animate:", ":any:");
            if neutral != keys[0] {
                keys.push(neutral);
            }
        }
        GrammarCell::Pronoun(pronoun) => {
            keys.push(pronoun_key(pronoun, None));
        }
        GrammarCell::Numeral(numeral) => {
            if numeral.gender.is_some() {
                keys.push(numeral_key(numeral, None, Some(numeral.animacy)));
            }
            keys.push(numeral_key(numeral, numeral.gender, None));
            if numeral.gender.is_some() {
                keys.push(numeral_key(numeral, None, None));
            }
        }
        _ => {}
    }
    keys.dedup();
    keys
}

fn pronoun_key(
    cell: synodal_church_slavonic_core::PronounCell,
    animacy: Option<synodal_church_slavonic_core::Animacy>,
) -> String {
    format!(
        "pronoun:{}:{}:{}:{}:{}",
        case_name(cell.case),
        number_name(cell.number),
        cell.gender.map_or("any", gender_name),
        cell.person.map_or("none", person_name),
        animacy.map_or("any", animacy_name),
    )
}

fn numeral_key(
    cell: synodal_church_slavonic_core::NumeralCell,
    gender: Option<synodal_church_slavonic_core::Gender>,
    animacy: Option<synodal_church_slavonic_core::Animacy>,
) -> String {
    format!(
        "numeral:{}:{}:{}:{}:{}",
        numeral_kind_name(cell.kind),
        case_name(cell.case),
        number_name(cell.number),
        gender.map_or("any", gender_name),
        animacy.map_or("any", animacy_name),
    )
}

fn exact_forms(
    inflector: Inflector,
    id: &LexemeId,
    key: &str,
    records: &[registry::ExactFormRecord],
) -> Result<FormSet> {
    let summary = registry::from_id(id)?;
    let irregular_evidence = registry::irregular_evidence_for(id, key);
    let variants = records
        .iter()
        .map(|record| {
            let (printed, accented, warning) = match inflector.orthography() {
                OrthographyProfile::Expanded => (record.expanded.to_owned(), None, None),
                OrthographyProfile::ExpandedAccentless => (
                    normalize_lookup_accentless(record.expanded),
                    None,
                    Some("accent and breathing marks removed by requested profile".into()),
                ),
                OrthographyProfile::SynodalLiturgical => (
                    record.printed.to_owned(),
                    Some(record.printed.to_owned()),
                    None,
                ),
            };
            let evidence_id = EvidenceId::from(record.evidence_id);
            let evidence = Evidence {
                id: evidence_id.clone(),
                source: SourceId::from(summary.source_id()),
                source_recension: Recension::SynodalRussian,
                kind: EvidenceKind::ExactTableCell,
                authority_roles: vec![AuthorityRole::Morphological, AuthorityRole::Orthographic],
                epistemic_role: if record.source_kind == "synodal-attestation" {
                    EpistemicRole::ExactSynodalAttestation
                } else {
                    EpistemicRole::SynodalNormativeAuthority
                },
                citation: record.evidence_id.into(),
                note: Some(record.source_kind.into()),
            };
            let rule_id = if irregular_evidence.is_some() {
                RuleId::from("SYN-REGISTRY-IRREGULAR-OVERRIDE")
            } else {
                RuleId::from("SYN-REGISTRY-NORMATIVE-TABLE")
            };
            let source = if record.source_kind == "synodal-attestation" {
                FormSource::SynodalAttestation {
                    evidence: evidence_id.clone(),
                }
            } else if let Some(irregular_evidence) = irregular_evidence {
                FormSource::SynodalIrregularOverride {
                    evidence: EvidenceId::from(irregular_evidence),
                }
            } else {
                FormSource::SynodalNormativeGeneration {
                    rule: rule_id.clone(),
                }
            };
            let mut evidence = vec![evidence];
            let mut trace_evidence = vec![evidence_id.clone()];
            if let Some(irregular_evidence) = irregular_evidence {
                let irregular_evidence = EvidenceId::from(irregular_evidence);
                evidence.push(Evidence {
                    id: irregular_evidence.clone(),
                    source: SourceId::from(summary.source_id()),
                    source_recension: Recension::SynodalRussian,
                    kind: EvidenceKind::ReviewedIrregularOverride,
                    authority_roles: vec![AuthorityRole::Morphological],
                    epistemic_role: EpistemicRole::SynodalNormativeAuthority,
                    citation: irregular_evidence.to_string(),
                    note: Some("reviewed irregular system override".into()),
                });
                trace_evidence.push(irregular_evidence);
            }
            FormVariant {
                expanded: record.expanded.into(),
                accented,
                printed: printed.clone(),
                romanization: None,
                source_recension: Some(Recension::SynodalRussian),
                target_recension: Recension::SynodalRussian,
                recension_mapping: None,
                confidence: Confidence::CERTAIN,
                source,
                assumptions: vec![],
                evidence,
                contradictions: vec![],
                warnings: warning.into_iter().collect(),
                rule_trace: RuleTrace::new(vec![TraceStep {
                    rule: rule_id,
                    stage: if irregular_evidence.is_some() {
                        "irregular-override-registry".into()
                    } else {
                        "exact-normative-registry".into()
                    },
                    input: format!("{}:{key}", id.as_str()),
                    output: printed,
                    source_recension: Some(Recension::SynodalRussian),
                    target_recension: Recension::SynodalRussian,
                    mapping: None,
                    evidence: trace_evidence,
                }]),
            }
        })
        .collect();
    FormSet::try_from_variants(variants)
}

fn case_name(value: synodal_church_slavonic_core::Case) -> &'static str {
    use synodal_church_slavonic_core::Case;
    match value {
        Case::Nominative => "nominative",
        Case::Genitive => "genitive",
        Case::Dative => "dative",
        Case::Accusative => "accusative",
        Case::Instrumental => "instrumental",
        Case::Locative => "locative",
        Case::Vocative => "vocative",
    }
}

fn number_name(value: synodal_church_slavonic_core::Number) -> &'static str {
    use synodal_church_slavonic_core::Number;
    match value {
        Number::Singular => "singular",
        Number::Dual => "dual",
        Number::Plural => "plural",
    }
}

fn gender_name(value: synodal_church_slavonic_core::Gender) -> &'static str {
    use synodal_church_slavonic_core::Gender;
    match value {
        Gender::Masculine => "masculine",
        Gender::Feminine => "feminine",
        Gender::Neuter => "neuter",
    }
}

fn person_name(value: synodal_church_slavonic_core::Person) -> &'static str {
    use synodal_church_slavonic_core::Person;
    match value {
        Person::First => "first",
        Person::Second => "second",
        Person::Third => "third",
    }
}

fn animacy_name(value: synodal_church_slavonic_core::Animacy) -> &'static str {
    use synodal_church_slavonic_core::Animacy;
    match value {
        Animacy::Inanimate => "inanimate",
        Animacy::Animate => "animate",
    }
}

fn tense_name(value: FiniteTense) -> &'static str {
    match value {
        FiniteTense::Present => "present",
        FiniteTense::Future => "future",
        FiniteTense::Past => "past",
        FiniteTense::Imperfect => "imperfect",
        FiniteTense::Aorist => "aorist",
    }
}

fn adjective_form_name(value: AdjectiveForm) -> &'static str {
    match value {
        AdjectiveForm::Short => "short",
        AdjectiveForm::Long => "long",
    }
}

fn comparison_name(value: synodal_church_slavonic_core::Comparison) -> &'static str {
    use synodal_church_slavonic_core::Comparison;
    match value {
        Comparison::Positive => "positive",
        Comparison::Comparative => "comparative",
        Comparison::Superlative => "superlative",
    }
}

fn numeral_kind_name(value: synodal_church_slavonic_core::NumeralKind) -> &'static str {
    use synodal_church_slavonic_core::NumeralKind;
    match value {
        NumeralKind::Cardinal => "cardinal",
        NumeralKind::Ordinal => "ordinal",
        NumeralKind::Collective => "collective",
    }
}
