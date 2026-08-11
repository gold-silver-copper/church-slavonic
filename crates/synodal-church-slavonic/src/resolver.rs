use synodal_church_slavonic_core::{
    AdjectiveCell, AdjectiveForm, Assumption, AuthorityRole, Comparison, Confidence, EpistemicRole,
    Error, Evidence, EvidenceId, EvidenceKind, FiniteTense, FormSet, FormSource, FormVariant,
    GenerationPolicy, GrammarCell, LexemeId, NumeralKind, OrthographyProfile, Recension, Result,
    RuleId, RuleTrace, SourceId, TraceStep, aorist, decline_adjective, decline_noun,
    decline_participle, imperative, imperfect, infinitive, l_participle,
    normalize_lookup_accentless, present,
};

use crate::{Inflector, registry};

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
        GrammarCell::LexicalForm => Err(Error::UnsupportedCell {
            reason: "a lexical-form cell must have exact reviewed target-recension evidence".into(),
        }),
        GrammarCell::Indeclinable => Err(Error::UnsupportedCell {
            reason: "an indeclinable lexeme must have an exact reviewed lexical form".into(),
        }),
        GrammarCell::Noun(cell) => {
            let forms = decline_noun(&registry::noun_lexeme(id)?, cell, rule_profile)?;
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
        GrammarCell::Adjective(cell) => {
            decline_adjective(&registry::adjective_lexeme(id)?, cell, rule_profile)
        }
        GrammarCell::FiniteVerb(cell) => {
            let verb = registry::verb_lexeme(id)?;
            match cell.tense {
                FiniteTense::Present => present(&verb, cell.person, cell.number, rule_profile),
                FiniteTense::Future => Err(Error::UnsupportedCell {
                    reason: "a simple future must have an exact reviewed normative paradigm".into(),
                }),
                FiniteTense::Past => Err(Error::UnsupportedCell {
                    reason: "an underspecified finite past must have exact reviewed evidence"
                        .into(),
                }),
                FiniteTense::Imperfect => imperfect(&verb, cell.person, cell.number, rule_profile),
                FiniteTense::Aorist => aorist(&verb, cell.person, cell.number, rule_profile),
            }
        }
        GrammarCell::Imperative(cell) => {
            imperative(&registry::verb_lexeme(id)?, cell, rule_profile)
        }
        GrammarCell::Infinitive => infinitive(&registry::verb_lexeme(id)?, rule_profile),
        GrammarCell::LParticiple(cell) => {
            l_participle(&registry::verb_lexeme(id)?, cell, rule_profile)
        }
        GrammarCell::Determiner(cell) => {
            let lexeme = registry::determiner_lexeme(id).map_err(|_| Error::UnsupportedCell {
                reason: "this determiner has no reviewed productive class for the requested cell"
                    .into(),
            })?;
            decline_adjective(&lexeme, cell, rule_profile)
        }
        GrammarCell::Supine => Err(Error::UnsupportedCell {
            reason: "the Synodal supine inventory remains under normative review".into(),
        }),
        GrammarCell::Participle(cell) => {
            decline_participle(&registry::verb_lexeme(id)?, cell, rule_profile)
        }
        GrammarCell::VerbalNoun(_) => Err(Error::UnsupportedCell {
            reason: "productive verbal nouns require lexical suffix metadata".into(),
        }),
        GrammarCell::Numeral(cell) if cell.kind == NumeralKind::Ordinal => {
            let gender = cell.gender.ok_or(Error::UnsupportedCell {
                reason: "productive ordinal inflection requires grammatical gender".into(),
            })?;
            decline_adjective(
                &registry::ordinal_lexeme(id)?,
                AdjectiveCell {
                    case: cell.case,
                    number: cell.number,
                    gender,
                    animacy: cell.animacy,
                    form: AdjectiveForm::Long,
                    comparison: Comparison::Positive,
                },
                rule_profile,
            )
        }
        GrammarCell::Pronoun(_) | GrammarCell::Numeral(_) => Err(Error::UnsupportedCell {
            reason: "this pronoun or numeral cell is absent from the exact normative registry"
                .into(),
        }),
    }?;
    apply_generated_presentation(inflector, id, &key, forms)
}

fn apply_generated_presentation(
    inflector: Inflector,
    id: &LexemeId,
    key: &str,
    forms: FormSet,
) -> Result<FormSet> {
    if inflector.orthography() != OrthographyProfile::SynodalLiturgical {
        return Ok(forms);
    }
    let mut variants = Vec::with_capacity(forms.variants().len());
    for source_variant in forms.variants() {
        let accent = registry::accent_for(id, key, &source_variant.expanded).ok_or(
            Error::OrthographicMetadataRequired {
                field: synodal_church_slavonic_core::MetadataField::AccentClass,
            },
        )?;
        let mut variant = source_variant.clone();
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
            note: Some("target-recension accent realization".into()),
        });
        let mut steps = variant.rule_trace.steps().to_vec();
        steps.push(TraceStep {
            rule: RuleId::from("SYN-ACCENT-REGISTRY"),
            stage: "accent-realization".into(),
            input: variant.expanded.clone(),
            output: accent.accented.into(),
            source_recension: Some(Recension::SynodalRussian),
            target_recension: Recension::SynodalRussian,
            mapping: variant.recension_mapping.clone(),
            evidence: vec![evidence_id],
        });
        variant.rule_trace = RuleTrace::new(steps);
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
            let rule_id = RuleId::from("SYN-REGISTRY-NORMATIVE-TABLE");
            let source = if record.source_kind == "synodal-attestation" {
                FormSource::SynodalAttestation {
                    evidence: evidence_id.clone(),
                }
            } else {
                FormSource::SynodalNormativeGeneration {
                    rule: rule_id.clone(),
                }
            };
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
                evidence: vec![evidence],
                contradictions: vec![],
                warnings: warning.into_iter().collect(),
                rule_trace: RuleTrace::new(vec![TraceStep {
                    rule: rule_id,
                    stage: "exact-normative-registry".into(),
                    input: format!("{}:{key}", id.as_str()),
                    output: printed,
                    source_recension: Some(Recension::SynodalRussian),
                    target_recension: Recension::SynodalRussian,
                    mapping: None,
                    evidence: vec![evidence_id],
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
