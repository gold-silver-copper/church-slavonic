use super::*;

pub(super) fn tag_tokens(
    tokens: &[PhraseToken],
    rule: &'static str,
    citation: &'static str,
) -> Result<Vec<PhraseToken>> {
    tokens
        .iter()
        .map(|token| {
            Ok(PhraseToken {
                role: token.role,
                forms: tag_form_set(&token.forms, rule, citation)?,
            })
        })
        .collect()
}

pub(super) fn tag_form_set(
    forms: &FormSet,
    rule: &'static str,
    citation: &'static str,
) -> Result<FormSet> {
    let rule_id = RuleId::from(rule);
    let construction_evidence = numeral_evidence(rule, citation);
    let construction_evidence_id = construction_evidence.id.clone();
    let mut variants = Vec::with_capacity(forms.variants().len());
    for source in forms.variants() {
        let mut variant = source.clone();
        if !variant
            .evidence
            .iter()
            .any(|known| known.id == construction_evidence_id)
        {
            variant.evidence.push(construction_evidence.clone());
        }
        let evidence = variant
            .evidence
            .iter()
            .map(|item| item.id.clone())
            .collect();
        variant.rule_trace.push(TraceStep {
            rule: rule_id.clone(),
            stage: "numeral-phrase-construction".into(),
            input: variant.expanded.clone(),
            output: variant.printed.clone(),
            source_recension: Some(Recension::SynodalRussian),
            target_recension: Recension::SynodalRussian,
            mapping: None,
            evidence,
        });
        variants.push(variant);
    }
    FormSet::try_from_variants(variants)
}

pub(super) fn numeral_token(forms: FormSet) -> PhraseToken {
    PhraseToken {
        role: PhraseRole::Numeral,
        forms,
    }
}

pub(super) fn single_cardinal_analysis(
    construction: NumeralComposition,
    forms: FormSet,
) -> CardinalPhraseAnalysis {
    CardinalPhraseAnalysis {
        construction,
        tokens: vec![numeral_token(forms)],
    }
}

pub(super) fn render_tokens(tokens: &[PhraseToken]) -> String {
    tokens
        .iter()
        .map(|token| token.forms.primary_text())
        .collect::<Vec<_>>()
        .join(" ")
}

pub(super) fn cardinal_requires_gender(value: u32) -> bool {
    let final_two = value % 100;
    let final_digit = value % 10;
    (1..=4).contains(&final_digit) && !(15..=19).contains(&final_two)
}

pub(super) fn following_government(value: u32, case: Case) -> Vec<NumeralGovernment> {
    let final_two = value % 100;
    let final_digit = value % 10;
    let mut patterns = if (11..=14).contains(&final_two) {
        vec![
            NumeralGovernment::Agreement {
                number: inherent_number(final_digit),
            },
            NumeralGovernment::GenitivePlural,
        ]
    } else if (1..=4).contains(&final_digit) {
        vec![NumeralGovernment::Agreement {
            number: inherent_number(final_digit),
        }]
    } else if matches!(case, Case::Dative | Case::Instrumental | Case::Locative) {
        vec![
            NumeralGovernment::Agreement {
                number: Number::Plural,
            },
            NumeralGovernment::GenitivePlural,
        ]
    } else {
        vec![NumeralGovernment::GenitivePlural]
    };
    if case == Case::Nominative && value >= 5 {
        patterns.push(NumeralGovernment::ContextualNominativePlural);
    }
    patterns
}

pub(super) fn preceding_government(value: u32, case: Case) -> Vec<NumeralGovernment> {
    let leading = match value {
        1..=10 => value,
        11..=19 => value - 10,
        20..=99 => value / 10,
        100..=999 => {
            let multiplier = value / 100;
            if multiplier == 1 { 100 } else { multiplier }
        }
        1_000..=999_999 => {
            let multiplier = value / 1_000;
            if multiplier == 1 {
                1_000
            } else {
                first_component_value(multiplier)
            }
        }
        1_000_000 => 1_000_000,
        _ => value,
    };
    following_government(leading, case)
}

pub(super) fn first_component_value(value: u32) -> u32 {
    match value {
        1..=10 => value,
        11..=19 => value - 10,
        20..=99 => value / 10,
        100..=999 => {
            let multiplier = value / 100;
            if multiplier == 1 { 100 } else { multiplier }
        }
        _ => value,
    }
}

pub(super) const fn inherent_number(digit: u32) -> Number {
    match digit {
        1 => Number::Singular,
        2 => Number::Dual,
        _ => Number::Plural,
    }
}

pub(super) fn deduplicate_analyses(analyses: &mut Vec<CardinalPhraseAnalysis>) {
    let mut seen = BTreeSet::new();
    analyses.retain(|analysis| seen.insert((analysis.construction, analysis.primary_text())));
}

pub(super) fn deduplicate_ordinal_analyses(analyses: &mut Vec<OrdinalPhraseAnalysis>) {
    let mut seen = BTreeSet::new();
    analyses.retain(|analysis| seen.insert((analysis.construction, analysis.primary_text())));
}

pub(super) fn deduplicate_phrases(phrases: &mut Vec<RealizedPhrase>) {
    let mut seen = BTreeSet::new();
    phrases.retain(|phrase| seen.insert(phrase.primary_text()));
}
