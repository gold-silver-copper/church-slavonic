use super::*;

pub(super) fn digit_form(
    digit: u8,
    case: Case,
    gender: Option<Gender>,
    animacy: Animacy,
    inflector: Inflector,
) -> Result<FormSet> {
    let (id, number) = match digit {
        1 => ("synodal:numeral:edin", Number::Singular),
        2 => ("synodal:numeral:dva", Number::Dual),
        3 => ("synodal:numeral:tri", Number::Plural),
        4 => ("synodal:numeral:chetyre", Number::Plural),
        5 => ("synodal:numeral:wikt-42c5d78bab14", Number::Singular),
        6 => ("synodal:numeral:wikt-58a4f8eb4197", Number::Singular),
        7 => ("synodal:numeral:wikt-2fe80b81eaf8", Number::Singular),
        8 => ("synodal:numeral:v06-7391e80a474691c3", Number::Singular),
        9 => ("synodal:numeral:wikt-04f311cf0bd0", Number::Singular),
        _ => {
            return Err(Error::InvalidNumeral {
                reason: "a cardinal digit component must be one through nine".into(),
            });
        }
    };
    let cell = GrammarCell::Numeral(NumeralCell {
        kind: NumeralKind::Cardinal,
        case,
        number,
        gender,
        animacy,
    });
    let id = LexemeId::from(id);
    match inflector.form_by_id(&id, cell) {
        Ok(forms) => Ok(forms),
        Err(Error::OrthographicMetadataRequired {
            field: MetadataField::AccentParadigm | MetadataField::PositionalParadigm,
        }) if inflector.orthography() == OrthographyProfile::SynodalLiturgical
            && (digit == 1 || (5..=9).contains(&digit)) =>
        {
            let expanded = expanded_composition_inflector(inflector).form_by_id(&id, cell)?;
            if digit == 1 {
                accent_component_form_set(
                    &expanded,
                    cell,
                    AccentPlacement::WordVowelFromStart(0),
                    AccentMark::Acute,
                    "SYN-NUMERAL-CARDINAL-ONE-ALYPY-62",
                    "Alypy (Gamanovich), §62 complete є҆ди́нъ cardinal paradigm",
                )
            } else {
                accent_cardinal_i_stem_digit_form(&expanded, cell, digit, case)
            }
        }
        Err(error) => Err(error),
    }
}

pub(super) fn accent_cardinal_i_stem_digit_form(
    forms: &FormSet,
    cell: GrammarCell,
    digit: u8,
    case: Case,
) -> Result<FormSet> {
    let accent_evidence = fused_accent_evidence(
        "SYN-NUMERAL-CARDINAL-I-STEM-ALYPY-62",
        "Alypy (Gamanovich), §62 pѧ́ть–де́вѧть third-declension cardinal paradigm",
    );
    let accent_evidence_id = accent_evidence.id.clone();
    let rule_id = RuleId::from("SYN-NUMERAL-CARDINAL-I-STEM-ALYPY-62");
    let placement = if matches!(case, Case::Nominative | Case::Accusative) {
        AccentPlacement::WordVowelFromStart(0)
    } else {
        AccentPlacement::EndingVowelFromEnd(0)
    };
    let paradigm = AccentParadigm {
        id: "component-accent:SYN-NUMERAL-CARDINAL-I-STEM-ALYPY-62".into(),
        accent_rules: vec![AccentRule {
            scope: AccentScope::All,
            placement,
            mark: AccentMark::Acute,
        }],
        breathing_rules: Vec::new(),
        evidence: accent_evidence.clone(),
    };
    let positional_evidence = (digit == 8).then(|| Evidence {
        id: EvidenceId::from("orthography:SYN-NUMERAL-CARDINAL-OSM-BROAD-ON-ALYPY-2-62"),
        source: SourceId::from("alypy-gamanovich-grammar-web-2023"),
        source_recension: Recension::SynodalRussian,
        kind: EvidenceKind::OrthographicParadigm,
        authority_roles: vec![AuthorityRole::Orthographic],
        epistemic_role: EpistemicRole::SynodalNormativeAuthority,
        citation: "Alypy (Gamanovich), §§2 and 62 initial ѻ҆́смь presentation".into(),
        note: Some("initial broad-on presentation is restricted to the numeral осмь".into()),
    });
    let mut variants = Vec::new();
    for source in forms.variants() {
        let positional = if digit == 8 {
            apply_initial_presentation(
                &SynodalWord::parse(&source.expanded)?,
                InitialPresentation::BroadOn,
            )?
            .normalized
        } else {
            source.expanded.clone()
        };
        let printed = paradigm.apply(cell, &positional)?;
        let mut variant = source.clone();
        variant.accented = Some(printed.clone());
        variant.printed = printed.clone();
        if !variant
            .evidence
            .iter()
            .any(|known| known.id == accent_evidence_id)
        {
            variant.evidence.push(accent_evidence.clone());
        }
        if let Some(evidence) = &positional_evidence
            && !variant.evidence.iter().any(|known| known.id == evidence.id)
        {
            variant.evidence.push(evidence.clone());
        }
        variant.rule_trace.push(TraceStep {
            rule: rule_id.clone(),
            stage: "numeral-component-liturgical-presentation".into(),
            input: source.expanded.clone(),
            output: printed,
            source_recension: Some(Recension::SynodalRussian),
            target_recension: Recension::SynodalRussian,
            mapping: None,
            evidence: variant
                .evidence
                .iter()
                .map(|item| item.id.clone())
                .collect(),
        });
        variants.push(variant);
    }
    FormSet::try_from_variants(variants)
}

pub(super) fn ten_form(
    case: Case,
    number: Number,
    animacy: Animacy,
    inflector: Inflector,
) -> Result<FormSet> {
    let cell = GrammarCell::Numeral(NumeralCell {
        kind: NumeralKind::Cardinal,
        case,
        number,
        gender: None,
        animacy,
    });
    if inflector.orthography() != OrthographyProfile::SynodalLiturgical {
        return inflector.form_by_id(&LexemeId::from("synodal:numeral:wikt-bc270882d39d"), cell);
    }
    let expanded = expanded_composition_inflector(inflector)
        .form_by_id(&LexemeId::from("synodal:numeral:wikt-bc270882d39d"), cell)?;
    let (placement, mark) = match (number, case) {
        (Number::Singular, Case::Nominative | Case::Accusative) => {
            (AccentPlacement::WordVowelFromStart(0), AccentMark::Acute)
        }
        (Number::Singular, _) => (AccentPlacement::EndingVowelFromEnd(0), AccentMark::Acute),
        (Number::Dual, Case::Nominative | Case::Genitive | Case::Accusative | Case::Locative) => {
            (AccentPlacement::WordVowelFromStart(1), AccentMark::Acute)
        }
        (Number::Dual, _) => (AccentPlacement::EndingVowelFromEnd(0), AccentMark::Acute),
        (Number::Plural, Case::Instrumental) => {
            (AccentPlacement::EndingVowelFromEnd(0), AccentMark::Acute)
        }
        (Number::Plural, _) => (AccentPlacement::WordVowelFromStart(1), AccentMark::Acute),
    };
    accent_component_form_set(
        &expanded,
        cell,
        placement,
        mark,
        "SYN-NUMERAL-CARDINAL-TEN-ALYPY-62",
        "Alypy (Gamanovich), §62 complete де́сѧть paradigm",
    )
}

pub(super) fn expanded_composition_inflector(inflector: Inflector) -> Inflector {
    Inflector::builder()
        .generation_policy(inflector.generation_policy())
        .orthography(OrthographyProfile::Expanded)
        .productive_mapping_threshold_basis_points(
            inflector.productive_mapping_threshold_basis_points(),
        )
        .build()
}

pub(super) fn accent_component_form_set(
    forms: &FormSet,
    cell: GrammarCell,
    placement: AccentPlacement,
    mark: AccentMark,
    rule: &'static str,
    citation: &'static str,
) -> Result<FormSet> {
    let evidence = fused_accent_evidence(rule, citation);
    let evidence_id = evidence.id.clone();
    let rule_id = RuleId::from(rule);
    let paradigm = AccentParadigm {
        id: format!("component-accent:{rule}"),
        accent_rules: vec![AccentRule {
            scope: AccentScope::All,
            placement,
            mark,
        }],
        breathing_rules: Vec::new(),
        evidence: evidence.clone(),
    };
    let mut variants = Vec::new();
    for source in forms.variants() {
        let printed = paradigm.apply(cell, &source.expanded)?;
        let mut variant = source.clone();
        variant.accented = Some(printed.clone());
        variant.printed = printed.clone();
        if !variant.evidence.iter().any(|known| known.id == evidence_id) {
            variant.evidence.push(evidence.clone());
        }
        variant.rule_trace.push(TraceStep {
            rule: rule_id.clone(),
            stage: "numeral-component-accent".into(),
            input: source.expanded.clone(),
            output: printed,
            source_recension: Some(Recension::SynodalRussian),
            target_recension: Recension::SynodalRussian,
            mapping: None,
            evidence: variant
                .evidence
                .iter()
                .map(|item| item.id.clone())
                .collect(),
        });
        variants.push(variant);
    }
    FormSet::try_from_variants(variants)
}

pub(super) fn magnitude_form(
    magnitude: Magnitude,
    case: Case,
    number: Number,
    inflector: Inflector,
) -> Result<FormSet> {
    let cell = GrammarCell::Numeral(NumeralCell {
        kind: NumeralKind::Cardinal,
        case,
        number,
        gender: None,
        animacy: Animacy::Inanimate,
    });
    let id = LexemeId::from(magnitude.id());
    match inflector.form_by_id(&id, cell) {
        Ok(forms) => Ok(forms),
        Err(Error::OrthographicMetadataRequired {
            field: MetadataField::AccentParadigm | MetadataField::PositionalParadigm,
        }) if magnitude == Magnitude::Hundred
            && inflector.orthography() == OrthographyProfile::SynodalLiturgical =>
        {
            let expanded = expanded_composition_inflector(inflector).form_by_id(&id, cell)?;
            accent_hundred_form(&expanded, cell, case, number)
        }
        Err(error) => Err(error),
    }
}

pub(super) fn accent_hundred_form(
    forms: &FormSet,
    cell: GrammarCell,
    case: Case,
    number: Number,
) -> Result<FormSet> {
    let (placement, mark) = match (number, case) {
        (Number::Singular, Case::Instrumental) => {
            (AccentPlacement::WordVowelFromStart(0), AccentMark::Acute)
        }
        (Number::Singular, _) => (AccentPlacement::EndingVowelFromEnd(0), AccentMark::Acute),
        (Number::Dual, Case::Nominative | Case::Genitive | Case::Accusative | Case::Locative)
        | (Number::Plural, Case::Nominative | Case::Accusative) => {
            (AccentPlacement::EndingVowelFromEnd(0), AccentMark::Kamora)
        }
        (Number::Dual, _) | (Number::Plural, Case::Genitive | Case::Dative | Case::Locative) => {
            (AccentPlacement::WordVowelFromStart(0), AccentMark::Acute)
        }
        (Number::Plural, Case::Instrumental) => {
            (AccentPlacement::EndingVowelFromEnd(0), AccentMark::Acute)
        }
        (_, Case::Vocative) => {
            return Err(Error::HistoricallyInvalidCell {
                reason: "Alypy §62 does not license a cardinal hundred vocative".into(),
            });
        }
    };
    let accent_evidence = fused_accent_evidence(
        "SYN-NUMERAL-CARDINAL-HUNDRED-ALYPY-62",
        "Alypy (Gamanovich), §62 complete сто̀ paradigm",
    );
    let positional_evidence = Evidence {
        id: EvidenceId::from("orthography:SYN-NUMERAL-CARDINAL-HUNDRED-OMEGA-ALYPY-62"),
        source: SourceId::from("alypy-gamanovich-grammar-web-2023"),
        source_recension: Recension::SynodalRussian,
        kind: EvidenceKind::OrthographicParadigm,
        authority_roles: vec![AuthorityRole::Orthographic],
        epistemic_role: EpistemicRole::SynodalNormativeAuthority,
        citation: "Alypy (Gamanovich), §62 plural сѡ́тъ / стѡ́мъ hundred forms".into(),
        note: Some("omega presentation is restricted to the cited plural cells".into()),
    };
    let positional = PositionalParadigm {
        id: "component-position:SYN-NUMERAL-CARDINAL-HUNDRED-OMEGA-ALYPY-62".into(),
        rules: vec![PositionalRule {
            scope: AccentScope::All,
            operations: if number == Number::Plural && matches!(case, Case::Genitive | Case::Dative)
            {
                vec![PositionalOperation::Replace {
                    replacement: PositionalReplacement::Omega,
                    occurrence: LetterOccurrence::FromStart(0),
                }]
            } else {
                Vec::new()
            },
        }],
        evidence: positional_evidence.clone(),
    };
    let accent = AccentParadigm {
        id: "component-accent:SYN-NUMERAL-CARDINAL-HUNDRED-ALYPY-62".into(),
        accent_rules: vec![AccentRule {
            scope: AccentScope::All,
            placement,
            mark,
        }],
        breathing_rules: Vec::new(),
        evidence: accent_evidence.clone(),
    };
    let rule_id = RuleId::from("SYN-NUMERAL-CARDINAL-HUNDRED-ALYPY-62");
    let two_hundred_variant_evidence = numeral_evidence(
        "SYN-NUMERAL-CARDINAL-TWO-HUNDRED-STI-ALYPY-63",
        "Alypy (Gamanovich), §63 двѣ́стѣ / двѣ́сти alternatives",
    );
    let mut variants = Vec::new();
    for source in forms.variants() {
        let mut expanded_forms = vec![source.expanded.clone()];
        if number == Number::Dual
            && matches!(case, Case::Nominative | Case::Accusative)
            && source.expanded == "стѣ"
        {
            expanded_forms.push("сти".into());
        }
        for expanded in expanded_forms {
            let positioned = positional.apply(cell, &expanded)?;
            let printed = accent.apply(cell, &positioned)?;
            let mut variant = source.clone();
            variant.expanded = expanded.clone();
            variant.accented = Some(printed.clone());
            variant.printed = printed.clone();
            for evidence in [&accent_evidence, &positional_evidence] {
                if !variant.evidence.iter().any(|known| known.id == evidence.id) {
                    variant.evidence.push(evidence.clone());
                }
            }
            if expanded == "сти"
                && !variant
                    .evidence
                    .iter()
                    .any(|known| known.id == two_hundred_variant_evidence.id)
            {
                variant.evidence.push(two_hundred_variant_evidence.clone());
            }
            variant.rule_trace.push(TraceStep {
                rule: rule_id.clone(),
                stage: "numeral-hundred-liturgical-presentation".into(),
                input: source.expanded.clone(),
                output: printed,
                source_recension: Some(Recension::SynodalRussian),
                target_recension: Recension::SynodalRussian,
                mapping: None,
                evidence: variant
                    .evidence
                    .iter()
                    .map(|evidence| evidence.id.clone())
                    .collect(),
            });
            variants.push(variant);
        }
    }
    FormSet::try_from_variants(variants)
}

pub(super) fn fixed_ten_accusative(profile: OrthographyProfile) -> Result<FormSet> {
    grammar_forms(
        &[("десѧть", "де́сѧть"), ("десѧте", "де́сѧте")],
        "SYN-NUMERAL-CARDINAL-TEEN-ALYPY-63-64",
        "Alypy (Gamanovich), §§63–64 invariant accusative ten in teens",
        profile,
    )
}

pub(super) fn fixed_genitive_plural_ten(profile: OrthographyProfile) -> Result<FormSet> {
    grammar_form(
        "десѧтъ",
        Some("десѧ́тъ"),
        "SYN-NUMERAL-CARDINAL-TENS-GOVERNMENT-ALYPY-63-64",
        "Alypy (Gamanovich), §§63–64 governed genitive-plural ten",
        profile,
    )
}

pub(super) fn grammar_forms(
    forms: &[(&str, &str)],
    rule: &'static str,
    citation: &'static str,
    profile: OrthographyProfile,
) -> Result<FormSet> {
    let mut variants = Vec::new();
    for (form, accented) in forms {
        variants.extend(
            grammar_form(form, Some(accented), rule, citation, profile)?
                .variants()
                .to_vec(),
        );
    }
    FormSet::try_from_variants(variants)
}

pub(super) fn grammar_form(
    expanded: &str,
    accented: Option<&str>,
    rule: &'static str,
    citation: &'static str,
    profile: OrthographyProfile,
) -> Result<FormSet> {
    let expanded = SynodalWord::parse(expanded)?.canonical().to_owned();
    let (accented, printed, warnings) = match profile {
        OrthographyProfile::Expanded => (None, expanded.clone(), Vec::new()),
        OrthographyProfile::ExpandedAccentless => {
            let form = normalize_lookup_accentless(&expanded);
            (
                None,
                form.clone(),
                vec!["accent and breathing marks removed".into()],
            )
        }
        OrthographyProfile::SynodalLiturgical => {
            let accented = accented.ok_or(Error::OrthographicMetadataRequired {
                field: MetadataField::AccentParadigm,
            })?;
            let accented = SynodalWord::parse(accented)?.canonical().to_owned();
            (Some(accented.clone()), accented, Vec::new())
        }
    };
    let rule_id = RuleId::from(rule);
    let evidence = numeral_evidence(rule, citation);
    let evidence_id = evidence.id.clone();
    FormSet::new(FormVariant {
        expanded: expanded.clone(),
        accented,
        printed: printed.clone(),
        romanization: None,
        source_recension: Some(Recension::SynodalRussian),
        target_recension: Recension::SynodalRussian,
        recension_mapping: None,
        confidence: Confidence::from_basis_points(9_500).unwrap_or(Confidence::CERTAIN),
        source: FormSource::SynodalNormativeGeneration {
            rule: rule_id.clone(),
        },
        assumptions: Vec::new(),
        evidence: vec![evidence],
        contradictions: Vec::new(),
        warnings,
        rule_trace: RuleTrace::new(vec![TraceStep {
            rule: rule_id,
            stage: "numeral-composition-token".into(),
            input: expanded,
            output: printed,
            source_recension: Some(Recension::SynodalRussian),
            target_recension: Recension::SynodalRussian,
            mapping: None,
            evidence: vec![evidence_id],
        }]),
    })
}
