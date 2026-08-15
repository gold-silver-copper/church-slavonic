use synodal_church_slavonic_core::{
    AccentParadigm, Assumption, AuthorityRole, Confidence, EpistemicRole, Error, Evidence,
    EvidenceId, EvidenceKind, FormSet, FormSource, FormVariant, GenerationPolicy, GrammarCell,
    LexemeId, OrthographyProfile, Recension, Result, RuleId, RuleTrace, SourceId, TraceStep,
    normalize_lookup_accentless,
};

use crate::{
    DefectKind, Inflector, LexemeSpec, SpecificationSource, SpecifiedForm,
    kernel::{ProductiveLexeme, generate_productive},
    registry,
    spec::LexemeSpecInner,
};

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
    specified_forms_with_rule(
        inflector,
        forms,
        accent,
        "SYN-PROVIDER-EXACT-OVERRIDE",
        "provider-exact-override",
    )
}

fn specified_forms_with_rule(
    inflector: Inflector,
    forms: &[&SpecifiedForm],
    accent: Option<&AccentParadigm>,
    rule: &'static str,
    stage: &'static str,
) -> Result<FormSet> {
    let mut variants = Vec::new();
    for form in forms {
        variants.extend(
            specified_form_with_rule(inflector, form, accent, rule, stage)?
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
    let irregular = context
        .irregular_forms
        .iter()
        .filter(|form| form.cell == cell)
        .collect::<Vec<_>>();
    if !irregular.is_empty() {
        return specified_forms_with_rule(
            inflector,
            &irregular,
            context.accent.as_ref(),
            "SYN-CALLER-IRREGULAR-OVERRIDE",
            "caller-irregular-override",
        );
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
        LexemeSpecInner::Determiner(spec) => generate_productive(
            ProductiveLexeme::Determiner(&spec.lexeme),
            cell,
            rule_profile,
        ),
        LexemeSpecInner::Numeral(spec) => {
            generate_productive(ProductiveLexeme::Numeral(&spec.lexeme), cell, rule_profile)
        }
        LexemeSpecInner::Pronoun(spec) => {
            generate_productive(ProductiveLexeme::Pronoun(&spec.lexeme), cell, rule_profile)
        }
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
            generate_productive(ProductiveLexeme::Determiner(&lexeme), cell, rule_profile)
        }
        GrammarCell::Numeral(_) => {
            let lexeme = registry::numeral_lexeme(id)?;
            generate_productive(ProductiveLexeme::Numeral(&lexeme), cell, rule_profile)
        }
        GrammarCell::Pronoun(_) => {
            let lexeme = registry::pronoun_lexeme(id)?;
            generate_productive(ProductiveLexeme::Pronoun(&lexeme), cell, rule_profile)
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
    cell.key()
}

pub(crate) fn exact_lookup_keys(cell: GrammarCell) -> Vec<String> {
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
        cell.case.code(),
        cell.number.code(),
        cell.gender
            .map_or("any", synodal_church_slavonic_core::Gender::code),
        cell.person
            .map_or("none", synodal_church_slavonic_core::Person::code),
        animacy.map_or("any", synodal_church_slavonic_core::Animacy::code),
    )
}

fn numeral_key(
    cell: synodal_church_slavonic_core::NumeralCell,
    gender: Option<synodal_church_slavonic_core::Gender>,
    animacy: Option<synodal_church_slavonic_core::Animacy>,
) -> String {
    format!(
        "numeral:{}:{}:{}:{}:{}",
        cell.kind.code(),
        cell.case.code(),
        cell.number.code(),
        gender.map_or("any", synodal_church_slavonic_core::Gender::code),
        animacy.map_or("any", synodal_church_slavonic_core::Animacy::code),
    )
}

fn exact_forms(
    inflector: Inflector,
    id: &LexemeId,
    key: &str,
    records: &[registry::ExactFormRecord],
) -> Result<FormSet> {
    let irregular_evidence = registry::irregular_evidence_for(id, key);
    let irregular_records = irregular_evidence
        .map(registry::reviewed_evidence)
        .transpose()?
        .unwrap_or_default();
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
            let evidence_records = registry::reviewed_evidence(record.evidence_id)?;
            let attestation_id = evidence_records
                .iter()
                .find(|evidence| {
                    evidence.role == "target-attestation"
                        || evidence.role.strip_prefix("reviewed-cell:") == Some(key)
                })
                .map(|evidence| EvidenceId::from(evidence.id));
            let rule_id = if irregular_evidence.is_some() {
                RuleId::from("SYN-REGISTRY-IRREGULAR-OVERRIDE")
            } else {
                RuleId::from("SYN-REGISTRY-NORMATIVE-TABLE")
            };
            let source = if record.source_kind == "synodal-attestation" {
                FormSource::SynodalAttestation {
                    evidence: attestation_id.ok_or_else(|| Error::ContradictoryMetadata {
                        reason: format!(
                            "Synodal attestation {} has no target-recension evidence",
                            record.evidence_id
                        ),
                    })?,
                }
            } else if let Some(first_irregular) = irregular_records.first() {
                let source_evidence = irregular_records
                    .iter()
                    .find(|candidate| {
                        evidence_records
                            .iter()
                            .any(|evidence| evidence.id == candidate.id)
                    })
                    .unwrap_or(first_irregular);
                FormSource::SynodalIrregularOverride {
                    evidence: EvidenceId::from(source_evidence.id),
                }
            } else {
                FormSource::SynodalNormativeGeneration {
                    rule: rule_id.clone(),
                }
            };
            let mut evidence = evidence_records
                .iter()
                .map(|record| {
                    let kind = if irregular_records
                        .iter()
                        .any(|irregular| irregular.id == record.id)
                    {
                        EvidenceKind::ReviewedIrregularOverride
                    } else {
                        EvidenceKind::ExactTableCell
                    };
                    reviewed_evidence(record, kind)
                })
                .collect::<Vec<_>>();
            for record in &irregular_records {
                if !evidence
                    .iter()
                    .any(|evidence| evidence.id.as_str() == record.id)
                {
                    evidence.push(reviewed_evidence(
                        record,
                        EvidenceKind::ReviewedIrregularOverride,
                    ));
                }
            }
            let trace_evidence = evidence.iter().map(|item| item.id.clone()).collect();
            Ok(FormVariant {
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
            })
        })
        .collect::<Result<Vec<_>>>()?;
    FormSet::try_from_variants(variants)
}

fn reviewed_evidence(
    record: &registry::ReviewedEvidenceRecord,
    default_kind: EvidenceKind,
) -> Evidence {
    let (kind, authority_roles, epistemic_role) = match record.role {
        role if role == "target-attestation" || role.starts_with("reviewed-cell:") => (
            EvidenceKind::CorpusObservation,
            vec![
                AuthorityRole::ExactForm,
                AuthorityRole::Orthographic,
                AuthorityRole::Accentual,
            ],
            EpistemicRole::ExactSynodalAttestation,
        ),
        "synodal-authority" => (
            default_kind,
            vec![
                AuthorityRole::Grammatical,
                AuthorityRole::Morphological,
                AuthorityRole::Orthographic,
            ],
            EpistemicRole::SynodalNormativeAuthority,
        ),
        "ocs-evidence" => (
            EvidenceKind::ComparativeObservation,
            vec![AuthorityRole::Lexical, AuthorityRole::Morphological],
            EpistemicRole::InheritedOcsEvidence,
        ),
        _ => (
            EvidenceKind::ComparativeObservation,
            vec![AuthorityRole::Lexical, AuthorityRole::Morphological],
            EpistemicRole::OtherRecensionComparativeEvidence,
        ),
    };
    Evidence {
        id: EvidenceId::from(record.id),
        source: SourceId::from(record.source_id),
        source_recension: record.source_recension,
        kind,
        authority_roles,
        epistemic_role,
        citation: record.citation.into(),
        note: (!record.note.is_empty()).then(|| record.note.into()),
    }
}
