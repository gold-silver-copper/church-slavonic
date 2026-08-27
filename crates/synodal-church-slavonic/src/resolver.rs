use church_slavonic_orthography::synodal::{
    antistich_letter, final_accent_alternate, lowercase_initial, present_initial_broad_on,
    unaccented_enclitic, widen_plural_ending_printed,
};
use synodal_church_slavonic_core::{
    AccentParadigm, Assumption, AuthorityRole, Case, Confidence, EpistemicRole, Error, Evidence,
    EvidenceId, EvidenceKind, FormSet, FormSource, FormVariant, Gender, GenerationPolicy,
    GrammarCell, LexemeId, Number, OrthographyProfile, PositionalParadigm, Recension, Result,
    RuleId, RuleTrace, SourceId, TraceStep, normalize_lookup_accentless,
};

use crate::{
    DefectKind, Inflector, LexemeSpec, SpecificationSource,
    kernel::{ProductiveLexeme, absent_synodal_supine, generate_productive},
    registry,
    spec::LexemeSpecInner,
};

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
        let presentation = variant.printed.clone();
        let accented = paradigm.apply(cell, &presentation)?;
        variant.accented = Some(accented.clone());
        variant.printed = accented.clone();
        variant.evidence.push(paradigm.evidence.clone());
        variant.rule_trace.push(TraceStep {
            rule: rule.clone(),
            stage: "accent-paradigm-realization".into(),
            input: presentation,
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

fn apply_positional_paradigm(
    forms: FormSet,
    cell: GrammarCell,
    paradigm: &PositionalParadigm,
) -> Result<FormSet> {
    paradigm.validate()?;
    let evidence_id = paradigm.evidence.id.clone();
    let rule = RuleId::from(format!("SYN-POSITIONAL-PARADIGM:{}", paradigm.id));
    let mut variants = Vec::with_capacity(forms.variants().len());
    for source_variant in forms.variants() {
        let mut variant = source_variant.clone();
        let presented = paradigm.apply(cell, &variant.printed)?;
        variant.evidence.push(paradigm.evidence.clone());
        variant.rule_trace.push(TraceStep {
            rule: rule.clone(),
            stage: "lexical-positional-realization".into(),
            input: variant.printed.clone(),
            output: presented.clone(),
            source_recension: Some(Recension::SynodalRussian),
            target_recension: Recension::SynodalRussian,
            mapping: variant.recension_mapping.clone(),
            evidence: vec![evidence_id.clone()],
        });
        variant.printed = presented;
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
        let positional =
            context
                .positional
                .as_ref()
                .ok_or(Error::OrthographicMetadataRequired {
                    field: synodal_church_slavonic_core::MetadataField::PositionalParadigm,
                })?;
        let forms = apply_positional_paradigm(forms, cell, positional)?;
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
        let forms = exact_forms(inflector, id, &exact_key, &exact)?;
        return present_liturgical_cell(inflector, id, cell, forms, false);
    }
    if let Some(defect) = registry::defect_for(id, &key)? {
        return match defect.kind {
            DefectKind::HistoricallyAbsent => Err(Error::HistoricallyInvalidCell {
                reason: defect.reason.into(),
            }),
            DefectKind::EvidenceIncomplete => Err(Error::EvidenceIncompleteCell {
                field: defect.field,
                reason: defect.reason.into(),
            }),
        };
    }
    if cell == GrammarCell::Supine {
        return Err(absent_synodal_supine());
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
    // Alypy §73: the enclitic is attached before accent realisation so that
    // exact accent rows, fitted paradigms, and the corpus prints they were
    // fitted against all describe the same surface (`боите́сѧ`, not `боите`).
    // The enclitic never carries a mark, so a placement counted from the
    // start is unaffected, and one counted from the end is fitted on the
    // enclitic-bearing form it will be applied to.
    let forms = if registry::is_reflexive_verb(id) {
        apply_reflexive(forms)?
    } else {
        forms
    };
    let forms = apply_generated_presentation(inflector, id, cell, &key, forms)?;
    present_liturgical_cell(inflector, id, cell, forms, true)
}

const PRESENTATION_RULE: &str = "SYN-ORTH-LITURGICAL-PRESENTATION";
const ANTISTICH_RULE: &str = "SYN-ORTH-ANTISTICH-ALYPY-36";
const ENCLITIC_RULE: &str = "SYN-ACCENT-ENCLITIC-ENVIRONMENT-ALYPY-5";
/// Assumption code carried by an enclitic-environment print (Alypy §5) so
/// that surface indexes can tell it from the isolated print.
pub const ENCLITIC_ENVIRONMENT_ASSUMPTION: &str = "enclitic-environment";

fn nominal_number_and_case(cell: GrammarCell) -> Option<(Number, Case)> {
    match cell {
        GrammarCell::Noun(cell) | GrammarCell::VerbalNoun(cell) => Some((cell.number, cell.case)),
        GrammarCell::Adjective(cell) | GrammarCell::Determiner(cell) => {
            Some((cell.number, cell.case))
        }
        GrammarCell::Numeral(cell) => Some((cell.number, cell.case)),
        GrammarCell::Participle(cell) => Some((cell.agreement.number, cell.agreement.case)),
        GrammarCell::Pronoun(cell) => Some((cell.number, cell.case)),
        _ => None,
    }
}

/// The singular cells a plural or dual nominal cell can be homographic with:
/// every case of the singular; for agreeing words also every gender, since
/// the print contrasts number across genders (небє́снаѧ plural neuter against
/// небе́снаѧ feminine singular). Length and comparison stay fixed.
fn singular_counterparts(cell: GrammarCell) -> Vec<GrammarCell> {
    let cases = [
        Case::Nominative,
        Case::Genitive,
        Case::Dative,
        Case::Accusative,
        Case::Instrumental,
        Case::Locative,
        Case::Vocative,
    ];
    let genders = [Gender::Masculine, Gender::Feminine, Gender::Neuter];
    let mut cells = Vec::new();
    for case in cases {
        match cell {
            GrammarCell::Noun(mut inner) => {
                inner.case = case;
                inner.number = Number::Singular;
                cells.push(GrammarCell::Noun(inner));
            }
            GrammarCell::VerbalNoun(mut inner) => {
                inner.case = case;
                inner.number = Number::Singular;
                cells.push(GrammarCell::VerbalNoun(inner));
            }
            GrammarCell::Adjective(mut inner) => {
                inner.case = case;
                inner.number = Number::Singular;
                for gender in genders {
                    inner.gender = gender;
                    cells.push(GrammarCell::Adjective(inner));
                }
            }
            GrammarCell::Determiner(mut inner) => {
                inner.case = case;
                inner.number = Number::Singular;
                for gender in genders {
                    inner.gender = gender;
                    cells.push(GrammarCell::Determiner(inner));
                }
            }
            GrammarCell::Numeral(mut inner) => {
                inner.case = case;
                inner.number = Number::Singular;
                if inner.gender.is_some() {
                    for gender in genders {
                        inner.gender = Some(gender);
                        cells.push(GrammarCell::Numeral(inner));
                    }
                } else {
                    cells.push(GrammarCell::Numeral(inner));
                }
            }
            GrammarCell::Participle(mut inner) => {
                inner.agreement.case = case;
                inner.agreement.number = Number::Singular;
                for gender in genders {
                    inner.agreement.gender = gender;
                    cells.push(GrammarCell::Participle(inner));
                }
            }
            GrammarCell::Pronoun(mut inner) => {
                inner.case = case;
                inner.number = Number::Singular;
                if inner.gender.is_some() {
                    for gender in genders {
                        inner.gender = Some(gender);
                        cells.push(GrammarCell::Pronoun(inner));
                    }
                } else {
                    cells.push(GrammarCell::Pronoun(inner));
                }
            }
            _ => {}
        }
    }
    cells
}

fn trace(rule: &str, stage: &str, input: String, output: String) -> TraceStep {
    TraceStep {
        rule: RuleId::from(rule),
        stage: stage.into(),
        input,
        output,
        source_recension: Some(Recension::SynodalRussian),
        target_recension: Recension::SynodalRussian,
        mapping: None,
        evidence: vec![],
    }
}

fn set_printed(variant: &mut FormVariant, rule: &str, stage: &str, printed: String) {
    if printed == variant.printed {
        return;
    }
    variant
        .rule_trace
        .push(trace(rule, stage, variant.printed.clone(), printed.clone()));
    if variant.accented.is_some() {
        variant.accented = Some(printed.clone());
    }
    variant.printed = printed;
}

/// The language-wide liturgical presentation of one cell, applied after the
/// lexical (exact, positional, accent) realisation of both the exact and the
/// generated path:
///
/// 1. gold contract §3.2 — a verse-initial capital in a reviewed print is
///    presentation, so the cell surface is the lowercase print;
/// 2. Alypy §36 — plural genitive/dative `-ѡвъ`/`-ѡмъ`/`-євъ`/`-ємъ` on
///    generated nominal cells (reviewed exact prints already carry theirs);
/// 3. Alypy §2 — word-initial broad on `ѻ`;
/// 4. Alypy §36 — the letter antistich (`ѡ`/`є`) on a generated plural or
///    dual cell that is homographic with a singular cell of the same lexeme;
/// 5. Alypy §5 — the pre-enclitic acute / isolated grave pair on a final
///    vowel, and the unaccented print of a monosyllabic grave-bearing
///    pronoun, emitted as additional variants after the isolated print.
fn present_liturgical_cell(
    inflector: Inflector,
    id: &LexemeId,
    cell: GrammarCell,
    forms: FormSet,
    generated: bool,
) -> Result<FormSet> {
    if inflector.orthography() != OrthographyProfile::SynodalLiturgical {
        return Ok(forms);
    }
    let nominal = nominal_number_and_case(cell);
    let mut variants: Vec<FormVariant> = forms.variants().to_vec();
    for variant in &mut variants {
        let printed = present_initial_broad_on(&lowercase_initial(&variant.printed));
        set_printed(
            variant,
            PRESENTATION_RULE,
            "liturgical-presentation",
            printed,
        );
    }
    if generated && matches!(nominal, Some((Number::Plural | Number::Dual, _))) {
        // The singular prints of the same lexeme (reviewed exact prints
        // included), computed once per plural or dual request.
        let mut singulars: Option<Vec<String>> = None;
        let mut singular_surfaces_of = |inflector, id, cell| {
            singulars
                .get_or_insert_with(|| singular_surfaces(inflector, id, cell))
                .clone()
        };
        match nominal {
            // The genitive plural ending is always wide (дарѡ́въ, ѻ҆тцє́въ).
            Some((Number::Plural, Case::Genitive)) => {
                for variant in &mut variants {
                    let widened = widen_plural_ending_printed(nominal, &variant.printed);
                    set_printed(variant, ANTISTICH_RULE, "wide-plural-ending", widened);
                }
            }
            // The dative plural ending is wide exactly where the declension
            // gives the instrumental singular the same letters (ѻ҆тцє́мъ /
            // ѻ҆тце́мъ, мꙋжє́мъ / мꙋ́жемъ, словесє́мъ / словесе́мъ, пꙋтє́мъ /
            // пꙋте́мъ); feminine i-stems with -їю instrumentals keep the
            // narrow letter (лю́демъ, за́повѣдемъ, ме́рзостемъ).
            Some((Number::Plural, Case::Dative)) => {
                let candidates: Vec<usize> = variants
                    .iter()
                    .enumerate()
                    .filter(|(_, variant)| {
                        widen_plural_ending_printed(nominal, &variant.printed) != variant.printed
                    })
                    .map(|(index, _)| index)
                    .collect();
                if !candidates.is_empty() {
                    let accentless: Vec<String> = singular_surfaces_of(inflector, id, cell)
                        .iter()
                        .map(|surface| normalize_lookup_accentless(surface))
                        .collect();
                    for index in candidates {
                        let variant = &mut variants[index];
                        if accentless.contains(&normalize_lookup_accentless(&variant.printed)) {
                            let widened = widen_plural_ending_printed(nominal, &variant.printed);
                            set_printed(variant, ANTISTICH_RULE, "wide-plural-ending", widened);
                        }
                    }
                }
            }
            _ => {}
        }
        // The letter antistich applies to a print that is fully homographic
        // (accents included: во́ды / воды̀ and дѣ́ла / дѣла̀ stay narrow;
        // жєны̀ / жены̀ and ю҆́нѡши / ю҆́ноши do not) with a singular print.
        let candidates: Vec<usize> = variants
            .iter()
            .enumerate()
            .filter(|(_, variant)| antistich_letter(&variant.printed).is_some())
            .map(|(index, _)| index)
            .collect();
        if !candidates.is_empty() {
            let singulars = singular_surfaces_of(inflector, id, cell);
            for index in candidates {
                let variant = &mut variants[index];
                if singulars.contains(&variant.printed) {
                    if let Some(substituted) = antistich_letter(&variant.printed) {
                        set_printed(variant, ANTISTICH_RULE, "number-antistich", substituted);
                    }
                }
            }
        }
    }
    let mut environment = Vec::new();
    for variant in &variants {
        let mut alternates = Vec::new();
        if let Some(acute) = final_accent_alternate(&variant.printed) {
            alternates.push(("pre-enclitic-or-isolated-accent", acute));
        }
        if matches!(cell, GrammarCell::Pronoun(_)) {
            if let Some(bare) = unaccented_enclitic(&variant.printed) {
                alternates.push(("unaccented-enclitic", bare));
            }
        }
        for (stage, printed) in alternates {
            let mut alternate = variant.clone();
            set_printed(&mut alternate, ENCLITIC_RULE, stage, printed);
            // The environment print is a variant of the cell, not an
            // isolated surface: the analyzer indexes the isolated print and
            // reaches these through the accent-insensitive projection.
            alternate.assumptions.push(Assumption {
                code: ENCLITIC_ENVIRONMENT_ASSUMPTION.into(),
                detail: format!("{stage}: {} beside {}", alternate.printed, variant.printed),
            });
            environment.push(alternate);
        }
    }
    for alternate in environment {
        if !variants
            .iter()
            .any(|variant| variant.printed == alternate.printed)
        {
            variants.push(alternate);
        }
    }
    FormSet::try_from_variants(variants)
}

/// Every printed surface of the lexeme's singular cells that share the
/// requested cell's other dimensions (reviewed exact prints included).
/// Unavailable cells contribute nothing.
fn singular_surfaces(inflector: Inflector, id: &LexemeId, cell: GrammarCell) -> Vec<String> {
    let mut surfaces = Vec::new();
    for singular in singular_counterparts(cell) {
        if let Ok(forms) = resolve_cell(inflector, id, singular) {
            surfaces.extend(
                forms
                    .variants()
                    .iter()
                    .map(|variant| variant.printed.clone()),
            );
        }
    }
    surfaces
}

/// Alypy §73: attach the reflexive enclitic to every generated cell of a
/// reflexive verb, on the expanded surface, before accent realisation.
fn apply_reflexive(forms: FormSet) -> Result<FormSet> {
    let variants = forms
        .variants()
        .iter()
        .cloned()
        .map(|mut variant| {
            let input = variant.printed.clone();
            variant.expanded = synodal_church_slavonic_core::reflexive_surface(&variant.expanded);
            variant.printed = synodal_church_slavonic_core::reflexive_surface(&variant.printed);
            variant.accented = variant
                .accented
                .as_deref()
                .map(synodal_church_slavonic_core::reflexive_surface);
            variant.rule_trace.push(TraceStep {
                rule: RuleId::from(synodal_church_slavonic_core::REFLEXIVE_RULE_ID),
                stage: "reflexive-enclitic".into(),
                input,
                output: variant.printed.clone(),
                source_recension: Some(Recension::SynodalRussian),
                target_recension: Recension::SynodalRussian,
                mapping: None,
                evidence: vec![],
            });
            variant
        })
        .collect();
    FormSet::try_from_variants(variants)
}

fn apply_generated_presentation(
    inflector: Inflector,
    id: &LexemeId,
    cell: GrammarCell,
    key: &str,
    forms: FormSet,
) -> Result<FormSet> {
    // The positional spelling belongs to the expanded orthography itself
    // (§36 градѡ́въ carries its omega in every profile), so a reviewed
    // positional paradigm applies under every profile, before the
    // liturgical-only accent realisation below.
    let positional = registry::positional_paradigm_for(id, cell)?;
    let forms = if let Some(paradigm) = &positional {
        let mut variants = Vec::with_capacity(forms.variants().len());
        for source_variant in forms.variants() {
            let mut variant = source_variant.clone();
            let positioned = paradigm.apply(cell, &variant.expanded)?;
            if positioned != variant.expanded {
                variant.evidence.push(paradigm.evidence.clone());
                variant.rule_trace.push(TraceStep {
                    rule: RuleId::from(format!("SYN-POSITIONAL-PARADIGM:{}", paradigm.id)),
                    stage: "positional-realization".into(),
                    input: variant.expanded.clone(),
                    output: positioned.clone(),
                    source_recension: Some(Recension::SynodalRussian),
                    target_recension: Recension::SynodalRussian,
                    mapping: variant.recension_mapping.clone(),
                    evidence: vec![paradigm.evidence.id.clone()],
                });
                if variant.printed == variant.expanded {
                    variant.printed = positioned.clone();
                }
                variant.expanded = positioned;
            }
            variants.push(variant);
        }
        FormSet::try_from_variants(variants)?
    } else {
        forms
    };
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
        // The printed profile writes the expanded word-initial `оу` as the
        // digraph `ᲂу`, exactly as every reviewed exact print in the registry
        // does; `accented` keeps the expanded letters under their marks.
        variant.printed =
            synodal_church_slavonic_core::present_initial_uk_digraph(&variant.printed);
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
            // Reviewed third-person dual and plural prints (ѧ҆̀, и҆́хъ, и҆̀мъ)
            // are keyed with gender `any`; a gender-specific request reaches
            // them through the gender-neutral key exactly as an animacy-
            // specific adjective request reaches its `any` row.
            if pronoun.gender.is_some() {
                let mut neutral = pronoun;
                neutral.gender = None;
                keys.push(pronoun_key(neutral, None));
            }
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
    // The merged irregular table: an exact row stamped with irregular
    // provenance by the extractor is the override; no second lookup exists.
    let irregular_evidence = records
        .iter()
        .map(|record| record.irregular_evidence)
        .find(|evidence| !evidence.is_empty());
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

#[cfg(test)]
mod positional_ordering_tests {
    use synodal_church_slavonic_core::{
        AccentMark, AccentParadigm, AccentPlacement, AccentRule, AccentScope, Animacy,
        AuthorityRole, Case, EpistemicRole, Evidence, EvidenceId, EvidenceKind, GrammarCell,
        InitialPresentation, NounCell, Number, PositionalOperation, PositionalParadigm,
        PositionalRule, Recension, SourceId,
    };

    fn evidence_of(kind: EvidenceKind) -> Evidence {
        Evidence {
            id: EvidenceId::from("positional-ordering-test"),
            source: SourceId::from("ponomar-elizabeth-bible-2026-08-09"),
            source_recension: Recension::SynodalRussian,
            kind,
            authority_roles: vec![AuthorityRole::Orthographic, AuthorityRole::Accentual],
            epistemic_role: EpistemicRole::SynodalNormativeAuthority,
            citation: "ordering test".into(),
            note: None,
        }
    }

    /// The v0.11 phase-3 defect, resolved: the positional paradigm rewrites
    /// the unaccented expanded form first, and the accent paradigm then
    /// realises its mark over the positioned form — the order the core
    /// permits. A `preserve` rule is a semantic no-op rather than an error.
    #[test]
    fn positional_runs_before_accent_and_preserve_is_a_no_op() {
        let cell = GrammarCell::Noun(NounCell {
            case: Case::Nominative,
            number: Number::Singular,
            animacy: Animacy::Inanimate,
        });
        let positional = PositionalParadigm {
            id: "test-wide-e".into(),
            rules: vec![PositionalRule {
                scope: AccentScope::All,
                operations: vec![PositionalOperation::Initial(InitialPresentation::WideE)],
            }],
            evidence: evidence_of(EvidenceKind::OrthographicParadigm),
        };
        let positioned = positional.apply(cell, "езеро").expect("positioned");
        assert_eq!(positioned, "єзеро");
        let accent = AccentParadigm {
            id: "test-accent".into(),
            accent_rules: vec![AccentRule {
                scope: AccentScope::Noun {
                    numbers: vec![Number::Singular],
                },
                placement: AccentPlacement::StemVowelFromStart(1),
                mark: AccentMark::Acute,
            }],
            breathing_rules: vec![],
            evidence: evidence_of(EvidenceKind::AccentParadigm),
        };
        assert_eq!(
            accent
                .apply(cell, &positioned)
                .expect("accent over the positioned form"),
            "є\u{486}зе\u{301}ро"
        );
        // The old ordering — positional over an already-accented print —
        // could never succeed; that is exactly why the resolver now runs
        // positional first.
        assert!(positional.apply(cell, "є\u{486}зе\u{301}ро").is_err());
        let preserve = PositionalParadigm::preserve(
            "test-preserve",
            AccentScope::All,
            evidence_of(EvidenceKind::OrthographicParadigm),
        );
        assert_eq!(
            preserve.apply(cell, "престоломъ").expect("preserve"),
            "престоломъ"
        );
    }
}
