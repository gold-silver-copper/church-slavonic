use super::*;

pub(super) fn fuse_form_sets(
    parts: &[&FormSet],
    accent_component: usize,
    rule: &'static str,
    citation: &'static str,
    profile: OrthographyProfile,
) -> Result<FormSet> {
    if accent_component >= parts.len() {
        return Err(Error::ContradictoryMetadata {
            reason: format!(
                "fused numeral accent component {accent_component} is outside {} components",
                parts.len()
            ),
        });
    }
    let mut products = vec![Vec::<FormVariant>::new()];
    for forms in parts {
        let mut next = Vec::new();
        for prefix in &products {
            for variant in forms.variants() {
                let mut combination = prefix.clone();
                combination.push(variant.clone());
                next.push(combination);
            }
        }
        products = next;
    }

    let rule_id = RuleId::from(rule);
    let construction_evidence = numeral_evidence(rule, citation);
    let construction_evidence_id = construction_evidence.id.clone();
    let mut variants = Vec::new();
    for product in products {
        let expanded = product
            .iter()
            .map(|item| item.expanded.as_str())
            .collect::<String>();
        let component_printed = product
            .iter()
            .map(|item| item.printed.as_str())
            .collect::<String>();
        let (accented, printed) = if profile == OrthographyProfile::SynodalLiturgical {
            let local_accent = accented_vowel_from_start(&product[accent_component].printed)
                .ok_or(Error::OrthographicMetadataRequired {
                    field: MetadataField::AccentParadigm,
                })?;
            let preceding_vowels: usize = product[..accent_component]
                .iter()
                .map(|item| synodal_vowel_count(&item.expanded))
                .sum();
            let word_vowel = preceding_vowels
                .checked_add(usize::from(local_accent))
                .and_then(|index| u8::try_from(index).ok())
                .ok_or_else(|| Error::ContradictoryMetadata {
                    reason: "fused numeral accent index exceeds the typed word-vowel range".into(),
                })?;
            let unmarked = strip_accent_and_breathing(&component_printed);
            let paradigm = AccentParadigm {
                id: format!("fused-accent:{rule}:{accent_component}"),
                accent_rules: vec![AccentRule {
                    scope: AccentScope::All,
                    placement: AccentPlacement::WordVowelFromStart(word_vowel),
                    mark: AccentMark::Acute,
                }],
                breathing_rules: Vec::new(),
                evidence: fused_accent_evidence(rule, citation),
            };
            let printed = paradigm.apply(GrammarCell::LexicalForm, &unmarked)?;
            (Some(printed.clone()), printed)
        } else {
            (None, component_printed)
        };
        let mut evidence = Vec::new();
        let mut evidence_ids = Vec::new();
        let mut trace = Vec::new();
        let mut assumptions = Vec::new();
        let mut contradictions = Vec::new();
        let mut warnings = Vec::new();
        let mut confidence = Confidence::CERTAIN;
        for item in product {
            confidence = confidence.min(item.confidence);
            assumptions.extend(item.assumptions);
            contradictions.extend(item.contradictions);
            warnings.extend(item.warnings);
            trace.extend(item.rule_trace.steps().iter().cloned());
            for item_evidence in item.evidence {
                if !evidence
                    .iter()
                    .any(|known: &Evidence| known.id == item_evidence.id)
                {
                    evidence_ids.push(item_evidence.id.clone());
                    evidence.push(item_evidence);
                }
            }
        }
        if !evidence
            .iter()
            .any(|known| known.id == construction_evidence_id)
        {
            evidence_ids.push(construction_evidence_id.clone());
            evidence.push(construction_evidence.clone());
        }
        trace.push(TraceStep {
            rule: rule_id.clone(),
            stage: "fuse-numeral-components".into(),
            input: "component form sets".into(),
            output: printed.clone(),
            source_recension: Some(Recension::SynodalRussian),
            target_recension: Recension::SynodalRussian,
            mapping: None,
            evidence: evidence_ids,
        });
        variants.push(FormVariant {
            expanded,
            accented,
            printed,
            romanization: None,
            source_recension: Some(Recension::SynodalRussian),
            target_recension: Recension::SynodalRussian,
            recension_mapping: None,
            confidence,
            source: FormSource::SynodalNormativeGeneration {
                rule: rule_id.clone(),
            },
            assumptions,
            evidence,
            contradictions,
            warnings,
            rule_trace: RuleTrace::new(trace),
        });
    }
    FormSet::try_from_variants(variants)
}

pub(super) fn fused_accent_evidence(rule: &'static str, citation: &'static str) -> Evidence {
    Evidence {
        id: EvidenceId::from(format!("accent:{rule}")),
        source: SourceId::from("alypy-gamanovich-grammar-web-2023"),
        source_recension: Recension::SynodalRussian,
        kind: EvidenceKind::AccentParadigm,
        authority_roles: vec![AuthorityRole::Accentual, AuthorityRole::Morphological],
        epistemic_role: EpistemicRole::SynodalNormativeAuthority,
        citation: citation.into(),
        note: Some(format!(
            "fused numeral stress licensed by stable rule {rule}"
        )),
    }
}

pub(super) fn accented_vowel_from_start(value: &str) -> Option<u8> {
    let mut next_vowel = 0_u8;
    let mut current_vowel = None;
    for character in value.nfd() {
        if is_synodal_vowel(character) {
            current_vowel = Some(next_vowel);
            next_vowel = next_vowel.checked_add(1)?;
        } else if matches!(character, '\u{0300}' | '\u{0301}' | '\u{0311}') {
            return current_vowel;
        }
    }
    None
}

pub(super) fn synodal_vowel_count(value: &str) -> usize {
    value
        .nfd()
        .filter(|character| is_synodal_vowel(*character))
        .count()
}

pub(super) fn strip_accent_and_breathing(value: &str) -> String {
    value
        .nfd()
        .filter(|character| {
            !matches!(
                character,
                '\u{0300}' | '\u{0301}' | '\u{0311}' | '\u{0484}' | '\u{0485}' | '\u{0486}'
            )
        })
        .nfc()
        .collect()
}

pub(super) const fn is_synodal_vowel(character: char) -> bool {
    matches!(
        character,
        'а' | 'е'
            | 'є'
            | 'и'
            | 'і'
            | 'ї'
            | 'о'
            | 'ѻ'
            | 'ѡ'
            | 'ꙍ'
            | 'у'
            | 'ꙋ'
            | 'ы'
            | 'ю'
            | 'я'
            | 'ѧ'
            | 'ѩ'
            | 'ѣ'
            | 'ѥ'
            | 'ѫ'
            | 'ѭ'
            | 'ѵ'
    )
}

pub(super) fn numeral_evidence(rule: &'static str, citation: &'static str) -> Evidence {
    Evidence {
        id: EvidenceId::from(format!("normative:{rule}")),
        source: SourceId::from("alypy-gamanovich-grammar-web-2023"),
        source_recension: Recension::SynodalRussian,
        kind: EvidenceKind::NormativeRule,
        authority_roles: vec![AuthorityRole::Grammatical, AuthorityRole::Morphological],
        epistemic_role: EpistemicRole::SynodalNormativeAuthority,
        citation: citation.into(),
        note: Some(format!("stable rule {rule}")),
    }
}
