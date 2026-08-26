use std::collections::BTreeSet;

use synodal_church_slavonic_core::{
    AnalyticConstruction, AuthorityRole, EncliticParticle, EpistemicRole, Error, Evidence,
    EvidenceId, EvidenceKind, FormSet, GrammarCell, LexemeId, NegativePronounBase,
    OrthographyProfile, PhraseRole, PhraseToken, PronounCell, PronounCliticProsody,
    PronounFormSelection, PronounPostpositive, RealizedPhrase, Recension, Result, RuleId, SourceId,
    TraceStep, decline_pronoun,
};
use unicode_normalization::UnicodeNormalization;

#[allow(unused_imports)]
use super::*;
use crate::{Inflector, PartOfSpeech, Pronoun};

/// Realizes Alypy §48 negative-pronoun interposition as three independently
/// evidenced tokens: `ни + preposition + inflected interrogative base`.
pub fn negative_pronoun_prepositional(
    preposition: &str,
    base: NegativePronounBase,
    postpositive: Option<PronounPostpositive>,
    cell: PronounCell,
) -> Result<RealizedPhrase> {
    negative_pronoun_prepositional_with(preposition, base, postpositive, cell, Inflector::default())
}

pub fn negative_pronoun_prepositional_with(
    preposition: &str,
    base: NegativePronounBase,
    postpositive: Option<PronounPostpositive>,
    cell: PronounCell,
    inflector: Inflector,
) -> Result<RealizedPhrase> {
    let negative_id = match (base, postpositive) {
        (NegativePronounBase::Who, None) => "synodal:pronoun:nikto",
        (NegativePronounBase::Who, Some(PronounPostpositive::Zhe)) => {
            "synodal:pronoun:v06-niktozhe"
        }
        (NegativePronounBase::What, None) => "synodal:pronoun:nichto",
        (NegativePronounBase::What, Some(PronounPostpositive::Zhe)) => {
            "synodal:pronoun:v06-nichtozhe"
        }
        (NegativePronounBase::Kii, None) => "synodal:pronoun:nikii",
        (NegativePronounBase::Kotoryi, None) => "synodal:pronoun:nikotoryi",
        (_, Some(_)) => {
            return Err(Error::ContradictoryMetadata {
                reason:
                    "preposition-interposed negative pronouns license only optional -же on кто/что"
                        .into(),
            });
        }
    };

    let preposition_summary = inflector.resolve(preposition)?;
    if preposition_summary.part_of_speech() != PartOfSpeech::Preposition {
        return Err(Error::ContradictoryMetadata {
            reason: format!(
                "negative-pronoun interposition requires a preposition, not {}",
                preposition_summary.part_of_speech().code()
            ),
        });
    }
    let negative = Pronoun::from_id_with(&LexemeId::from(negative_id), inflector)?;
    let base_forms = separated_negative_base(negative.form(cell)?)?;
    RealizedPhrase::new(
        AnalyticConstruction::NegativePronounPrepositional,
        vec![
            PhraseToken {
                role: PhraseRole::Particle,
                forms: inflector.form_by_id(
                    &LexemeId::from("synodal:particle:negative-ni"),
                    GrammarCell::Indeclinable,
                )?,
            },
            PhraseToken {
                role: PhraseRole::Preposition,
                forms: inflector.form_by_id(preposition_summary.id(), GrammarCell::Indeclinable)?,
            },
            PhraseToken {
                role: PhraseRole::Pronoun,
                forms: base_forms,
            },
        ],
    )
}

/// Joins a previously realized host and one source-licensed short personal or
/// reflexive pronoun as a typed enclitic construction (Alypy §47).
pub fn pronoun_enclitic_after_host(
    host: FormSet,
    pronoun_id: &LexemeId,
    cell: PronounCell,
    prosody: PronounCliticProsody,
) -> Result<RealizedPhrase> {
    pronoun_enclitic_after_host_with(host, pronoun_id, cell, prosody, Inflector::default())
}

pub fn pronoun_enclitic_after_host_with(
    host: FormSet,
    pronoun_id: &LexemeId,
    cell: PronounCell,
    prosody: PronounCliticProsody,
    inflector: Inflector,
) -> Result<RealizedPhrase> {
    let mut lexeme = crate::registry::pronoun_lexeme(pronoun_id)?;
    lexeme.selection = PronounFormSelection::Enclitic;
    let enclitic_expanded = decline_pronoun(&lexeme, cell, OrthographyProfile::Expanded)?
        .variants()
        .iter()
        .map(|variant| variant.expanded.clone())
        .collect::<BTreeSet<_>>();
    let registered = inflector.form_by_id(pronoun_id, GrammarCell::Pronoun(cell))?;
    let variants = registered
        .variants()
        .iter()
        .filter(|variant| enclitic_expanded.contains(&variant.expanded))
        .cloned()
        .collect::<Vec<_>>();
    if variants.is_empty() {
        return Err(Error::ContradictoryMetadata {
            reason: "the registered pronoun cell did not realize its licensed enclitic form".into(),
        });
    }
    let clitic = FormSet::try_from_variants(variants)?;
    let (host, clitic) = match prosody {
        PronounCliticProsody::AfterFinalVowelStress => {
            (stress_final_host_vowel(host)?, unaccent_enclitic(clitic)?)
        }
        PronounCliticProsody::LogicallyStressed => (
            trace_pronoun_clitic(host, "logically-stressed-host")?,
            trace_pronoun_clitic(clitic, "logically-stressed-short-pronoun")?,
        ),
    };
    RealizedPhrase::new(
        AnalyticConstruction::EncliticPronoun,
        vec![
            PhraseToken {
                role: PhraseRole::Host,
                forms: host,
            },
            PhraseToken {
                role: PhraseRole::Pronoun,
                forms: clitic,
            },
        ],
    )
}

/// Joins a host and one of Alypy §3's closed postpositive particles. A
/// word-final grave on the host becomes acute; nonfinal lexical stress is not
/// moved. The particle remains a separately sourced token.
pub fn enclitic_particle_after_host(
    host: FormSet,
    particle: EncliticParticle,
) -> Result<RealizedPhrase> {
    enclitic_particle_after_host_with(host, particle, Inflector::default())
}

pub fn enclitic_particle_after_host_with(
    host: FormSet,
    particle: EncliticParticle,
    inflector: Inflector,
) -> Result<RealizedPhrase> {
    let host = transform_enclitic_particle_forms(host, "pre-enclitic-host-accent", |text| {
        Ok(final_grave_to_acute(text))
    })?;
    let particle_id = match particle {
        EncliticParticle::Zhe => "synodal:conjunction:wikt-d01902db4fbc",
        EncliticParticle::Bo => "synodal:conjunction:wikt-b8c98d0a9447",
        EncliticParticle::Li => "synodal:conjunction:li",
    };
    let particle = transform_enclitic_particle_forms(
        inflector.form_by_id(&LexemeId::from(particle_id), GrammarCell::Indeclinable)?,
        "postpositive-enclitic-particle",
        |text| Ok(text.to_owned()),
    )?;
    RealizedPhrase::new(
        AnalyticConstruction::EncliticParticle,
        vec![
            PhraseToken {
                role: PhraseRole::Host,
                forms: host,
            },
            PhraseToken {
                role: PhraseRole::Particle,
                forms: particle,
            },
        ],
    )
}

pub(super) fn transform_enclitic_particle_forms(
    forms: FormSet,
    stage: &'static str,
    transform: impl Fn(&str) -> Result<String>,
) -> Result<FormSet> {
    let rule = RuleId::from("SYN-ORTH-FINAL-ACUTE-BEFORE-ENCLITIC-ALYPY-3");
    let evidence = enclitic_particle_evidence();
    let mut variants = Vec::with_capacity(forms.variants().len());
    for source in forms.variants() {
        let mut variant = source.clone();
        let input = variant.printed.clone();
        variant.printed = transform(&variant.printed)?;
        variant.accented = variant.accented.as_deref().map(&transform).transpose()?;
        variant.romanization = None;
        if !variant.evidence.iter().any(|item| item.id == evidence.id) {
            variant.evidence.push(evidence.clone());
        }
        let evidence_ids = variant
            .evidence
            .iter()
            .map(|item| item.id.clone())
            .collect();
        variant.rule_trace.push(TraceStep {
            rule: rule.clone(),
            stage: stage.into(),
            input,
            output: variant.printed.clone(),
            source_recension: Some(Recension::SynodalRussian),
            target_recension: Recension::SynodalRussian,
            mapping: None,
            evidence: evidence_ids,
        });
        variants.push(variant);
    }
    FormSet::try_from_variants(variants)
}

pub(super) fn final_grave_to_acute(text: &str) -> String {
    let mut decomposed = text.nfd().collect::<Vec<_>>();
    let Some(final_base) = decomposed.iter().rposition(|character| {
        !matches!(
            *character as u32,
            0x0300..=0x036f | 0x0483..=0x0489 | 0x2de0..=0x2dff | 0xfe20..=0xfe2f
        )
    }) else {
        return text.to_owned();
    };
    if is_synodal_vowel(decomposed[final_base]) {
        for mark in &mut decomposed[final_base + 1..] {
            if *mark == '\u{0300}' {
                *mark = '\u{0301}';
            }
        }
    }
    decomposed.into_iter().nfc().collect()
}

pub(super) fn enclitic_particle_evidence() -> Evidence {
    Evidence {
        id: EvidenceId::from("alypy-3-enclitic-accent"),
        source: SourceId::from("alypy-gamanovich-grammar-web-2023"),
        source_recension: Recension::SynodalRussian,
        kind: EvidenceKind::NormativeRule,
        authority_roles: vec![AuthorityRole::Grammatical, AuthorityRole::Accentual],
        epistemic_role: EpistemicRole::SynodalNormativeAuthority,
        citation: "Alypy (Gamanovich), §3.a".into(),
        note: Some("word-final acute before же, бо, or ли".into()),
    }
}

/// Realizes the source-listed masculine singular inanimate accusative
/// contractions `на(н)и → нань` and `въ(н)и → вонь` (Alypy §47).
pub fn contracted_third_person_accusative(preposition: &str) -> Result<RealizedPhrase> {
    contracted_third_person_accusative_with(preposition, Inflector::default())
}

pub fn contracted_third_person_accusative_with(
    preposition: &str,
    inflector: Inflector,
) -> Result<RealizedPhrase> {
    let preposition = inflector.resolve(preposition)?;
    if preposition.part_of_speech() != PartOfSpeech::Preposition {
        return Err(Error::ContradictoryMetadata {
            reason: format!(
                "third-person contraction requires a preposition, not {}",
                preposition.part_of_speech().code()
            ),
        });
    }
    let contraction_id = match preposition.lemma() {
        "на" => "synodal:pronoun:v07-2f384f6138f1b4bd",
        "въ" => "synodal:pronoun:v07-61cdc8112ee912fe",
        other => {
            return Err(Error::HistoricallyInvalidCell {
                reason: format!(
                    "Alypy §47 explicitly licenses contracted -нь only after на and въ, not {other}"
                ),
            });
        }
    };
    let forms = append_phrase_trace(
        inflector.form_by_id(&LexemeId::from(contraction_id), GrammarCell::LexicalForm)?,
        "SYN-PRONOUN-THIRD-PREPOSITION-CONTRACTION-ALYPY-47",
        "third-person-preposition-contraction",
    )?;
    RealizedPhrase::new(
        AnalyticConstruction::ThirdPersonPrepositionalContraction,
        vec![PhraseToken {
            role: PhraseRole::FusedPrepositionPronoun,
            forms,
        }],
    )
}

pub(super) fn stress_final_host_vowel(forms: FormSet) -> Result<FormSet> {
    transform_pronoun_clitic_forms(forms, "final-vowel-stressed-host", |text| {
        let unaccented = strip_stress_marks(text);
        let final_base = unaccented.chars().rev().find(|character| {
            !matches!(
                *character as u32,
                0x0300..=0x036f | 0x0483..=0x0489 | 0x2de0..=0x2dff | 0xfe20..=0xfe2f
            )
        });
        if !final_base.is_some_and(is_synodal_vowel) {
            return Err(Error::ContradictoryMetadata {
                reason: "AfterFinalVowelStress requires a host ending in a vowel".into(),
            });
        }
        Ok(format!("{unaccented}\u{0301}"))
    })
}

pub(super) fn unaccent_enclitic(forms: FormSet) -> Result<FormSet> {
    transform_pronoun_clitic_forms(forms, "unaccented-enclitic", |text| {
        Ok(strip_stress_marks(text))
    })
}

pub(super) fn trace_pronoun_clitic(forms: FormSet, stage: &'static str) -> Result<FormSet> {
    transform_pronoun_clitic_forms(forms, stage, |text| Ok(text.to_owned()))
}

pub(super) fn transform_pronoun_clitic_forms(
    forms: FormSet,
    stage: &'static str,
    transform: impl Fn(&str) -> Result<String>,
) -> Result<FormSet> {
    let rule = RuleId::from("SYN-PRONOUN-ENCLITIC-PROSODY-ALYPY-47");
    let mut variants = Vec::with_capacity(forms.variants().len());
    for source in forms.variants() {
        let mut variant = source.clone();
        let input = variant.printed.clone();
        variant.printed = transform(&variant.printed)?;
        variant.accented = match &variant.accented {
            Some(accented) => Some(transform(accented)?),
            None if stage == "final-vowel-stressed-host" => Some(variant.printed.clone()),
            None => None,
        };
        variant.romanization = None;
        let construction_evidence = pronoun_phrase_evidence(rule.as_ref())?;
        if !variant
            .evidence
            .iter()
            .any(|item| item.id == construction_evidence.id)
        {
            variant.evidence.push(construction_evidence);
        }
        let evidence = variant
            .evidence
            .iter()
            .map(|item| item.id.clone())
            .collect();
        variant.rule_trace.push(TraceStep {
            rule: rule.clone(),
            stage: stage.into(),
            input,
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

pub(super) fn strip_stress_marks(text: &str) -> String {
    text.chars()
        .filter(|character| !matches!(character, '\u{0300}' | '\u{0301}' | '\u{0311}' | '\u{0484}'))
        .collect()
}

pub(super) fn is_synodal_vowel(character: char) -> bool {
    matches!(
        character.to_lowercase().next().unwrap_or(character),
        'а' | 'е'
            | 'є'
            | 'ё'
            | 'и'
            | 'і'
            | 'ї'
            | 'о'
            | 'ѡ'
            | 'ꙍ'
            | 'у'
            | 'ꙋ'
            | 'ы'
            | 'э'
            | 'ю'
            | 'я'
            | 'ѧ'
            | 'ѩ'
            | 'ѣ'
    )
}

pub(super) fn append_phrase_trace(
    forms: FormSet,
    rule: &'static str,
    stage: &'static str,
) -> Result<FormSet> {
    let rule = RuleId::from(rule);
    let mut variants = Vec::with_capacity(forms.variants().len());
    for source in forms.variants() {
        let mut variant = source.clone();
        let construction_evidence = pronoun_phrase_evidence(rule.as_ref())?;
        if !variant
            .evidence
            .iter()
            .any(|item| item.id == construction_evidence.id)
        {
            variant.evidence.push(construction_evidence);
        }
        let evidence = variant
            .evidence
            .iter()
            .map(|item| item.id.clone())
            .collect();
        variant.rule_trace.push(TraceStep {
            rule: rule.clone(),
            stage: stage.into(),
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

pub(super) fn pronoun_phrase_evidence(rule: &str) -> Result<Evidence> {
    let (id, citation, note, roles) = match rule {
        "SYN-PRONOUN-ENCLITIC-PROSODY-ALYPY-47" => (
            "alypy-47-pronoun-enclisis",
            "Alypy (Gamanovich), §47 note 2",
            "short-pronoun enclisis and accent",
            vec![AuthorityRole::Grammatical, AuthorityRole::Accentual],
        ),
        "SYN-PRONOUN-THIRD-PREPOSITION-CONTRACTION-ALYPY-47" => (
            "alypy-47-third-person-contraction",
            "Alypy (Gamanovich), §47 note 1",
            "third-person prepositional contraction",
            vec![AuthorityRole::Grammatical, AuthorityRole::Morphological],
        ),
        "SYN-PRONOUN-NEGATIVE-PREPOSITION-ALYPY-48" => (
            "alypy-48-negative-pronoun-interposition",
            "Alypy (Gamanovich), §48",
            "negative-pronoun preposition interposition",
            vec![AuthorityRole::Grammatical, AuthorityRole::Morphological],
        ),
        _ => {
            return Err(Error::ContradictoryMetadata {
                reason: format!("unknown pronoun phrase rule {rule}"),
            });
        }
    };
    Ok(Evidence {
        id: EvidenceId::from(id),
        source: SourceId::from("alypy-gamanovich-grammar-web-2023"),
        source_recension: Recension::SynodalRussian,
        kind: EvidenceKind::NormativeRule,
        authority_roles: roles,
        epistemic_role: EpistemicRole::SynodalNormativeAuthority,
        citation: citation.into(),
        note: Some(note.into()),
    })
}

pub(super) fn separated_negative_base(forms: FormSet) -> Result<FormSet> {
    let rule = RuleId::from("SYN-PRONOUN-NEGATIVE-PREPOSITION-ALYPY-48");
    let mut variants = Vec::with_capacity(forms.variants().len());
    for source in forms.variants() {
        let mut variant = source.clone();
        let input = variant.printed.clone();
        variant.expanded = strip_negative_prefix(&variant.expanded)?;
        variant.printed = strip_negative_prefix(&variant.printed)?;
        variant.accented = variant
            .accented
            .as_deref()
            .map(strip_negative_prefix)
            .transpose()?;
        let construction_evidence = pronoun_phrase_evidence(rule.as_ref())?;
        if !variant
            .evidence
            .iter()
            .any(|item| item.id == construction_evidence.id)
        {
            variant.evidence.push(construction_evidence);
        }
        let evidence = variant
            .evidence
            .iter()
            .map(|item| item.id.clone())
            .collect();
        variant.rule_trace.push(TraceStep {
            rule: rule.clone(),
            stage: "negative-pronoun-preposition-interposition".into(),
            input,
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

pub(super) fn strip_negative_prefix(text: &str) -> Result<String> {
    text.strip_prefix("ни")
        .or_else(|| text.strip_prefix("Ни"))
        .map(str::to_owned)
        .ok_or_else(|| Error::ContradictoryMetadata {
            reason: "negative-pronoun interposition requires a surface ни- prefix".into(),
        })
}

/// Constructs a typed analytic phrase from already inflected tokens. This is the
/// escape hatch for reviewed future, pluperfect, conditional, passive, and
/// periphrastic combinations whose auxiliary lexeme is not yet in the registry.
pub fn from_tokens(
    construction: AnalyticConstruction,
    tokens: Vec<PhraseToken>,
) -> Result<RealizedPhrase> {
    RealizedPhrase::new(construction, tokens)
}
